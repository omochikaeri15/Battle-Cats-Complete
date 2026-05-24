use std::path::Path;
use std::collections::HashMap;
use eframe::egui;

use core::stage::data::stage::{BossType, EnemyAmount};
use core::global::utils::autocrop;
use core::stage::registry::Stage;
use core::enemy::logic::scanner::EnemyEntry;

use super::treasure::center_header;

// --- FORMATTERS ---

fn format_enemy_amount(spawn_amount: &EnemyAmount) -> String {
    match spawn_amount {
        EnemyAmount::Infinite => "∞".to_string(),
        EnemyAmount::Limit(limited_amount) => limited_amount.to_string(),
    }
}

fn format_enemy_respawn(spawn_amount: &EnemyAmount, respawn_min_frames: u32, respawn_max_frames: u32) -> String {
    let is_singular_enemy_spawn = spawn_amount == &EnemyAmount::Limit(1);
    if is_singular_enemy_spawn {
        return "-".to_string();
    }

    if respawn_min_frames == respawn_max_frames {
        return format!("{}f", respawn_min_frames);
    }

    format!("{}f ~ {}f", respawn_min_frames, respawn_max_frames)
}

fn format_layer(layer_min: i32, layer_max: i32) -> String {
    if layer_min == layer_max {
        return layer_min.to_string();
    }
    format!("{} ~ {}", layer_min, layer_max)
}

fn format_boss_type(boss_type: &BossType) -> String {
    match boss_type {
        BossType::None => "-".to_string(),
        BossType::Boss => "Yes".to_string(),
        BossType::ScreenShake => "Yes (Shake)".to_string(),
        BossType::Unknown(_) => "Unknown".to_string(),
    }
}

fn format_kill_count(kill_count: u32) -> String {
    if kill_count == 0 {
        return "-".to_string();
    }
    kill_count.to_string()
}

fn format_score(score: u32) -> String {
    if score == 0 {
        return "-".to_string();
    }
    score.to_string()
}

fn format_base_hp_percentage(base_hp_percentage: u32, is_dojo_mechanic: bool) -> String {
    if is_dojo_mechanic {
        return base_hp_percentage.to_string();
    }

    if base_hp_percentage == 100 {
        return "-".to_string();
    }
    format!("{}%", base_hp_percentage)
}

fn process_enemy_icon_texture(icon_file_path: &Path) -> Option<egui::ColorImage> {
    let Ok(loaded_raw_image_data) = image::open(icon_file_path) else {
        return None;
    };

    let autocropped_rgba_image = autocrop(loaded_raw_image_data.to_rgba8());
    let image_dimensions = [autocropped_rgba_image.width() as usize, autocropped_rgba_image.height() as usize];

    Some(egui::ColorImage::from_rgba_unmultiplied(image_dimensions, autocropped_rgba_image.as_flat_samples().as_slice()))
}

fn center_enemy_text(ui: &mut egui::Ui, display_text: impl Into<String>) {
    ui.centered_and_justified(|ui| {
        ui.add(egui::Label::new(display_text.into()).wrap_mode(egui::TextWrapMode::Extend));
    });
}

// --- MAIN UI DRAW LOOP ---

pub fn draw(
    egui_context: &egui::Context,
    ui: &mut egui::Ui,
    stage_data: &Stage,
    enemy_registry: &HashMap<u32, EnemyEntry>,
    enemy_name_registry: &[String],
    texture_cache: &mut HashMap<u32, egui::TextureHandle>
) {
    ui.strong("Battleground");
    ui.separator();

    if stage_data.enemies.is_empty() {
        ui.label("No enemies defined for this stage.");
        return;
    }

    let show_score_column = stage_data.enemies.iter().any(|e| e.score > 0);
    let is_dojo_mechanic = stage_data.enemies.iter().any(|e| e.base_hp_perc > 100);

    egui::Grid::new("enemy_grid")
        .striped(true)
        .spacing([15.0, 4.0])
        .min_row_height(32.0)
        .show(ui, |grid| {
            center_header(grid, "Enemy");
            center_header(grid, "Count");
            center_header(grid, "HP %");
            center_header(grid, "Atk %");
            center_header(grid, if is_dojo_mechanic { "Dmg #" } else { "Base %" });
            center_header(grid, "Spawn");
            center_header(grid, "Respawn");
            center_header(grid, "Layer");
            center_header(grid, "Boss");
            if show_score_column {
                center_header(grid, "Score");
            }
            center_header(grid, "Kills");
            grid.end_row();

            for enemy_data in &stage_data.enemies {
                let resolved_enemy_name = enemy_name_registry
                    .get(enemy_data.id as usize)
                    .filter(|s| !s.is_empty())
                    .cloned()
                    .unwrap_or_else(|| format!("{:03}-E", enemy_data.id));

                grid.with_layout(egui::Layout::bottom_up(egui::Align::Center), |icon_layout| {
                    let has_rendered_icon = 'icon: {
                        let Some(located_enemy_entry) = enemy_registry.get(&enemy_data.id) else { break 'icon false; };
                        let Some(enemy_icon_path) = &located_enemy_entry.icon_path else { break 'icon false; };

                        if !texture_cache.contains_key(&enemy_data.id) {
                            let Some(processed_color_image) = process_enemy_icon_texture(enemy_icon_path) else { break 'icon false; };
                            let generated_texture_handle = egui_context.load_texture(format!("stage_enemy_icon_{}", enemy_data.id), processed_color_image, egui::TextureOptions::LINEAR);
                            texture_cache.insert(enemy_data.id, generated_texture_handle);
                        }

                        let Some(cached_texture_handle) = texture_cache.get(&enemy_data.id) else { break 'icon false; };
                        let image_response = icon_layout.add(egui::Image::new(cached_texture_handle).max_size(egui::vec2(32.0, 32.0)));
                        image_response.on_hover_text(resolved_enemy_name.clone());
                        true
                    };

                    if !has_rendered_icon {
                        icon_layout.add_space(6.0);
                        let label_response = icon_layout.add(egui::Label::new(format!("{:03}", enemy_data.id)).wrap_mode(egui::TextWrapMode::Extend));
                        label_response.on_hover_text(resolved_enemy_name);
                    }
                });

                let formatted_amount = format_enemy_amount(&enemy_data.amount);
                let formatted_base_hp = format_base_hp_percentage(enemy_data.base_hp_perc, is_dojo_mechanic);
                let formatted_respawn = format_enemy_respawn(&enemy_data.amount, enemy_data.respawn_min, enemy_data.respawn_max);
                let formatted_layer = format_layer(enemy_data.layer_min, enemy_data.layer_max);
                let formatted_boss_type = format_boss_type(&enemy_data.boss_type);
                let formatted_score = format_score(enemy_data.score);
                let formatted_kill_count = format_kill_count(enemy_data.kill_count);

                center_enemy_text(grid, formatted_amount);
                center_enemy_text(grid, format!("{}%", enemy_data.magnification));
                center_enemy_text(grid, format!("{}%", enemy_data.atk_magnification));
                center_enemy_text(grid, formatted_base_hp);
                center_enemy_text(grid, format!("{}f", enemy_data.start_frame));
                center_enemy_text(grid, formatted_respawn);
                center_enemy_text(grid, formatted_layer);
                center_enemy_text(grid, formatted_boss_type);
                if show_score_column {
                    center_enemy_text(grid, formatted_score);
                }
                center_enemy_text(grid, formatted_kill_count);

                grid.end_row();
            }
        });
}