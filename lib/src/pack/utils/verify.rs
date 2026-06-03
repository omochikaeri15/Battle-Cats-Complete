/// Verifies the integrity of decrypted data by checking file magic bytes or text encoding.
///
/// # Arguments
/// * `data` - The raw, decrypted byte array.
/// * `filename` - The target filename used to determine the expected file signature.
///
/// # Returns
/// Returns `true` if the data matches the expected format, `false` otherwise.
pub fn check_integrity(data: &[u8], filename: &str) -> bool {
    let lower_name = filename.to_lowercase();

    if lower_name.ends_with(".png") {
        return data.len() >= 4 && data.starts_with(&[0x89, 0x50, 0x4E, 0x47]);
    }

    if lower_name.ends_with(".csv") || lower_name.ends_with(".tsv") || lower_name.ends_with(".list") || lower_name.ends_with(".json")
        || lower_name.ends_with(".maanim") || lower_name.ends_with(".mamodel") || lower_name.ends_with(".imgcut")
    {
        return std::str::from_utf8(data).is_ok();
    }

    true
}