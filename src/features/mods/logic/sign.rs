use anyhow::{Context, Result};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use rasn_pkix::Certificate;
use rsa::pkcs8::{DecodePrivateKey, EncodePublicKey};
use rsa::pkcs1::DecodeRsaPrivateKey;
use rsa::{Pkcs1v15Sign, RsaPublicKey, RsaPrivateKey};
use sha2::{Digest as _, Sha256};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64_STANDARD};
use std::fs::File;
use std::io::{Cursor, Read, Seek, SeekFrom, Write};
use std::path::Path;

// 100-Year Universal Native Signer Key
// Valid until May 2126
const DEBUG_PEM: &str = r#"-----BEGIN PRIVATE KEY-----
MIIEvQIBADANBgkqhkiG9w0BAQEFAASCBKcwggSjAgEAAoIBAQCmBNx3G6wn5h63
9cvUxyul2ik3/a4uBBfmGAccldsdawLzg4X7y4nYvBjNo1KWWnKekIWnDHxULtH3
zwEwRAZPFmNPwKvJ3pwYlUE/RvunAVM3PuLGnAFSmDghE3Sylc02HitS0qrWuW/Z
wWPjLIUkmLD/CQnqA1eZL5io+KOZpYx6+iD9XW9aR2ANdHX/813tnGp0HPelUhBg
tFuDdjAJvuzXhhQWFncvYmD5u2wqDVES3o7PkitCFM9xCrg35bIvpTBvyltca2cw
uNmngx3sldU0G0MqmsCpwRvgun9f3vMtlQ3KXEzN+dYP6oYTOIlpUT9pjwCod9yk
CbtRXcn1AgMBAAECggEAH4K2sai/8Ua9N99gU7+F6lHRFv6AS92dB6Ax4VwUHa5M
/hlNmfAU9t0kvAsuxrjeHniB1aYKBxRn5+gTaqzOob43FVEVihhFemkB3FfFtfoL
aGX4NwgvPBUGOkjuEmNactYhFPRFVsIVl7gcFGdD0iFlHtMBXbhKrRmamR+wNZ4m
+dgOWCocvpCMz5/xtxapEfKL+PouHjOonWLLPET+Ire7k+AprW2z3Ww6eZvkc5OU
FJnOM22aznnloQV8rIfG3ZRF2znQQ5uUS8F7ER+OdAE0i5cAbWGGUQ3JGJFgrTMI
A572fhcz+un4/cqPJoC4fYSiNTgXyZ5vKWiqMN1qgQKBgQDaozQ8KVw7F5iBYl/Y
4ZXsWLUs7TIe2bKWhE+3huTyuPzeGZwQ7T8trxRR4DjCqRpdyGJCzXYFbHPaqfee
INEfoXiDJcfVBLGNPpEC/ahc/lPmB/XOsrLsVQ8+hXV32ohLfa3nE/YZsUtdQnyH
Zj1v1xNfo7zIyu3Wf8hU7omxlQKBgQDCY71pLSZVRzRokvxiMjennQUVZ6xSUKOO
AhvQcGOqhW0TLLl46JoDXEmIjFIxp3mYOAb40TxE3jzJ/hqzzhfGXpw8BlPegCYw
UKpiRMqwZJ9mNsEqiRyf3AJPfQMF3M+0ablgxM/RJZLAgnGeQU6DNKjwbOMNiZEB
WYAobZRe4QKBgGasyCYMomSZ0yPHyA04+0gv7H15stTsFUM8RZeBgNk/6HiA/Fqy
n73bf6ZnryAze89ZAFQw2uD3Kn0g3slizfKVyNuGDY9LEfqrzDvkVYG+ajYXvObh
4sa7t1n8IMs1VFZnYhintiYgrazRQVvwtp9kGJQMd+av7fuSrMi98On1AoGBALKx
Z0wJEiTwiM/c1p8aFKlDIYo0vGcK8962N4Vb23LEpqkqwvDPucx/CKW6gFBe6Nsy
Hc6a4TFZrj3tFfTV7msPS8Wt92khGnntnUMqg7y1MwaOLPICCss1PvZ9L8sy2ci6
K4w2P+e+B3JqNzHITPk17lrdbbdjD2ZTNQl0+iBhAoGAeitNc38UpvYWgmUZ1EJu
cpKtg2aQCvCLImnd7LyTu1sbbg00TFpQacSOEgeAcIWP6HfgtrnTX+OwyA5/yCHG
a89zCRmQCdo7kzdfJfDweN5ztCmgpfdLC+Q2kalcQfINyYBxOf+3UmoNTBlqSeCa
5sXXMkroiS5edT9nN7JoTW4=
-----END PRIVATE KEY-----
-----BEGIN CERTIFICATE-----
MIIDRzCCAi+gAwIBAgIUScYjHBliUxuB5JT9tECieV3ku5cwDQYJKoZIhvcNAQEL
BQAwMjELMAkGA1UEBhMCVVMxDzANBgNVBAoMBk9tb2NoaTESMBAGA1UEAwwJQkND
IERlYnVnMCAXDTI2MDUxNzIwNTAyMVoYDzIxMjYwNDIzMjA1MDIxWjAyMQswCQYD
VQQGEwJVUzEPMA0GA1UECgwGT21vY2hpMRIwEAYDVQQDDAlCQ0MgRGVidWcwggEi
MA0GCSqGSIb3DQEBAQUAA4IBDwAwggEKAoIBAQCmBNx3G6wn5h639cvUxyul2ik3
/a4uBBfmGAccldsdawLzg4X7y4nYvBjNo1KWWnKekIWnDHxULtH3zwEwRAZPFmNP
wKvJ3pwYlUE/RvunAVM3PuLGnAFSmDghE3Sylc02HitS0qrWuW/ZwWPjLIUkmLD/
CQnqA1eZL5io+KOZpYx6+iD9XW9aR2ANdHX/813tnGp0HPelUhBgtFuDdjAJvuzX
hhQWFncvYmD5u2wqDVES3o7PkitCFM9xCrg35bIvpTBvyltca2cwuNmngx3sldU0
G0MqmsCpwRvgun9f3vMtlQ3KXEzN+dYP6oYTOIlpUT9pjwCod9ykCbtRXcn1AgMB
AAGjUzBRMB0GA1UdDgQWBBQ1jNEP84Ahqea+IGcTsLsrmKvIYDAfBgNVHSMEGDAW
gBQ1jNEP84Ahqea+IGcTsLsrmKvIYDAPBgNVHRMBAf8EBTADAQH/MA0GCSqGSIb3
DQEBCwUAA4IBAQClgShVAxP5eeCgNvgOySVOFXDNhLRHKWWGOPNkVxb2j5nCMO+y
6LGsHdH1a/a9YsLyQ/08Prb6Q15cVZ3RwzwTCCnSote43i7hDhCWHrxLSTccCWl3
uosSA7VXy943j7l/goKhIkV01Vuful2/PkPCfh6u+yZ66fZe0E56TXY7Ei9znBfk
vna+hVemUkD1ezLTGjoT56Zd63zVF1YI66r37jZ1uEGpKeuFeG9ATgTce6rzWtWg
R8lCToYI1d9YTN3UwkzWp1Id0b6DLMrKznir6uiWsiOKc9s4fMILOK0ehSlZ6V6H
0JkeMoqTC9BNIOYSCKyFcUmGZ1YUhU8Mf4Si
-----END CERTIFICATE-----"#;

