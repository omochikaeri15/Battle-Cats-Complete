use crate::graphics::game::{timeline, transform, construct};
use crate::graphics::utils::boundary::calculate_animation_bounds;
use crate::graphics::utils::periodicity::calculate_difference;
use crate::graphics::data::imgcut::SpriteSheet;

pub use crate::graphics::data::mamodel::Model;
pub use crate::graphics::data::maanim::Animation as Anim;

/// A comprehensive container representing a fully parsed in-game entity.
///
/// This structure aggregates the raw skeletal hierarchy (`Model`) and the mapped
/// texture atlas (`SpriteSheet`) required to geometrically construct and render
/// a unit or enemy within the engine.
pub struct Unit {
    /// The hierarchical skeletal structure and baseline component definitions.
    pub model: Model,
    /// The parsed texture atlas mapping for the entity's sliced sprite regions.
    pub sheet: SpriteSheet,
}

impl Unit {
    /// Deserializes and validates raw binary streams into a cohesive `Unit` structure.
    ///
    /// This function serves as the primary ingestion point for a unit's core visual assets.
    /// It coordinates the parsing of the unencrypted raw byte streams for the texture atlas (`.png`),
    /// the sprite mapping definitions (`.imgcut`), and the skeletal structure (`.mamodel`).
    ///
    /// # Arguments
    /// * `png` - An object implementing `AsRef<[u8]>` containing the raw PNG image data.
    /// * `imgcut` - An object implementing `AsRef<[u8]>` containing the delimited `.imgcut` map.
    /// * `mamodel` - An object implementing `AsRef<[u8]>` containing the `.mamodel` skeletal hierarchy.
    ///
    /// # Returns
    /// An `Option<Self>`. Returns `Some(Unit)` if all discrete files are successfully parsed
    /// and mathematically linked. Returns `None` if any individual file fails strict structural validation.
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

    /// Calculates the maximum spatial bounding box of the `Unit` across a sequence of animations.
    ///
    /// This function iterates through the provided animation data to determine the absolute
    /// spatial extremities (min/max X and Y) reached by the unit's sprites during playback.
    /// This computation is necessary for strict rendering culling and localized UI alignment.
    ///
    /// # Arguments
    /// * `animations` - A slice of references to `Anim` objects representing the animation sequences to evaluate.
    /// * `tolerance` - A floating-point threshold between `0.0` and `1.0`. A value of `1.0` aggressively culls geometry resulting from scale extreme visual artifacts, while `0.0` strictly calculates the absolute mathematical outer limits.
    ///
    /// # Returns
    /// An `Option` containing a tuple representing `(min_x, min_y, total_width, total_height)`.
    /// Returns `None` if the unit evaluates to completely invisible across all provided frames.
    pub fn calculate_bounds(
        &self,
        animations: &[&Anim],
        tolerance: f32
    ) -> Option<(f32, f32, f32, f32)> {
        let tolerance = crate::graphics::utils::boundary::Tolerance::new(tolerance);
        let bounds = calculate_animation_bounds(&self.model, &self.sheet, animations, tolerance)?;

        Some((bounds.min_x, bounds.min_y, bounds.width(), bounds.height()))
    }

    /// Scans a unit's chronological animation timeline to programmatically detect repeating spatial loops.
    ///
    /// This function simulates the skeleton's state frame-by-frame and compares the resulting
    /// world-space transformation matrices. A loop is identified when the mathematical difference
    /// between the current frame's hierarchy and a previously recorded frame's hierarchy falls
    /// beneath the provided tolerance threshold.
    ///
    /// # Arguments
    /// * `animation` - The specific `Anim` sequence timeline to analytically evaluate.
    /// * `tolerance` - The maximum allowable delta between two hierarchical matrix states to be considered a visual match.
    /// * `minimum_frame` - An optional floor constraint; the earliest chronological frame where a valid loop is permitted to begin.
    /// * `maximum_frame` - An optional ceiling constraint; if the search exceeds this frame without finding a match, the algorithm aborts.
    /// * `progress_callback` - A mutable closure invoked at the start of every frame evaluation. If the closure evaluates to `false`, the search immediately terminates. Useful for threaded cancellations.
    ///
    /// # Returns
    /// An `Option` containing a tuple of `(loop_start_frame, loop_end_frame)`.
    /// Returns `None` if the timeline exhausts the `maximum_frame` or triggers user abortion without identifying a valid loop.
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

            if let Some(maximum) = maximum_frame
                && current_frame > maximum as usize {
                return None;
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
/// model part for a specific, locked animation frame.
///
/// This structure acts as the intermediary payload between the pure mathematical domain
/// library and the hardware-accelerated orchestrator, providing flat, contiguous memory
/// buffers inherently suitable for direct GPU ingestion or vertex array allocation.
#[derive(Clone, Debug, Default)]
pub struct FrameData {
    /// The index mapping to the localized sprite cut within the parent `SpriteSheet`.
    pub sprite_index: usize,
    /// The 3x3 affine transformation matrix representing the part's absolute world-space position, rotation, and scale.
    pub final_matrix: [f32; 9],
    /// The local spatial coordinates bounding the part, formatted as a flat array of 12 floats (two triangles, 6 vertices, structured as x/y pairs).
    pub vertices: [f32; 12],
    /// The texture mapping coordinates corresponding exactly to the `vertices`, formatted as a flat array of 12 floats.
    pub uvs: [f32; 12],
    /// The chronologically calculated alpha transparency of the part, bounded between 0.0 and 1.0.
    pub opacity: f32,
    /// A localized flag indicating whether the additive blending (glow) rasterization shader must be applied to this specific part.
    pub glow: i32,
}

/// Computes the complete world-space geometry for a unit at a specific temporal coordinate.
///
/// This function acts as the primary front-facing data extraction API for the rendering pipeline. It fully
/// isolates the mathematical computation of the skeletal timeline, hierarchical matrix resolution, and sprite
/// extraction from any external graphics context, hardware state, or projection math.
///
/// # Arguments
/// * `unit` - A reference to the parsed `Unit` containing the base skeletal model and mapped texture atlas.
/// * `anim` - An optional reference to an `Anim`. If `Some`, the timeline interpolates the model's parts to the specified frame. If `None`, the model remains statically evaluated in its default rest pose.
/// * `frame` - The precise chronological floating-point time value to execute matrix evaluations against.
///
/// # Returns
/// A dynamically allocated `Vec` containing the populated `FrameData` payloads for every visible part,
/// sorted strictly by their chronological drawing order (z-index) layer.
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

    construct::build_geometry(&world_parts, &unit.sheet)
}