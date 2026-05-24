use std::path::PathBuf;
use crate::global::formats::mamodel::Model;
use crate::global::formats::maanim::Animation;
use crate::global::formats::imgcut::SpriteSheet;
use crate::animation::logic::constants;
use crate::animation::logic::{animator, transform};

#[derive(Clone, Copy, Debug)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct BoundingBox {
    pub min_x: f32,
    pub min_y: f32,
    pub max_x: f32,
    pub max_y: f32,
}

impl BoundingBox {
    pub fn width(&self) -> f32 {
        self.max_x - self.min_x
    }

    pub fn height(&self) -> f32 {
        self.max_y - self.min_y
    }

    pub fn center(&self) -> Vector2 {
        Vector2 {
            x: (self.min_x + self.max_x) / 2.0,
            y: (self.min_y + self.max_y) / 2.0,
        }
    }

    pub fn union(&self, other: &BoundingBox) -> BoundingBox {
        BoundingBox {
            min_x: self.min_x.min(other.min_x),
            min_y: self.min_y.min(other.min_y),
            max_x: self.max_x.max(other.max_x),
            max_y: self.max_y.max(other.max_y),
        }
    }
}

pub fn calculate_showcase_bounds(
    model: &Model,
    sheet: &SpriteSheet,
    available_anims: &[(usize, PathBuf)],
    walk_len: i32,
    idle_len: i32,
    attack_len: i32,
    kb_len: i32,
    use_tight_bounds: bool,
) -> Option<BoundingBox> {
    let mut master_bounds: Option<BoundingBox> = None;

    let target_animations = [
        (constants::IDX_WALK, walk_len),
        (constants::IDX_IDLE, idle_len),
        (constants::IDX_ATTACK, attack_len),
        (constants::IDX_KB, kb_len),
    ];

    for (anim_index, anim_length) in target_animations {
        if anim_length > 0 {
            if let Some((_, path)) = available_anims.iter().find(|(i, _)| *i == anim_index) {
                if let Some(loaded_animation) = Animation::load(path) {
                    let bounds_option = if use_tight_bounds {
                        calculate_tight_bounds(model, Some(&loaded_animation), sheet)
                    } else {
                        calculate_loose_bounds(model, Some(&loaded_animation), sheet)
                    };

                    if let Some(new_bounds) = bounds_option {
                        // FIX: Avoid map_or consuming the variable
                        master_bounds = Some(if let Some(existing_bounds) = master_bounds {
                            existing_bounds.union(&new_bounds)
                        } else {
                            new_bounds
                        });
                    }
                }
            }
        }
    }
    master_bounds
}

pub fn calculate_tight_bounds(
    model: &Model,
    anim: Option<&Animation>,
    sheet: &SpriteSheet
) -> Option<BoundingBox> {
    let solid_bounds = scan_bounds(model, anim, sheet, true, None);

    if solid_bounds.is_some() {
        return solid_bounds;
    }

    scan_bounds(model, anim, sheet, false, None)
}

pub fn calculate_loose_bounds(
    model: &Model,
    anim: Option<&Animation>,
    sheet: &SpriteSheet
) -> Option<BoundingBox> {
    scan_bounds(model, anim, sheet, false, None)
}

pub fn calculate_initial_view(
    model: &Model,
    anim: Option<&Animation>,
    sheet: &SpriteSheet,
    viewport_width: f32,
    viewport_height: f32,
    use_tight_bounds: bool,
) -> Option<(Vector2, f32)> {

    let frame_zero = Some((0, 0));

    let bounds_option = if use_tight_bounds {
        scan_bounds(model, anim, sheet, true, frame_zero)
            .or_else(|| scan_bounds(model, anim, sheet, false, frame_zero))
    } else {
        scan_bounds(model, anim, sheet, false, frame_zero)
    };

    if let Some(bounds) = bounds_option {
        let center = bounds.center();
        let pan = Vector2 {
            x: -center.x,
            y: -center.y,
        };

        let width = bounds.width().max(1.0);
        let height = bounds.height().max(1.0);

        let scale_x = viewport_width / width;
        let scale_y = viewport_height / height;

        let breathing_room = 0.45;

        let zoom = scale_x.min(scale_y).clamp(0.05, 5.0) * breathing_room;

        return Some((pan, zoom));
    }

    None
}