const APK_SIGNING_BLOCK_MAGIC: &[u8] = b"APK Sig Block 42";
const APK_SIGNING_BLOCK_V2_ID: u32 = 0x7109871a;
const RSA_PKCS1V15_SHA2_256: u32 = 0x0103;
const MAX_CHUNK_SIZE: usize = 1024 * 1024;

pub struct ZipInfo {
    pub central_directory_start: u64,
    pub eocd_start: u64,
}

impl ZipInfo {
    pub fn new<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        let mut eocd_magic = [0u8; 4];
        let file_length = reader.seek(SeekFrom::End(0))?;

        let mut search_position = file_length.saturating_sub(22);
        let mut magic_found = false;

        while search_position > 0 && search_position >= file_length.saturating_sub(0xFFFF + 22) {
            reader.seek(SeekFrom::Start(search_position))?;
            reader.read_exact(&mut eocd_magic)?;
            if eocd_magic == [0x50, 0x4b, 0x05, 0x06] {
                magic_found = true;
                break;
            }
            search_position -= 1;
        }

        anyhow::ensure!(magic_found, "End of Central Directory (EOCD) not found. Is this a valid ZIP?");

        reader.seek(SeekFrom::Start(search_position + 16))?;
        let central_directory_start = reader.read_u32::<LittleEndian>()? as u64;

