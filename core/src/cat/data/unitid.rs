#![allow(dead_code)]
use std::fs;
use std::path::Path;
use nyanko::cat::unit::Battle;
use crate::cat::paths;

// Keep UI constants in the Orchestrator where they belong
pub const ICON_SIZE: f32 = 40.0;

pub fn load_from_id(cat_id: i32, priority: &[String]) -> Option<Vec<Battle>> {
    let path_object = paths::stats(Path::new(paths::DIR_CATS), cat_id as u32);

    // Flat architecture: early returns to safely eradicate unwrap()
    let base_dir = path_object.parent()?;
    let file_name = path_object.file_name()?.to_str()?;

    let resolved_path = crate::global::resolver::get(base_dir, &[file_name], priority).into_iter().next()?;

    // Read bytes from disk
    let bytes = fs::read(resolved_path).ok()?;

    // THE WAITER HAND-OFF: Pass raw bytes to the nyanko engine, convert Result to Option
    Battle::parse(&bytes).ok()
}