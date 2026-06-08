use std::fs;
use std::path::Path;
use crate::cat::paths;
use nyanko::cat::unit::SkillDescriptions;

pub fn load(cats_directory: &Path, priority: &[String]) -> Vec<String> {
    let base_dir = cats_directory.join(paths::DIR_SKILL_DESCRIPTIONS);

    let Some(file_path) = crate::global::resolver::get(&base_dir, ["SkillDescriptions.csv"], priority).into_iter().next() else {
        return Vec::new();
    };

    if let Ok(bytes) = fs::read(&file_path) {
        // THE WAITER HAND-OFF
        if let Ok(parsed_data) = SkillDescriptions::parse(&bytes) {
            return parsed_data.texts;
        }
    }

    Vec::new()
}