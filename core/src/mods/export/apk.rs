use std::fs;
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use zip::ZipArchive;
use resand::res_value::ResValueType;

use crate::mods::logic::state::ModDataState;
use crate::global::region::Region;
use crate::data::utilities::keys;
use crate::settings::logic::state::Settings;
use crate::addons::apktool::xapk;
use crate::mods::export::{modify, sign, pack};
use crate::mods::export::patch::{EVENT_RECEIVER, ExportEvent, spawn_log_adapter};

pub fn start_export(state: &mut ModDataState, settings: &Settings) {
    if state.export.is_busy { return; }

    state.export.log_content.clear();
    state.export.is_busy = true;

    let app_title = state.export.app_title.clone();
    let suffix = state.export.package_suffix.clone();
    let replace_on_update = settings.mods.replace_on_update;
    let enforce_keys = settings.game_data.enforce_key_validation;

    let Some(mod_folder) = state.selected_mod.clone() else {
        state.export.is_busy = false; return;
    };
    let Some(input_apk_path) = state.export.selected_apk.clone() else {
        state.export.is_busy = false; return;
    };
    let detected_region = state.export.target_region.clone();

    let (transmitter, receiver) = mpsc::channel();
    if let Ok(mut guard) = EVENT_RECEIVER.lock() { *guard = Some(receiver); }

    let original_filename = input_apk_path.file_stem().and_then(|n| n.to_str()).unwrap_or("battlecats").to_string();

    thread::spawn(move || {
        let string_transmitter = spawn_log_adapter(transmitter.clone());
        let log_callback = |message: String| { let _ = transmitter.send(ExportEvent::Log(message)); };

        let user_keys = match keys::verify(enforce_keys, &string_transmitter) {
            Ok(keys) => keys,
            Err(error) => {
                let _ = transmitter.send(ExportEvent::Error(error)); return;
            }
        };

        let base_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("../../../.."));
        let mod_dir = base_dir.join("mods").join(&mod_folder);
        let export_dir = base_dir.join("exports");
        let app_dir = mod_dir.join("app");
        let temp_bin_dir = app_dir.join("binaries");
        let assets_dir = app_dir.join("assets");
        let xapk_dir = app_dir.join("xapk");

        let _ = fs::remove_dir_all(&app_dir);
        let _ = fs::create_dir_all(&export_dir);
        let _ = fs::create_dir_all(&temp_bin_dir);
        let _ = fs::create_dir_all(&assets_dir);

        let mut working_apk = input_apk_path.clone();
        let extension = input_apk_path.extension().and_then(|e| e.to_str()).unwrap_or("");

        // Handle Splits (XAPK/APKM)
        if extension == "xapk" || extension == "apkm" || extension == "apks" {
            log_callback("Merging split APKs...".to_string());
            let _ = fs::create_dir_all(&xapk_dir);
            let merged_temp_path = xapk_dir.join("merged_xapk.apk");

            if let Err(error) = xapk::merge_xapk(&working_apk, &merged_temp_path, &log_callback) {
                let _ = transmitter.send(ExportEvent::Error(error.to_string())); return;
            }
            working_apk = merged_temp_path;
        }

        // Look-Ahead Identity Check
        log_callback("Analyzing APK identity...".to_string());
        let manifest_path = temp_bin_dir.join("AndroidManifest.xml");
        let arsc_path = temp_bin_dir.join("resources.arsc");
        let mut extracted_arsc = false;

        let source_file = match fs::File::open(&working_apk) {
            Ok(f) => f,
            Err(e) => { let _ = transmitter.send(ExportEvent::Error(format!("Failed to open APK: {}", e))); return; }
        };

        let mut archive = match ZipArchive::new(source_file) {
            Ok(a) => a,
            Err(e) => { let _ = transmitter.send(ExportEvent::Error(format!("Failed to read APK archive: {}", e))); return; }
        };

        for i in 0..archive.len() {
            if let Ok(mut file) = archive.by_index(i) {
                if file.name() == "AndroidManifest.xml" {
                    if let Ok(mut out) = fs::File::create(&manifest_path) {
                        let _ = std::io::copy(&mut file, &mut out);
                    }
                } else if file.name() == "resources.arsc" {
                    if let Ok(mut out) = fs::File::create(&arsc_path) {
                        let _ = std::io::copy(&mut file, &mut out);
                        extracted_arsc = true;
                    }
                }
            }
        }
        drop(archive);

        let mut editor = match modify::ApkEditor::from_paths(&manifest_path, if extracted_arsc { Some(arsc_path.as_path()) } else { None }) {
            Ok(ed) => ed,
            Err(e) => { let _ = transmitter.send(ExportEvent::Error(format!("Failed to parse APK binaries: {}", e))); return; }
        };

        // Check current package
        let mut is_update = false;
        let target_package = format!("jp.co.ponos.battlecats{}", suffix.trim());

        if let Some(root) = editor.manifest.root.get_element(&["manifest"], &editor.manifest.string_pool) {
            if let Some(attr) = root.get_attribute("package", &editor.manifest.string_pool) {
                if let ResValueType::String(ref s) = attr.typed_value.data {
                    let current_pkg = s.resolve(&mut editor.manifest.string_pool).unwrap_or_default().to_string();
                    if current_pkg == target_package {
                        is_update = true;
                    }
                }
            }
        }

        // Apply Patches (If Necessary)
        let final_id = if is_update {
            log_callback("Package identity matches target APK.".to_string());
            log_callback("Updating target APK...".to_string());
            target_package
        } else {
            log_callback("New package identity found.".to_string());
            log_callback("Creating new APK...".to_string());
            match editor.apply_patches(&suffix, &app_title) {
                Ok(id) => {
                    if let Err(e) = editor.save_to_paths(&manifest_path, if extracted_arsc { Some(arsc_path.as_path()) } else { None }) {
                        let _ = transmitter.send(ExportEvent::Error(format!("Failed to save binaries: {}", e))); return;
                    }
                    id
                },
                Err(e) => {
                    let _ = transmitter.send(ExportEvent::Error(format!("Patch Error: {}", e))); return;
                }
            }
        };

        let region_key = match detected_region {
            Region::En => &user_keys.en,
            Region::Ja => &user_keys.ja,
            Region::Ko => &user_keys.ko,
            Region::Tw => &user_keys.tw,
        };

        // Pack & Inject
        log_callback("Packing modded game data...".to_string());
        if let Err(error) = pack::stream_pack_and_list(&mod_dir.join("patch"), &assets_dir, "DownloadLocal", region_key, &log_callback) {
            let _ = transmitter.send(ExportEvent::Error(error)); return;
        }

        log_callback("Rebuilding APK with patch...".to_string());
        let unsigned_apk_path = app_dir.join("unsigned_final.apk");

        match modify::inject_and_build_apk(
            &working_apk,
            &unsigned_apk_path,
            &assets_dir,
            &mod_dir.join("icons"),
            &mod_dir.join("loose"),
            if is_update { None } else { Some(manifest_path.as_path()) },
            if is_update || !extracted_arsc { None } else { Some(arsc_path.as_path()) }
        ) {
            Ok(count) => log_callback(format!("Injected {} files.", count)),
            Err(e) => { let _ = transmitter.send(ExportEvent::Error(format!("Build Error: {}", e))); return; }
        }

        // Normalize & Sign
        log_callback("Normalizing binaries...".to_string());
        let normalized_apk_path = app_dir.join("normalized_final.apk");
        if let Err(error) = modify::normalize_apk(&unsigned_apk_path, &normalized_apk_path, &working_apk) {
            let _ = transmitter.send(ExportEvent::Error(format!("Normalization Error: {}", error))); return;
        }

        log_callback("Signing APK...".to_string());
        if let Err(error) = sign::sign(&normalized_apk_path, None) {
            let _ = transmitter.send(ExportEvent::Error(format!("Native Signing Error: {}", error))); return;
        }

        // Output Routing
        let output_name = if app_title.trim().is_empty() { final_id } else { app_title.trim().to_string() };

        let is_in_exports = input_apk_path.parent().map_or(false, |parent| {
            parent.canonicalize().unwrap_or_else(|_| parent.to_path_buf()) == export_dir.canonicalize().unwrap_or_else(|_| export_dir.clone())
        });

        let final_apk_path = if replace_on_update && is_in_exports {
            input_apk_path.clone()
        } else if is_update && !replace_on_update && is_in_exports {
            export_dir.join(format!("{}_updated.apk", original_filename))
        } else {
            export_dir.join(format!("{}.apk", output_name))
        };

        if let Err(error) = fs::copy(&normalized_apk_path, &final_apk_path) {
            let _ = transmitter.send(ExportEvent::Error(format!("Filesystem Error: {}", error))); return;
        }

        let _ = fs::remove_dir_all(&app_dir);

        let final_filename = final_apk_path.file_name().unwrap_or_default().to_string_lossy();
        let success_message = if is_update && replace_on_update && is_in_exports {
            format!("Successfully Updated {}!", final_filename)
        } else {
            format!("Successfully Built {}!", final_filename)
        };

        let _ = transmitter.send(ExportEvent::Success(success_message));
    });
}