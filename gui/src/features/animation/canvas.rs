use eframe::egui;
use std::sync::{Arc, Mutex};

// STRICT BOUNDARY: Only import the public engine API
use nyanko::animation::engine::{Unit, Anim, GlowRenderer, frame};

pub fn paint(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    renderer_reference: Arc<Mutex<Option<GlowRenderer>>>,
    unit: Arc<Unit>,
    animation: Option<Arc<Anim>>,
    current_frame: f32,
    pan: egui::Vec2,
    zoom: f32,
) {
    let paint_callback = egui::PaintCallback {
        rect,
        callback: Arc::new(eframe::egui_glow::CallbackFn::new(move |info, painter| {
            let Ok(mut renderer_lock) = renderer_reference.lock() else {
                return;
            };

            if renderer_lock.is_none() {
                // Safely handle the Result without unwrapping/panicking
                let Ok(new_renderer) = GlowRenderer::new(&**painter.gl()) else {
                    return;
                };
                *renderer_lock = Some(new_renderer);
            }

            let Some(renderer) = renderer_lock.as_mut() else {
                return;
            };

            let viewport_width = info.viewport.width();
            let viewport_height = info.viewport.height();

            // Delegate purely to the Nyanko engine's unified pipeline
            let _ = frame(
                renderer,
                &**painter.gl(),
                &unit,
                animation.as_deref(), // Converts Option<Arc<Anim>> to Option<&Anim>
                current_frame,
                viewport_width,
                viewport_height,
                pan.x,
                pan.y,
                zoom
            );
        })),
    };

    ui.painter().add(paint_callback);
}