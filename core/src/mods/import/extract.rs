use std::fs;
use std::path::Path;
use std::sync::mpsc::Sender;
use zip::ZipArchive;
use crate::mods::import::decrypt;
use crate::settings::logic::keys::UserKeys;

pub fn run_archive(archive_path: &Path, _target_dir: &Path, tx: Sender<String>, user_keys: &UserKeys) -> Result<(), String> {
    let _ = tx.send("Opening archive...".to_string());

    let mods_root = Path::new("mods");
    let mut mod_num = 1;
    while mods_root.join(format!("NewMod{}", mod_num)).exists() {
        mod_num += 1;
    }

    let mod_dir = mods_root.join(format!("NewMod{}", mod_num));

    for dir in [&mod_dir, &mod_dir.join("icons"), &mod_dir.join("patch"), &mod_dir.join("loose")] {
        fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }

    let file = fs::File::open(archive_path).map_err(|e| e.to_string())?;
    let mut archive = ZipArchive::new(file).map_err(|e| e.to_string())?;
    let mut has_pack_data = false;

    for icon in ["icon.png", "icon_foreground.png", "push_icon.png"] {
        for dpi in ["xxxhdpi", "xxhdpi", "xhdpi"] {
            let zip_path = format!("res/drawable-{}/{}", dpi, icon);

            let Ok(mut file_in_zip) = archive.by_name(&zip_path) else { continue };

            if let Ok(mut out_file) = fs::File::create(mod_dir.join("icons").join(icon)) {
                let _ = std::io::copy(&mut file_in_zip, &mut out_file);
            }
            break;
        }
    }

    for i in 0..archive.len() {
        let Ok(mut file_in_zip) = archive.by_index(i) else { continue };
        if file_in_zip.is_dir() { continue; }
        let name = file_in_zip.name().to_string();
        let path = Path::new(&name);
        let Some(safe_name) = path.file_name() else { continue };
        let safe_name_str = safe_name.to_string_lossy();

        let save_path = if name == "assets/DownloadLocal.list" || name == "assets/DownloadLocal.pack" {
            has_pack_data = true;
            Some(mod_dir.join(safe_name))

        } else if safe_name_str == "metadata.json" {
            Some(mod_dir.join("patch").join(safe_name))

        } else if path.parent() == Some(Path::new("assets")) {
            let is_download_tsv = safe_name_str.starts_with("download") && safe_name_str.ends_with(".tsv");
            let is_junk = safe_name_str.ends_with(".dat")
                || safe_name_str.ends_with(".lock")
                || safe_name_str.ends_with(".pack")
                || safe_name_str.ends_with(".list")
                || safe_name_str.ends_with(".dex");

            if !is_download_tsv && !is_junk {
                Some(mod_dir.join("loose").join(safe_name))
            } else { None }

        } else { None };

        if let Some(out_path) = save_path
            && let Ok(mut out_file) = fs::File::create(&out_path) {
                let _ = std::io::copy(&mut file_in_zip, &mut out_file);
            }
    }

    if has_pack_data && mod_dir.join("DownloadLocal.list").exists() && mod_dir.join("DownloadLocal.pack").exists() {
        let _ = tx.send("Decrypting pack data...".to_string());
        decrypt::run(&mod_dir, tx.clone(), user_keys)?;
        let _ = fs::remove_file(mod_dir.join("DownloadLocal.list"));
        let _ = fs::remove_file(mod_dir.join("DownloadLocal.pack"));
    }

    let mut final_name = mod_dir.file_name().unwrap_or_default().to_string_lossy().into_owned();

    if mod_dir.join("patch").join("metadata.json").exists() {
        let meta = crate::mods::logic::metadata::ModMetadata::load(&mod_dir);
        let safe_title = meta.title.replace(&['<', '>', ':', '"', '/', '\\', '|', '?', '*'][..], "").trim().to_string();

        if !safe_title.is_empty() && safe_title != final_name {
            let mut attempt = safe_title.clone();
            let mut counter = 1;

            while mods_root.join(&attempt).exists() {
                attempt = format!("{}{}", safe_title, counter);
                counter += 1;
            }
            if fs::rename(&mod_dir, mods_root.join(&attempt)).is_ok() {
                final_name = attempt;
            }
        }
    }

    let _ = tx.send(format!("Successfully imported mod as '{}'", final_name));
    Ok(())
}