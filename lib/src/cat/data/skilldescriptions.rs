use std::fmt;
use crate::common::utils::csv;

#[derive(Debug)]
pub enum SkillDescriptionsError {
    EmptyFile,
}

impl fmt::Display for SkillDescriptionsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyFile => write!(f, "The provided file bytes contained no valid skill descriptions."),
        }
    }
}

impl std::error::Error for SkillDescriptionsError {}

/// Represents the ordered sequence of localized textual descriptions for skills.
///
/// This structure maintains a synchronized index mapping where the vector index
/// corresponds directly to the internal skill identifier. It retains the cleansed,
/// newline-formatted display strings extracted from the raw data buffer.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct SkillDescriptions {
    /// The sequence of cleansed description strings, indexed by skill ID.
    pub texts: Vec<String>,
}

impl SkillDescriptions {
    pub fn parse<B: AsRef<[u8]>>(bytes: B) -> Result<Self, SkillDescriptionsError> {
        parse_inner(bytes.as_ref())
    }
}

fn parse_inner(bytes: &[u8]) -> Result<SkillDescriptions, SkillDescriptionsError> {
    let file_content = csv::scrub(bytes);
    let separator = csv::detect_separator(&file_content);

    let mut texts = Vec::new();

    for line in file_content.lines() {
        if line.trim().is_empty() {
            texts.push(String::new());
            continue;
        }

        let raw_text = match line.split_once(separator) {
            Some((_id, text_part)) => text_part,
            None => line,
        };

        texts.push(raw_text.replace("<br>", "\n").trim().to_string());
    }

    if texts.is_empty() {
        return Err(SkillDescriptionsError::EmptyFile);
    }

    Ok(SkillDescriptions { texts })
}