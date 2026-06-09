use std::fmt;
use std::error;
use crate::common::utils::csv;

/// Represents errors that can occur during the parsing of localized enemy names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnemyNameError {
    EmptyData,
}

impl fmt::Display for EnemyNameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyData => write!(f, "The provided enemy name file contained no valid entries."),
        }
    }
}

impl error::Error for EnemyNameError {}

/// Represents the localized display name for an enemy entity.
///
/// This structure cleanly encapsulates a sanitized string. It automatically identifies
/// and rejects internal developer placeholders (such as "ダミー") to ensure the UI
/// does not render invalid terminology. Missing or invalid names evaluate to `None`.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct EnemyName {
    /// The parsed display name. `None` if the name is a placeholder or empty.
    pub name: Option<String>,
}

impl EnemyName {
    /// Parses a raw byte stream into a vector of `EnemyName` structures.
    ///
    /// # Arguments
    /// * `b` - The raw byte slice of the terminology file.
    ///
    /// # Returns
    /// A `Result` containing the vector of structured `EnemyName`s on success, or an
    /// `EnemyNameError` if the file contained no parseable text.
    pub fn parse_all<T: AsRef<[u8]>>(b: T) -> Result<Vec<Self>, EnemyNameError> {
        parse_all_inner(b.as_ref())
    }

    /// Safely extracts and parses a single `EnemyName` based on its internal ID line index.
    ///
    /// # Arguments
    /// * `b` - The raw byte slice of the terminology file.
    /// * `id` - The specific line index corresponding to the enemy's internal ID.
    ///
    /// # Returns
    /// A `Result` containing an `Option<EnemyName>` if the line exists, or `None` if the ID is out of bounds.
    pub fn parse<T: AsRef<[u8]>>(b: T, id: usize) -> Result<Option<Self>, EnemyNameError> {
        parse_inner(b.as_ref(), id)
    }
}

fn parse_line_data(line: &str, separator: char) -> EnemyName {
    let raw_name = line.split(separator).next().unwrap_or("").trim().to_string();
    let is_invalid = raw_name.is_empty() || raw_name == "ダミー";

    EnemyName {
        name: if is_invalid { None } else { Some(raw_name) },
    }
}

fn parse_all_inner(bytes: &[u8]) -> Result<Vec<EnemyName>, EnemyNameError> {
    let content = csv::scrub(bytes);
    let separator = csv::detect_separator(&content);
    let mut names = Vec::new();

    for line in content.lines() {
        names.push(parse_line_data(line, separator));
    }

    if names.is_empty() {
        return Err(EnemyNameError::EmptyData);
    }

    Ok(names)
}

fn parse_inner(bytes: &[u8], id: usize) -> Result<Option<EnemyName>, EnemyNameError> {
    let content = csv::scrub(bytes);
    let separator = csv::detect_separator(&content);

    let Some(target_line) = content.lines().nth(id) else {
        return Ok(None);
    };

    Ok(Some(parse_line_data(target_line, separator)))
}