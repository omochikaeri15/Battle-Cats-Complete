use std::fs;
use std::path::Path;
use crate::cat::paths;
use nyanko::cat::unit::UnitExplanation;

pub fn load(cat_id: u32, original_folder_path: &Path, priority: &[String]) -> UnitExplanation {
    let mut final_explanation = UnitExplanation::default();
    let cats_root_dir = Path::new(paths::DIR_CATS);

    let lang_directory = paths::lang(cats_root_dir, cat_id);
    let base_filename = format!("Unit_Explanation{}.csv", cat_id + 1);

    // Build the directory fallback chain
    let mut search_dirs = vec![original_folder_path.to_path_buf()];
    if lang_directory.exists() {
        search_dirs.insert(0, lang_directory);
    }

    // Iterate through the directories
    for dir in search_dirs {
        let resolved_paths = crate::global::resolver::get(&dir, [base_filename.as_str()], priority);

        for file_path in resolved_paths {
            if let Ok(bytes) = fs::read(&file_path) {
                // THE WAITER HAND-OFF
                if let Ok(parsed_explanation) = UnitExplanation::parse(&bytes) {

                    // ORCHESTRATOR LOGIC: Merge the fallback languages for the UI
                    for i in 0..4 {
                        if final_explanation.names[i].is_none() && parsed_explanation.names[i].is_some() {
                            final_explanation.names[i] = parsed_explanation.names[i].clone();
                            final_explanation.descriptions[i] = parsed_explanation.descriptions[i].clone();
                        }
                    }

                }
            }
        }

        // If we found any valid names in this directory's fallback chain, stop searching deeper
        if final_explanation.names.iter().any(|name| name.is_some()) {
            break;
        }
    }

    final_explanation
}