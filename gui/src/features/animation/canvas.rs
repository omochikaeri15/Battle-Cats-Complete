use eframe::egui;
use std::sync::{Arc, Mutex};
use core::animation::logic::canvas::GlowRenderer;
use core::animation::logic::transform::WorldTransform;
use crate::global::sheet::GuiSpriteSheet;

pub fn paint(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    renderer_ref: Arc<Mutex<Option<GlowRenderer>>>,
    sheet: Arc<GuiSpriteSheet>,
    parts: Vec<WorldTransform>,
    pan: egui::Vec2,
    zoom: f32,
    allow_update: bool
) {
    let callback = egui::PaintCallback {
        rect,
        callback: Arc::new(eframe::egui_glow::CallbackFn::new(move |info, painter| {
            let Ok(mut renderer_lock) = renderer_ref.lock() else { return; };

            if renderer_lock.is_none() {
                *renderer_lock = Some(GlowRenderer::new(&**painter.gl()));
            }

            let Some(renderer) = renderer_lock.as_mut() else { return; };

            let viewport_width = info.viewport.width();
            let viewport_height = info.viewport.height();
            let pan_x = pan.x;
            let pan_y = pan.y;

            renderer.paint(
                &**painter.gl(),
                viewport_width,
                viewport_height,
                &parts,
                &sheet.core,
                pan_x,
                pan_y,
                zoom,
                allow_update
            );
        })),
    };

    ui.painter().add(callback);
}