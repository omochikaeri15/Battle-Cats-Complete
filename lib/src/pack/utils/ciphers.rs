use aes::Aes128;
use cipher::{BlockModeDecrypt, BlockModeEncrypt, KeyInit, KeyIvInit, block_padding::Pkcs7};
use crate::pack::cryptology::PackError;
use cbc;
use ecb;
use md5;

type Aes128CbcDec = cbc::Decryptor<Aes128>;
type Aes128EcbDec = ecb::Decryptor<Aes128>;
type Aes128CbcEnc = cbc::Encryptor<Aes128>;
type Aes128EcbEnc = ecb::Encryptor<Aes128>;

/// Generates a 16-byte MD5 hash from a plaintext string for ECB encryption.
pub fn get_md5_key(text: &str) -> [u8; 16] {
    let digest = md5::compute(text.as_bytes());
    let mut key = [0u8; 16];
    let hex_string = hex::encode(digest.0);
    key.copy_from_slice(&hex_string.as_bytes()[0..16]);
    key
}

/// Performs AES-128 CBC decryption with PKCS7 padding.
pub fn decrypt_cbc(data: &[u8], key: &[u8; 16], iv: &[u8; 16]) -> Result<Vec<u8>, PackError> {
    let decryptor = Aes128CbcDec::new(key.into(), iv.into());
    let mut buffer = data.to_vec();
    let decrypted_slice = decryptor
        .decrypt_padded::<Pkcs7>(&mut buffer)
        .map_err(|_| PackError::DecryptionFailed)?;
    Ok(decrypted_slice.to_vec())
}

/// Performs AES-128 ECB decryption with PKCS7 padding.
pub fn decrypt_ecb(data: &[u8], key: &[u8; 16]) -> Result<Vec<u8>, PackError> {
    let decryptor = Aes128EcbDec::new(key.into());
    let mut buffer = data.to_vec();
    let decrypted_slice = decryptor
        .decrypt_padded::<Pkcs7>(&mut buffer)
        .map_err(|_| PackError::DecryptionFailed)?;
    Ok(decrypted_slice.to_vec())
}

/// Performs AES-128 CBC encryption with PKCS7 padding.
pub fn encrypt_cbc(data: &[u8], key: &[u8; 16], iv: &[u8; 16]) -> Result<Vec<u8>, PackError> {
    let encryptor = Aes128CbcEnc::new(key.into(), iv.into());
    let mut buffer = data.to_vec();
    let pos = buffer.len();
    buffer.resize(pos + 16, 0);

    let encrypted_slice = encryptor
        .encrypt_padded::<Pkcs7>(&mut buffer, pos)
        .map_err(|_| PackError::EncryptionFailed)?;

    Ok(encrypted_slice.to_vec())
}

/// Performs AES-128 ECB encryption with PKCS7 padding.
pub fn encrypt_ecb(data: &[u8], key: &[u8; 16]) -> Result<Vec<u8>, PackError> {
    let encryptor = Aes128EcbEnc::new(key.into());
    let mut buffer = data.to_vec();
    let pos = buffer.len();
    buffer.resize(pos + 16, 0);

    let encrypted_slice = encryptor
        .encrypt_padded::<Pkcs7>(&mut buffer, pos)
        .map_err(|_| PackError::EncryptionFailed)?;

    Ok(encrypted_slice.to_vec())
}