pub fn is_valid(data: &[u8], filename: &str) -> bool {
    let lower_name = filename.to_lowercase();
    if lower_name.ends_with(".png") {
        return data.len() >= 4 && data.starts_with(&[0x89, 0x50, 0x4E, 0x47]);
    }
    if lower_name.ends_with(".csv") || lower_name.ends_with(".list") || lower_name.ends_with(".json")
        || lower_name.ends_with(".maanim") || lower_name.ends_with(".mamodel") || lower_name.ends_with(".imgcut")
    {
        return std::str::from_utf8(data).is_ok();
    }
    true
}