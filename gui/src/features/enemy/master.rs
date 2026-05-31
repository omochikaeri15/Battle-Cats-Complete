use eframe::egui;
use core::enemy::logic::scanner::{self, EnemyEntry};
use crate::features::enemy::state::EnemyDetailTab;
use core::settings::logic::Settings;
use core::enemy::registry::Magnification;
use crate::global::sheet::GuiSpriteSheet;
use nyanko::graphics::animation::Unit;
use std::sync::Arc;
use crate::features::animation::viewer::AnimViewer;
use crate::global::assets::CustomAssets;
use nyanko::common::Param;
use core::global::context::GlobalContext;
use core::enemy::logic::context::EnemyRenderContext;
use crate::features::statblock::builder::{generate_and_copy, generate_and_save};
use crate::features::enemy::statblock::build_enemy_statblock;
use crate::global::shared::DragGuard;
use super::{header, stats, abilities, details, viewer};
use super::header::ExportAction;

pub fn show(
    ctx: &egui::Context,
    ui: &mut egui::Ui,
    enemy_entry: &EnemyEntry,
    current_tab: &mut EnemyDetailTab,
    mag_input: &mut String,
    magnification: &mut Magnification,
    img015_sheets: &mut Vec<GuiSpriteSheet>,
    unit_sync: &mut Option<Arc<Unit>>, // Swapped Model/Sheet for Rig
    anim_viewer: &mut AnimViewer,
    settings: &mut Settings,
    detail_texture: &mut Option<egui::TextureHandle>,
    detail_key: &mut String,
    param: &Param,
    assets: &CustomAssets,
    drag_guard: &mut DragGuard,
) {
    crate::global::img015::ensure_loaded(ctx, img015_sheets, settings);

    let export_action = header::render(
        ctx, ui, enemy_entry, current_tab, mag_input, magnification, detail_texture, detail_key,
    );

    let dynamic_entry = scanner::scan_single(enemy_entry.id, &settings.scanner_config());
    let stats = dynamic_entry.as_ref().map(|e| &e.stats).unwrap_or(&enemy_entry.stats);

    let global_ctx = GlobalContext { param };

    let enemy_ctx = EnemyRenderContext {
        global: global_ctx,
        stats,
        magnification: *magnification,
    };

    match export_action {
        ExportAction::Copy | ExportAction::Save => {
            let data = build_enemy_statblock(&enemy_ctx, enemy_entry);

            let priority_clone = settings.general.language_priority.clone();
            let mut cuts_clone = std::collections::HashMap::new();
            for sheet in img015_sheets.iter().rev() {
                cuts_clone.extend(sheet.core.cuts_map.clone());
            }

            if export_action == ExportAction::Copy {
                generate_and_copy(ctx.clone(), priority_clone, data, cuts_clone);
            } else {
                generate_and_save(ctx.clone(), priority_clone, data, cuts_clone);
            }
        },
        ExportAction::None => {}
    }

    ui.separator();
    ui.add_space(0.0);

    // FIX: Clear the new held_unit structure
    if *current_tab != EnemyDetailTab::Animation {
        if !anim_viewer.loaded_id.is_empty() {
            anim_viewer.held_unit = None;
            anim_viewer.current_anim = None;
            anim_viewer.loaded_id.clear();
            *unit_sync = None;
        }
    }

    match current_tab {
        EnemyDetailTab::Abilities => {
            stats::render(ui, enemy_entry, *magnification);
            ui.spacing_mut().item_spacing.y = 7.0;
            ui.separator();
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    abilities::render(
                        ui,
                        &enemy_ctx,
                        img015_sheets,
                        assets
                    );
                });
        },
        EnemyDetailTab::Details => {
            details::render(ui, &enemy_entry.description);
        },
        EnemyDetailTab::Animation => {
            viewer::show(ui, ctx, enemy_entry, anim_viewer, unit_sync, settings, drag_guard);
        }
    }
}