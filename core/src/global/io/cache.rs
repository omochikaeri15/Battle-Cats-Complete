use directories::BaseDirs;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fs::{self, File};
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::io::{BufReader, BufWriter};
use rayon::prelude::*;
use rustc_hash::FxHasher;
use bincode::Options;

pub fn get_cache_dir() -> Option<PathBuf> {
    if let Some(base_dirs) = BaseDirs::new() {
        let cache_directory = base_dirs.data_local_dir().join("battle_cats_complete").join("cache");
        if !cache_directory.exists() {
            tracing::debug!("Creating missing cache directory at {:?}", cache_directory);
            let _ = fs::create_dir_all(&cache_directory);
        }
        Some(cache_directory)
    } else {
        tracing::error!("Failed to retrieve BaseDirs. Caching system is disabled.");
        None
    }
}

fn hash_directory_parallel(directory_path: &Path) -> u64 {
    if !directory_path.exists() { return 0; }

    let mut file_entries = Vec::new();
    if let Ok(read_directory) = fs::read_dir(directory_path) {
        for directory_entry in read_directory.flatten() {
            file_entries.push(directory_entry.path());
        }
    }

    let child_hashes: Vec<u64> = file_entries.par_iter().map(|child_path| {
        let mut local_hasher = FxHasher::default();
        if child_path.is_dir() {
            let subdirectory_hash = hash_directory_parallel(child_path);
            subdirectory_hash.hash(&mut local_hasher);
        } else if let Ok(file_metadata) = child_path.metadata()
            && let Ok(modified_time) = file_metadata.modified() {
            modified_time.hash(&mut local_hasher);
        }
        local_hasher.finish()
    }).collect();

    let mut final_hasher = FxHasher::default();
    for child_hash in child_hashes {
        child_hash.hash(&mut final_hasher);
    }
    file_entries.len().hash(&mut final_hasher);
    final_hasher.finish()
}

#[tracing::instrument(level = "debug", skip(active_mod))]
pub fn get_game_hash(active_mod: Option<&str>) -> u64 {
    tracing::trace!("Calculating global game hash across assets and tables...");
    let mut final_game_hasher = FxHasher::default();

    let target_paths = ["game/tables", "game/cats", "game/enemies", "mods"];
    for path_string in target_paths {
        let directory_hash = hash_directory_parallel(Path::new(path_string));
        directory_hash.hash(&mut final_game_hasher);
    }

    if let Some(mod_name) = active_mod {
        tracing::trace!("Including active mod in hash: {}", mod_name);
        mod_name.hash(&mut final_game_hasher);
    } else {
        "vanilla_base_game".hash(&mut final_game_hasher);
    }

    let hash_result = final_game_hasher.finish();
    tracing::debug!("Generated game hash: {}", hash_result);
    hash_result
}

#[derive(Serialize, Deserialize)]
struct CachePayload<T> {
    hash: u64,
    data: T,
}

#[tracing::instrument(level = "debug", skip_all, fields(file = %filename))]
pub fn load_with_hash<T: DeserializeOwned>(filename: &str) -> Option<(u64, T)> {
    let cache_path = get_cache_dir()?.join(filename);

    let cache_file = match File::open(&cache_path) {
        Ok(f) => f,
        Err(_) => {
            tracing::trace!("Cache file {} does not exist", filename);
            return None;
        }
    };

    let reader = BufReader::new(cache_file);

    let options = bincode::DefaultOptions::new()
        .with_limit(1024 * 1024 * 100);

    match options.deserialize_from::<_, CachePayload<T>>(reader) {
        Ok(payload) => {
            tracing::debug!("Successfully loaded cache payload");
            Some((payload.hash, payload.data))
        },
        Err(e) => {
            tracing::warn!("Failed to deserialize cache payload: {}. Purging corrupted cache file.", e);
            let _ = fs::remove_file(&cache_path);
            None
        }
    }
}

#[tracing::instrument(level = "debug", skip(data))]
pub fn save<T: Serialize>(filename: &str, hash: u64, data: &T) {
    if let Some(cache_directory) = get_cache_dir() {
        let target_path = cache_directory.join(filename);
        let tmp_path = target_path.with_extension("tmp");

        if let Ok(cache_file) = File::create(&tmp_path) {
            let mut writer = BufWriter::new(cache_file);
            let payload = CachePayload { hash, data };
            let options = bincode::DefaultOptions::new()
                .with_limit(1024 * 1024 * 100);

            if let Err(e) = options.serialize_into(&mut writer, &payload) {
                tracing::error!("Failed to serialize cache payload: {}", e);
                let _ = fs::remove_file(&tmp_path);
                return;
            }

            if writer.into_inner().is_ok() {
                if let Err(e) = fs::rename(&tmp_path, &target_path) {
                    tracing::error!("Failed to commit cache file rename: {}", e);
                } else {
                    tracing::debug!("Successfully committed cache file to disk");
                }
            } else {
                tracing::error!("Failed to flush cache file writer");
            }
        } else {
            tracing::error!("Failed to create temporary cache file at {:?}", tmp_path);
        }
    }
}