use std::collections::HashMap;
use nyanko::cat::unit::{Battle, LevelCurve};
use nyanko::cat::unit::Talent;
use crate::global::context::GlobalContext;

#[derive(Clone, Copy)]
pub struct CatRenderContext<'a> {
    pub global: GlobalContext<'a>,
    pub base_stats: &'a Battle,
    pub final_stats: &'a Battle,
    pub current_level: i32,
    pub level_curve: Option<&'a LevelCurve>,
    pub talent_data: Option<&'a Talent>,
    pub talent_levels: Option<&'a HashMap<u8, u8>>,
    pub is_conjure_unit: bool,
}