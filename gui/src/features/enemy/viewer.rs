use eframe::egui;
use std::path::{Path, PathBuf};
use std::cell::RefCell;
use std::sync::Arc;

use core::enemy::logic::scanner::EnemyEntry;
use nyanko::animation::engine::Unit;
use crate::features::animation::viewer::AnimViewer;
use core::settings::logic::state::Settings;
use core::enemy::paths::{self, AnimType};
use core::animation::constants::{IDX_WALK, IDX_IDLE, IDX_ATTACK, IDX_KB, IDX_BURROW, IDX_SURFACE};
use crate::global::shared::DragGuard;

thread_local! {
    static PATH_CACHE: RefCell<(String, Vec<(usize, PathBuf)>, Option<(PathBuf, PathBuf, PathBuf)>)> = Default::default();
}

pub fn show(
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    enemy_entry: &EnemyEntry,
    anim_viewer: &mut AnimViewer,
    unit_sync: &mut Option<Arc<Unit>>, // Waiter Pattern Bridge
    settings: &mut Settings,
    drag_guard: &mut DragGuard,
) {
    let primary_id = format!("{}_{}", enemy_entry.id_str(), anim_viewer.texture_version);

    PATH_CACHE.with(|cache| {
        let mut c = cache.borrow_mut();

        if c.0 != primary_id {
            let root = Path::new(paths::DIR_ENEMIES);
            let priority = &settings.general.language_priority;
            let mut available_anims = Vec::new();

            let resolve = |p: PathBuf| {
                let parent = p.parent()?;
                let name = p.file_name()?.to_str()?;
                let iname = format!("i{}", name);
                core::global::get(parent, &[name, &iname], priority).into_iter().next()
            };

            for idx in [IDX_WALK, IDX_IDLE, IDX_ATTACK, IDX_KB] {
                if let Some(path) = resolve(paths::maanim(root, enemy_entry.id, idx)) {
                    available_anims.push((idx, path));
                }
            }

            if let Some(p) = resolve(paths::zombie_maanim(root, enemy_entry.id, 0)) { available_anims.push((IDX_BURROW, p)); }
            if let Some(p) = resolve(paths::zombie_maanim(root, enemy_entry.id, 1)) { available_anims.push((7, p)); }
            if let Some(p) = resolve(paths::zombie_maanim(root, enemy_entry.id, 2)) { available_anims.push((IDX_SURFACE, p)); }

            let primary_assets = (|| {
                let png = resolve(paths::anim(root, enemy_entry.id, AnimType::Png))?;
                let cut = resolve(paths::anim(root, enemy_entry.id, AnimType::Imgcut))?;
                let model = resolve(paths::anim(root, enemy_entry.id, AnimType::Mamodel))?;
                Some((png, cut, model))
            })();

            c.0 = primary_id.clone();
            c.1 = available_anims;
            c.2 = primary_assets;
        }

        anim_viewer.show(ui, ctx, &c.0, &String::new(), &c.1, c.2.clone(), None, unit_sync, settings, drag_guard);
    });
}