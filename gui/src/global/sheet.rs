use eframe::egui;
use std::path::Path;
use core::global::formats::imgcut::SpriteSheet;

#[derive(Default, Clone)]
pub struct GuiSpriteSheet {
    pub core: SpriteSheet,
    pub texture_handle: Option<egui::TextureHandle>,
}

impl GuiSpriteSheet {
    pub fn load(&mut self, png_path: &Path, imgcut_path: &Path, id_str: String) {
        // Forward the load request to the background thread in the core
        self.core.load(png_path, imgcut_path, id_str);
    }

    pub fn update(&mut self, ctx: &egui::Context) {
        // Tick the core to see if the thread finished loading the raw image
        self.core.update();

        // If the core has the image data, but we haven't uploaded it to the GPU yet:
        if self.texture_handle.is_none() && !self.core.is_loading_active
            && let Some(image_data) = &self.core.image_data {
                let size = [image_data.width() as usize, image_data.height() as usize];
                let pixels = image_data.as_flat_samples();
                let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());

                self.texture_handle = Some(ctx.load_texture(
                    &self.core.sheet_name,
                    color_image,
                    egui::TextureOptions::LINEAR
                ));
            }
    }
}