/// Verifies the integrity of decrypted data by checking file magic bytes or text encoding.
///
/// # Arguments
/// * `data` - The raw, decrypted byte array.
/// * `filename` - The target filename used to determine the expected file signature.
///
/// # Returns
/// Returns `true` if the data matches the expected format, `false` otherwise.
pub fn check_integrity(data: &[u8], filename: &str) -> bool {
    let ext = filename
        .rsplit_once('.')
        .map(|(_, e)| e.to_lowercase());

    match ext.as_deref() {
        Some("png") => data.starts_with(&[0x89, 0x50, 0x4E, 0x47]),
        Some("csv" | "tsv" | "list" | "json" | "maanim" | "mamodel" | "imgcut") => {
            std::str::from_utf8(data).is_ok()
        }
        _ => true,
    }
}