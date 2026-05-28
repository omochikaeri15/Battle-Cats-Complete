use crate::common::utils::csv;
use crate::animation::utils::math;

/// Represents a single point of transformation in time.
#[derive(Clone, Debug)]
pub struct Keyframe {
    /// The frame this value targets.
    pub frame: i32,
    /// The target value.
    pub value: i32,
    /// The interpolation curve type.
    pub ease_mode: i32,
    /// The strength of the easing curve.
    pub ease_power: i32,
}

/// A collection of keyframes defining the transformation curve for a structural part.
#[derive(Clone, Debug)]
pub struct AnimModification {
    /// The index of the targeted model part.
    pub part_id: usize,
    /// The property being modified.
    pub modification_type: i32,
    /// Defines loop behavior (infinite, single, or normal).
    pub loop_count: i32,
    /// The list of keyframes.
    pub keyframes: Vec<Keyframe>,
    pub min_frame: i32,
    pub max_frame: i32,
}

/// A parsed representation of a `.maanim` file.
#[derive(Clone, Debug, Default)]
pub struct Animation {
    /// Every transformation track in the file.
    pub curves: Vec<AnimModification>,
    /// The highest frame index found across all keyframes.
    pub max_frame: i32,
}

impl Animation {
    /// Parses `.maanim` byte data into an `Animation` object.
    ///
    /// # Arguments
    /// * `bytes` - The raw byte data of the file.
    ///
    /// # Returns
    /// Returns `Some(Animation)` if parsed successfully, or `None` if invalid.
    #[inline(always)]
    pub fn parse(bytes: impl AsRef<[u8]>) -> Option<Self> {
        Self::parse_inner(bytes.as_ref())
    }

    fn parse_inner(bytes: &[u8]) -> Option<Self> {
        let content = csv::scrub(bytes);
        let delimiter = ',';

        let lines: Vec<&str> = content.lines().filter(|line_ref| !line_ref.trim().is_empty()).collect();

        if lines.is_empty() { return None; }

        fn parse_num<T: std::str::FromStr + Default>(input_string: &str) -> T {
            input_string.trim().parse().unwrap_or_default()
        }

        let mut curves = Vec::new();
        let mut current_line_idx = 0;

        if current_line_idx < lines.len() && lines[current_line_idx].trim().starts_with('[') {
            current_line_idx += 1;
        }
        if current_line_idx < lines.len() { current_line_idx += 1; }
        if current_line_idx < lines.len() { current_line_idx += 1; }

        while current_line_idx < lines.len() {
            let current_line = lines[current_line_idx];
            let parts: Vec<&str> = current_line.split(delimiter).collect();
            current_line_idx += 1;

            if parts.len() < 5 { continue; }

            let part_id: usize = parse_num(parts[0]);
            let mod_type: i32 = parse_num(parts[1]);
            let loop_behavior: i32 = parse_num(parts[2]);
            let min_frame: i32 = parse_num(parts[3]);
            let max_frame: i32 = parse_num(parts[4]);

            if current_line_idx >= lines.len() { break; }
            let count_line = lines[current_line_idx];
            current_line_idx += 1;

            let count_str = count_line.split(delimiter).next().unwrap_or_default();
            let keyframe_count: usize = parse_num(count_str);

            let mut keyframes = Vec::new();

            for _ in 0..keyframe_count {
                if current_line_idx >= lines.len() { break; }
                let keyframe_line = lines[current_line_idx];
                current_line_idx += 1;
                let keyframe_parts: Vec<&str> = keyframe_line.split(delimiter).collect();

                if keyframe_parts.len() >= 2 {
                    let frame: i32 = parse_num(keyframe_parts[0]);
                    let value: i32 = parse_num(keyframe_parts[1]);

                    let ease_mode = keyframe_parts.get(2).map_or(0, |text_part| parse_num(text_part));
                    let ease_power = keyframe_parts.get(3).map_or(0, |text_part| parse_num(text_part));

                    keyframes.push(Keyframe { frame, value, ease_mode, ease_power });
                }
            }

            if !keyframes.is_empty() {
                curves.push(AnimModification {
                    part_id, modification_type: mod_type, loop_count: loop_behavior,
                    keyframes, min_frame, max_frame,
                });
            }
        }

        let mut max_len = 0;
        for curve in &curves {
            if let Some(last_keyframe) = curve.keyframes.last() {
                if last_keyframe.frame > max_len { max_len = last_keyframe.frame; }
            }
        }

        Some(Self { curves, max_frame: max_len })
    }

