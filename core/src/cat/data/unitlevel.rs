use std::fs;
use std::path::Path;
use crate::cat::paths;
use nyanko::cat::unit::LevelCurve;

pub fn load_level_curves(cats_directory: &Path, priority: &[String]) -> Vec<LevelCurve> {
    let mut curves_list = Vec::new();

    let Some(level_file_path) = crate::global::resolver::get(cats_directory, [paths::UNIT_LEVEL], priority).into_iter().next() else {
        return curves_list;
    };

    if let Ok(bytes) = fs::read(&level_file_path)
        && let Ok(parsed_data) = LevelCurve::parse(&bytes) {
            curves_list = parsed_data;
        }

    curves_list
}