use std::fs;
use std::path::Path;
use std::sync::mpsc::Sender;
use std::sync::atomic::{AtomicUsize, Ordering};
use rayon::prelude::*;
use crate::data::utilities::crypto;
use crate::settings::logic::keys::UserKeys;

struct PackEntry {
    name: String,
    offset: usize,
    size: usize,
}

pub fn run(pack_dir: &Path, tx: Sender<String>, user_keys: &UserKeys) -> Result<(), String> {
    let list_path = pack_dir.join("DownloadLocal.list");
    let pack_path = pack_dir.join("DownloadLocal.pack");

    if !list_path.exists() || !pack_path.exists() {
        return Ok(());
    }

    let patch_dir = pack_dir.join("patch");
    fs::create_dir_all(&patch_dir).map_err(|e| e.to_string())?;

    let list_data = fs::read(&list_path).map_err(|e| e.to_string())?;
    let content = decrypt_list_content(&list_data)?;

    let mut entries = Vec::new();
    for line in content.lines() {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() < 3 { continue; }

        entries.push(PackEntry {
            name: parts[0].to_string(),
            offset: parts[1].parse().unwrap_or(0),
            size: parts[2].parse().unwrap_or(0),
        });
    }

    let pack_data = fs::read(&pack_path).map_err(|e| e.to_string())?;

    let extracted_count = AtomicUsize::new(0);
    let failed_count = AtomicUsize::new(0);

    entries.into_par_iter().for_each(|entry| {
        let aligned_size = if entry.size % 16 == 0 { entry.size } else { ((entry.size / 16) + 1) * 16 };

        if entry.offset + aligned_size <= pack_data.len() {
            let chunk = &pack_data[entry.offset .. entry.offset + aligned_size];

            match crypto::decrypt_pack_chunk(chunk, &entry.name, user_keys) {
                Ok((decrypted_bytes, _)) => {
                    let final_data = &decrypted_bytes[..std::cmp::min(entry.size, decrypted_bytes.len())];

                    let out_file = patch_dir.join(&entry.name);
                    if let Some(parent) = out_file.parent() { let _ = fs::create_dir_all(parent); }
                    let _ = fs::write(out_file, final_data);

                    extracted_count.fetch_add(1, Ordering::Relaxed);
                },
                Err(_) => {
                    failed_count.fetch_add(1, Ordering::Relaxed);
                }
            }
        }
    });

    let final_errors = failed_count.load(Ordering::Relaxed);
    if final_errors > 0 {
        let _ = tx.send(format!("Encountered {} errors decrypting pack chunks.", final_errors));
    }

    Ok(())
}

fn decrypt_list_content(data: &[u8]) -> Result<String, String> {
    let pack_key = crypto::get_md5_key("pack");
    if let Ok(bytes) = crypto::decrypt_ecb_with_key(data, &pack_key) {
        if let Ok(s) = String::from_utf8(bytes) { return Ok(s); }
    }

    let bc_key = crypto::get_md5_key("battlecats");
    if let Ok(bytes) = crypto::decrypt_ecb_with_key(data, &bc_key) {
        if let Ok(s) = String::from_utf8(bytes) { return Ok(s); }
    }

    Err("List decryption failed (Invalid keys or corrupted file)".into())
}