//! Core domain definitions for enemy abilities and combat mechanics.
//!
//! This module bridges the raw, byte-centric data stored in the `Battle` struct
//! with dynamically evaluated combat mechanics. It strictly maps raw values to
//! strongly-typed attributes and handles engine quirks.

use crate::enemy::unit::Battle;
use crate::common::data::img015;

/// Represents the mathematical or logical unit of measurement for an ability's attribute.
///
/// Because the game data stores all values as flat integers (e.g., `50`), this enum provides
/// the necessary context to determine whether that integer represents a percentage, a
/// frame count, a spatial distance, or a raw numerical value. This is critical for both
/// accurate calculations and UI formatting.
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum AttrUnit {
    /// A raw integer value. Used for standard counts (e.g., number of burrows) or flat numerical values.
    None,
    /// A percentage modifier (0-100+). Used for probability chances, damage multipliers, and stat reductions.
    Percent,
    /// A temporal measurement defined in engine ticks (30 frames = 1 second). Used for effect durations.
    Frames,
    /// A spatial measurement defined in engine coordinate units. Used for attack ranges, spawn anchors, and widths.
    Range,
}

/// A comprehensive enumeration acting as the unique domain identifier for every known
/// trait, ability, immunity, and stat modifier belonging to enemies.
///
/// This serves as a strongly-typed key, avoiding the need to pass around string
/// comparisons or raw, contextless integer IDs when checking an enemy's capabilities.
#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum Identity {
    SingleAttack, AreaAttack, TypeRed, TypeFloating, TypeDark, TypeMetal, TypeAngel,
    TypeAlien, TypeZombie, TypeRelic, TypeAku, TypeTraitless, TypeDojo, TypeStarredAlien,
    TypeCatGod, TypeColossus, TypeBehemoth, TypeSage, TypeSupervillain, TypeWitch, TypeEva,
    Kamikaze, Stop, BaseDestroyer, WaveBlock, MultiHit, LongDistance, OmniStrike,
    WaveAttack, MiniWave, SurgeAttack, MiniSurge, DeathSurge, Explosion, CriticalHit,
    SavageBlow, Strengthen, Survive, Barrier, AkuShield, Burrow, Revive, Toxic, Drain,
    Dodge, Weaken, Freeze, Slow, Knockback, Curse, Warp, Unknown, ImmuneWave,
    ImmuneSurge, ImmuneExplosion, ImmuneWeaken, ImmuneFreeze, ImmuneSlow, ImmuneKnockback,
    ImmuneCurse, ImmuneWarp, CounterSurge
}

/// The pure domain definition of an enemy combat mechanic (Ability, Trait, or Immunity).
///
/// This structure bridges the gap between raw, static game data and dynamically evaluated
/// combat mechanics. It dictates how an ability is identified, its mathematical schema,
/// and how its raw attributes are dynamically extracted from the core `Battle` stat block.
pub struct Ability {
    /// The strongly-typed domain identifier for this ability.
    pub identity: Identity,
    /// An optional reference to the sprite index in the `img015` icon atlas used for UI rendering.
    pub icon_id: Option<usize>,
    /// The human-readable display name of the ability.
    pub name: &'static str,
    /// A descriptive string detailing the ability's mechanical behavior.
    pub description: &'static str,
    /// A localized schema outlining the expected parameters for this ability.
    /// Pairs a descriptive string (e.g., "Duration") with its expected `AttrUnit` (e.g., `AttrUnit::Frames`).
    pub schema: &'static [(&'static str, AttrUnit)],
    /// A closure that evaluates an enemy's `Battle` stat block and dynamically extracts the localized
    /// values for this specific ability, returning them mapped to their schema names and units.
    pub attributes: fn(&Battle) -> Vec<(&'static str, i32, AttrUnit)>,
    /// An engine-specific flag indicating whether a raw value of `-1` should be mathematically
    /// treated as "infinite" (e.g., infinite burrows or infinite revives).
    pub minus_one_is_inf: bool,
}

