use std::fs;
use std::path::Path;
use std::collections::HashMap;
use crate::cat::paths;
use nyanko::cat::unit::UnitEvolve;

pub fn load(cats_directory: &Path, priority: &[String]) -> HashMap<u32, UnitEvolve> {
    let mut final_map: HashMap<u32, UnitEvolve> = HashMap::new();
    let base_directory = cats_directory.join(paths::DIR_UNIT_EVOLVE);

    let has_content = |data: &[String]| data.iter().any(|s| !s.is_empty());

    // Iterate through fallback languages
    for file_path in crate::global::resolver::get(&base_directory, ["unitevolve.csv"], priority) {
        if let Ok(bytes) = fs::read(&file_path) {

            // THE WAITER HAND-OFF
            if let Ok(parsed_map) = UnitEvolve::parse(&bytes) {

                // ORCHESTRATOR LOGIC: Merge the fallbacks for the UI
                for (cat_id, parsed_evolve) in parsed_map {
                    let entry = final_map.entry(cat_id).or_default();

                    if !has_content(&entry.texts[2]) && has_content(&parsed_evolve.texts[2]) {
                        entry.texts[2] = parsed_evolve.texts[2].clone();
                    }
                    if !has_content(&entry.texts[3]) && has_content(&parsed_evolve.texts[3]) {
                        entry.texts[3] = parsed_evolve.texts[3].clone();
                    }
                }
            }
        }
    }

    final_map
}