        Ok(ZipInfo {
            central_directory_start,
            eocd_start: search_position,
        })
    }
}

pub struct Signer {
    private_key: RsaPrivateKey,
    public_key: RsaPublicKey,
    certificate_der: Certificate,
}

impl Signer {
    pub fn new(pem_string: &str) -> Result<Self> {
        let cert_start_tag = "-----BEGIN CERTIFICATE-----";
        let cert_end_tag = "-----END CERTIFICATE-----";

        let cert_start_idx = pem_string.find(cert_start_tag).context("No BEGIN CERTIFICATE tag found in PEM")?;
        let cert_end_idx = pem_string.find(cert_end_tag).context("No END CERTIFICATE tag found in PEM")?;

        // Extract ONLY the private key portion before the certificate
        let priv_key_str = &pem_string[..cert_start_idx].trim();

        let private_key = RsaPrivateKey::from_pkcs8_pem(priv_key_str)
            .or_else(|_| RsaPrivateKey::from_pkcs1_pem(priv_key_str))
            .context("Failed to parse RSA Private Key from PEM. Ensure your debug.pem contains a valid PKCS#8 or PKCS#1 RSA key.")?;

        let public_key = private_key.to_public_key();

        let base64_cert = &pem_string[cert_start_idx + cert_start_tag.len()..cert_end_idx]
            .replace('\n', "")
            .replace('\r', "");

        let raw_der_bytes = BASE64_STANDARD.decode(base64_cert).context("Failed to base64 decode certificate")?;
        let certificate_der = rasn::der::decode::<Certificate>(&raw_der_bytes)
            .map_err(|err| anyhow::anyhow!("Failed to parse ASN.1 Certificate: {}", err))?;

        Ok(Self {
            private_key,
            public_key,
            certificate_der,
        })
    }

    pub fn cert(&self) -> &Certificate {
        &self.certificate_der
    }

    pub fn pubkey(&self) -> &RsaPublicKey {
        &self.public_key
    }

    pub fn sign(&self, data: &[u8]) -> Vec<u8> {
        let digest = Sha256::digest(data);
        let padding = Pkcs1v15Sign::new::<Sha256>();
        self.private_key.sign(padding, &digest).expect("RSA signing failed")
    }
}

pub fn sign(apk_path: &Path, custom_signer: Option<Signer>) -> Result<()> {
    let identity = custom_signer.map(Ok).unwrap_or_else(|| Signer::new(DEBUG_PEM))?;

    let apk_bytes = std::fs::read(apk_path)?;
    let mut reader = Cursor::new(&apk_bytes);

    let block_info = parse_apk_signing_block(&mut reader)?;
    let zip_hash = compute_digest(
        &mut reader,
        block_info.signing_block_start,
        block_info.central_directory_start,
        block_info.eocd_start
    )?;

    let mut new_signature_block = vec![];
    let mut writer = Cursor::new(&mut new_signature_block);
    write_apk_signing_block(&mut writer, zip_hash, &identity)?;

    let mut output_file = File::create(apk_path)?;

    // Contents before signing block
    output_file.write_all(&apk_bytes[..(block_info.signing_block_start as usize)])?;

    // v2 signing block
    output_file.write_all(&new_signature_block)?;
    let new_cd_start_offset = output_file.stream_position()?;

    // Central Directory
    output_file.write_all(&apk_bytes[(block_info.central_directory_start as usize)..(block_info.eocd_start as usize)])?;
    let new_eocd_offset = output_file.stream_position()?;

    // End of Central Directory
    output_file.write_all(&apk_bytes[(block_info.eocd_start as usize)..])?;

    // Update EOCD pointer to the new Central Directory offset
    output_file.seek(SeekFrom::Start(new_eocd_offset + 16))?;
    output_file.write_u32::<LittleEndian>(new_cd_start_offset as u32)?;

    Ok(())
}