/// Locates and returns a static reference to an `Ability` definition based on its domain identity.
///
/// # Arguments
/// * `identity` - The `Identity` enum variant representing the target ability.
///
/// # Returns
/// An `Option` containing a static reference to the corresponding `Ability`, or `None` if the identity is somehow unmapped.
pub fn get_ability(identity: Identity) -> Option<&'static Ability> {
    REGISTRY.iter().find(|ability| ability.identity == identity)
}

/// The global, statically allocated registry containing the domain definitions for every known
/// enemy ability, trait, immunity, and stat modifier in the game.
pub static REGISTRY: &[Ability] = &[
    Ability {
        identity: Identity::SingleAttack,
        icon_id: Some(img015::ICON_SINGLE_ATTACK),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| if stats.area_attack == 0 { vec![("Active", 1, AttrUnit::None)] } else { vec![] },
        minus_one_is_inf: false,
    },
    Ability {
        identity: Identity::AreaAttack,
        icon_id: Some(img015::ICON_AREA_ATTACK),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| if stats.area_attack == 1 { vec![("Active", 1, AttrUnit::None)] } else { vec![] },
        minus_one_is_inf: false,
    },
    Ability {
        identity: Identity::TypeRed,
        icon_id: Some(img015::ICON_TRAIT_RED),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| if stats.type_red > 0 { vec![("Active", 1, AttrUnit::None)] } else { vec![] },
        minus_one_is_inf: false,
    },
    Ability {
        identity: Identity::TypeFloating,
        icon_id: Some(img015::ICON_TRAIT_FLOATING),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| if stats.type_floating > 0 { vec![("Active", 1, AttrUnit::None)] } else { vec![] },
        minus_one_is_inf: false,
    },
    Ability {
        identity: Identity::TypeDark,
        icon_id: Some(img015::ICON_TRAIT_BLACK),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| if stats.type_dark > 0 { vec![("Active", 1, AttrUnit::None)] } else { vec![] },
        minus_one_is_inf: false,
    },
    Ability {
        identity: Identity::TypeMetal,
        icon_id: Some(img015::ICON_TRAIT_METAL),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| if stats.type_metal > 0 { vec![("Active", 1, AttrUnit::None)] } else { vec![] },
        minus_one_is_inf: false,
    },
    Ability {
        identity: Identity::TypeAngel,
        icon_id: Some(img015::ICON_TRAIT_ANGEL),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| if stats.type_angel > 0 { vec![("Active", 1, AttrUnit::None)] } else { vec![] },
        minus_one_is_inf: false,
    },
    Ability {
        identity: Identity::TypeAlien,
        icon_id: Some(img015::ICON_TRAIT_ALIEN),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| if stats.type_alien > 0 { vec![("Active", 1, AttrUnit::None)] } else { vec![] },
        minus_one_is_inf: false,
    },
    Ability {
        identity: Identity::TypeZombie,
        icon_id: Some(img015::ICON_TRAIT_ZOMBIE),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| if stats.type_zombie > 0 { vec![("Active", 1, AttrUnit::None)] } else { vec![] },
        minus_one_is_inf: false,
    },
    Ability {
        identity: Identity::TypeRelic,
        icon_id: Some(img015::ICON_TRAIT_RELIC),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| if stats.type_relic > 0 { vec![("Active", 1, AttrUnit::None)] } else { vec![] },
        minus_one_is_inf: false,
    },
    Ability {
        identity: Identity::TypeAku,
        icon_id: Some(img015::ICON_TRAIT_AKU),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| if stats.type_aku > 0 { vec![("Active", 1, AttrUnit::None)] } else { vec![] },
        minus_one_is_inf: false,
    },
    Ability {
        identity: Identity::TypeTraitless,
        icon_id: Some(img015::ICON_TRAIT_TRAITLESS),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| if stats.type_traitless > 0 { vec![("Active", 1, AttrUnit::None)] } else { vec![] },
        minus_one_is_inf: false,
    },
    Ability {
        identity: Identity::TypeDojo,
        icon_id: None,
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| if stats.type_dojo > 0 { vec![("Active", 1, AttrUnit::None)] } else { vec![] },
        minus_one_is_inf: false,
    },
    Ability {
        identity: Identity::TypeStarredAlien,
        icon_id: None,
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| if stats.type_starred_alien == 1 { vec![("Active", 1, AttrUnit::None)] } else { vec![] },
        minus_one_is_inf: false,
    },
    Ability {
        identity: Identity::TypeCatGod,
        icon_id: None,
        name: "",
        description: "",
        schema: &[("Type", AttrUnit::None)],
        attributes: |stats| {
            if stats.type_starred_alien >= 2 && stats.type_starred_alien <= 4 {
                vec![("Type", stats.type_starred_alien, AttrUnit::None)]
            } else { vec![] }
        },
        minus_one_is_inf: false,
    },
    Ability {
        identity: Identity::TypeColossus,
        icon_id: Some(img015::ICON_COLOSSUS),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| if stats.type_colossus > 0 { vec![("Active", 1, AttrUnit::None)] } else { vec![] },
        minus_one_is_inf: false,
    },
    Ability {
        identity: Identity::TypeBehemoth,
        icon_id: Some(img015::ICON_BEHEMOTH),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| if stats.type_behemoth > 0 { vec![("Active", 1, AttrUnit::None)] } else { vec![] },
        minus_one_is_inf: false,
    },
    Ability {
        identity: Identity::TypeSage,
        icon_id: Some(img015::ICON_SAGE),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| if stats.type_sage > 0 { vec![("Active", 1, AttrUnit::None)] } else { vec![] },
        minus_one_is_inf: false,
    },
    Ability {
        identity: Identity::TypeSupervillain,
        icon_id: Some(img015::ICON_SUPERVILLIAN),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| if stats.type_supervillain > 0 { vec![("Active", 1, AttrUnit::None)] } else { vec![] },
        minus_one_is_inf: false,
    },
    Ability {
        identity: Identity::TypeWitch,
        icon_id: Some(img015::ICON_WITCH),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| if stats.type_witch > 0 { vec![("Active", 1, AttrUnit::None)] } else { vec![] },
        minus_one_is_inf: false,
    },
    Ability {
        identity: Identity::TypeEva,
        icon_id: Some(img015::ICON_EVA),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| if stats.type_eva > 0 { vec![("Active", 1, AttrUnit::None)] } else { vec![] },
        minus_one_is_inf: false,
    },
    Ability {
        identity: Identity::Kamikaze,
        icon_id: None,
        name: "",
        description: "",
        schema: &[("Attacks", AttrUnit::None)],
        attributes: |stats| {
            if stats.attack_count_total > -1 && stats.attack_count_state == 2 {
                vec![("Attacks", stats.attack_count_total, AttrUnit::None)]
            } else { vec![] }
        },
        minus_one_is_inf: false,
    },
    Ability {
        identity: Identity::Stop,
        icon_id: None,
        name: "",
        description: "",
        schema: &[("Attacks", AttrUnit::None)],
        attributes: |stats| {
            if stats.attack_count_total > -1 && stats.attack_count_state == 0 {
                vec![("Attacks", stats.attack_count_total, AttrUnit::None)]
            } else { vec![] }
        },
        minus_one_is_inf: false,
    },
    Ability {
        identity: Identity::BaseDestroyer,
        icon_id: Some(img015::ICON_BASE_DESTROYER),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| if stats.base_destroyer > 0 { vec![("Active", 1, AttrUnit::None)] } else { vec![] },
        minus_one_is_inf: false,
    },
    Ability {
        identity: Identity::WaveBlock,
        icon_id: Some(img015::ICON_WAVE_BLOCK),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| if stats.wave_blocker > 0 { vec![("Active", 1, AttrUnit::None)] } else { vec![] },
        minus_one_is_inf: false,
    },
    Ability {
        identity: Identity::MultiHit,
        icon_id: None,
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| if stats.attack_2 > 0 { vec![("Active", 1, AttrUnit::None)] } else { vec![] },
        minus_one_is_inf: false,
    },
    Ability {
        identity: Identity::LongDistance,
        icon_id: Some(img015::ICON_LONG_DISTANCE),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| {
            let has_omni = (stats.long_distance_span_1 < 0 || (stats.long_distance_span_1 == 0 && stats.long_distance_anchor_1 != 0)) ||
                (stats.long_distance_2_flag > 0 && (stats.long_distance_2_span < 0 || (stats.long_distance_2_span == 0 && stats.long_distance_2_anchor != 0))) ||
                (stats.long_distance_3_flag > 0 && (stats.long_distance_3_span < 0 || (stats.long_distance_3_span == 0 && stats.long_distance_3_anchor != 0)));

            let has_ld = (stats.long_distance_span_1 > 0) ||
                (stats.long_distance_2_flag > 0 && stats.long_distance_2_span > 0) ||
                (stats.long_distance_3_flag > 0 && stats.long_distance_3_span > 0);

            if has_ld && !has_omni { vec![("Active", 1, AttrUnit::None)] } else { vec![] }
        },
        minus_one_is_inf: false,
    },
    Ability {
        identity: Identity::OmniStrike,
        icon_id: Some(img015::ICON_OMNI_STRIKE),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| {
            let has_omni = (stats.long_distance_span_1 < 0 || (stats.long_distance_span_1 == 0 && stats.long_distance_anchor_1 != 0)) ||
                (stats.long_distance_2_flag > 0 && (stats.long_distance_2_span < 0 || (stats.long_distance_2_span == 0 && stats.long_distance_2_anchor != 0))) ||
                (stats.long_distance_3_flag > 0 && (stats.long_distance_3_span < 0 || (stats.long_distance_3_span == 0 && stats.long_distance_3_anchor != 0)));

            if has_omni { vec![("Active", 1, AttrUnit::None)] } else { vec![] }
        },
        minus_one_is_inf: false,
    },
    Ability {
        identity: Identity::WaveAttack,
        icon_id: Some(img015::ICON_WAVE),
        name: "",
        description: "",
        schema: &[("Chance", AttrUnit::Percent), ("Level", AttrUnit::None), ("Max Reach", AttrUnit::Range)],
        attributes: |stats| {
            if stats.mini_wave == 0 && stats.wave_chance > 0 {
                let maximum_reach = (467.5 + ((stats.wave_level - 1) as f32 * 200.0)).round() as i32;
                vec![
                    ("Chance", stats.wave_chance, AttrUnit::Percent),
                    ("Level", stats.wave_level, AttrUnit::None),
                    ("Max Reach", maximum_reach, AttrUnit::Range),
                ]
            } else { vec![] }
        },
        minus_one_is_inf: false,
    },
    Ability {
        identity: Identity::MiniWave,
        icon_id: Some(img015::ICON_MINI_WAVE),
        name: "",
        description: "",
        schema: &[("Chance", AttrUnit::Percent), ("Level", AttrUnit::None), ("Max Reach", AttrUnit::Range)],
        attributes: |stats| {
            if stats.mini_wave > 0 && stats.wave_chance > 0 {
                let maximum_reach = (467.5 + ((stats.wave_level - 1) as f32 * 200.0)).round() as i32;
                vec![
                    ("Chance", stats.wave_chance, AttrUnit::Percent),
                    ("Level", stats.wave_level, AttrUnit::None),
                    ("Max Reach", maximum_reach, AttrUnit::Range),
                ]
            } else { vec![] }
        },
        minus_one_is_inf: false,
    },
    Ability {
        identity: Identity::SurgeAttack,
        icon_id: Some(img015::ICON_SURGE),
        name: "",
        description: "",
        schema: &[("Chance", AttrUnit::Percent), ("Level", AttrUnit::None), ("Min Range", AttrUnit::Range), ("Max Range", AttrUnit::Range), ("Width", AttrUnit::Range)],
        attributes: |stats| {
            if stats.mini_surge == 0 && stats.surge_chance > 0 {
                vec![
                    ("Chance", stats.surge_chance, AttrUnit::Percent),
                    ("Level", stats.surge_level, AttrUnit::None),
                    ("Min Range", stats.surge_spawn_min, AttrUnit::Range),
                    ("Max Range", stats.surge_spawn_min + stats.surge_spawn_max, AttrUnit::Range),
                    ("Width", stats.surge_spawn_max, AttrUnit::Range),
                ]
            } else { vec![] }
        },
        minus_one_is_inf: false,
    },
    Ability {
        identity: Identity::MiniSurge,
        icon_id: Some(img015::ICON_MINI_SURGE),
        name: "",
        description: "",
        schema: &[("Chance", AttrUnit::Percent), ("Level", AttrUnit::None), ("Min Range", AttrUnit::Range), ("Max Range", AttrUnit::Range), ("Width", AttrUnit::Range)],
        attributes: |stats| {
            if stats.mini_surge > 0 && stats.surge_chance > 0 {
                vec![
                    ("Chance", stats.surge_chance, AttrUnit::Percent),
                    ("Level", stats.surge_level, AttrUnit::None),
                    ("Min Range", stats.surge_spawn_min, AttrUnit::Range),
                    ("Max Range", stats.surge_spawn_min + stats.surge_spawn_max, AttrUnit::Range),
                    ("Width", stats.surge_spawn_max, AttrUnit::Range),
                ]
            } else { vec![] }
        },
        minus_one_is_inf: false,
    },
    Ability {
        identity: Identity::DeathSurge,
        icon_id: Some(img015::ICON_DEATH_SURGE),
        name: "",
        description: "",
        schema: &[("Chance", AttrUnit::Percent), ("Level", AttrUnit::None), ("Min Range", AttrUnit::Range), ("Max Range", AttrUnit::Range), ("Width", AttrUnit::Range)],
        attributes: |stats| {
            if stats.death_surge_chance > 0 {
                vec![
                    ("Chance", stats.death_surge_chance, AttrUnit::Percent),
                    ("Level", stats.death_surge_level, AttrUnit::None),
                    ("Min Range", stats.death_surge_spawn_min, AttrUnit::Range),
                    ("Max Range", stats.death_surge_spawn_min + stats.death_surge_spawn_max, AttrUnit::Range),
                    ("Width", stats.death_surge_spawn_max, AttrUnit::Range),
                ]
            } else { vec![] }
        },
        minus_one_is_inf: false,
    },
    Ability {
        identity: Identity::Explosion,
        icon_id: Some(img015::ICON_EXPLOSION),
        name: "",
        description: "",
        schema: &[("Chance", AttrUnit::Percent), ("Min Range", AttrUnit::Range), ("Max Range", AttrUnit::Range), ("Width", AttrUnit::Range)],
        attributes: |stats| {
            if stats.explosion_chance > 0 {
                vec![
                    ("Chance", stats.explosion_chance, AttrUnit::Percent),
                    ("Min Range", stats.explosion_anchor, AttrUnit::Range),
                    ("Max Range", stats.explosion_anchor + stats.explosion_span, AttrUnit::Range),
                    ("Width", stats.explosion_span, AttrUnit::Range),
                ]
            } else { vec![] }
        },
        minus_one_is_inf: false,
    },
    Ability {
        identity: Identity::CriticalHit,
        icon_id: Some(img015::ICON_CRITICAL_HIT),
        name: "",
        description: "",
        schema: &[("Chance", AttrUnit::Percent)],
        attributes: |stats| if stats.critical_chance > 0 { vec![("Chance", stats.critical_chance, AttrUnit::Percent)] } else { vec![] },
        minus_one_is_inf: false,
    },
    Ability {
        identity: Identity::SavageBlow,
        icon_id: Some(img015::ICON_SAVAGE_BLOW),
        name: "",
        description: "",
        schema: &[("Chance", AttrUnit::Percent), ("Boost", AttrUnit::Percent)],
        attributes: |stats| {
            if stats.savage_blow_chance > 0 {
                vec![
                    ("Chance", stats.savage_blow_chance, AttrUnit::Percent),
                    ("Boost", stats.savage_blow_boost, AttrUnit::Percent),
                ]
            } else { vec![] }
        },
        minus_one_is_inf: false,
    },
    Ability {
        identity: Identity::Strengthen,
        icon_id: Some(img015::ICON_STRENGTHEN),
        name: "",
        description: "",
        schema: &[("HP", AttrUnit::Percent), ("Boost", AttrUnit::Percent)],
        attributes: |stats| {
            if stats.strengthen_threshold > 0 {
                vec![
                    ("HP", stats.strengthen_threshold, AttrUnit::Percent),
                    ("Boost", stats.strengthen_boost, AttrUnit::Percent),
                ]
            } else { vec![] }
        },
        minus_one_is_inf: false,
    },
    Ability {
        identity: Identity::Survive,
        icon_id: Some(img015::ICON_SURVIVE),
        name: "",
        description: "",
        schema: &[("Chance", AttrUnit::Percent)],
        attributes: |stats| if stats.survive_chance > 0 { vec![("Chance", stats.survive_chance, AttrUnit::Percent)] } else { vec![] },
        minus_one_is_inf: false,
    },
    Ability {
        identity: Identity::Barrier,
        icon_id: Some(img015::ICON_BARRIER),
        name: "",
        description: "",
        schema: &[("Hitpoints", AttrUnit::None)],
        attributes: |stats| if stats.barrier_hitpoints > 0 { vec![("Hitpoints", stats.barrier_hitpoints, AttrUnit::None)] } else { vec![] },
        minus_one_is_inf: false,
    },
    Ability {
        identity: Identity::AkuShield,
        icon_id: Some(img015::ICON_SHIELD),
        name: "",
        description: "",
        schema: &[("Hitpoints", AttrUnit::None), ("Regen", AttrUnit::Percent)],
        attributes: |stats| {
            if stats.shield_hitpoints > 0 {
                vec![
                    ("Hitpoints", stats.shield_hitpoints, AttrUnit::None),
                    ("Regen", stats.shield_regen, AttrUnit::Percent),
                ]
            } else { vec![] }
        },
        minus_one_is_inf: false,
    },
    Ability {
        identity: Identity::Burrow,
        icon_id: None,
        name: "",
        description: "",
        schema: &[("Count", AttrUnit::None), ("Distance", AttrUnit::Range)],
        attributes: |stats| {
            if stats.burrow_amount != 0 {
                vec![
                    ("Count", stats.burrow_amount, AttrUnit::None),
                    ("Distance", stats.burrow_distance, AttrUnit::Range),
                ]
            } else { vec![] }
        },
        minus_one_is_inf: true,
    },
    Ability {
        identity: Identity::Revive,
        icon_id: None,
        name: "",
        description: "",
        schema: &[("Count", AttrUnit::None), ("Duration", AttrUnit::Frames), ("Hitpoints", AttrUnit::Percent)],
        attributes: |stats| {
            if stats.revive_count != 0 {
                vec![
                    ("Count", stats.revive_count, AttrUnit::None),
                    ("Duration", stats.revive_time, AttrUnit::Frames),
                    ("Hitpoints", stats.revive_hp, AttrUnit::Percent),
                ]
            } else { vec![] }
        },
        minus_one_is_inf: true,
    },
    Ability {
        identity: Identity::Toxic,
        icon_id: Some(img015::ICON_TOXIC),
        name: "",
        description: "",
        schema: &[("Chance", AttrUnit::Percent), ("Damage", AttrUnit::Percent)],
        attributes: |stats| {
            if stats.toxic_chance > 0 {
                vec![
                    ("Chance", stats.toxic_chance, AttrUnit::Percent),
                    ("Damage", stats.toxic_damage, AttrUnit::Percent),
                ]
            } else { vec![] }
        },
        minus_one_is_inf: false,
    },
    Ability {
        identity: Identity::Drain,
        icon_id: Some(img015::ICON_DRAIN),
        name: "",
        description: "",
        schema: &[("Chance", AttrUnit::Percent), ("Amount", AttrUnit::Percent)],
        attributes: |stats| {
            if stats.drain_chance > 0 {
                vec![
                    ("Chance", stats.drain_chance, AttrUnit::Percent),
                    ("Amount", stats.drain_percent, AttrUnit::Percent),
                ]
            } else { vec![] }
        },
        minus_one_is_inf: false,
    },
    Ability {
        identity: Identity::Dodge,
        icon_id: Some(img015::ICON_DODGE),
        name: "",
        description: "",
        schema: &[("Chance", AttrUnit::Percent), ("Duration", AttrUnit::Frames)],
        attributes: |stats| {
            if stats.dodge_chance > 0 {
                vec![
                    ("Chance", stats.dodge_chance, AttrUnit::Percent),
                    ("Duration", stats.dodge_duration, AttrUnit::Frames),
                ]
            } else { vec![] }
        },
        minus_one_is_inf: false,
    },
    Ability {
        identity: Identity::Weaken,
        icon_id: Some(img015::ICON_WEAKEN),
        name: "",
        description: "",
        schema: &[("Chance", AttrUnit::Percent), ("Reduced To", AttrUnit::Percent), ("Duration", AttrUnit::Frames)],
        attributes: |stats| {
            if stats.weaken_chance > 0 {
                vec![
                    ("Chance", stats.weaken_chance, AttrUnit::Percent),
                    ("Reduced To", stats.weaken_percent, AttrUnit::Percent),
                    ("Duration", stats.weaken_duration, AttrUnit::Frames),
                ]
            } else { vec![] }
        },
        minus_one_is_inf: false,
    },
    Ability {
        identity: Identity::Freeze,
        icon_id: Some(img015::ICON_FREEZE),
        name: "",
        description: "",
        schema: &[("Chance", AttrUnit::Percent), ("Duration", AttrUnit::Frames)],
        attributes: |stats| {
            if stats.freeze_chance > 0 {
                vec![
                    ("Chance", stats.freeze_chance, AttrUnit::Percent),
                    ("Duration", stats.freeze_duration, AttrUnit::Frames),
                ]
            } else { vec![] }
        },
        minus_one_is_inf: false,
    },
    Ability {
        identity: Identity::Slow,
        icon_id: Some(img015::ICON_SLOW),
        name: "",
        description: "",
        schema: &[("Chance", AttrUnit::Percent), ("Duration", AttrUnit::Frames)],
        attributes: |stats| {
            if stats.slow_chance > 0 {
                vec![
                    ("Chance", stats.slow_chance, AttrUnit::Percent),
                    ("Duration", stats.slow_duration, AttrUnit::Frames),
                ]
            } else { vec![] }
        },
        minus_one_is_inf: false,
    },
    Ability {
        identity: Identity::Knockback,
        icon_id: Some(img015::ICON_KNOCKBACK),
        name: "",
        description: "",
        schema: &[("Chance", AttrUnit::Percent)],
        attributes: |stats| if stats.knockback_chance > 0 { vec![("Chance", stats.knockback_chance, AttrUnit::Percent)] } else { vec![] },
        minus_one_is_inf: false,
    },
    Ability {
        identity: Identity::Curse,
        icon_id: Some(img015::ICON_CURSE),
        name: "",
        description: "",
        schema: &[("Chance", AttrUnit::Percent), ("Duration", AttrUnit::Frames)],
        attributes: |stats| {
            if stats.curse_chance > 0 {
                vec![
                    ("Chance", stats.curse_chance, AttrUnit::Percent),
                    ("Duration", stats.curse_duration, AttrUnit::Frames),
                ]
            } else { vec![] }
        },
        minus_one_is_inf: false,
    },
    Ability {
        identity: Identity::Warp,
        icon_id: Some(img015::ICON_WARP),
        name: "",
        description: "",
        schema: &[("Chance", AttrUnit::Percent), ("Duration", AttrUnit::Frames), ("Min Distance", AttrUnit::Range), ("Max Distance", AttrUnit::Range)],
        attributes: |stats| {
            if stats.warp_chance > 0 {
                vec![
                    ("Chance", stats.warp_chance, AttrUnit::Percent),
                    ("Duration", stats.warp_duration, AttrUnit::Frames),
                    ("Min Distance", stats.warp_distance_minimum, AttrUnit::Range),
                    ("Max Distance", stats.warp_distance_maximum, AttrUnit::Range),
                ]
            } else { vec![] }
        },
        minus_one_is_inf: false,
    },
    Ability {
        identity: Identity::Unknown,
        icon_id: None,
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| if stats.has_unknown_abilities > 0 { vec![("Active", 1, AttrUnit::None)] } else { vec![] },
        minus_one_is_inf: false,
    },
    Ability {
        identity: Identity::ImmuneWave,
        icon_id: Some(img015::ICON_IMMUNE_WAVE),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| if stats.wave_immune > 0 { vec![("Active", 1, AttrUnit::None)] } else { vec![] },
        minus_one_is_inf: false,
    },
    Ability {
        identity: Identity::ImmuneSurge,
        icon_id: Some(img015::ICON_IMMUNE_SURGE),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| if stats.surge_immune > 0 { vec![("Active", 1, AttrUnit::None)] } else { vec![] },
        minus_one_is_inf: false,
    },
    Ability {
        identity: Identity::ImmuneExplosion,
        icon_id: Some(img015::ICON_IMMUNE_EXPLOSION),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| if stats.explosion_immune > 0 { vec![("Active", 1, AttrUnit::None)] } else { vec![] },
        minus_one_is_inf: false,
    },
    Ability {
        identity: Identity::ImmuneWeaken,
        icon_id: Some(img015::ICON_IMMUNE_WEAKEN),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| if stats.weaken_immune > 0 { vec![("Active", 1, AttrUnit::None)] } else { vec![] },
        minus_one_is_inf: false,
    },
    Ability {
        identity: Identity::ImmuneFreeze,
        icon_id: Some(img015::ICON_IMMUNE_FREEZE),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| if stats.freeze_immune > 0 { vec![("Active", 1, AttrUnit::None)] } else { vec![] },
        minus_one_is_inf: false,
    },
    Ability {
        identity: Identity::ImmuneSlow,
        icon_id: Some(img015::ICON_IMMUNE_SLOW),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| if stats.slow_immune > 0 { vec![("Active", 1, AttrUnit::None)] } else { vec![] },
        minus_one_is_inf: false,
    },
    Ability {
        identity: Identity::ImmuneKnockback,
        icon_id: Some(img015::ICON_IMMUNE_KNOCKBACK),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| if stats.knockback_immune > 0 { vec![("Active", 1, AttrUnit::None)] } else { vec![] },
        minus_one_is_inf: false,
    },
    Ability {
        identity: Identity::ImmuneCurse,
        icon_id: Some(img015::ICON_IMMUNE_CURSE),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| if stats.curse_immune > 0 { vec![("Active", 1, AttrUnit::None)] } else { vec![] },
        minus_one_is_inf: false,
    },
    Ability {
        identity: Identity::ImmuneWarp,
        icon_id: Some(img015::ICON_IMMUNE_WARP),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| if stats.warp_immune > 0 { vec![("Active", 1, AttrUnit::None)] } else { vec![] },
        minus_one_is_inf: false,
    },
    Ability {
        identity: Identity::CounterSurge,
        icon_id: Some(img015::ICON_COUNTER_SURGE),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| if stats.counter_surge > 0 { vec![("Active", 1, AttrUnit::None)] } else { vec![] },
        minus_one_is_inf: false,
    },
];