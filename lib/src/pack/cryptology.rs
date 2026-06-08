use crate::pack::utils::ciphers::{
    decrypt_cbc, decrypt_ecb,
    encrypt_cbc, encrypt_ecb,
    get_md5_key
};
pub use crate::pack::utils::verify::check_integrity;
use std::fmt;
use std::error::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackError {
    InvalidHexFormat,
    InvalidKeyLength,
    MissingCipherParameters,
    DecryptionFailed,
    EncryptionFailed,
    ListDecryptionFailed,
}

impl fmt::Display for PackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidHexFormat => write!(f, "Invalid hexadecimal format"),
            Self::InvalidKeyLength => write!(f, "Decoded key or IV must be exactly 16 bytes"),
            Self::MissingCipherParameters => write!(f, "Required key and IV parameters were not provided"),
            Self::DecryptionFailed => write!(f, "AES decryption or padding validation failed"),
            Self::EncryptionFailed => write!(f, "AES encryption or padding application failed"),
            Self::ListDecryptionFailed => write!(f, "List manifest decryption failed (Invalid keys or corrupted file)"),
        }
    }
}

impl Error for PackError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Region { En, Jp, Kr, Tw }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackType { Standard, Server, ImageData }

#[derive(Clone)]
pub struct RegionalCipher {
    pub region: Region,
    pub key: [u8; 16],
    pub iv: [u8; 16],
}

#[derive(Default)]
pub struct Keys {
    pub ciphers: Vec<RegionalCipher>,
}

impl Keys {
    /// Parses a collection of raw hexadecimal key and initialization vector (IV) strings
    /// into a structured `Keys` instance.
    ///
    /// This function decodes the hexadecimal strings and strictly validates that
    /// the resulting byte arrays conform to the required 16-byte length for AES-128.
    ///
    /// # Arguments
    /// * `tuples` - A slice of tuples containing the `Region`, the hex-encoded key string, and the hex-encoded IV string.
    ///
    /// # Returns
    /// A `Result` containing the populated `Keys` instance on success, or a `PackError`
    /// if any of the hex strings are invalid or improperly sized.
    pub fn parse(tuples: &[(Region, &str, &str)]) -> Result<Self, PackError> {
        let mut ciphers = Vec::with_capacity(tuples.len());
        for (region, hex_key, hex_iv) in tuples {
            ciphers.push(Self::parse_cipher(*region, hex_key, hex_iv)?);
        }
        Ok(Self { ciphers })
    }

    fn parse_cipher(region: Region, hex_key: &str, hex_iv: &str) -> Result<RegionalCipher, PackError> {
        let key_bytes = hex::decode(hex_key).map_err(|_| PackError::InvalidHexFormat)?;
        let iv_bytes = hex::decode(hex_iv).map_err(|_| PackError::InvalidHexFormat)?;

        let key: [u8; 16] = key_bytes.try_into().map_err(|_| PackError::InvalidKeyLength)?;
        let iv: [u8; 16] = iv_bytes.try_into().map_err(|_| PackError::InvalidKeyLength)?;

        Ok(RegionalCipher { region, key, iv })
    }
}

/// Attempts to decrypt a data chunk by iterating through a set of regional ciphers
/// and falling back to a hardcoded server key if necessary.
///
/// The function verifies a successful decryption by checking the integrity of the
/// output against the expected internal filename. If all decryption attempts fail
/// the integrity check, it assumes the data is unencrypted and returns it as-is.
///
/// # Arguments
/// * `data` - A byte slice containing the encrypted raw chunk data.
/// * `internal_filename` - The name of the file expected within the chunk, used to verify the integrity of the decrypted data.
/// * `keys` - A reference to a `Keys` struct containing the available regional ciphers.
///
/// # Returns
/// A tuple containing the processed byte vector and an `Option<Region>`. The region is `Some`
/// if a regional CBC cipher succeeded, `None` if the ECB server cipher succeeded, or `None`
/// if it fell back to returning the unencrypted raw data.
pub fn decrypt_chunk(data: &[u8], internal_filename: &str, keys: &Keys) -> (Vec<u8>, Option<Region>) {
    for cipher in &keys.ciphers {
        if let Ok(result) = decrypt_cbc(data, &cipher.key, &cipher.iv)
            && check_integrity(&result, internal_filename) {
            return (result, Some(cipher.region));
        }
    }

    let server_key = get_md5_key("battlecats");
    if let Ok(result) = decrypt_ecb(data, &server_key)
        && check_integrity(&result, internal_filename) {
        return (result, None);
    }

    (data.to_vec(), None)
}

/// Encrypts a raw data chunk based on the specified pack classification.
///
/// This handles pass-through for image data, ECB encryption for server packs using
/// a derived MD5 key, and CBC encryption for standard regional packs which require
/// explicitly provided key and IV parameters.
///
/// # Arguments
/// * `data` - A byte slice containing the unencrypted raw chunk data.
/// * `pack_type` - The `PackType` determining the specific encryption strategy (Standard, Server, or ImageData).
/// * `key` - An optional reference to a 16-byte array used as the AES encryption key for `Standard` packs.
/// * `iv` - An optional reference to a 16-byte array used as the initialization vector for `Standard` packs.
///
/// # Returns
/// A `Result` containing the encrypted byte vector on success, or a `PackError` if
/// required cipher parameters are missing or the encryption engine fails.
pub fn encrypt_chunk(data: &[u8], pack_type: PackType, key: Option<&[u8; 16]>, iv: Option<&[u8; 16]>) -> Result<Vec<u8>, PackError> {
    match pack_type {
        PackType::ImageData => Ok(data.to_vec()),
        PackType::Server => encrypt_ecb(data, &get_md5_key("battlecats")),
        PackType::Standard => {
            let (Some(cipher_key), Some(cipher_iv)) = (key, iv) else {
                return Err(PackError::MissingCipherParameters);
            };
            encrypt_cbc(data, cipher_key, cipher_iv)
        }
    }
}

/// Decrypts a list manifest file using predefined ECB keys.
///
/// The function attempts decryption with a standard "pack" key first. If that fails
/// or yields invalid UTF-8 data, it falls back to a secondary "battlecats" key before
/// failing entirely.
///
/// # Arguments
/// * `data` - A byte slice containing the encrypted manifest list data.
///
/// # Returns
/// A `Result` containing the decrypted manifest as a UTF-8 `String` on success, or a
/// `PackError::ListDecryptionFailed` if all decryption attempts fail or produce invalid text.
pub fn decrypt_list(data: &[u8]) -> Result<String, PackError> {
    let pack_key = get_md5_key("pack");
    if let Ok(bytes) = decrypt_ecb(data, &pack_key)
        && let Ok(manifest_text) = String::from_utf8(bytes) { return Ok(manifest_text); }

    let bc_key = get_md5_key("battlecats");
    if let Ok(bytes) = decrypt_ecb(data, &bc_key)
        && let Ok(manifest_text) = String::from_utf8(bytes) { return Ok(manifest_text); }

    Err(PackError::ListDecryptionFailed)
}

/// Encrypts a string-based list manifest into raw bytes.
///
/// The manifest is encoded using the standard "pack" ECB cipher configuration
/// expected by the game client when parsing list files.
///
/// # Arguments
/// * `data` - A string slice representing the unencrypted manifest content.
///
/// # Returns
/// A `Result` containing the encrypted byte vector on success, or a `PackError`
/// if the underlying AES encryption process fails.
pub fn encrypt_list(data: &str) -> Result<Vec<u8>, PackError> {
    encrypt_ecb(data.as_bytes(), &get_md5_key("pack"))
}