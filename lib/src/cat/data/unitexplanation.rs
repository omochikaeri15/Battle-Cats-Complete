use std::fmt;
use crate::common::utils::csv;

#[derive(Debug)]
pub enum UnitExplanationError {
    EmptyFile,
}

impl fmt::Display for UnitExplanationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyFile => write!(f, "The provided file bytes contained no valid explanation text."),
        }
    }
}

impl std::error::Error for UnitExplanationError {}

/// Represents the localized display names and dictionary descriptions for an entity.
///
/// This structure maintains parallel fixed arrays mapping cleansed, multi-line string data
/// to their corresponding form indices (0 = Normal, 1 = Evolved, 2 = True, 3 = Ultra).
/// It automatically identifies and deduplicates identical sequential entries to optimize
/// payload size. Missing or deduplicated forms are explicitly represented as `None`.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct UnitExplanation {
    /// An array of parsed display names, indexed by form. `None` if the form does not exist or was deduplicated.
    pub names: [Option<String>; 4],
    /// An array of parsed multi-line descriptions, indexed by form. `None` if the form does not exist or was deduplicated.
    pub descriptions: [Option<Vec<String>>; 4],
}

impl UnitExplanation {
    /// Parses a raw byte stream into a `UnitExplanation` struct.
    ///
    /// # Arguments
    /// * `bytes` - The raw, decrypted byte slice of a unit's explanation `.csv` file.
    ///
    /// # Returns
    /// A `Result` containing the structured `UnitExplanation` on success, or a
    /// `UnitExplanationError` if the file contained no parseable text.
    pub fn parse<B: AsRef<[u8]>>(bytes: B) -> Result<Self, UnitExplanationError> {
        parse_inner(bytes.as_ref())
    }
}

fn parse_inner(bytes: &[u8]) -> Result<UnitExplanation, UnitExplanationError> {
    let file_content = csv::scrub(bytes);
    let separator_char = csv::detect_separator(&file_content);

    let mut names: [Option<String>; 4] = [const { None }; 4];
    let mut descriptions: [Option<Vec<String>>; 4] = [const { None }; 4];
    let mut has_content = false;

    for (line_index, file_line) in file_content.lines().enumerate().take(4) {
        let parts: Vec<&str> = file_line.split(separator_char).collect();
        
        if let Some(name_part) = parts.first() {
            let sanitized_name: String = name_part.trim().chars()
                .filter(|c| !is_problematic_char(*c))
                .collect();

            if !sanitized_name.is_empty() && !looks_like_garbage_id(&sanitized_name) {
                names[line_index] = Some(sanitized_name);
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
            descriptions[line_index] = Some(desc_lines);
        }
    }

    if !has_content {
        return Err(UnitExplanationError::EmptyFile);
    }

    for form_index in 1..4 {
        if names[form_index].is_some()
            && names[form_index] == names[form_index - 1]
            && descriptions[form_index] == descriptions[form_index - 1]
        {
            names[form_index] = None;
            descriptions[form_index] = None;
        }
    }

    Ok(UnitExplanation { names, descriptions })
}

fn is_problematic_char(character: char) -> bool {
    let codepoint = character as u32;
    if (0xE0100..=0xE01EF).contains(&codepoint) { return true; } // Variation Selectors
    if (0xFE00..=0xFE0F).contains(&codepoint) { return true; }   // Variation Selectors
    false
}

fn looks_like_garbage_id(text: &str) -> bool {
    text.chars().all(|c| c.is_ascii_digit() || c == '-' || c == '_')
}