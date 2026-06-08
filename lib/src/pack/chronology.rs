/// Calculates the chronological load weight of a pack file based on its filename.
///
/// The Battle Cats uses a specific naming convention to determine which files overwrite
/// others in memory during extraction or loading. Higher calculated weights indicate
/// that the file should be loaded last.
///
/// # Arguments
/// * `filename` - The raw name of the file, with or without its extension.
/// * `is_update_pack` - A boolean indicating if this file originates from a split install/update APK.
///
/// # Returns
/// A `u64` representing the sorting weight. Higher values take precedence in memory overlays.
pub fn calculate_weight(filename: &str, is_update_pack: bool) -> u64 {
    let stem = filename.rsplit_once('.').map_or(filename, |(name, _ext)| name);

    let mut weight = 5_000;
    let parts: Vec<&str> = stem.split('_').collect();

    if parts.len() >= 3
        && let (Ok(version_major), Ok(version_minor)) = (parts[1].parse::<u64>(), parts[2].parse::<u64>()) {
            weight = 100_000_000 + (version_major * 100) + version_minor;
        }

    if weight == 5_000 && stem.ends_with("Server") {
        let chars: Vec<char> = stem.chars().collect();
        weight = if chars.len() > 1 && chars[0].is_ascii_uppercase() && chars[1].is_ascii_uppercase() {
            20_000 + (chars[0] as u64)
        } else {
            10_000
        };
    }

    if weight == 5_000 && (stem == "DataLocal" || stem == "UpdateLocal" || stem.ends_with("Local")) {
        weight = 0;
    }

    if is_update_pack {
        weight += 500_000_000;
    }

    weight
}