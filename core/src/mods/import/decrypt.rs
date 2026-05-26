use std::fs;
use std::path::Path;
use std::sync::mpsc::Sender;
use std::sync::atomic::{AtomicUsize, Ordering};
use rayon::prelude::*;

use nyanko::pack::cryptology;
use nyanko::pack::cryptology::Region as NyankoRegion;
use crate::settings::logic::keys::UserKeys;

struct PackEntry {
    name: String,
    offset: usize,
    size: usize,
}

fn map_keys_to_nyanko(user_keys: &UserKeys) -> cryptology::PackKeys {
    let tuples = user_keys.as_tuples().into_iter().map(|(k, iv, r)| {
        let nyanko_region = match r {
            crate::global::region::Region::En => NyankoRegion::En,
            crate::global::region::Region::Ja => NyankoRegion::Jp,
            crate::global::region::Region::Ko => NyankoRegion::Kr,
            crate::global::region::Region::Tw => NyankoRegion::Tw,
        };
        (k.to_string(), iv.to_string(), nyanko_region)
    }).collect();

    cryptology::PackKeys { tuples }
}

pub fn run(pack_dir: &Path, tx: Sender<String>, user_keys: &UserKeys) -> Result<(), String> {
    // Restored to the game-accurate file names
    let list_path = pack_dir.join("DownloadLocal.list");
    let pack_path = pack_dir.join("DownloadLocal.pack");

    if !list_path.exists() || !pack_path.exists() {
        return Ok(());
    }
    
    let patch_dir = pack_dir.join("patch");
    fs::create_dir_all(&patch_dir).map_err(|e| e.to_string())?;

    let list_data = fs::read(&list_path).map_err(|e| e.to_string())?;
    let content = cryptology::decrypt_list(&list_data)?;

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
    let nyanko_keys = map_keys_to_nyanko(user_keys);

    let extracted_count = AtomicUsize::new(0);
    let failed_count = AtomicUsize::new(0);

    entries.into_par_iter().for_each(|entry| {
        let aligned_size = if entry.size % 16 == 0 { entry.size } else { ((entry.size / 16) + 1) * 16 };

        if entry.offset + aligned_size <= pack_data.len() {
            let chunk = &pack_data[entry.offset .. entry.offset + aligned_size];

            match cryptology::decrypt_pack_chunk(chunk, &entry.name, &nyanko_keys) {
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