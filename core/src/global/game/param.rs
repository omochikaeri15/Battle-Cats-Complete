use std::fs;
use std::path::Path;
use nyanko::common::Param; // Adjust import based on where you exposed it!

pub fn load_param(data_directory: &Path, priority: &[String]) -> Option<Param> {
    let file_path = crate::global::resolver::get(data_directory, ["param.tsv"], priority).into_iter().next()?;

    let bytes = fs::read(&file_path).ok()?;
    Param::parse(&bytes).ok()
}