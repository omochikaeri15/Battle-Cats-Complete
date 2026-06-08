pub use crate::enemy::data::t_unit::{Battle, BattleError};
pub use crate::enemy::data::enemyname::{EnemyName, EnemyNameError};
pub use crate::enemy::data::enemypicturebook::{EnemyPictureBook, EnemyPictureBookError};

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Unit {
    pub id: u16,
    pub name: Option<String>,
    pub description: Option<Vec<String>>,
    pub battle: Option<Battle>,
    pub attack_frames: Option<i32>,
}