fn scan_bounds(
    model: &Model,
    anim: Option<&Animation>,
    sheet: &SpriteSheet,
    strict_mode: bool,
    override_range: Option<(i32, i32)>
) -> Option<BoundingBox> {
    let mut minimum_x = f32::MAX;
    let mut minimum_y = f32::MAX;
    let mut maximum_x = f32::MIN;
    let mut maximum_y = f32::MIN;
    let mut found_any_valid_parts = false;

    let (start_frame, end_frame) = if let Some(range) = override_range {
        range
    } else if let Some(loaded_anim) = anim {
        (0, loaded_anim.max_frame)
    } else {
        (0, 0)
    };

    for frame_index in start_frame..=end_frame {
        let current_frame = frame_index as f32;
        let posed_parts = if let Some(loaded_anim) = anim {
            animator::animate(model, loaded_anim, current_frame)
        } else {
            model.parts.clone()
        };
        let world_parts = transform::solve_hierarchy(&posed_parts, model);

        for part in world_parts {

            if strict_mode {
                if part.opacity < 0.25 { continue; }
                if part.glow > 0 && part.opacity < 0.75 { continue; }

                let scale_x = (part.matrix[0].powi(2) + part.matrix[1].powi(2)).sqrt();
                let scale_y = (part.matrix[3].powi(2) + part.matrix[4].powi(2)).sqrt();
                let max_scale = scale_x.max(scale_y);

                if max_scale > 3.0 {
                    if part.opacity < 0.95 || part.glow > 0 {
                        continue;
                    }
                }
            } else {
                if part.opacity <= 0.01 || part.hidden { continue; }
            }

            if let Some(cut) = sheet.cuts_map.get(&part.sprite_index) {
                let sprite_width = cut.original_size.x;
                let sprite_height = cut.original_size.y;
                let pivot_x = part.pivot.x;
                let pivot_y = part.pivot.y;

                let local_corners = [
                    Vector2 { x: -pivot_x, y: -pivot_y },
                    Vector2 { x: sprite_width - pivot_x, y: -pivot_y },
                    Vector2 { x: sprite_width - pivot_x, y: sprite_height - pivot_y },
                    Vector2 { x: -pivot_x, y: sprite_height - pivot_y },
                ];

                let transform_matrix = part.matrix;
                let mut part_min_x = f32::MAX;
                let mut part_min_y = f32::MAX;
                let mut part_max_x = f32::MIN;
                let mut part_max_y = f32::MIN;

                for point in local_corners {
                    let world_x = point.x * transform_matrix[0] + point.y * transform_matrix[3] + transform_matrix[6];
                    let world_y = point.x * transform_matrix[1] + point.y * transform_matrix[4] + transform_matrix[7];

                    part_min_x = part_min_x.min(world_x);
                    part_max_x = part_max_x.max(world_x);
                    part_min_y = part_min_y.min(world_y);
                    part_max_y = part_max_y.max(world_y);
                }

                if strict_mode {
                    let part_total_height = part_max_y - part_min_y;
                    let part_total_width = part_max_x - part_min_x;

                    if part_total_height > 1000.0 && part_total_height > part_total_width * 2.0 {
                        continue;
                    }

                    if part_max_y < -1200.0 {
                        continue;
                    }
                }

                minimum_x = minimum_x.min(part_min_x);
                maximum_x = maximum_x.max(part_max_x);
                minimum_y = minimum_y.min(part_min_y);
                maximum_y = maximum_y.max(part_max_y);

                found_any_valid_parts = true;
            }
        }
    }

    if found_any_valid_parts {
        Some(BoundingBox {
            min_x: minimum_x,
            min_y: minimum_y,
            max_x: maximum_x,
            max_y: maximum_y,
        })
    } else {
        None
    }
}