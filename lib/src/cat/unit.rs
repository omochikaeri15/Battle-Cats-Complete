//! Core data aggregation module for in-game entities.
//!
//! This module acts as the primary data facade, re-exporting the specialized
//! parsers, structures, and error types required to process a unit's mechanical
//! and visual data.

pub use crate::cat::data::unitexplanation::{UnitExplanation, UnitExplanationError};
pub use crate::cat::data::unitid::{Battle, BattleError};
pub use crate::cat::data::unitlevel::{LevelCurve, LevelError};
pub use crate::cat::data::skillacquisition::{Talent, TalentGroup, SkillAcquisitionError};
pub use crate::cat::data::unitbuy::{UnitBuy, UnitBuyError};
pub use crate::cat::data::unitevolve::{UnitEvolve, UnitEvolveError};
pub use crate::cat::data::skilllevel::{TalentCost, SkillLevelError};
pub use crate::cat::data::skilldescriptions::{SkillDescriptions, SkillDescriptionsError};

use std::collections::BTreeMap;

/// The comprehensive, fully-aggregated representation of a Cat unit.
///
/// Because the game's native architecture scatters a single unit's definition across
/// multiple discrete localized and mechanical files, this structure serves as an
/// aggregate root. It combines localized strings, statistical battle arrays,
/// experience progression curves, and extracted animation data into a single,
/// cohesive payload.
///
/// This struct is inherently designed to be serialized (typically to JSON) as the
/// final output state of the data extraction pipeline.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Unit {
    /// The base mathematical identifier for the unit, corresponding to its directory and file prefixes.
    pub id: u32,
    /// A fixed array containing the unit's display names across all 4 potential evolutionary forms.
    /// Evaluates to `None` if the form does not exist or lacks a unique name.
    pub names: [Option<String>; 4],
    /// A fixed array containing the multi-line flavor text and lore explanations for all 4 forms.
    /// Evaluates to `None` if the form does not exist or lacks unique text.
    pub descriptions: [Option<Vec<String>>; 4],
    /// The raw mechanical combat data, hitboxes, and active abilities, mapped strictly to each of the 4 forms.
    /// Missing forms evaluate to `None`.
    pub battle: [Option<Battle>; 4],
    /// The absolute duration of the primary attack animation, parsed chronologically from the `maanim` sequence data.
    pub attack_frames: [i32; 4],
    /// The numeric IDs mapping this unit to its base and evolved unhatched egg counterparts.
    /// Evaluates to `None` if this unit is not linked to the egg system.
    pub egg_ids: Option<(i32, i32)>,
    /// The mathematical progression curve dictating the required XP to reach subsequent levels.
    pub level_curve: Option<LevelCurve>,
    /// The collection of unlockable NP upgrades and abilities for the unit's True/Ultra forms.
    pub talents: Option<Talent>,
    /// The financial and progression prerequisites, including deployment cost, cooldown frames, and unlock conditions.
    pub unitbuy: UnitBuy,
    /// Localized instructional text detailing the specific items and XP required to evolve the unit.
    pub evolve_text: [Vec<String>; 4],
    /// A deterministically sorted dictionary associating specific, raw talent integer IDs with their escalating NP upgrade costs.
    pub talent_costs: BTreeMap<u8, TalentCost>,
    /// Localized, human-readable descriptions explaining the mechanical function of the unit's assigned skills.
    pub skill_descriptions: Vec<String>,
}