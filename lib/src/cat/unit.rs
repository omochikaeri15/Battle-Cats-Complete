pub use crate::cat::data::unitexplanation::{UnitExplanation, UnitExplanationError};
pub use crate::cat::data::unitid::{Battle, BattleError};
pub use crate::cat::data::unitlevel::{LevelCurve, LevelError};
pub use crate::cat::data::skillacquisition::{Talent, TalentGroup, SkillAcquisitionError};
pub use crate::cat::data::unitbuy::{UnitBuy, UnitBuyError};
pub use crate::cat::data::unitevolve::{UnitEvolve, UnitEvolveError};
pub use crate::cat::data::skilllevel::{TalentCost, SkillLevelError};
pub use crate::cat::data::skilldescriptions::{SkillDescriptions, SkillDescriptionsError};

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct CatUnit {
    /// The base numerical identifier for the unit
    pub id: u32,

    /// Localized display names across all forms
    pub names: Vec<String>,

    /// Localized flavor text/explanations across all forms
    pub descriptions: Vec<Vec<String>>,

    /// Tracks which forms (0-3) actually exist in the game files
    pub forms: [bool; 4],

    /// The raw mechanical battle data and abilities
    pub battle: Vec<Option<Battle>>,

    /// Attack animation duration calculated directly from maanim data
    pub attack_frames: [i32; 4],

    /// Base and evolved egg IDs linked to this unit, if applicable
    pub egg_ids: (i32, i32),

    /// XP requirements and level scaling math
    pub level_curve: Option<LevelCurve>,
    //
    /// The unlocked talent abilities (NP upgrades)
    pub talents: Option<Talent>,
    //
    /// Deployment cost, cooldown, and unlock criteria
    pub unitbuy: UnitBuy,
    //
    /// Localized text explaining the evolution requirements
    pub evolve_text: [Vec<String>; 4],
    //
    /// NP costs mapped to specific talent levels
    pub talent_costs: std::collections::HashMap<u8, TalentCost>,
    //
    /// Localized descriptions of the specific skills
    pub skill_descriptions: Vec<String>,
}