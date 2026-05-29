use std::collections::HashMap;
use std::path::{Path, PathBuf};
use crate::global::formats::gatyaitembuy::GatyaItemBuy;
use crate::global::formats::gatyaitemname::GatyaItemName;
use nyanko::cat::unit::UnitBuy;
use crate::cat::data::unitexplanation;
use crate::cat::paths;

pub struct ResolvedDrop {
    pub name: String,
    pub image_path: Option<PathBuf>,
    pub amount_display: String,
}

pub fn resolve_drop(
    target_item_id: u32,
    raw_amount: u32,
    item_buy_registry: &HashMap<u32, GatyaItemBuy>,
    item_name_registry: &HashMap<usize, GatyaItemName>,
    drop_chara_registry: &HashMap<u32, u32>,
    unit_buy_registry: &HashMap<u32, UnitBuy>,
    active_language_priority_array: &[String]
) -> ResolvedDrop {

    // =========================================================================
    // 1. Regular Items
    // =========================================================================
    if let Some(located_item_unitbuy) = item_buy_registry.get(&target_item_id) {
        let target_name_row_index = located_item_unitbuy.row_index;
        let name = item_name_registry.get(&target_name_row_index)
            .map(|d| d.name.clone())
            .unwrap_or_else(|| target_item_id.to_string());

        let resolved_image_identifier = if located_item_unitbuy.img_id != -1 {
            located_item_unitbuy.img_id as u32
        } else {
            located_item_unitbuy.row_index as u32
        };

        let gatya_directory_path = PathBuf::from("game/ui/gatyaitemD");
        let gatya_file_name = format!("gatyaitemD_{:02}_f.png", resolved_image_identifier);
        let image_path = crate::global::resolver::get(&gatya_directory_path, &[&gatya_file_name], active_language_priority_array).into_iter().next();

        return ResolvedDrop {
            name,
            image_path,
            amount_display: raw_amount.to_string(),
        };
    }

    // =========================================================================
    // 2. Base Cat Drops (Normal / Evolved form)
    // =========================================================================
    if let Some(&located_chara_id) = drop_chara_registry.get(&target_item_id) {
        let cat_folder = Path::new(paths::DIR_CATS).join(format!("{:03}", located_chara_id));

        // THE WAITER HAND-OFF: Let the unitexplanation module manage the language fallback directories
        let explanation = unitexplanation::load(located_chara_id, &cat_folder, active_language_priority_array);

        let mut name = format!("{}-1", located_chara_id);
        let first_form_name = &explanation.names[0];
        if !first_form_name.is_empty() {
            name = first_form_name.clone();
        }

        let img_directory_path = PathBuf::from(format!("game/cats/{:03}/f", located_chara_id));
        let img_file_name = format!("uni{:03}_f00.png", located_chara_id);
        let image_path = crate::global::resolver::get(&img_directory_path, &[&img_file_name], active_language_priority_array).into_iter().next();

        return ResolvedDrop {
            name,
            image_path,
            amount_display: "-".to_string(),
        };
    }

    // =========================================================================
    // 3. True Form Drops
    // =========================================================================
    if let Some((&unit_id, _)) = unit_buy_registry.iter().find(|(_, row_data)| row_data.true_form_id == target_item_id as i32) {
        let cat_folder = Path::new(paths::DIR_CATS).join(format!("{:03}", unit_id));
        let explanation = unitexplanation::load(unit_id, &cat_folder, active_language_priority_array);
        let mut name = format!("{}-3", unit_id);
        let true_form_name = &explanation.names[2];
        if !true_form_name.is_empty() {
            name = true_form_name.clone();
        }

        let img_directory_path = PathBuf::from(format!("game/cats/{:03}/s", unit_id));
        let img_file_name = format!("uni{:03}_s00.png", unit_id);
        let image_path = crate::global::resolver::get(&img_directory_path, &[&img_file_name], active_language_priority_array).into_iter().next();

        return ResolvedDrop {
            name,
            image_path,
            amount_display: "-".to_string(),
        };
    }

    // Fallback if absolutely nothing matches
    ResolvedDrop {
        name: target_item_id.to_string(),
        image_path: None,
        amount_display: raw_amount.to_string(),
    }
}