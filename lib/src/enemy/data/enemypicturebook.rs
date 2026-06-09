use std::fmt;
use std::error;
use crate::common::utils::csv;

/// Represents errors that can occur during the parsing of enemy picture book descriptions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnemyPictureBookError {
    EmptyData,
}

impl fmt::Display for EnemyPictureBookError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyData => write!(f, "The provided enemy picture book file contained no valid entries."),
        }
    }
}

impl error::Error for EnemyPictureBookError {}

/// Represents the localized multi-line dictionary description for an enemy entity.
///
/// Automatically strips internal placeholders (such as "仮") and null characters,
/// returning a clean, structured array of strings ready for rendering.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct EnemyPictureBook {
    /// A vector of parsed text lines. Evaluates to `None` if the description is missing or invalid.
    pub description: Option<Vec<String>>,
}

impl EnemyPictureBook {
    /// Parses a raw byte stream into a vector of `EnemyPictureBook` structures.
    ///
    /// # Arguments
    /// * `b` - The raw byte slice of the picture book description file.
    ///
    /// # Returns
    /// A `Result` containing the vector of structured `EnemyPictureBook`s on success, or an
    /// `EnemyPictureBookError` if the file contained no parseable text.
    pub fn parse_all<T: AsRef<[u8]>>(b: T) -> Result<Vec<Self>, EnemyPictureBookError> {
        parse_all_inner(b.as_ref())
    }

    /// Safely extracts and parses a single `EnemyPictureBook` based on its internal ID line index.
    ///
    /// # Arguments
    /// * `b` - The raw byte slice of the picture book description file.
    /// * `id` - The specific line index corresponding to the enemy's internal ID.
    ///
    /// # Returns
    /// A `Result` containing an `Option<EnemyPictureBook>` if the line exists, or `None` if the ID is out of bounds.
    pub fn parse<T: AsRef<[u8]>>(b: T, id: usize) -> Result<Option<Self>, EnemyPictureBookError> {
        parse_inner(b.as_ref(), id)
    }
}

fn parse_line_data(line: &str, separator: char) -> EnemyPictureBook {
    let cols: Vec<&str> = line.split(separator).collect();
    let mut desc_lines = Vec::new();

    for col in cols.into_iter().skip(1) {
        let text = col.trim();
        if text.is_empty() || text.starts_with("仮") { continue; }
        desc_lines.push(text.to_string());
    }

    EnemyPictureBook {
        description: if desc_lines.is_empty() { None } else { Some(desc_lines) },
    }
}

fn parse_all_inner(bytes: &[u8]) -> Result<Vec<EnemyPictureBook>, EnemyPictureBookError> {
    let content = csv::scrub(bytes);
    let separator = csv::detect_separator(&content);
    let mut descriptions = Vec::new();

    for line in content.lines() {
        descriptions.push(parse_line_data(line, separator));
    }

    if descriptions.is_empty() {
        return Err(EnemyPictureBookError::EmptyData);
    }

    Ok(descriptions)
}

fn parse_inner(bytes: &[u8], id: usize) -> Result<Option<EnemyPictureBook>, EnemyPictureBookError> {
    let content = csv::scrub(bytes);
    let separator = csv::detect_separator(&content);

    let Some(target_line) = content.lines().nth(id) else {
        return Ok(None);
    };

    Ok(Some(parse_line_data(target_line, separator)))
}