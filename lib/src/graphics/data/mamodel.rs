use crate::common::utils::csv;

/// Represents the baseline geometric and hierarchical state of a singular skeletal component.
///
/// Defines initial affine transformations, structural pivot points, and drawing order assignments
/// for a model part before timeline-based animation modifications are calculated.
#[derive(Clone, Debug)]
pub struct ModelPart {
    /// The index of the structural parent governing this part's hierarchical inheritance.
    pub parent_id: i32,
    /// An internal identifier mapping this component to a generalized unit structure.
    pub unit_id: i32,
    /// The integer key mapping this part to a specific `SpriteCut` in the associated `SpriteSheet`.
    pub sprite_index: i32,
    /// The absolute Z-order dictating depth sorting during rendering (higher values draw in front).
    pub drawing_layer: i32,
    /// The resting spatial coordinate along the X-axis.
    pub position_x: f32,
    /// The resting spatial coordinate along the Y-axis.
    pub position_y: f32,
    /// The local rotational axis anchor along the X-axis.
    pub pivot_x: f32,
    /// The local rotational axis anchor along the Y-axis.
    pub pivot_y: f32,
    /// The initial scaling multiplier along the X-axis.
    pub scale_x: f32,
    /// The initial scaling multiplier along the Y-axis.
    pub scale_y: f32,
    /// The baseline rotational angle.
    pub rotation: f32,
    /// The baseline alpha transparency value.
    pub alpha: f32,
    /// An integer flag indicating if additive blending (glow) is enabled.
    pub glow_mode: i32,
    /// Indicates if the sprite geometry is horizontally inverted.
    pub flip_x: bool,
    /// Indicates if the sprite geometry is vertically inverted.
    pub flip_y: bool,
    /// The internal logical name of the part.
    #[allow(dead_code)] pub name: String,
}

impl Default for ModelPart {
    fn default() -> Self {
        Self {
            parent_id: -1, unit_id: 0, sprite_index: 0, drawing_layer: 0,
            position_x: 0.0, position_y: 0.0, pivot_x: 0.0, pivot_y: 0.0,
            scale_x: 1000.0, scale_y: 1000.0, rotation: 0.0, alpha: 1000.0,
            glow_mode: 0, flip_x: false, flip_y: false, name: String::new(),
        }
    }
}

/// The root hierarchical skeletal structure of a graphical entity.
///
/// This structure encapsulates the localized `ModelPart` components and defines the global
/// arithmetic divisors required to decode spatial, rotational, and opacity transformations
/// from their integer storage format into standard floating-point space.
#[derive(Clone, Debug)]
pub struct Model {
    /// The ordered collection of individual skeletal components dictating the model's geometry.
    pub parts: Vec<ModelPart>,
    /// The specification version of the parsed format.
    #[allow(dead_code)] pub version: u32,
    /// The global denominator used to normalize raw scale values.
    pub scale_unit: f32,
    /// The global denominator used to normalize raw rotational values into degrees.
    pub angle_unit: f32,
    /// The global denominator used to normalize raw alpha values into a 0.0 - 1.0 range.
    pub alpha_unit: f32,
}

impl Default for Model {
    fn default() -> Self {
        Self { parts: Vec::new(), version: 0, scale_unit: 1000.0, angle_unit: 3600.0, alpha_unit: 1000.0 }
    }
}

impl Model {
    /// Parses a `.mamodel` byte stream into a structured `Model` hierarchy.
    ///
    /// # Arguments
    /// * `bytes` - The raw byte data of the `.mamodel` file.
    ///
    /// # Returns
    /// Returns `Some(Model)` if the structure is successfully parsed, or `None` if the input is malformed.
    #[inline(always)]
    pub fn parse(bytes: impl AsRef<[u8]>) -> Option<Self> {
        Self::parse_inner(bytes.as_ref())
    }

