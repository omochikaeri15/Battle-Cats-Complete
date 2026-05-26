use aes::Aes128;
use cipher::{BlockModeDecrypt, BlockModeEncrypt, KeyInit, KeyIvInit, block_padding::Pkcs7};
use cbc;
use ecb;
use md5;

type Aes128CbcDec = cbc::Decryptor<Aes128>;
type Aes128EcbDec = ecb::Decryptor<Aes128>;
type Aes128CbcEnc = cbc::Encryptor<Aes128>;
type Aes128EcbEnc = ecb::Encryptor<Aes128>;

pub(crate) fn get_md5_key(text: &str) -> [u8; 16] {
    let digest = md5::compute(text.as_bytes());
    let mut key = [0u8; 16];
    let hex_string = hex::encode(&digest.0);
    key.copy_from_slice(&hex_string.as_bytes()[0..16]);
    key
}

pub(crate) fn decrypt_cbc(data: &[u8], key: &[u8; 16], iv: &[u8; 16]) -> Result<Vec<u8>, String> {
    let decryptor = Aes128CbcDec::new(key.into(), iv.into());
    let mut buffer = data.to_vec();
    let decrypted_slice = decryptor
        .decrypt_padded::<Pkcs7>(&mut buffer)
        .map_err(|_| "Padding Error".to_string())?;
    Ok(decrypted_slice.to_vec())
}

pub(crate) fn decrypt_ecb(data: &[u8], key: &[u8; 16]) -> Result<Vec<u8>, String> {
    let decryptor = Aes128EcbDec::new(key.into());
    let mut buffer = data.to_vec();
    let decrypted_slice = decryptor
        .decrypt_padded::<Pkcs7>(&mut buffer)
        .map_err(|_| "Padding Error".to_string())?;
    Ok(decrypted_slice.to_vec())
}

pub(crate) fn encrypt_cbc(data: &[u8], key: &[u8; 16], iv: &[u8; 16]) -> Result<Vec<u8>, String> {
    let encryptor = Aes128CbcEnc::new(key.into(), iv.into());
    let mut buffer = data.to_vec();
    let pos = buffer.len();
    buffer.resize(pos + 16, 0);

    let encrypted_slice = encryptor
        .encrypt_padded::<Pkcs7>(&mut buffer, pos)
        .map_err(|_| "CBC Encryption Error".to_string())?;

    Ok(encrypted_slice.to_vec())
}

pub(crate) fn encrypt_ecb(data: &[u8], key: &[u8; 16]) -> Result<Vec<u8>, String> {
    let encryptor = Aes128EcbEnc::new(key.into());
    let mut buffer = data.to_vec();
    let pos = buffer.len();
    buffer.resize(pos + 16, 0);

    let encrypted_slice = encryptor
        .encrypt_padded::<Pkcs7>(&mut buffer, pos)
        .map_err(|_| "ECB Encryption Error".to_string())?;

    Ok(encrypted_slice.to_vec())
}