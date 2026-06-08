use std::fmt;
use crate::common::utils::csv;

#[derive(Debug)]
pub enum UnitExplanationError {
    EmptyFile,
}

impl fmt::Display for UnitExplanationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnitExplanationError::EmptyFile => write!(f, "The provided file bytes contained no valid explanation text."),
        }
    }
}

impl std::error::Error for UnitExplanationError {}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct UnitExplanation {
    pub names: [String; 4],
    pub descriptions: [Vec<String>; 4],
}

impl UnitExplanation {
    /// PUBLIC API: Parses a byte slice into a UnitExplanation struct.
    pub fn parse<B: AsRef<[u8]>>(bytes: B) -> Result<Self, UnitExplanationError> {
        parse_inner(bytes.as_ref())
    }
}

/// PRIVATE INNER: Does the heavy lifting and domain sanitization.
fn parse_inner(bytes: &[u8]) -> Result<UnitExplanation, UnitExplanationError> {
    let file_content = csv::scrub(bytes);
    let separator_char = csv::detect_separator(&file_content);

    let mut names: [String; 4] = Default::default();
    let mut descriptions: [Vec<String>; 4] = Default::default();
    let mut has_content = false;

    for (line_index, file_line) in file_content.lines().enumerate().take(4) {
        let parts: Vec<&str> = file_line.split(separator_char).collect();

        if let Some(name_part) = parts.first() {
            let trimmed_name = name_part.trim();

            // Domain Rule: Clean up PONOS text encoding quirks
            let sanitized_name: String = trimmed_name.chars()
                .filter(|c| !is_problematic_char(*c))
                .collect();

            // Domain Rule: Ignore placeholder dummy IDs
            if !sanitized_name.is_empty() && !looks_like_garbage_id(&sanitized_name) {
                names[line_index] = sanitized_name;
                has_content = true;
            }
        }

        let desc_lines: Vec<String> = parts.iter()
            .skip(1)
            .take(3)
            .map(|s| s.trim().to_string())
            .collect();

        if !desc_lines.is_empty() && desc_lines.iter().any(|s| !s.is_empty()) {
            has_content = true;
        }

        descriptions[line_index] = desc_lines;
    }

    if !has_content {
        return Err(UnitExplanationError::EmptyFile);
    }

    // Domain Rule: Clear identical redundant evolutions to save memory
    for i in 1..4 {
        if !names[i].is_empty() && names[i] == names[i - 1] && descriptions[i] == descriptions[i - 1] {
            names[i].clear();
            descriptions[i].clear();
        }
    }

    Ok(UnitExplanation { names, descriptions })
}

// --- Domain Helpers ---

fn is_problematic_char(c: char) -> bool {
    let u = c as u32;
    if (0xE0100..=0xE01EF).contains(&u) { return true; }
    if (0xFE00..=0xFE0F).contains(&u) { return true; }
    false
}

fn looks_like_garbage_id(text: &str) -> bool {
    text.chars().all(|char_check| char_check.is_ascii_digit() || char_check == '-' || char_check == '_')
}