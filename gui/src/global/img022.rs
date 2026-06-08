use eframe::egui;
use core::global::io::paths;
use core::global::resolver;
use core::settings::logic::state::Settings;
use crate::global::sheet::GuiSpriteSheet;

pub fn ensure_loaded(ctx: &egui::Context, sheets: &mut Vec<GuiSpriteSheet>, settings: &Settings) {
    let base_dir = paths::img022_folder(std::path::Path::new(""));

    let png_paths = resolver::get(&base_dir, ["img022.png"], &settings.general.language_priority);
    let cut_paths = resolver::get(&base_dir, ["img022.imgcut"], &settings.general.language_priority);

    if sheets.len() != png_paths.len() {
        sheets.resize_with(png_paths.len(), GuiSpriteSheet::default);
    }

    for (i, (png_path, imgcut_path)) in png_paths.into_iter().zip(cut_paths).enumerate() {
        sheets[i].update(ctx);

        if sheets[i].texture_handle.is_none() && !sheets[i].core.is_loading_active {
            let key = png_path.file_stem().unwrap().to_string_lossy().into_owned();
            sheets[i].load(&png_path, &imgcut_path, key);
        }
    }
}