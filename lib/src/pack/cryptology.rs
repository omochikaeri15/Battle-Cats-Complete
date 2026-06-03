use crate::pack::utils::ciphers::{
    decrypt_cbc, decrypt_ecb,
    encrypt_cbc, encrypt_ecb,
    get_md5_key
};
pub use crate::pack::utils::verify::check_integrity;
use std::fmt;
use std::error::Error;

/// Represents failures that can occur during pack cryptographic operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackError {
    /// The provided string contains invalid hexadecimal characters.
    InvalidHexFormat,
    /// The decoded hexadecimal string does not equal exactly 16 bytes.
    InvalidKeyLength,
    /// Required cryptographic keys or initialization vectors were not provided.
    MissingCipherParameters,
    /// Block cipher padding validation or AES decryption failure.
    DecryptionFailed,
    /// Block cipher padding application or AES encryption failure.
    EncryptionFailed,
    /// List manifest decryption failed due to invalid keys or corrupted data.
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

/// Represents the regional version of The Battle Cats.
///
/// Cryptographic keys differ based on the target game region. This enum identifies
/// which key successfully decrypted a file, allowing downstream orchestrators to track active regions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Region {
    /// English / Global version (`en`)
    En,
    /// Japanese version (`jp`)
    Jp,
    /// Korean version (`kr`)
    Kr,
    /// Taiwanese version (`tw`)
    Tw,
}

/// Represents the encryption strategy required for a specific type of pack file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackType {
    /// Standard game data requiring regional CBC encryption.
    Standard,
    /// Server data requiring global ECB encryption.
    Server,
    /// Image data requiring no encryption.
    ImageData,
}

/// A parsed 128-bit AES cipher key and initialization vector for a specific region.
#[derive(Clone)]
pub struct RegionalCipher {
    /// The region associated with this cipher.
    pub region: Region,
    /// The 16-byte AES decryption key.
    pub key: [u8; 16],
    /// The 16-byte initialization vector.
    pub iv: [u8; 16],
}

/// An in-memory keystore for `.pack` decryption.
///
/// Holds pre-parsed, hardware-ready byte arrays to prevent redundant
/// hex decoding and memory allocations during massive bulk decryption operations.
#[derive(Default)]
pub struct Keys {
    /// The collection of verified regional ciphers.
    pub ciphers: Vec<RegionalCipher>,
}

impl Keys {
    /// Initializes an empty keystore.
    pub fn new() -> Self {
        Self { ciphers: Vec::new() }
    }

    /// Parses a collection of raw hex strings into a fully verified, immutable keystore.
    ///
    /// # Arguments
    /// * `tuples` - A slice of tuples containing `(Region, Key Hex, IV Hex)`.
    ///
    /// # Returns
    /// Returns the initialized keystore, or a `PackError` if any hex string is invalid or incorrectly sized.
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

/// Attempts to decrypt a raw chunk of pack data using a brute-force key resolution strategy.
///
/// Tests the provided regional keys against the cipher. If a regional key succeeds, it verifies
/// the decrypted output against known file magic bytes. If all regional keys fail, it automatically
/// falls back to testing the global server keys.
///
/// # Arguments
/// * `data` - The raw, encrypted byte array extracted directly from the pack.
/// * `internal_filename` - The target name of the file, used internally to verify the decrypted content type.
/// * `keys` - A `Keys` keystore containing the pre-parsed regional ciphers.
///
/// # Returns
/// A tuple containing the resulting bytes and an `Option<Region>`. The `Region` indicates which
/// regional key succeeded. If the global server key was used, or if no encryption was detected, the region will be `None`.
pub fn decrypt_chunk(
    data: &[u8],
    internal_filename: &str,
    keys: &Keys
) -> (Vec<u8>, Option<Region>) {

    for cipher in &keys.ciphers {
        if let Ok(result) = decrypt_cbc(data, &cipher.key, &cipher.iv) {
            if check_integrity(&result, internal_filename) {
                return (result, Some(cipher.region));
            }
        }
    }

    let server_key = get_md5_key("battlecats");
    if let Ok(result) = decrypt_ecb(data, &server_key) {
        if check_integrity(&result, internal_filename) {
            return (result, None);
        }
    }

    (data.to_vec(), None)
}

/// Encrypts a raw chunk of game data based on its designated `PackType`.
///
/// # Arguments
/// * `data` - The raw, plaintext byte array of the file to be encrypted.
/// * `pack_type` - A strict enum defining the target encryption strategy.
/// * `key` - An optional 16-byte array representing the specific regional cipher key.
/// * `iv` - An optional 16-byte array representing the specific regional initialization vector.
///
/// # Returns
/// Returns the encrypted byte array, or a `PackError` if AES parameters are missing or padding fails.
pub fn encrypt_chunk(
    data: &[u8],
    pack_type: PackType,
    key: Option<&[u8; 16]>,
    iv: Option<&[u8; 16]>
) -> Result<Vec<u8>, PackError> {
    match pack_type {
        PackType::ImageData => Ok(data.to_vec()),
        PackType::Server => {
            let server_key = get_md5_key("battlecats");
            encrypt_ecb(data, &server_key)
        },
        PackType::Standard => {
            let (Some(cipher_key), Some(cipher_iv)) = (key, iv) else {
                return Err(PackError::MissingCipherParameters);
            };
            encrypt_cbc(data, cipher_key, cipher_iv)
        }
    }
}

/// Decrypts a `.list` file containing pack manifest data.
///
/// List files act as the table of contents for `.pack` files and utilize specific
/// ECB encryption patterns distinct from standard asset CBC encryption.
///
/// # Arguments
/// * `data` - The raw, encrypted bytes of the `.list` file.
///
/// # Returns
/// Returns the decrypted manifest as a valid UTF-8 `String`, or a `PackError` if decryption fails.
pub fn decrypt_list(data: &[u8]) -> Result<String, PackError> {
    let pack_key = get_md5_key("pack");
    if let Ok(bytes) = decrypt_ecb(data, &pack_key) {
        if let Ok(s) = String::from_utf8(bytes) { return Ok(s); }
    }

    let bc_key = get_md5_key("battlecats");
    if let Ok(bytes) = decrypt_ecb(data, &bc_key) {
        if let Ok(s) = String::from_utf8(bytes) { return Ok(s); }
    }

    Err(PackError::ListDecryptionFailed)
}

/// Encrypts a string into a `.list` file format suitable for game engine packing.
///
/// # Arguments
/// * `data` - The raw, plaintext manifest string containing file entries, offsets, and sizes.
///
/// # Returns
/// Returns the fully padded and ECB-encrypted byte array.
pub fn encrypt_list(data: &str) -> Result<Vec<u8>, PackError> {
    let pack_key = get_md5_key("pack");
    encrypt_ecb(data.as_bytes(), &pack_key)
}