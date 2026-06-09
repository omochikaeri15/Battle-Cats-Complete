use std::fmt;
use std::error;
use crate::common::utils::csv;

/// Represents errors that can occur during the parsing of raw enemy battle data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BattleError {
    EmptyData,
}

impl fmt::Display for BattleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyData => write!(f, "The provided battle data file contained no valid entries."),
        }
    }
}

impl error::Error for BattleError {}

/// Represents the complete statistical and behavioral combat profile for an enemy.
///
/// This structure defines the strictly ordered array of combat parameters, execution
/// timings, targeting flags, and specialized ability modifiers mapping directly to
/// the application's internal simulation engine.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Battle {
    pub hitpoints: i32,
    pub knockbacks: i32,
    pub speed: i32,
    pub attack_1: i32,
    pub time_between_attacks: i32,
    pub standing_range: i32,
    pub cash_drop: i32,
    pub hitbox_position: i32,
    pub hitbox_width: i32,
    pub unused: i32,
    pub type_red: i32,
    pub area_attack: i32,
    pub time_until_attack_1: i32,
    pub type_floating: i32,
    pub type_dark: i32,
    pub type_metal: i32,
    pub type_traitless: i32,
    pub type_angel: i32,
    pub type_alien: i32,
    pub type_zombie: i32,
    pub knockback_chance: i32,
    pub freeze_chance: i32,
    pub freeze_duration: i32,
    pub slow_chance: i32,
    pub slow_duration: i32,
    pub critical_chance: i32,
    pub base_destroyer: i32,
    pub wave_chance: i32,
    pub wave_level: i32,
    pub weaken_chance: i32,
    pub weaken_duration: i32,
    pub weaken_percent: i32,
    pub strengthen_threshold: i32,
    pub strengthen_boost: i32,
    pub survive_chance: i32,
    pub long_distance_anchor_1: i32,
    pub long_distance_span_1: i32,
    pub wave_immune: i32,
    pub wave_blocker: i32,
    pub knockback_immune: i32,
    pub freeze_immune: i32,
    pub slow_immune: i32,
    pub weaken_immune: i32,
    pub burrow_amount: i32,
    pub burrow_distance: i32,
    pub revive_count: i32,
    pub revive_time: i32,
    pub revive_hp: i32,
    pub type_witch: i32,
    pub type_dojo: i32,
    pub attack_count_total: i32,
    pub time_before_death: i32,
    pub attack_count_state: i32,
    pub spawn_animation: i32,
    pub soul_animation: i32,
    pub attack_2: i32,
    pub attack_3: i32,
    pub time_until_attack_2: i32,
    pub time_until_attack_3: i32,
    pub attack_1_abilities: i32,
    pub attack_2_abilities: i32,
    pub attack_3_abilities: i32,
    pub spawn_animation_flag: i32,
    pub soul_animation_flag: i32,
    pub barrier_hitpoints: i32,
    pub warp_chance: i32,
    pub warp_duration: i32,
    pub warp_distance_minimum: i32,
    pub warp_distance_maximum: i32,
    pub type_starred_alien: i32,
    pub warp_immune: i32,
    pub type_eva: i32,
    pub type_relic: i32,
    pub curse_chance: i32,
    pub curse_duration: i32,
    pub savage_blow_chance: i32,
    pub savage_blow_boost: i32,
    pub dodge_chance: i32,
    pub dodge_duration: i32,
    pub toxic_chance: i32,
    pub toxic_damage: i32,
    pub surge_chance: i32,
    pub surge_spawn_min: i32,
    pub surge_spawn_max: i32,
    pub surge_level: i32,
    pub surge_immune: i32,
    pub mini_wave: i32,
    pub shield_hitpoints: i32,
    pub shield_regen: i32,
    pub death_surge_chance: i32,
    pub death_surge_spawn_min: i32,
    pub death_surge_spawn_max: i32,
    pub death_surge_level: i32,
    pub type_aku: i32,
    pub type_colossus: i32,
    pub long_distance_2_flag: i32,
    pub long_distance_2_anchor: i32,
    pub long_distance_2_span: i32,
    pub long_distance_3_flag: i32,
    pub long_distance_3_anchor: i32,
    pub long_distance_3_span: i32,
    pub type_behemoth: i32,
    pub mini_surge: i32,
    pub counter_surge: i32,
    pub type_sage: i32,
    pub curse_immune: i32,
    pub explosion_chance: i32,
    pub explosion_anchor: i32,
    pub explosion_span: i32,
    pub explosion_immune: i32,
    pub type_supervillain: i32,
    pub drain_chance: i32,
    pub drain_percent: i32,
    pub has_unknown_abilities: i32,
}

