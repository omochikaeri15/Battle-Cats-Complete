use std::fs;
use std::path::Path;
use aes::Aes128;
use cbc::Encryptor as CbcEncryptor;
use ecb::Encryptor as EcbEncryptor;
use block_padding::Pkcs7;
use aes::cipher::{BlockEncryptMut, KeyIvInit, KeyInit};
use crate::features::settings::logic::keys::RegionKey;
use crate::features::data::utilities::crypto::get_md5_key;

pub fn build_pack_and_list(
    source_dir: &Path,
    pack_name: &str,
    region_key: &RegionKey,
    log_cb: &impl Fn(String)
) -> Result<(Vec<u8>, Vec<u8>), String> {

    let mut files_with_size = Vec::new();

    if let Ok(entries) = fs::read_dir(source_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Ok(meta) = fs::metadata(&path) {
                    files_with_size.push((path, meta.len() as usize));
                }
            }
        }
    }

    if files_with_size.is_empty() {
        return Err("No files found in the source directory.".to_string());
    }

    let mut list_string = format!("{}\n", files_with_size.len());
    let mut current_address = 0;
    let mut pack_buffer = Vec::new();

    let is_imagedata = pack_name.to_lowercase().contains("imagedatalocal");
    let is_server = pack_name.to_lowercase().contains("server");

    let total_files = files_with_size.len();
    let log_interval = if total_files > 100 { 50 } else { 10 };
    log_cb(format!("Found {} files to encrypt.", total_files));

    for (index, (file_path, _orig_size)) in files_with_size.iter().enumerate() {
        let filename = file_path.file_name().unwrap_or_default().to_string_lossy().to_string();

        if index % log_interval == 0 || index == total_files - 1 {
            log_cb(format!("Processing {} ({}/{})", filename, index + 1, total_files));
        }

        let mut data = fs::read(&file_path).map_err(|e| format!("Failed to read {}: {}", filename, e))?;

        if !is_imagedata {
            if is_server {
                data = encrypt_ecb(&data, &get_md5_key("battlecats"))?;
            } else {
                let key_bytes = hex::decode(&region_key.key).map_err(|_| "Invalid Region Key Hex".to_string())?;
                let iv_bytes = hex::decode(&region_key.iv).map_err(|_| "Invalid Region IV Hex".to_string())?;

                if key_bytes.len() != 16 || iv_bytes.len() != 16 {
                    return Err("Region Key/IV length is incorrect. Check settings.".to_string());
                }

                let key_arr: [u8; 16] = key_bytes.try_into().unwrap_or([0; 16]);
                let iv_arr: [u8; 16] = iv_bytes.try_into().unwrap_or([0; 16]);

                data = encrypt_cbc(&data, &key_arr, &iv_arr)?;
            }
        }

        let new_size = data.len();
        list_string.push_str(&format!("{},{},{}\n", filename, current_address, new_size));

        pack_buffer.extend(data);
        current_address += new_size;
    }

    log_cb("Encrypting list file...".to_string());
    let list_bytes = encrypt_ecb(list_string.as_bytes(), &get_md5_key("pack"))?;

    Ok((pack_buffer, list_bytes))
}

fn encrypt_cbc(data: &[u8], key: &[u8; 16], iv: &[u8; 16]) -> Result<Vec<u8>, String> {
    let encryptor = CbcEncryptor::<Aes128>::new(key.into(), iv.into());
    let mut buffer = data.to_vec();
    let pos = buffer.len();
    buffer.resize(pos + 16, 0);
    let encrypted_len = encryptor
        .encrypt_padded_mut::<Pkcs7>(&mut buffer, pos)
        .map_err(|_| "CBC Encryption Error".to_string())?
        .len();
    buffer.truncate(encrypted_len);
    Ok(buffer)
}

fn encrypt_ecb(data: &[u8], key: &[u8; 16]) -> Result<Vec<u8>, String> {
    let encryptor = EcbEncryptor::<Aes128>::new(key.into());
    let mut buffer = data.to_vec();
    let pos = buffer.len();
    buffer.resize(pos + 16, 0);
    let encrypted_len = encryptor
        .encrypt_padded_mut::<Pkcs7>(&mut buffer, pos)
        .map_err(|_| "ECB Encryption Error".to_string())?
        .len();
    buffer.truncate(encrypted_len);
    Ok(buffer)
}