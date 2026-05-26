use crate::pack::utils::ciphers::{
    decrypt_cbc, decrypt_ecb,
    encrypt_cbc, encrypt_ecb,
    get_md5_key
};
use crate::pack::utils::verify;

/// Represents the regional version of The Battle Cats.
/// Cryptographic keys differ based on the target game region.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Region {
    /// English / Global
    En,
    /// Japanese
    Jp,
    /// Korean
    Kr,
    /// Taiwanese
    Tw,
}

/// A pure representation of the cryptographic keys required for pack decryption.
pub struct PackKeys {
    /// A list of tuples containing `(Key Hex String, IV Hex String, Region)`.
    pub tuples: Vec<(String, String, Region)>,
}

/// Attempts to decrypt a raw chunk of pack data using a brute-force key resolution strategy.
///
/// This function tests the provided regional keys against the cipher. If successful, it
/// verifies the decrypted output against known file magic bytes (e.g., PNG headers) or valid UTF-8.
/// If regional keys fail, it falls back to testing global server keys.
///
/// # Arguments
/// * `data` - The raw, encrypted byte array.
/// * `internal_filename` - The name of the file, used to verify the decrypted content type.
/// * `keys` - A `PackKeys` struct containing the user's regional keys.
///
/// # Returns
/// A `Result` containing a tuple of the decrypted bytes and an `Option<Region>` indicating
/// which regional key succeeded. Returns `None` for the region if the server key was used.
pub fn decrypt_pack_chunk(
    data: &[u8],
    internal_filename: &str,
    keys: &PackKeys
) -> Result<(Vec<u8>, Option<Region>), String> {
    for (k_hex, iv_hex, region) in &keys.tuples {
        let Ok(key_bytes) = hex::decode(k_hex) else { continue; };
        let Ok(iv_bytes) = hex::decode(iv_hex) else { continue; };
        let (Ok(key_arr), Ok(iv_arr)) = (key_bytes.try_into(), iv_bytes.try_into()) else { continue; };

        if let Ok(result) = decrypt_cbc(data, &key_arr, &iv_arr) {
            if verify::is_valid(&result, internal_filename) {
                return Ok((result, Some(*region)));
            }
        }
    }

    let server_key = get_md5_key("battlecats");
    if let Ok(result) = decrypt_ecb(data, &server_key) {
        if verify::is_valid(&result, internal_filename) {
            return Ok((result, None));
        }
    }

    Ok((data.to_vec(), None))
}

/// Decrypts a `.list` file containing pack manifest data.
///
/// List files use a specific ECB encryption pattern separate from standard game assets.
pub fn decrypt_list(data: &[u8]) -> Result<String, String> {
    let pack_key = get_md5_key("pack");
    if let Ok(bytes) = decrypt_ecb(data, &pack_key) {
        if let Ok(s) = String::from_utf8(bytes) { return Ok(s); }
    }

    let bc_key = get_md5_key("battlecats");
    if let Ok(bytes) = decrypt_ecb(data, &bc_key) {
        if let Ok(s) = String::from_utf8(bytes) { return Ok(s); }
    }

    Err("List decryption failed (Invalid keys or corrupted file)".into())
}

/// Encrypts a string into a `.list` file format suitable for packing.
pub fn encrypt_list(data: &str) -> Result<Vec<u8>, String> {
    let pack_key = get_md5_key("pack");
    encrypt_ecb(data.as_bytes(), &pack_key)
}

/// Encrypts raw data using the global server ECB algorithm.
pub fn encrypt_server_data(data: &[u8]) -> Result<Vec<u8>, String> {
    let server_key = get_md5_key("battlecats");
    encrypt_ecb(data, &server_key)
}

/// Encrypts raw game asset data using a specific regional CBC key and initialization vector.
pub fn encrypt_game_data(data: &[u8], key: &[u8; 16], iv: &[u8; 16]) -> Result<Vec<u8>, String> {
    encrypt_cbc(data, key, iv)
}