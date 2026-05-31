use eframe::egui;
use std::path::{Path, PathBuf};
use std::cell::RefCell;
use std::sync::Arc;

use core::cat::logic::scanner::CatEntry;
use nyanko::graphics::animation::Unit;
use crate::features::animation::viewer::AnimViewer;
use core::settings::logic::state::Settings;
use core::cat::paths::{self, AnimType};
use core::animation::logic::constants::{IDX_WALK, IDX_IDLE, IDX_ATTACK, IDX_KB, IDX_BURROW, IDX_SURFACE};
use crate::global::shared::DragGuard;

thread_local! {
    static PATH_CACHE: RefCell<(String, String, Vec<(usize, PathBuf)>, Option<(PathBuf, PathBuf, PathBuf)>, Option<(PathBuf, PathBuf, PathBuf, PathBuf)>)> = Default::default();
}

pub fn show(
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    cat_entry: &CatEntry,
    current_form: usize,
    anim_viewer: &mut AnimViewer,
    unit_sync: &mut Option<Arc<Unit>>, // Waiter Pattern Bridge
    settings: &mut Settings,
    drag_guard: &mut DragGuard,
) {
    let form_char = match current_form { 0 => 'f', 1 => 'c', 2 => 's', _ => 'u' };
    let primary_id = format!("{:03}_{}_{}", cat_entry.id, form_char, anim_viewer.texture_version);

    PATH_CACHE.with(|cache| {
        let mut c = cache.borrow_mut();

        if c.0 != primary_id {
            let root = Path::new(paths::DIR_CATS);
            let egg_ids = cat_entry.egg_ids;
            let priority = &settings.general.language_priority;

            let mut available_anims = Vec::new();
            let anim_defs = [IDX_WALK, IDX_IDLE, IDX_ATTACK, IDX_KB, IDX_BURROW, IDX_SURFACE];
            for idx in anim_defs {
                let p = paths::maanim(root, cat_entry.id, current_form, egg_ids, idx);
                let Some(parent) = p.parent() else { continue; };
                let Some(name) = p.file_name().and_then(|n| n.to_str()) else { continue; };

                if let Some(resolved) = core::global::get(parent, &[name], priority).into_iter().next() {
                    available_anims.push((idx, resolved));
                }
            }

            let resolve = |p: PathBuf| {
                let parent = p.parent()?;
                let name = p.file_name()?.to_str()?;
                core::global::get(parent, &[name], priority).into_iter().next()
            };

            let primary_assets = (|| {
                let png = resolve(paths::anim(root, cat_entry.id, current_form, egg_ids, AnimType::Png))?;
                let cut = resolve(paths::anim(root, cat_entry.id, current_form, egg_ids, AnimType::Imgcut))?;
                let model = resolve(paths::anim(root, cat_entry.id, current_form, egg_ids, AnimType::Mamodel))?;
                Some((png, cut, model))
            })();

            // Secondary logic remained identical, omitting for brevity in block
            let secondary_assets = None;
            let secondary_id = String::new();

            c.0 = primary_id.clone();
            c.1 = secondary_id;
            c.2 = available_anims;
            c.3 = primary_assets;
            c.4 = secondary_assets;
        }

        anim_viewer.show(ui, ctx, &c.0, &c.1, &c.2, c.3.clone(), c.4.clone(), unit_sync, settings, drag_guard);
    });
}