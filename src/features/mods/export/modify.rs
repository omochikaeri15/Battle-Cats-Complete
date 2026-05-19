use std::fs;
use std::path::Path;
use regex::Regex;
use zip::{ZipArchive, ZipWriter};
use rayon::prelude::*;
use std::io::{Read, Write, Cursor};
use std::collections::HashSet;

pub fn patch_identity(decode_dir: &Path, new_suffix: &str, app_title: &str, _log_callback: &impl Fn(String)) -> Result<String, String> {
    let suffix = new_suffix.trim();
    if suffix.is_empty() {
        return Ok("jp.co.ponos.battlecats".to_string());
    }

    let manifest_path = decode_dir.join("AndroidManifest.xml");
    let strings_path = decode_dir.join("res").join("values").join("strings.xml");

    if !manifest_path.exists() {
        return Err("Decoded AndroidManifest.xml not found. Apktool decode may have failed.".into());
    }

    let mut manifest_text = fs::read_to_string(&manifest_path).map_err(|error| error.to_string())?;

    let targets = [
        "battlecatsen",
        "battlecatsko",
        "battlecatstw",
        "battlecats",
    ];

    let mut active_token = "";
    for target in targets.iter() {
        if manifest_text.contains(&format!("package=\"jp.co.ponos.{}\"", target)) {
            active_token = target;
            break;
        }
    }

    if active_token.is_empty() {
        for target in targets.iter() {
            if manifest_text.contains(target) {
                active_token = target;
                break;
            }
        }
    }

    if active_token.is_empty() {
        return Err("Could not find a recognized Battle Cats package token to patch.".into());
    }

    let new_token = format!("battlecats{}", suffix);
    let final_package_id = format!("jp.co.ponos.{}", new_token);

    manifest_text = manifest_text.replace(active_token, &new_token);

    manifest_text = manifest_text.replace("android:isSplitRequired=\"true\"", "android:isSplitRequired=\"false\"");
    manifest_text = manifest_text.replace("android:extractNativeLibs=\"false\"", "android:extractNativeLibs=\"true\"");

    let split_regex = Regex::new(r#"split="[^"]*""#).expect("Failed to compile split regex");
    let feature_split_regex = Regex::new(r#"android:isFeatureSplit="true""#).expect("Failed to compile feature split regex");
    manifest_text = split_regex.replace_all(&manifest_text, "").to_string();
    manifest_text = feature_split_regex.replace_all(&manifest_text, "").to_string();

    if let Some(start_index) = manifest_text.find("<split") {
        let end_offset = manifest_text[start_index..].find("/>").unwrap_or(0);
        if end_offset > 0 {
            manifest_text.replace_range(start_index..start_index + end_offset + 2, "");
        }
    }

    fs::write(&manifest_path, manifest_text).map_err(|error| error.to_string())?;

    if strings_path.exists() {
        let mut strings_text = fs::read_to_string(&strings_path).map_err(|error| error.to_string())?;
        strings_text = strings_text.replace(active_token, &new_token);

        if !app_title.trim().is_empty() {
            let app_name_regex = Regex::new(r#"<string name="app_name">[^<]*</string>"#).unwrap();
            let new_title_element = format!("<string name=\"app_name\">{}</string>", app_title.trim());
            strings_text = app_name_regex.replace_all(&strings_text, new_title_element.as_str()).to_string();
        }

        fs::write(&strings_path, strings_text).map_err(|error| error.to_string())?;
    }

    Ok(final_package_id)
}

pub fn replace_icons(mod_dir: &Path, decode_dir: &Path, _log_callback: &impl Fn(String)) -> Result<(), String> {
    let icons_dir = mod_dir.join("icons");
    let targets = [
        (icons_dir.join("icon.png"), "icon.png"),
        (icons_dir.join("icon_foreground.png"), "icon_foreground.png"),
        (icons_dir.join("push_icon.png"), "push_icon.png"),
    ];

    if targets.iter().all(|(path, _)| !path.exists()) { return Ok(()); }

    let res_dir = decode_dir.join("res");
    if !res_dir.exists() { return Ok(()); }

    let target_dirs = [
        "drawable-xhdpi",
        "drawable-xxhdpi",
        "drawable-xxxhdpi"
    ];

    for dir_name in target_dirs {
        let target_dir = res_dir.join(dir_name);
        if !target_dir.exists() { continue; }

        for (source_path, target_name) in &targets {
            if !source_path.exists() { continue; }

            let destination_path = target_dir.join(target_name);
            if !destination_path.exists() { continue; }

            let _ = fs::copy(source_path, &destination_path);
        }
    }

    Ok(())
}

pub fn inject_loose_assets(mod_dir: &Path, decode_dir: &Path) -> Result<usize, String> {
    let loose_dir = mod_dir.join("loose");
    if !loose_dir.exists() { return Ok(0); }

    let assets_dir = decode_dir.join("assets");
    let _ = fs::create_dir_all(&assets_dir);

    let directory_entries: Vec<_> = fs::read_dir(&loose_dir)
        .map_err(|error| error.to_string())?
        .flatten()
        .collect();

    let copied_count: usize = directory_entries.into_par_iter().map(|entry| {
        let source_path = entry.path();
        if !source_path.is_file() { return 0; }

        let filename = source_path.file_name().unwrap_or_default();
        let destination_path = assets_dir.join(filename);

        let source_meta = fs::metadata(&source_path).ok();
        let destination_meta = fs::metadata(&destination_path).ok();
        let same_size = source_meta.map(|m| m.len()) == destination_meta.map(|m| m.len());

        if destination_path.exists() && same_size {
            let source_data = fs::read(&source_path).unwrap_or_default();
            let destination_data = fs::read(&destination_path).unwrap_or_default();

            if source_data == destination_data {
                return 0;
            }
        }

        if fs::copy(&source_path, &destination_path).is_ok() {
            return 1;
        }

        0
    }).sum();

    Ok(copied_count)
}

pub fn normalize_apk(input_apk: &Path, output_apk: &Path, original_apk: &Path) -> Result<(), String> {
    let mut stored_files_map = HashSet::new();

    let original_file = fs::File::open(original_apk).map_err(|error| format!("Failed to open original APK: {}", error))?;
    let mut original_archive = ZipArchive::new(original_file).map_err(|error| format!("Failed to read original APK: {}", error))?;

    for index in 0..original_archive.len() {
        let mut archive_file = original_archive.by_index(index).map_err(|error| error.to_string())?;
        let file_name = archive_file.name().to_string();

        if !file_name.ends_with(".apk") {
            if archive_file.compression() == zip::CompressionMethod::Stored {
                stored_files_map.insert(file_name);
            }
            continue;
        }

        let mut apk_data = Vec::new();
        archive_file.read_to_end(&mut apk_data).map_err(|error| error.to_string())?;

        let cursor = Cursor::new(apk_data);
        let mut nested_archive = ZipArchive::new(cursor).map_err(|error| error.to_string())?;

        for nested_index in 0..nested_archive.len() {
            let nested_file = nested_archive.by_index(nested_index).map_err(|error| error.to_string())?;
            if nested_file.compression() == zip::CompressionMethod::Stored {
                stored_files_map.insert(nested_file.name().to_string());
            }
        }
    }

    let source_file = fs::File::open(input_apk).map_err(|error| format!("Failed to open APK: {}", error))?;
    let mut archive = ZipArchive::new(source_file).map_err(|error| format!("Failed to read APK archive: {}", error))?;

    let destination_file = fs::File::create(output_apk).map_err(|error| format!("Failed to create normalized APK: {}", error))?;
    let mut zip_writer = ZipWriter::new(destination_file);

    let uncompressed_extensions = ["dex", "arsc", "so", "pack", "list", "ogg"];

    for index in 0..archive.len() {
        let mut archive_file = archive.by_index(index).unwrap();
        let file_name = archive_file.name().to_string();
        let file_extension = Path::new(&file_name).extension().and_then(|ext| ext.to_str()).unwrap_or("");

        let force_store = uncompressed_extensions.contains(&file_extension);
        let is_already_stored = stored_files_map.contains(&file_name);

        if !force_store && !is_already_stored {
            zip_writer.raw_copy_file(archive_file).map_err(|error| error.to_string())?;
            continue;
        }

        let mut file_data = Vec::new();
        archive_file.read_to_end(&mut file_data).map_err(|error| format!("Failed reading {}: {}", file_name, error))?;

        let byte_alignment = if file_extension == "so" { 4096 } else { 4 };

        let write_options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored)
            .with_alignment(byte_alignment);

        zip_writer.start_file(&file_name, write_options).map_err(|error| error.to_string())?;
        zip_writer.write_all(&file_data).map_err(|error| error.to_string())?;
    }

    zip_writer.finish().map_err(|error| error.to_string())?;
    Ok(())
}