    /// Calculates the looping duration using the Least Common Multiple (LCM) of its curves.
    ///
    /// # Returns
    /// An `Option` containing the loop frame. Returns `None` if the animation
    /// contains non-looping curves.
    pub fn calculate_true_loop(&self) -> Option<i32> {
        let mut overall_lcm: i64 = 1;
        let mut found_looping_part = false;

        for curve in &self.curves {
            if curve.loop_count == 1 {
                return None;
            }

            let first_keyframe = match curve.keyframes.first() {
                Some(k) => k,
                None => continue,
            };

            let last_keyframe = match curve.keyframes.last() {
                Some(k) => k,
                None => continue,
            };

            let duration = (last_keyframe.frame - first_keyframe.frame) as i32;
            if duration <= 0 {
                continue;
            }

            overall_lcm = math::lcm(overall_lcm as i32, duration) as i64;

            if overall_lcm > 999_999 {
                return None;
            }

            found_looping_part = true;
        }

        if !found_looping_part {
            return Some(self.max_frame);
        }

        Some(std::cmp::max(overall_lcm as i32, self.max_frame))
    }

    /// Scans the raw bytes of an animation file to determine its total duration.
    ///
    /// # Arguments
    /// * `bytes` - The raw byte data of the file.
    ///
    /// # Returns
    /// An integer representing the highest frame count.
    #[inline(always)]
    pub fn scan_duration(bytes: impl AsRef<[u8]>) -> i32 {
        Self::scan_duration_inner(bytes.as_ref())
    }

    fn scan_duration_inner(bytes: &[u8]) -> i32 {
        let content = csv::scrub(bytes);
        let delimiter = ',';

        let lines: Vec<&str> = content.lines().filter(|l| !l.trim().is_empty()).collect();
        if lines.is_empty() { return 0; }

        let mut max_frame_count = 0;
        let mut i = 0;

        if i < lines.len() && lines[i].trim().starts_with('[') { i += 1; }
        if i < lines.len() { i += 1; }
        if i < lines.len() { i += 1; }

        while i < lines.len() {
            let parts: Vec<&str> = lines[i].split(delimiter).collect();
            i += 1;

            if parts.len() < 5 { continue; }

            let loop_count: i32 = parts[2].trim().parse().unwrap_or(1);
            let repeats = std::cmp::max(loop_count, 1);

            if i >= lines.len() { break; }

            let count_line = lines[i];
            let keyframe_count: usize = count_line.split(delimiter)
                .next()
                .and_then(|s| s.trim().parse().ok())
                .unwrap_or_default();
            i += 1;

            if keyframe_count > 0 {
                let first_frame_line = lines[i];
                let first_frame: i32 = first_frame_line.split(delimiter)
                    .next()
                    .and_then(|s| s.trim().parse().ok())
                    .unwrap_or_default();

                let last_idx = i + keyframe_count - 1;
                let last_frame: i32 = if last_idx < lines.len() {
                    lines[last_idx].split(delimiter)
                        .next()
                        .and_then(|s| s.trim().parse().ok())
                        .unwrap_or_default()
                } else {
                    0
                };

                let duration = last_frame - first_frame;
                max_frame_count = std::cmp::max((duration * repeats) + first_frame, max_frame_count);

                i += keyframe_count;
            }
        }

        max_frame_count
    }
}