fn compute_digest<R: Read + Seek>(
    reader: &mut R,
    signing_block_start: u64,
    central_directory_start: u64,
    eocd_start: u64,
) -> Result<[u8; 32]> {
    let mut hash_chunks = vec![];
    let mut hasher = Sha256::new();
    let mut buffer_chunk = vec![0u8; MAX_CHUNK_SIZE];

    // Contents
    reader.rewind()?;
    let mut current_position = 0;
    while current_position < signing_block_start {
        hash_chunk(&mut hash_chunks, reader, signing_block_start, &mut hasher, &mut buffer_chunk, &mut current_position)?;
    }

    // Central Directory
    current_position = reader.seek(SeekFrom::Start(central_directory_start))?;
    while current_position < eocd_start {
        hash_chunk(&mut hash_chunks, reader, eocd_start, &mut hasher, &mut buffer_chunk, &mut current_position)?;
    }

    // EOCD (with adjusted CD offset)
    buffer_chunk.clear();
    reader.read_to_end(&mut buffer_chunk)?;

    let mut eocd_cursor = Cursor::new(&mut buffer_chunk);
    eocd_cursor.seek(SeekFrom::Start(16))?;
    eocd_cursor.write_u32::<LittleEndian>(signing_block_start as u32)?;

    hasher.update([0xa5]);
    assert!(buffer_chunk.len() <= MAX_CHUNK_SIZE);
    hasher.update((buffer_chunk.len() as u32).to_le_bytes());
    hasher.update(buffer_chunk);
    hash_chunks.push(hasher.finalize_reset().into());

    // Final Top-Level Hash
    hasher.update([0x5a]);
    hasher.update((hash_chunks.len() as u32).to_le_bytes());
    for chunk in &hash_chunks {
        hasher.update(chunk);
    }

    Ok(hasher.finalize().into())
}

fn hash_chunk<R: Read + Seek>(
    chunks: &mut Vec<[u8; 32]>,
    reader: &mut R,
    target_size: u64,
    hasher: &mut Sha256,
    buffer: &mut Vec<u8>,
    position: &mut u64,
) -> Result<()> {
    let end_position = std::cmp::min(*position + MAX_CHUNK_SIZE as u64, target_size);
    let chunk_length = (end_position - *position) as usize;

    buffer.resize(chunk_length, 0);
    reader.read_exact(buffer).unwrap();

    hasher.update([0xa5]);
    hasher.update((chunk_length as u32).to_le_bytes());
    hasher.update(buffer);

    chunks.push(hasher.finalize_reset().into());
    *position = end_position;

    Ok(())
}

#[derive(Debug, Default)]
struct Digest {
    pub algorithm: u32,
    pub digest: Vec<u8>,
}

impl Digest {
    fn new(hash: [u8; 32]) -> Self {
        Self {
            algorithm: RSA_PKCS1V15_SHA2_256,
            digest: hash.to_vec(),
        }
    }

    fn size(&self) -> u32 {
        self.digest.len() as u32 + 12
    }

    fn write(&self, writer: &mut impl Write) -> Result<()> {
        writer.write_u32::<LittleEndian>(self.digest.len() as u32 + 8)?;
        writer.write_u32::<LittleEndian>(self.algorithm)?;
        writer.write_u32::<LittleEndian>(self.digest.len() as u32)?;
        writer.write_all(&self.digest)?;
        Ok(())
    }
}

#[derive(Debug, Default)]
struct SignedData {
    pub digests: Vec<Digest>,
    pub certificates: Vec<Vec<u8>>,
    pub additional_attributes: Vec<(u32, Vec<u8>)>,
}

impl SignedData {
    fn new(hash: [u8; 32], signer: &Signer) -> Result<Self> {
        Ok(Self {
            digests: vec![Digest::new(hash)],
            certificates: vec![
                rasn::der::encode(signer.cert()).map_err(|err| anyhow::anyhow!("{}", err))?
            ],
            additional_attributes: vec![],
        })
    }

