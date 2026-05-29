use std::fmt;
use crate::common::utils::csv;

#[derive(Debug)]
pub enum SkillDescriptionsError {
    EmptyFile,
}

impl fmt::Display for SkillDescriptionsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SkillDescriptionsError::EmptyFile => write!(f, "The provided file bytes contained no valid skill descriptions."),
        }
    }
}

impl std::error::Error for SkillDescriptionsError {}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct SkillDescriptions {
    pub texts: Vec<String>,
}

impl SkillDescriptions {
    /// PUBLIC API: Parses a byte slice into a SkillDescriptions struct.
    pub fn parse<B: AsRef<[u8]>>(bytes: B) -> Result<Self, SkillDescriptionsError> {
        parse_inner(bytes.as_ref())
    }
}

/// PRIVATE INNER: Does the heavy lifting.
fn parse_inner(bytes: &[u8]) -> Result<SkillDescriptions, SkillDescriptionsError> {
    let file_content = csv::scrub(bytes);
    let separator = csv::detect_separator(&file_content);

    let mut texts = Vec::new();

    for line in file_content.lines() {
        if line.trim().is_empty() {
            // Domain Rule: Maintain index sync by pushing empty strings
            texts.push(String::new());
            continue;
        }

        let raw_text = match line.split_once(separator) {
            Some((_id, text_part)) => text_part,
            None => line,
        };

        // Domain Rule: Convert HTML line breaks to pure string newlines
        texts.push(raw_text.replace("<br>", "\n").trim().to_string());
    }

    if texts.is_empty() {
        return Err(SkillDescriptionsError::EmptyFile);
    }

    Ok(SkillDescriptions { texts })
}