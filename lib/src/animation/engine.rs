use glow;
pub use super::graphics::canvas::{GlowRenderer, CanvasError};

use crate::animation::data::mamodel::Model;
use crate::common::data::imgcut::SpriteSheet;
use crate::animation::graphics::{timeline, transform};
use crate::animation::utils::boundary::calculate_animation_bounds;
use crate::animation::utils::periodicity::calculate_difference;

pub use crate::animation::data::maanim::Animation as Anim;

/// A parsed container holding a unit's skeletal model and texture atlas.
pub struct Unit {
    pub(crate) model: Model,
    pub(crate) sheet: SpriteSheet,
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

/// Calculates and draws an animation frame.
///
/// # Arguments
/// * `renderer` - A mutable reference to the `GlowRenderer`.
/// * `gl_context` - The active OpenGL context.
/// * `unit` - The `Unit` to render.
/// * `anim` - An optional reference to an `Anim`. If `None`, the unit is rendered in its rest pose.
/// * `current_frame` - The time value for interpolation.
/// * `viewport_width` - The width of the viewport.
/// * `viewport_height` - The height of the viewport.
/// * `pan_x` - The horizontal camera translation.
/// * `pan_y` - The vertical camera translation.
/// * `zoom` - The camera scaling factor.
///
/// # Returns
/// A `Result` indicating whether the paint commands were successful.
pub fn frame(
    renderer: &mut GlowRenderer,
    gl_context: &glow::Context,
    unit: &Unit,
    anim: Option<&Anim>,
    current_frame: f32,
    viewport_width: f32,
    viewport_height: f32,
    pan_x: f32,
    pan_y: f32,
    zoom: f32,
) -> Result<(), CanvasError> {

    let parts = if let Some(animation) = anim {
        let mut state_buffer = unit.model.parts.clone();
        let _ = timeline::animate(&unit.model, animation, current_frame, &mut state_buffer);
        state_buffer
    } else {
        unit.model.parts.clone()
    };

    let world_parts = transform::solve_hierarchy(&parts, &unit.model);

    renderer.paint(
        gl_context,
        viewport_width,
        viewport_height,
        &world_parts,
        &unit.sheet,
        pan_x,
        pan_y,
        zoom
    )
}