impl Battle {
    /// Calculates the absolute duration of an attack cycle in engine frames.
    ///
    /// # Arguments
    /// * `frames` - The chronological frame count parsed from the unit's animation sequence.
    ///
    /// # Returns
    /// The total mathematical duration of the attack cycle.
    pub fn attack_cycle(&self, frames: i32) -> i32 {
        let mut effective_foreswing = self.time_until_attack_1;

        if self.attack_3 > 0 && self.time_until_attack_3 > 0 {
            effective_foreswing = self.time_until_attack_3;
        }
        else if self.attack_2 > 0 && self.time_until_attack_2 > 0 {
            effective_foreswing = self.time_until_attack_2;
        }

        let cooldown_frames = self.time_between_attacks.saturating_sub(1);

        (effective_foreswing + cooldown_frames).max(frames)
    }

    /// Parses a raw byte stream into a comprehensive vector of `Battle` stat blocks.
    ///
    /// # Arguments
    /// * `b` - The raw byte slice representing the data matrix.
    ///
    /// # Returns
    /// A `Result` containing the vector of `Battle`s on success, or a
    /// `BattleError` if the file was empty or corrupted.
    pub fn parse_all<T: AsRef<[u8]>>(b: T) -> Result<Vec<Self>, BattleError> {
        parse_all_inner(b.as_ref())
    }

    /// Safely extracts and parses a single `Battle` stat block based on its internal ID line index.
    ///
    /// # Arguments
    /// * `b` - The raw byte slice representing the data matrix.
    /// * `id` - The specific line index corresponding to the enemy's internal ID.
    ///
    /// # Returns
    /// A `Result` containing an `Option<Battle>` if the line exists, or `None` if the ID is out of bounds.
    pub fn parse<T: AsRef<[u8]>>(b: T, id: usize) -> Result<Option<Self>, BattleError> {
        parse_inner(b.as_ref(), id)
    }
}

fn parse_cell(cols: &[&str], index: usize, max_read: &mut usize, default_value: i32) -> i32 {
    *max_read = (*max_read).max(index);
    cols.get(index).and_then(|s| s.trim().parse::<i32>().ok()).unwrap_or(default_value)
}

