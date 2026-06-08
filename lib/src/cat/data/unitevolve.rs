use std::collections::HashMap;
use std::fmt;
use crate::common::utils::csv;

#[derive(Debug)]
pub enum UnitEvolveError {
    EmptyFile,
}

impl fmt::Display for UnitEvolveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyFile => write!(f, "The provided file bytes contained no valid evolution text."),
        }
    }
}

impl std::error::Error for UnitEvolveError {}

/// Represents the localized evolutionary text parameters for an entity.
///
/// This structure maps sequences of cleansed, newline-formatted strings to
/// their corresponding form indices, detailing the specific requirements or
/// localized text associated with each evolutionary stage.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct UnitEvolve {
    /// An array of parsed string vectors, indexed by form.
    pub texts: [Vec<String>; 4],
}

impl UnitEvolve {
    pub fn parse<B: AsRef<[u8]>>(bytes: B) -> Result<HashMap<u32, Self>, UnitEvolveError> {
        parse_inner(bytes.as_ref())
    }
}

fn parse_inner(bytes: &[u8]) -> Result<HashMap<u32, UnitEvolve>, UnitEvolveError> {
    let file_content = csv::scrub(bytes);
    let delimiter = csv::detect_separator(&file_content);

    let mut map = HashMap::new();

    for (line_index, line_content) in file_content.lines().enumerate() {
        if line_content.trim().is_empty() {
            continue;
        }

        let column_parts: Vec<&str> = line_content.split(delimiter).collect();
        let cat_id = line_index as u32;

        let get_text = |index: usize| -> String {
            let raw_string = column_parts.get(index).map(|s| s.trim()).unwrap_or("");
            if raw_string == "@" || raw_string == "＠" || raw_string.is_empty() {
                return String::new();
            }
            raw_string.replace("<br>", "\n")
        };

        let true_form = vec![get_text(0), get_text(1), get_text(2)];
        let ultra_form = vec![get_text(4), get_text(5), get_text(6)];

        let mut texts: [Vec<String>; 4] = Default::default();

        for form_index in 1..4 {
            if !texts[form_index].is_empty() && texts[form_index] == texts[form_index - 1] {
                texts[form_index].clear();
            }
        }

        texts[2] = true_form;
        texts[3] = ultra_form;

        map.insert(cat_id, UnitEvolve { texts });
    }

    if map.is_empty() {
        return Err(UnitEvolveError::EmptyFile);
    }

    Ok(map)
}