use std::fs;
use std::path::Path;
use std::collections::HashMap;
use crate::cat::paths;
use nyanko::cat::unit::Talent;

pub fn load(cats_directory: &Path, priority: &[String]) -> HashMap<u16, Talent> {
    let mut map = HashMap::new();

    let Some(file_path) = crate::global::resolver::get(cats_directory, [paths::SKILL_ACQUISITION], priority).into_iter().next() else {
        return map;
    };

    if let Ok(bytes) = fs::read(&file_path)
        && let Ok(parsed_data) = Talent::parse(&bytes) {
            map = parsed_data;
        }

    map
}