fn parse_line_data(cols: &[&str]) -> Battle {
    let mut max_read = 0;

    let mut raw = Battle {
        hitpoints: parse_cell(cols, 0, &mut max_read, 0),
        knockbacks: parse_cell(cols, 1, &mut max_read, 0),
        speed: parse_cell(cols, 2, &mut max_read, 0),
        attack_1: parse_cell(cols, 3, &mut max_read, 0),
        time_between_attacks: parse_cell(cols, 4, &mut max_read, 0) * 2,
        standing_range: parse_cell(cols, 5, &mut max_read, 0),
        cash_drop: parse_cell(cols, 6, &mut max_read, 0),
        hitbox_position: parse_cell(cols, 7, &mut max_read, 0),
        hitbox_width: parse_cell(cols, 8, &mut max_read, 0),
        unused: parse_cell(cols, 9, &mut max_read, 0),
        type_red: parse_cell(cols, 10, &mut max_read, 0),
        area_attack: parse_cell(cols, 11, &mut max_read, 0),
        time_until_attack_1: parse_cell(cols, 12, &mut max_read, 0),
        type_floating: parse_cell(cols, 13, &mut max_read, 0),
        type_dark: parse_cell(cols, 14, &mut max_read, 0),
        type_metal: parse_cell(cols, 15, &mut max_read, 0),
        type_traitless: parse_cell(cols, 16, &mut max_read, 0),
        type_angel: parse_cell(cols, 17, &mut max_read, 0),
        type_alien: parse_cell(cols, 18, &mut max_read, 0),
        type_zombie: parse_cell(cols, 19, &mut max_read, 0),
        knockback_chance: parse_cell(cols, 20, &mut max_read, 0),
        freeze_chance: parse_cell(cols, 21, &mut max_read, 0),
        freeze_duration: parse_cell(cols, 22, &mut max_read, 0),
        slow_chance: parse_cell(cols, 23, &mut max_read, 0),
        slow_duration: parse_cell(cols, 24, &mut max_read, 0),
        critical_chance: parse_cell(cols, 25, &mut max_read, 0),
        base_destroyer: parse_cell(cols, 26, &mut max_read, 0),
        wave_chance: parse_cell(cols, 27, &mut max_read, 0),
        wave_level: parse_cell(cols, 28, &mut max_read, 0),
        weaken_chance: parse_cell(cols, 29, &mut max_read, 0),
        weaken_duration: parse_cell(cols, 30, &mut max_read, 0),
        weaken_percent: parse_cell(cols, 31, &mut max_read, 0),
        strengthen_threshold: parse_cell(cols, 32, &mut max_read, 0),
        strengthen_boost: parse_cell(cols, 33, &mut max_read, 0),
        survive_chance: parse_cell(cols, 34, &mut max_read, 0),
        long_distance_anchor_1: parse_cell(cols, 35, &mut max_read, 0),
        long_distance_span_1: parse_cell(cols, 36, &mut max_read, 0),
        wave_immune: parse_cell(cols, 37, &mut max_read, 0),
        wave_blocker: parse_cell(cols, 38, &mut max_read, 0),
        knockback_immune: parse_cell(cols, 39, &mut max_read, 0),
        freeze_immune: parse_cell(cols, 40, &mut max_read, 0),
        slow_immune: parse_cell(cols, 41, &mut max_read, 0),
        weaken_immune: parse_cell(cols, 42, &mut max_read, 0),
        burrow_amount: parse_cell(cols, 43, &mut max_read, 0),
        burrow_distance: parse_cell(cols, 44, &mut max_read, 0) / 4,
        revive_count: parse_cell(cols, 45, &mut max_read, 0),
        revive_time: parse_cell(cols, 46, &mut max_read, 0),
        revive_hp: parse_cell(cols, 47, &mut max_read, 0),
        type_witch: parse_cell(cols, 48, &mut max_read, 0),
        type_dojo: parse_cell(cols, 49, &mut max_read, 0),
        attack_count_total: parse_cell(cols, 50, &mut max_read, -1),
        time_before_death: parse_cell(cols, 51, &mut max_read, -1),
        attack_count_state: parse_cell(cols, 52, &mut max_read, 0),
        spawn_animation: parse_cell(cols, 53, &mut max_read, 0),
        soul_animation: parse_cell(cols, 54, &mut max_read, 0),
        attack_2: parse_cell(cols, 55, &mut max_read, 0),
        attack_3: parse_cell(cols, 56, &mut max_read, 0),
        time_until_attack_2: parse_cell(cols, 57, &mut max_read, 0),
        time_until_attack_3: parse_cell(cols, 58, &mut max_read, 0),
        attack_1_abilities: parse_cell(cols, 59, &mut max_read, 0),
        attack_2_abilities: parse_cell(cols, 60, &mut max_read, 0),
        attack_3_abilities: parse_cell(cols, 61, &mut max_read, 0),
        spawn_animation_flag: parse_cell(cols, 62, &mut max_read, 0),
        soul_animation_flag: parse_cell(cols, 63, &mut max_read, 0),
        barrier_hitpoints: parse_cell(cols, 64, &mut max_read, 0),
        warp_chance: parse_cell(cols, 65, &mut max_read, 0),
        warp_duration: parse_cell(cols, 66, &mut max_read, 0),
        warp_distance_minimum: parse_cell(cols, 67, &mut max_read, 0) / 4,
        warp_distance_maximum: parse_cell(cols, 68, &mut max_read, 0) / 4,
        type_starred_alien: parse_cell(cols, 69, &mut max_read, 0),
        warp_immune: parse_cell(cols, 70, &mut max_read, 0),
        type_eva: parse_cell(cols, 71, &mut max_read, 0),
        type_relic: parse_cell(cols, 72, &mut max_read, 0),
        curse_chance: parse_cell(cols, 73, &mut max_read, 0),
        curse_duration: parse_cell(cols, 74, &mut max_read, 0),
        savage_blow_chance: parse_cell(cols, 75, &mut max_read, 0),
        savage_blow_boost: parse_cell(cols, 76, &mut max_read, 0),
        dodge_chance: parse_cell(cols, 77, &mut max_read, 0),
        dodge_duration: parse_cell(cols, 78, &mut max_read, 0),
        toxic_chance: parse_cell(cols, 79, &mut max_read, 0),
        toxic_damage: parse_cell(cols, 80, &mut max_read, 0),
        surge_chance: parse_cell(cols, 81, &mut max_read, 0),
        surge_spawn_min: parse_cell(cols, 82, &mut max_read, 0) / 4,
        surge_spawn_max: parse_cell(cols, 83, &mut max_read, 0) / 4,
        surge_level: parse_cell(cols, 84, &mut max_read, 0),
        surge_immune: parse_cell(cols, 85, &mut max_read, 0),
        mini_wave: parse_cell(cols, 86, &mut max_read, 0),
        shield_hitpoints: parse_cell(cols, 87, &mut max_read, 0),
        shield_regen: parse_cell(cols, 88, &mut max_read, 0),
        death_surge_chance: parse_cell(cols, 89, &mut max_read, 0),
        death_surge_spawn_min: parse_cell(cols, 90, &mut max_read, 0) / 4,
        death_surge_spawn_max: parse_cell(cols, 91, &mut max_read, 0) / 4,
        death_surge_level: parse_cell(cols, 92, &mut max_read, 0),
        type_aku: parse_cell(cols, 93, &mut max_read, 0),
        type_colossus: parse_cell(cols, 94, &mut max_read, 0),
        long_distance_2_flag: parse_cell(cols, 95, &mut max_read, 0),
        long_distance_2_anchor: parse_cell(cols, 96, &mut max_read, 0),
        long_distance_2_span: parse_cell(cols, 97, &mut max_read, 0),
        long_distance_3_flag: parse_cell(cols, 98, &mut max_read, 0),
        long_distance_3_anchor: parse_cell(cols, 99, &mut max_read, 0),
        long_distance_3_span: parse_cell(cols, 100, &mut max_read, 0),
        type_behemoth: parse_cell(cols, 101, &mut max_read, 0),
        mini_surge: parse_cell(cols, 102, &mut max_read, 0),
        counter_surge: parse_cell(cols, 103, &mut max_read, 0),
        type_sage: parse_cell(cols, 104, &mut max_read, 0),
        curse_immune: parse_cell(cols, 105, &mut max_read, 0),
        explosion_chance: parse_cell(cols, 106, &mut max_read, 0),
        explosion_anchor: parse_cell(cols, 107, &mut max_read, 0) / 4,
        explosion_span: parse_cell(cols, 108, &mut max_read, 0) / 4,
        explosion_immune: parse_cell(cols, 109, &mut max_read, 0),
        type_supervillain: parse_cell(cols, 110, &mut max_read, 0),
        drain_chance: parse_cell(cols, 111, &mut max_read, 0),
        drain_percent: parse_cell(cols, 112, &mut max_read, 0),
        has_unknown_abilities: 0,
    };

    raw.has_unknown_abilities = if cols.iter()
        .skip(max_read + 1)
        .any(|col| col.trim().parse::<i32>().unwrap_or(0) != 0)
    {
        1
    } else {
        0
    };

    raw
}

fn parse_all_inner(bytes: &[u8]) -> Result<Vec<Battle>, BattleError> {
    let content = csv::scrub(bytes);
    let separator = csv::detect_separator(&content);
    let mut enemies = Vec::new();

    for line in content.lines().skip(2) {
        let cols: Vec<&str> = line.split(separator).collect();
        if cols.len() < 10 {
            continue;
        }
        enemies.push(parse_line_data(&cols));
    }

    if enemies.is_empty() {
        return Err(BattleError::EmptyData);
    }

    Ok(enemies)
}

fn parse_inner(bytes: &[u8], id: usize) -> Result<Option<Battle>, BattleError> {
    let content = csv::scrub(bytes);
    let separator = csv::detect_separator(&content);

    let Some(target_line) = content.lines().skip(2).nth(id) else {
        return Ok(None);
    };

    let cols: Vec<&str> = target_line.split(separator).collect();
    if cols.len() < 10 {
        return Ok(None);
    }

    Ok(Some(parse_line_data(&cols)))
}