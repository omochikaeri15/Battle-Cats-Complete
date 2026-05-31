use crate::graphics::data::maanim::Animation;
use crate::graphics::data::mamodel::Model;
use crate::graphics::game::timeline;
use crate::graphics::game::transform::{self, Vector};
use crate::graphics::data::imgcut::SpriteSheet;

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

    #[allow(dead_code)]
    pub fn center(&self) -> Vector {
        Vector {
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

#[derive(Clone, Copy, Debug)]
pub struct Tolerance {
    pub minimum_opacity: f32,
    pub minimum_glow_opacity: f32,
    pub maximum_scale: f32,
    pub scale_opacity_threshold: f32,
    pub maximum_vertical_stretch: f32,
    pub maximum_height_threshold: f32,
    pub minimum_y_bound: f32,
}

impl Tolerance {
    pub fn new(level: f32) -> Self {
        let clamped_level = level.clamp(0.0, 1.0);
        let inverse_level = 1.0 - clamped_level;

        Self {
            minimum_opacity: 0.01 + (0.24 * clamped_level),
            minimum_glow_opacity: 0.75 * clamped_level,
            maximum_scale: 3.0 + (inverse_level * 100.0),
            scale_opacity_threshold: 0.95 * clamped_level,
            maximum_vertical_stretch: 2.0 + (inverse_level * 50.0),
            maximum_height_threshold: 1000.0 + (inverse_level * 10000.0),
            minimum_y_bound: -1200.0 - (inverse_level * 10000.0),
        }
    }
}

pub fn calculate_animation_bounds(
    model: &Model,
    sheet: &SpriteSheet,
    animations: &[&Animation],
    tolerance: Tolerance,
) -> Option<BoundingBox> {
    let mut master_bounds: Option<BoundingBox> = None;

    for loaded_animation in animations {
        let new_bounds_option = scan_bounds(model, Some(*loaded_animation), sheet, tolerance, None);

        let Some(new_bounds) = new_bounds_option else {
            continue;
        };

        master_bounds = Some(match master_bounds {
            Some(existing_bounds) => existing_bounds.union(&new_bounds),
            None => new_bounds,
        });
    }

    master_bounds
}

pub fn scan_bounds(
    model: &Model,
    animation: Option<&Animation>,
    sheet: &SpriteSheet,
    tolerance: Tolerance,
    override_range: Option<(i32, i32)>,
) -> Option<BoundingBox> {
    let mut minimum_x = f32::MAX;
    let mut minimum_y = f32::MAX;
    let mut maximum_x = f32::MIN;
    let mut maximum_y = f32::MIN;
    let mut found_any_valid_parts = false;

    let (start_frame, end_frame) = match override_range {
        Some(range) => range,
        None => match animation {
            Some(loaded_anim) => (0, loaded_anim.max_frame),
            None => (0, 0),
        },
    };

    let mut state_buffer = model.parts.clone();

    for frame_index in start_frame..=end_frame {
        let current_frame = frame_index as f32;

        let posed_parts = match animation {
            Some(loaded_anim) => {
                let _ = timeline::animate(model, loaded_anim, current_frame, &mut state_buffer);
                &state_buffer
            }
            None => &model.parts,
        };

        let world_parts = transform::solve_hierarchy(posed_parts, model);

        for part in world_parts {
            if part.opacity <= 0.01 || part.hidden {
                continue;
            }
            
            if part.opacity < tolerance.minimum_opacity {
                continue;
            }

            if part.glow > 0 && part.opacity < tolerance.minimum_glow_opacity {
                continue;
            }

            let scale_x = (part.matrix[0].powi(2) + part.matrix[1].powi(2)).sqrt();
            let scale_y = (part.matrix[3].powi(2) + part.matrix[4].powi(2)).sqrt();
            let maximum_scale = scale_x.max(scale_y);

            if maximum_scale > tolerance.maximum_scale && (part.opacity < tolerance.scale_opacity_threshold || part.glow > 0) {
                continue;
            }
            
            let Some(cut) = sheet.cuts_map.get(&part.sprite_index) else {
                continue;
            };

            let sprite_width = cut.original_size.x;
            let sprite_height = cut.original_size.y;
            let pivot_x = part.pivot.x;
            let pivot_y = part.pivot.y;

            let local_corners = [
                Vector { x: -pivot_x, y: -pivot_y },
                Vector { x: sprite_width - pivot_x, y: -pivot_y },
                Vector { x: sprite_width - pivot_x, y: sprite_height - pivot_y },
                Vector { x: -pivot_x, y: sprite_height - pivot_y },
            ];

            let transform_matrix = part.matrix;
            let mut part_minimum_x = f32::MAX;
            let mut part_minimum_y = f32::MAX;
            let mut part_maximum_x = f32::MIN;
            let mut part_maximum_y = f32::MIN;

            for point in local_corners {
                let world_x = point.x * transform_matrix[0] + point.y * transform_matrix[3] + transform_matrix[6];
                let world_y = point.x * transform_matrix[1] + point.y * transform_matrix[4] + transform_matrix[7];

                part_minimum_x = part_minimum_x.min(world_x);
                part_maximum_x = part_maximum_x.max(world_x);
                part_minimum_y = part_minimum_y.min(world_y);
                part_maximum_y = part_maximum_y.max(world_y);
            }
            
            let part_total_height = part_maximum_y - part_minimum_y;
            let part_total_width = part_maximum_x - part_minimum_x;

            if part_total_height > tolerance.maximum_height_threshold && part_total_height > part_total_width * tolerance.maximum_vertical_stretch {
                continue;
            }

            if part_maximum_y < tolerance.minimum_y_bound {
                continue;
            }
            
            minimum_x = minimum_x.min(part_minimum_x);
            maximum_x = maximum_x.max(part_maximum_x);
            minimum_y = minimum_y.min(part_minimum_y);
            maximum_y = maximum_y.max(part_maximum_y);

            found_any_valid_parts = true;
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