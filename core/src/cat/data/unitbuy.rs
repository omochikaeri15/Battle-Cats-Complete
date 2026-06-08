use std::fs;
use std::path::Path;
use std::collections::HashMap;
use crate::cat::paths;
use nyanko::cat::unit::UnitBuy;

pub fn load_unitbuy(cats_directory: &Path, priority: &[String]) -> HashMap<u32, UnitBuy> {
    let mut map = HashMap::new();

    let Some(file_path) = crate::global::resolver::get(cats_directory, [paths::UNIT_BUY], priority).into_iter().next() else {
        return map;
    };

    let Ok(bytes) = fs::read(&file_path) else {
        return map;
    };

    // THE WAITER HAND-OFF: Pass pure bytes to the nyanko engine
    if let Ok(parsed_data) = UnitBuy::parse(&bytes) {
        map = parsed_data;
    }

    map
}