    fn parse_inner(bytes: &[u8]) -> Option<Self> {
        let content = csv::scrub(bytes);
        let delimiter = csv::detect_separator(&content);

        let lines: Vec<&str> = content.lines().filter(|line_ref| !line_ref.trim().is_empty()).collect();

        if lines.is_empty() { return None; }

        let mut part_count = 0;
        let mut data_start_index = 0;

        for (index, line) in lines.iter().take(5).enumerate() {
            if line.contains(delimiter) { break; }
            let Ok(parsed_count) = line.trim().parse::<usize>() else { continue; };
            if parsed_count == 0 || parsed_count >= 1000 { continue; }

            part_count = parsed_count;
            data_start_index = index + 1;
        }

        if part_count == 0 { return None; }

        let unit_line_index = data_start_index + part_count;
        let mut scale_unit = 1000.0;
        let mut angle_unit = 3600.0;
        let mut alpha_unit = 1000.0;
        let mut metadata_start_index = usize::MAX;

        for index in unit_line_index..lines.len() {
            let columns: Vec<&str> = lines[index].split(delimiter).collect();
            if columns.len() < 3 { continue; }

            let Ok(scale_val) = columns[0].trim().parse::<f32>() else { continue; };
            let Ok(angle_val) = columns[1].trim().parse::<f32>() else { continue; };
            let Ok(alpha_val) = columns[2].trim().parse::<f32>() else { continue; };

            if scale_val != 0.0 { scale_unit = scale_val; }
            if angle_val != 0.0 { angle_unit = angle_val; }
            if alpha_val != 0.0 { alpha_unit = alpha_val; }

            metadata_start_index = index + 1;
            break;
        }

        let mut parts = Vec::new();

        for index in 0..part_count {
            let target_line_idx = data_start_index + index;
            if target_line_idx >= lines.len() { break; }

            let columns: Vec<&str> = lines[target_line_idx].split(delimiter).collect();
            if columns.len() < 13 { continue; }

            let is_root = parts.is_empty();
            let raw_name = if columns.len() > 13 { columns[13].trim().to_string() } else { String::new() };

            parts.push(ModelPart {
                parent_id:     columns[0].trim().parse().unwrap_or(-1),
                unit_id:       columns[1].trim().parse().unwrap_or(0),
                sprite_index:  columns[2].trim().parse().unwrap_or(0),
                drawing_layer: columns[3].trim().parse().unwrap_or(0),
                position_x:    if is_root { 0.0 } else { columns[4].trim().parse().unwrap_or(0.0) },
                position_y:    if is_root { 0.0 } else { columns[5].trim().parse().unwrap_or(0.0) },
                pivot_x:       columns[6].trim().parse().unwrap_or(0.0),
                pivot_y:       columns[7].trim().parse().unwrap_or(0.0),
                scale_x:       columns[8].trim().parse().unwrap_or(scale_unit),
                scale_y:       columns[9].trim().parse().unwrap_or(scale_unit),
                rotation:      columns[10].trim().parse().unwrap_or(0.0),
                alpha:         columns[11].trim().parse().unwrap_or(alpha_unit),
                glow_mode:     columns[12].trim().parse().unwrap_or(0),
                flip_x:        false, flip_y: false, name: raw_name,
            });
        }

        let _ = (|| -> Option<()> {
            if parts.is_empty() { return None; }
            let metadata_count = lines.get(metadata_start_index)?.trim().parse::<usize>().ok()?;
            if metadata_count == 0 { return None; }

            let columns: Vec<&str> = lines.get(metadata_start_index + 1)?.split(delimiter).collect();
            if columns.len() < 4 { return None; }

            parts[0].position_x = -columns[2].trim().parse::<f32>().unwrap_or(0.0);
            parts[0].position_y = -columns[3].trim().parse::<f32>().unwrap_or(0.0);
            Some(())
        })();

        Some(Model { parts, version: 1, scale_unit, angle_unit, alpha_unit })
    }
}