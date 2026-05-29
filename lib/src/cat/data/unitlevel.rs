use std::fmt;
use crate::common::utils::csv;

#[derive(Debug)]
pub enum LevelError {
    EmptyFile,
}

impl fmt::Display for LevelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LevelError::EmptyFile => write!(f, "The provided file bytes contained no valid unit level data."),
        }
    }
}

impl std::error::Error for LevelError {}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct LevelCurve {
    pub increments: Vec<u16>,
}

impl LevelCurve {
    fn from_csv_line(csv_line: &str, delimiter: char) -> Self {
        let line_parts: Vec<&str> = csv_line.split(delimiter).collect();
        let mut increment_values = Vec::new();
        for part in line_parts {
            if let Ok(value) = part.trim().parse::<u16>() {
                increment_values.push(value);
            }
        }
        Self { increments: increment_values }
    }

    /// Pure, platform-agnostic leveling math.
    pub fn calculate_stat(&self, base_value: i32, target_level: i32) -> i32 {
        let base_float = base_value as f64;
        let mut current_stat = base_float;
        let max_scaled_level = (self.increments.len() * 10) as i32;
        let level_limit = std::cmp::min(target_level, max_scaled_level);

        for level_step in 2..=level_limit {
            let curve_index = ((level_step as f64 / 10.0).ceil() as usize).saturating_sub(1);
            if let Some(&scaling_factor) = self.increments.get(curve_index) {
                current_stat += base_float * (scaling_factor as f64) / 100.0;
            }
        }

        if target_level > max_scaled_level {
            let levels_above_limit = target_level - max_scaled_level;
            if let Some(&last_scaling_factor) = self.increments.last() {
                current_stat += base_float * (last_scaling_factor as f64) * (levels_above_limit as f64) / 100.0;
            }
        }

        let rounded_stat = current_stat.round();
        let final_stat = (rounded_stat * 2.5).floor();
        final_stat as i32
    }

    /// PUBLIC API: Parses a byte slice into a vector of LevelCurves.
    pub fn parse<B: AsRef<[u8]>>(bytes: B) -> Result<Vec<Self>, LevelError> {
        parse_inner(bytes.as_ref())
    }
}

/// PRIVATE INNER: Does the heavy lifting.
fn parse_inner(bytes: &[u8]) -> Result<Vec<LevelCurve>, LevelError> {
    let file_content = csv::scrub(bytes);
    let delimiter = csv::detect_separator(&file_content);

    let mut curves_list = Vec::new();

    for csv_line in file_content.lines() {
        if csv_line.trim().is_empty() { continue; }
        curves_list.push(LevelCurve::from_csv_line(csv_line, delimiter));
    }

    if curves_list.is_empty() {
        return Err(LevelError::EmptyFile);
    }

    Ok(curves_list)
}