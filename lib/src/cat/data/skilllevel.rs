use std::collections::HashMap;
use std::fmt;
use crate::common::utils::csv;

#[derive(Debug)]
pub enum SkillLevelError {
    EmptyFile,
}

impl fmt::Display for SkillLevelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SkillLevelError::EmptyFile => write!(f, "The provided file bytes contained no valid skill level data."),
        }
    }
}

impl std::error::Error for SkillLevelError {}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TalentCost {
    pub costs: Vec<u16>,
}

impl TalentCost {
    /// PUBLIC API: Parses a byte slice into a HashMap of Talent Costs.
    pub fn parse<B: AsRef<[u8]>>(bytes: B) -> Result<HashMap<u8, Self>, SkillLevelError> {
        parse_inner(bytes.as_ref())
    }
}

/// PRIVATE INNER: Does the heavy lifting without monomorphization bloat.
fn parse_inner(bytes: &[u8]) -> Result<HashMap<u8, TalentCost>, SkillLevelError> {
    let file_content = csv::scrub(bytes);
    let delimiter = csv::detect_separator(&file_content);

    let mut map = HashMap::new();

    for line in file_content.lines() {
        if line.trim().is_empty() { continue; }

        let parts: Vec<&str> = line.split(delimiter).collect();
        if parts.is_empty() { continue; }

        if let Ok(id) = parts[0].trim().parse::<u8>() {
            let costs: Vec<u16> = parts.iter()
                .skip(1)
                .filter_map(|s| s.trim().parse::<u16>().ok())
                .collect();
            map.insert(id, TalentCost { costs });
        }
    }

    if map.is_empty() {
        return Err(SkillLevelError::EmptyFile);
    }

    Ok(map)
}