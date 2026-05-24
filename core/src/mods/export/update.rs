use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;
use std::collections::HashMap;
use std::io::{Read, Write};
use zip::{ZipArchive, ZipWriter};

use crate::mods::logic::state::ModDataState;
use crate::global::region::Region;
use crate::data::utilities::keys;
use crate::settings::logic::state::Settings;
use crate::mods::export::pack;
use crate::mods::export::sign;
use crate::mods::export::patch::{EVENT_RECEIVER, ExportEvent, spawn_log_adapter};

pub fn start_fast_track_export(state: &mut ModDataState) {
    if state.export.is_busy { return; }

    state.export.log_content.clear();
    state.export.is_busy = true;

    let Some(mod_folder) = state.selected_mod.clone() else {
        state.export.is_busy = false;
        return;
    };

    let Some(input_apk_path) = state.export.selected_apk.clone() else {
        state.export.is_busy = false;
        return;
    };

    let target_region = state.export.target_region.clone();

    let original_filename = input_apk_path
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("updated_battlecats")
        .to_string();

    let (transmitter, receiver) = mpsc::channel();
    if let Ok(mut guard) = EVENT_RECEIVER.lock() { *guard = Some(receiver); }

    thread::spawn(move || {
        let string_transmitter = spawn_log_adapter(transmitter.clone());
        let log_callback = |message: String| { let _ = transmitter.send(ExportEvent::Log(message)); };

        let settings: Settings = crate::global::io::json::load("settings.json").unwrap_or_default();
        let user_keys = match keys::verify(settings.game_data.enforce_key_validation, &string_transmitter) {
            Ok(keys) => keys,
            Err(error) => {
                let _ = transmitter.send(ExportEvent::Error(error));
                return;
            }
        };

        let base_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("../../../.."));
        let mod_dir = base_dir.join("mods").join(&mod_folder);
        let export_dir = base_dir.join("exports");

        let app_dir = mod_dir.join("app");
        let patch_dir = mod_dir.join("patch");
        let loose_dir = mod_dir.join("loose");
        let icons_dir = mod_dir.join("icons");

        let _ = fs::remove_dir_all(&app_dir);
        let _ = fs::create_dir_all(&export_dir);
        let _ = fs::create_dir_all(&app_dir);

        let temp_assets_dir = app_dir.join("assets");
        let _ = fs::create_dir_all(&temp_assets_dir);

        let region_key = match target_region {
            Region::En => &user_keys.en,
            Region::Ja => &user_keys.ja,
            Region::Ko => &user_keys.ko,
            Region::Tw => &user_keys.tw,
        };

        if let Err(error) = pack::stream_pack_and_list(&patch_dir, &temp_assets_dir, "DownloadLocal", region_key, &log_callback) {
            let _ = transmitter.send(ExportEvent::Error(error));
            return;
        }

        let mut inject_map: HashMap<String, PathBuf> = HashMap::new();
        inject_map.insert("assets/DownloadLocal.pack".to_string(), temp_assets_dir.join("DownloadLocal.pack"));
        inject_map.insert("assets/DownloadLocal.list".to_string(), temp_assets_dir.join("DownloadLocal.list"));

        let mut loose_count = 0;
        if loose_dir.exists() {
            if let Ok(entries) = fs::read_dir(&loose_dir) {
                for entry in entries.flatten() {
                    if !entry.path().is_file() { continue; }

                    let filename = entry.file_name().to_string_lossy().to_string();
                    inject_map.insert(format!("assets/{}", filename), entry.path());
                    loose_count += 1;
                }
            }
        }

        if loose_count > 0 {
            log_callback(format!("Injected {} loose files.", loose_count));
        }

        let icon_png = icons_dir.join("icon.png");
        let icon_foreground = icons_dir.join("icon_foreground.png");
        let push_icon = icons_dir.join("push_icon.png");

        let source_file = match File::open(&input_apk_path) {
            Ok(file) => file,
            Err(error) => {
                let _ = transmitter.send(ExportEvent::Error(format!("Failed to open source APK: {}", error)));
                return;
            }
        };

        let mut archive = match ZipArchive::new(source_file) {
            Ok(zip_archive) => zip_archive,
            Err(error) => {
                let _ = transmitter.send(ExportEvent::Error(format!("Invalid Source APK format: {}", error)));
                return;
            }
        };

        let mut compression_catalog: HashMap<String, zip::CompressionMethod> = HashMap::new();
        for index in 0..archive.len() {
            if let Ok(archive_file) = archive.by_index(index) {
                compression_catalog.insert(archive_file.name().to_string(), archive_file.compression());
            }
        }

        let unsigned_apk_path = app_dir.join("unsigned_fast.apk");
        let destination_file = match File::create(&unsigned_apk_path) {
            Ok(file) => file,
            Err(error) => {
                let _ = transmitter.send(ExportEvent::Error(format!("Failed to create temp APK: {}", error)));
                return;
            }
        };
        let mut zip_writer = ZipWriter::new(destination_file);

        log_callback("Injecting and aligning files...".to_string());
        let mut injected_count = 0;

        for index in 0..archive.len() {
            let archive_result = archive.by_index(index);

            if let Err(error) = archive_result {
                let _ = transmitter.send(ExportEvent::Error(format!("Failed to access internal APK file at index {}: {}", index, error)));
                return;
            }

            let mut archive_file = archive_result.unwrap();
            let file_name = archive_file.name().to_string();

            if file_name.starts_with("META-INF/") { continue; }

            let mut replacement_path = inject_map.get(&file_name).cloned();

            if replacement_path.is_none() && file_name.starts_with("res/") {
                let short_filename = Path::new(&file_name).file_name().unwrap_or_default().to_string_lossy();

                if short_filename == "icon.png" && icon_png.exists() {
                    replacement_path = Some(icon_png.clone());
                } else if short_filename == "icon_foreground.png" && icon_foreground.exists() {
                    replacement_path = Some(icon_foreground.clone());
                } else if short_filename == "push_icon.png" && push_icon.exists() {
                    replacement_path = Some(push_icon.clone());
                }
            }

            if let Some(local_path) = replacement_path {
                let file_data = fs::read(&local_path).unwrap_or_default();
                let compression_method = compression_catalog.get(&file_name).copied().unwrap_or(zip::CompressionMethod::Deflated);

                let mut write_options = zip::write::SimpleFileOptions::default().compression_method(compression_method);

                if compression_method == zip::CompressionMethod::Stored {
                    let file_extension = Path::new(&file_name).extension().and_then(|extension| extension.to_str()).unwrap_or("");
                    let byte_alignment = if file_extension == "so" { 4096 } else { 4 };
                    write_options = write_options.with_alignment(byte_alignment);
                }

                if zip_writer.start_file(&file_name, write_options).is_ok() {
                    let _ = zip_writer.write_all(&file_data);
                    injected_count += 1;
                }
                inject_map.remove(&file_name);
                continue;
            }

            if archive_file.compression() == zip::CompressionMethod::Stored {
                let mut chunk_data = Vec::new();
                if archive_file.read_to_end(&mut chunk_data).is_ok() {
                    let file_extension = Path::new(&file_name).extension().and_then(|extension| extension.to_str()).unwrap_or("");
                    let byte_alignment = if file_extension == "so" { 4096 } else { 4 };
                    let stored_options = zip::write::SimpleFileOptions::default()
                        .compression_method(zip::CompressionMethod::Stored)
                        .with_alignment(byte_alignment);

                    if zip_writer.start_file(&file_name, stored_options).is_ok() {
                        let _ = zip_writer.write_all(&chunk_data);
                    }
                }
                continue;
            }

            if let Err(error) = zip_writer.raw_copy_file(archive_file) {
                let _ = transmitter.send(ExportEvent::Error(format!("Failed to copy internal zip chunk {}: {}", file_name, error)));
                return;
            }
        }

        drop(archive);

        for (zip_path, local_path) in inject_map {
            let chunk_data = fs::read(&local_path).unwrap_or_default();
            let file_extension = Path::new(&zip_path).extension().and_then(|extension| extension.to_str()).unwrap_or("");

            let is_stored = matches!(file_extension, "pack" | "list" | "dex" | "arsc" | "so" | "ogg");
            let compression_method = if is_stored { zip::CompressionMethod::Stored } else { zip::CompressionMethod::Deflated };

            let mut final_options = zip::write::SimpleFileOptions::default().compression_method(compression_method);
            if is_stored {
                let byte_alignment = if file_extension == "so" { 4096 } else { 4 };
                final_options = final_options.with_alignment(byte_alignment);
            }

            if zip_writer.start_file(&zip_path, final_options).is_ok() {
                let _ = zip_writer.write_all(&chunk_data);
                injected_count += 1;
            }
        }

        if let Err(error) = zip_writer.finish() {
            let _ = transmitter.send(ExportEvent::Error(format!("Failed to finalize APK Zip: {}", error)));
            return;
        }

        log_callback(format!("Successfully patched {} files.", injected_count));
        log_callback("Signing APK...".to_string());

        if let Err(error) = sign::sign(&unsigned_apk_path, None) {
            let _ = transmitter.send(ExportEvent::Error(format!("Native Signing Error: {}", error)));
            return;
        }

        let is_in_exports = input_apk_path.parent().map_or(false, |parent| {
            let parent_canon = parent.canonicalize().unwrap_or_else(|_| parent.to_path_buf());
            let export_canon = export_dir.canonicalize().unwrap_or_else(|_| export_dir.clone());
            parent_canon == export_canon
        });

        let final_name = if !settings.mods.replace_on_update && is_in_exports {
            format!("{}_updated", original_filename)
        } else {
            original_filename.clone()
        };

        let final_apk_path = if settings.mods.replace_on_update {
            input_apk_path.with_extension("apk")
        } else {
            export_dir.join(format!("{}.apk", final_name))
        };

        if let Err(error) = fs::copy(&unsigned_apk_path, &final_apk_path) {
            let _ = transmitter.send(ExportEvent::Error(format!("Filesystem Error: {}", error)));
            return;
        }

        let _ = fs::remove_dir_all(&app_dir);
        let _ = transmitter.send(ExportEvent::Success(format!("Successfully Updated {}.apk!", final_name)));
    });
}