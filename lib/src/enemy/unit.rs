//! Core data aggregation module for enemy entities.
//!
//! This module acts as the primary data facade, re-exporting the specialized
//! parsers, structures, and error types required to process an enemy's mechanical
//! and visual data.

pub use crate::enemy::data::t_unit::{Battle, BattleError};
pub use crate::enemy::data::enemyname::{EnemyName, EnemyNameError};
pub use crate::enemy::data::enemypicturebook::{EnemyPictureBook, EnemyPictureBookError};

/// The comprehensive, fully-aggregated representation of an Enemy unit.
///
/// Unlike Cats, Enemies are structurally simpler as they only possess a single
/// form and lack complex progression systems (like XP curves or evolutions).
/// This structure serves as an aggregate root. It combines localized strings,
/// statistical battle arrays, and extracted animation data into a single,
/// cohesive payload.
///
/// This struct is inherently designed to be serialized (typically to JSON) as the
/// final output state of the data extraction pipeline.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Unit {
    /// The base mathematical identifier for the enemy, corresponding to its directory and file prefixes.
    pub id: u32,
    /// The parsed, localized display name of the enemy.
    /// Evaluates to `None` if the enemy is a placeholder (e.g., "ダミー") or lacks a unique name.
    pub name: Option<String>,
    /// A parsed array containing the multi-line flavor text and lore explanation for the enemy.
    /// Evaluates to `None` if the enemy lacks a dictionary entry.
    pub description: Option<Vec<String>>,
    /// The raw mechanical combat data, hitboxes, and active abilities.
    /// Wrapped in an `Option` to gracefully handle missing or corrupted rows in the raw data matrix.
    pub battle: Option<Battle>,
    /// The absolute duration of the primary attack animation, parsed chronologically from the `maanim` sequence data.
    pub attack_frames: Option<i32>,
}