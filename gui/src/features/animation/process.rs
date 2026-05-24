use eframe::egui;
use std::sync::{Arc, Mutex};
use std::sync::atomic::Ordering;

use core::global::formats::mamodel::Model;
use core::global::formats::maanim::Animation;
use crate::global::sheet::GuiSpriteSheet;
use core::animation::export::state::ExporterState;
use core::animation::export::process::calculate_export_frame;
use core::animation::export::encoding::{self, EncoderMessage};
use core::animation::logic::canvas::GlowRenderer;

pub fn process_frame(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    state: &mut ExporterState,
    model: &Model,
    anim: Option<&Animation>,
    sheet: &GuiSpriteSheet,
    renderer_ref: Arc<Mutex<Option<GlowRenderer>>>,
    current_time: f32,
) {
    if state.tx.is_none() { return; }

    if let Some(abort) = &state.abort {
        if abort.load(Ordering::Relaxed) {
            state.tx = None;
            state.abort = None;
            return;
        }
    }

    let frame_count = (state.frame_end - state.frame_start).abs() + 1;
    if state.current_progress >= frame_count {
        if let Some(sender) = state.tx.take() {
            let _ = sender.send(EncoderMessage::Finish);
        }
        return;
    }

    // Call the core to calculate the bone positions
    let world_parts = calculate_export_frame(state, model, anim, current_time);

    let frame_delay_ms = 1000.0 / state.fps as f32;

    // De-couple egui::Vec2 into raw floats for the core
    let pan_x = -state.region_x - (state.region_w as f32 / (2.0 * state.zoom));
    let pan_y = -state.region_y - (state.region_h as f32 / (2.0 * state.zoom));

    let bg_color = if state.background { [80, 80, 80, 255] } else { [0, 0, 0, 0] };

    let renderer_arc = renderer_ref.clone();
    let sheet_arc = Arc::new(sheet.clone());
    let Some(sender) = state.tx.as_ref().cloned() else { return; };

    let width = state.region_w;
    let height = state.region_h;
    let zoom = state.zoom;

    ui.painter().add(egui::PaintCallback {
        rect,
        callback: Arc::new(eframe::egui_glow::CallbackFn::new(move |_, painter| {
            let Ok(mut lock) = renderer_arc.lock() else { return; };
            let Some(renderer) = lock.as_mut() else { return; };

            let raw_pixels = encoding::render_frame(
                renderer,
                &**painter.gl(),
                width as u32,
                height as u32,
                &world_parts,
                &sheet_arc.core,
                pan_x,
                pan_y,
                zoom,
                bg_color
            );

            let _ = sender.send(EncoderMessage::Frame(raw_pixels, width as u32, height as u32, frame_delay_ms as u32));
        })),
    });

    state.current_progress += 1;
}