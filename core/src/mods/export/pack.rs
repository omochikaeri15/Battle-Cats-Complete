use std::fs::{self, File};
use std::io::{Write, BufWriter};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;

use aes::Aes128;
use cbc::Encryptor as CbcEncryptor;
use ecb::Encryptor as EcbEncryptor;
use block_padding::Pkcs7;
use aes::cipher::{BlockEncryptMut, KeyIvInit, KeyInit};

use crate::mods::logic::state::ModDataState;
use crate::global::region::Region;
use crate::data::utilities::keys;
use crate::settings::logic::state::Settings;
use crate::settings::logic::keys::RegionKey;
use crate::data::utilities::crypto::get_md5_key;
use crate::mods::export::patch::{EVENT_RECEIVER, ExportEvent, spawn_log_adapter};

pub fn start_pack_export(state: &mut ModDataState) {
    if state.export.is_busy { return; }

    let Some(mod_folder) = state.selected_mod.clone() else { return; };

    state.export.log_content.clear();
    state.export.is_busy = true;
    state.export.status_message = "Initializing Pack Export...".to_string();

    let pack_name = if state.export.pack_name.trim().is_empty() { "mod".to_string() } else { state.export.pack_name.clone() };
    let target_region = state.export.target_region.clone();

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
        let mod_path = base_dir.join("mods").join(&mod_folder);
        let export_dir = base_dir.join("exports");

        let patch_dir = mod_path.join("patch");
        let _ = fs::create_dir_all(&export_dir);

        let region_key = match target_region {
            Region::En => &user_keys.en,
            Region::Ja => &user_keys.ja,
            Region::Ko => &user_keys.ko,
            Region::Tw => &user_keys.tw,
        };

        if let Err(error) = stream_pack_and_list(&patch_dir, &export_dir, &pack_name, region_key, &log_callback) {
            let _ = transmitter.send(ExportEvent::Error(error)); return;
        }

        let _ = transmitter.send(ExportEvent::Success(format!("Successfully Built {}.pack!", pack_name)));
    });
}

pub fn stream_pack_and_list(
    source_dir: &Path,
    dest_dir: &Path,
    pack_name: &str,
    region_key: &RegionKey,
    log_callback: &impl Fn(String)
) -> Result<(), String> {

    let mut files_with_size = Vec::new();

    if let Ok(entries) = fs::read_dir(source_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Ok(metadata) = fs::metadata(&path) {
                    files_with_size.push((path, metadata.len() as usize));
                }
            }
        }
    }

    let total_files = files_with_size.len();
    if total_files == 0 {
        return Err("No files found in the patch directory.".to_string());
    }

    let is_imagedata = pack_name.to_lowercase().contains("imagedatalocal");
    let is_server = pack_name.to_lowercase().contains("server");

    log_callback(format!("Found {} files to patch.", total_files));

    let log_interval = (total_files / 10).max(1);

    let pack_path = dest_dir.join(format!("{}.pack", pack_name));
    let list_path = dest_dir.join(format!("{}.list", pack_name));

    let pack_file = File::create(&pack_path).map_err(|error| format!("Failed to create pack stream file: {}", error))?;
    let mut pack_writer = BufWriter::new(pack_file);

    let mut list_string = format!("{}\n", total_files);
    let mut current_address = 0;

    for (index, (file_path, _file_size)) in files_with_size.iter().enumerate() {
        let filename = file_path.file_name().unwrap_or_default().to_string_lossy().to_string();

        if index > 0 && index % log_interval == 0 {
            log_callback(format!("Packed {} files | Streaming: {}", index, filename));
        }

        let mut file_data = fs::read(&file_path).map_err(|error| format!("Failed to read {}: {}", filename, error))?;

        if !is_imagedata {
            if is_server {
                file_data = encrypt_ecb(file_data, &get_md5_key("battlecats"))?;
            } else {
                let key_bytes = hex::decode(&region_key.key).map_err(|_| "Invalid Region Key Hex".to_string())?;
                let iv_bytes = hex::decode(&region_key.iv).map_err(|_| "Invalid Region IV Hex".to_string())?;

                if key_bytes.len() != 16 || iv_bytes.len() != 16 {
                    return Err("Region Key/IV length is incorrect. Check settings.".to_string());
                }

                let key_array: [u8; 16] = key_bytes.try_into().unwrap_or([0; 16]);
                let iv_array: [u8; 16] = iv_bytes.try_into().unwrap_or([0; 16]);

                file_data = encrypt_cbc(file_data, &key_array, &iv_array)?;
            }
        }

        pack_writer.write_all(&file_data).map_err(|error| format!("Failed to write to pack buffer: {}", error))?;

        let new_size = file_data.len();
        list_string.push_str(&format!("{},{},{}\n", filename, current_address, new_size));
        current_address += new_size;
    }

    pack_writer.flush().map_err(|error| format!("Failed to flush pack stream to disk: {}", error))?;

    let list_bytes = encrypt_ecb(list_string.into_bytes(), &get_md5_key("pack"))?;
    fs::write(list_path, list_bytes).map_err(|error| format!("Failed to write list file: {}", error))?;

    Ok(())
}

fn encrypt_cbc(mut buffer: Vec<u8>, key: &[u8; 16], iv: &[u8; 16]) -> Result<Vec<u8>, String> {
    let encryptor = CbcEncryptor::<Aes128>::new(key.into(), iv.into());
    let current_position = buffer.len();
    buffer.resize(current_position + 16, 0);

    let encrypted_length = encryptor
        .encrypt_padded_mut::<Pkcs7>(&mut buffer, current_position)
        .map_err(|_| "CBC Encryption Error".to_string())?
        .len();

    buffer.truncate(encrypted_length);
    Ok(buffer)
}

fn encrypt_ecb(mut buffer: Vec<u8>, key: &[u8; 16]) -> Result<Vec<u8>, String> {
    let encryptor = EcbEncryptor::<Aes128>::new(key.into());
    let current_position = buffer.len();
    buffer.resize(current_position + 16, 0);

    let encrypted_length = encryptor
        .encrypt_padded_mut::<Pkcs7>(&mut buffer, current_position)
        .map_err(|_| "ECB Encryption Error".to_string())?
        .len();

    buffer.truncate(encrypted_length);
    Ok(buffer)
}