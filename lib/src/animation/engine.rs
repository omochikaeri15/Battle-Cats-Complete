use crate::animation::graphics::{timeline, transform};
use crate::animation::utils::boundary::calculate_animation_bounds;
use crate::animation::utils::periodicity::calculate_difference;
use crate::common::SpriteSheet;

pub use crate::animation::data::mamodel::Model;
pub use crate::animation::data::maanim::Animation as Anim;

/// A parsed container holding a unit's skeletal model and texture atlas.
pub struct Unit {
    pub model: Model,
    pub sheet: SpriteSheet,
}

impl Unit {
    /// Parses raw byte streams into a `Unit`.
    ///
    /// # Arguments
    /// * `png` - The raw bytes of the texture atlas image.
    /// * `imgcut` - The raw bytes of the `.imgcut` map defining the sprite regions.
    /// * `mamodel` - The raw bytes of the `.mamodel` file defining the skeletal structure.
    ///
    /// # Returns
    /// Returns `Some(Unit)` if all files are successfully parsed, or `None` if any file is invalid.
    #[inline(always)]
    pub fn parse(
        png: impl AsRef<[u8]>,
        imgcut: impl AsRef<[u8]>,
        mamodel: impl AsRef<[u8]>,
    ) -> Option<Self> {
        Self::parse_inner(png.as_ref(), imgcut.as_ref(), mamodel.as_ref())
    }

    fn parse_inner(png: &[u8], imgcut: &[u8], mamodel: &[u8]) -> Option<Self> {
        let sheet = SpriteSheet::parse(png, imgcut)?;
        let model = Model::parse(mamodel)?;

        Some(Self { model, sheet })
    }

    /// Calculates the maximum spatial boundaries of the `Unit` across the provided animations.
    ///
    /// # Arguments
    /// * `animations` - A slice of animation references to be evaluated.
    /// * `tolerance` - A float between `0.0` and `1.0`. `1.0` enforces culling of extreme visual artifacts, while `0.0` calculates absolute bounds.
    ///
    /// # Returns
    /// An `Option` containing a tuple representing `(x, y, width, height)`. Returns `None` if the unit is completely invisible.
    pub fn calculate_bounds(
        &self,
        animations: &[&Anim],
        tolerance: f32
    ) -> Option<(f32, f32, f32, f32)> {
        let tolerance = crate::animation::utils::boundary::Tolerance::new(tolerance);
        let bounds = calculate_animation_bounds(&self.model, &self.sheet, animations, tolerance)?;

        Some((bounds.min_x, bounds.min_y, bounds.width(), bounds.height()))
    }

    /// Scans the `Unit` to find a repeating animation loop.
    ///
    /// # Arguments
    /// * `animation` - The animation sequence to evaluate.
    /// * `tolerance` - The maximum allowable deviation between two transformation matrices to be considered a match.
    /// * `minimum_frame` - The earliest frame a loop is permitted to begin.
    /// * `maximum_frame` - An optional upper frame limit for the search.
    /// * `progress_callback` - A closure called every frame. If the closure returns `false`, the search aborts.
    ///
    /// # Returns
    /// An `Option` containing a tuple of `(loop_start_frame, loop_end_frame)`. Returns `None` if no loop is found.
    pub fn calculate_cycle(
        &self,
        animation: &Anim,
        tolerance: f32,
        minimum_frame: Option<i32>,
        maximum_frame: Option<i32>,
        mut progress_callback: impl FnMut(usize) -> bool,
    ) -> Option<(i32, i32)> {
        let mut frame_states: Vec<Vec<([f32; 9], f32)>> = Vec::new();
        let mut state_buffer = self.model.parts.clone();
        let mut current_frame = 0;

        let minimum_loop_length = minimum_frame.unwrap_or(1);

        loop {
            if !progress_callback(current_frame) {
                return None;
            }

            if let Some(maximum) = maximum_frame {
                if current_frame > maximum as usize {
                    return None;
                }
            }

            let frame_float = current_frame as f32;
            let _ = timeline::animate(&self.model, animation, frame_float, &mut state_buffer);
            let world_parts = transform::solve_hierarchy(&state_buffer, &self.model);

            let mut current_state = Vec::with_capacity(world_parts.len());
            for part in &world_parts {
                current_state.push((part.matrix, part.opacity));
            }

            for (past_frame_index, past_state) in frame_states.iter().enumerate() {
                let loop_length = current_frame as i32 - past_frame_index as i32;

                if loop_length < minimum_loop_length {
                    continue;
                }

                let difference = calculate_difference(&current_state, past_state);

                if difference <= tolerance {
                    return Some((past_frame_index as i32, current_frame as i32));
                }
            }

            frame_states.push(current_state);
            current_frame += 1;
        }
    }
}

/// Represents the pre-calculated, hardware-agnostic geometric and visual state of a single
/// model part for a specific animation frame.
///
/// This structure acts as the intermediary payload between the pure mathematical domain
/// library and the hardware-accelerated orchestrator, providing flat, contiguous memory
/// buffers suitable for direct GPU ingestion.
#[derive(Clone, Debug, Default)]
pub struct FrameData {
    /// The index mapping to the specific sprite cut in the associated `SpriteSheet`.
    pub sprite_index: usize,
    /// The 3x3 affine transformation matrix representing the part's absolute world-space position, rotation, and scale.
    pub final_matrix: [f32; 9],
    /// The local spatial coordinates bounding the part, formatted as a flat array of 12 floats (two triangles, 6 vertices, x/y pairs).
    pub vertices: [f32; 12],
    /// The texture mapping coordinates corresponding to the `vertices`, formatted as a flat array of 12 floats.
    pub uvs: [f32; 12],
    /// The final calculated alpha transparency of the part, bounded between 0.0 and 1.0.
    pub opacity: f32,
    /// A boolean flag indicating whether the additive blending (glow) shader should be applied to this part.
    pub is_glow: bool,
}

/// Computes the complete world-space geometry for a unit at a specific point in time.
///
/// This function acts as the primary front-facing API for the rendering pipeline. It fully isolates
/// the mathematical computation of the skeletal timeline, hierarchical matrix resolution, and sprite
/// bounding from any external graphics context or camera projection math.
///
/// # Arguments
/// * `unit` - A reference to the parsed `Unit` containing the base skeletal model and texture atlas.
/// * `anim` - An optional reference to an `Anim`. If `Some`, the timeline interpolates the model's parts to the specified frame. If `None`, the model remains evaluated in its default rest pose.
/// * `frame` - The precise chronological time value to evaluate.
///
/// # Returns
/// A dynamically allocated `Vec` containing the compiled `FrameData` for every visible part, sorted by drawing order layer.
pub fn resolve_frame(
    unit: &Unit,
    anim: Option<&Anim>,
    frame: f32,
) -> Vec<FrameData> {

    let parts = if let Some(animation) = anim {
        let mut state_buffer = unit.model.parts.clone();
        let _ = timeline::animate(&unit.model, animation, frame, &mut state_buffer);
        state_buffer
    } else {
        unit.model.parts.clone()
    };

    let world_parts = transform::solve_hierarchy(&parts, &unit.model);

    crate::animation::graphics::construct::build_geometry(&world_parts, &unit.sheet)
}