use eframe::egui;
use std::path::Path;
use crate::global::assets;
use crate::global::io::json;
use crate::global::game::param::load_param;
use crate::updater;
use crate::features::settings::logic::{lang, upd::UpdateMode};
use crate::app::BattleCatsApp;
use crate::features::cat::paths as cat_paths;
use crate::features::cat::data::{skilllevel, skilldescriptions};

#[cfg(not(debug_assertions))]
use crate::app::frame::Page;

impl BattleCatsApp {
    pub fn new(creation_context: &eframe::CreationContext<'_>) -> Self {
        let mut app: Self = json::load("settings.json").unwrap_or_default();

        #[cfg(target_os = "linux")]
        {
            let _ = crate::features::settings::logic::desktop::sync_desktop_data();
        }

        #[cfg(not(debug_assertions))]
        if app.current_page == Page::Stages {
            app.current_page = Page::Home;
        }

        lang::ensure_complete_list(&mut app.settings.general.language_priority);

        setup_custom_fonts(&creation_context.egui_ctx);

        app.mod_state.refresh_mods();
        updater::cleanup_temp_files();

        app.param = load_param(Path::new("game/tables"), &app.settings.general.language_priority).unwrap_or_default();

        let mut expected_hash = 0;
        let mut needs_validation = false;
        let priority = &app.settings.general.language_priority;

        if let Some((hash, cached_cats)) = crate::global::io::cache::load_with_hash::<Vec<crate::features::cat::logic::scanner::CatEntry>>("cats_cache.bin") {
            expected_hash = hash;
            needs_validation = true;
            let cats_directory = Path::new(cat_paths::DIR_CATS);
            let costs_arc = std::sync::Arc::new(skilllevel::load(cats_directory, priority));
            let descriptions_arc = std::sync::Arc::new(skilldescriptions::load(cats_directory, priority));

            app.cat_list_state.cats = cached_cats.into_iter().map(|mut cat| {
                cat.talent_costs = std::sync::Arc::clone(&costs_arc);
                cat.skill_descriptions = std::sync::Arc::clone(&descriptions_arc);
                cat
            }).collect();
            app.cat_list_state.initialized = true;
        } else {
            app.cat_list_state.restart_scan(app.settings.scanner_config());
        }

        if let Some((hash, cached_enemies)) = crate::global::io::cache::load_with_hash::<Vec<crate::features::enemy::logic::scanner::EnemyEntry>>("enemies_cache.bin") {
            expected_hash = hash;
            needs_validation = true;
            app.enemy_list_state.enemies = cached_enemies;
            app.enemy_list_state.initialized = true;
        } else {
            app.enemy_list_state.restart_scan(app.settings.scanner_config());
        }

        app.stage_list_state.restart_scan(app.settings.scanner_config());

        if needs_validation {
            let (transmitter, receiver) = std::sync::mpsc::channel();
            app.hash_rx = Some(receiver);
            let active_mod = crate::global::resolver::get_active_mod();

            std::thread::spawn(move || {
                let current_hash = crate::global::io::cache::get_game_hash(active_mod.as_deref());
                let _ = transmitter.send(current_hash == expected_hash && active_mod.is_none());
            });
        }

        if app.settings.general.update_mode != UpdateMode::Ignore {
            app.updater.check_for_updates(creation_context.egui_ctx.clone(), false);
        }

        app
    }
}

fn setup_custom_fonts(context: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert("jp_font".to_owned(), egui::FontData::from_static(assets::FONT_JP));
    fonts.font_data.insert("kr_font".to_owned(), egui::FontData::from_static(assets::FONT_KR));
    fonts.font_data.insert("tc_font".to_owned(), egui::FontData::from_static(assets::FONT_TC));
    fonts.font_data.insert("thai_font".to_owned(), egui::FontData::from_static(assets::FONT_TH));

    let families = [egui::FontFamily::Proportional, egui::FontFamily::Monospace];
    for family in families {
        let Some(list_reference) = fonts.families.get_mut(&family) else { continue; };

        list_reference.push("jp_font".to_owned());
        list_reference.push("kr_font".to_owned());
        list_reference.push("tc_font".to_owned());
        list_reference.push("thai_font".to_owned());
    }
    context.set_fonts(fonts);
}