    fn write(&self, writer: &mut impl Write) -> Result<()> {
        writer.write_u32::<LittleEndian>(self.digests.iter().map(|d| d.size()).sum())?;
        for digest in &self.digests { digest.write(writer)?; }

        writer.write_u32::<LittleEndian>(self.certificates.iter().map(|c| c.len() as u32 + 4).sum())?;
        for cert in &self.certificates {
            writer.write_u32::<LittleEndian>(cert.len() as u32)?;
            writer.write_all(cert)?;
        }

        writer.write_u32::<LittleEndian>(self.additional_attributes.iter().map(|(_, v)| v.len() as u32 + 8).sum())?;
        for (id, value) in &self.additional_attributes {
            writer.write_u32::<LittleEndian>(value.len() as u32 + 4)?;
            writer.write_u32::<LittleEndian>(*id)?;
            writer.write_all(value)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
struct ApkSignatureBlockV2 {
    pub signers: Vec<ApkSigner>,
}

#[derive(Debug)]
struct ApkSigner {
    pub signed_data: Vec<u8>,
    pub signatures: Vec<ApkSignature>,
    pub public_key: Vec<u8>,
}

#[derive(Debug)]
struct ApkSignature {
    pub algorithm: u32,
    pub signature: Vec<u8>,
}

impl ApkSignatureBlockV2 {
    fn new(hash: [u8; 32], signer: &Signer) -> Result<Self> {
        let mut signed_data = vec![];
        SignedData::new(hash, signer)?.write(&mut signed_data)?;
        let signature = signer.sign(&signed_data);

        Ok(Self {
            signers: vec![ApkSigner {
                signed_data,
                signatures: vec![ApkSignature {
                    algorithm: RSA_PKCS1V15_SHA2_256,
                    signature,
                }],
                public_key: signer.pubkey().to_public_key_der()?.as_ref().to_vec(),
            }],
        })
    }

    fn write(&self, writer: &mut impl Write) -> Result<()> {
        let mut buffer = vec![];
        for signer in &self.signers {
            let mut signer_buffer = vec![];
            signer_buffer.write_u32::<LittleEndian>(signer.signed_data.len() as u32)?;
            signer_buffer.write_all(&signer.signed_data)?;

            let mut sig_buffer = vec![];
            for sig in &signer.signatures {
                sig_buffer.write_u32::<LittleEndian>(sig.signature.len() as u32 + 8)?;
                sig_buffer.write_u32::<LittleEndian>(sig.algorithm)?;
                sig_buffer.write_u32::<LittleEndian>(sig.signature.len() as u32)?;
                sig_buffer.write_all(&sig.signature)?;
            }
            signer_buffer.write_u32::<LittleEndian>(sig_buffer.len() as u32)?;
            signer_buffer.write_all(&sig_buffer)?;

            signer_buffer.write_u32::<LittleEndian>(signer.public_key.len() as u32)?;
            signer_buffer.write_all(&signer.public_key)?;

            buffer.write_u32::<LittleEndian>(signer_buffer.len() as u32)?;
            buffer.write_all(&signer_buffer)?;
        }
        writer.write_u32::<LittleEndian>(buffer.len() as u32)?;
        writer.write_all(&buffer)?;
        Ok(())
    }
}

#[derive(Debug, Default)]
struct ApkSignatureBlock {
    pub signing_block_start: u64,
    pub central_directory_start: u64,
    pub eocd_start: u64,
}

fn write_apk_signing_block<W: Write + Seek>(
    writer: &mut W,
    hash: [u8; 32],
    signer: &Signer,
) -> Result<()> {
    let mut buffer = vec![];
    ApkSignatureBlockV2::new(hash, signer)?.write(&mut buffer)?;

    let size = buffer.len() as u64 + 36;
    writer.write_u64::<LittleEndian>(size)?;
    writer.write_u64::<LittleEndian>(buffer.len() as u64 + 4)?;
    writer.write_u32::<LittleEndian>(APK_SIGNING_BLOCK_V2_ID)?;
    writer.write_all(&buffer)?;
    writer.write_u64::<LittleEndian>(size)?;
    writer.write_all(APK_SIGNING_BLOCK_MAGIC)?;

    Ok(())
}

fn parse_apk_signing_block<R: Read + Seek>(reader: &mut R) -> Result<ApkSignatureBlock> {
    let zip_info = ZipInfo::new(reader)?;
    let mut block = ApkSignatureBlock {
        eocd_start: zip_info.eocd_start,
        central_directory_start: zip_info.central_directory_start,
        ..Default::default()
    };

    reader.seek(SeekFrom::Start(block.central_directory_start - 16 - 8))?;
    let remaining_size = reader.read_u64::<LittleEndian>()?;
    let mut magic = [0; 16];
    reader.read_exact(&mut magic)?;

    if magic != APK_SIGNING_BLOCK_MAGIC {
        block.signing_block_start = block.central_directory_start;
        return Ok(block);
    }

    let current_position = reader.seek(SeekFrom::Current(-(remaining_size as i64)))?;
    block.signing_block_start = current_position - 8;

    Ok(block)
}