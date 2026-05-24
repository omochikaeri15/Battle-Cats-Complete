use eframe::egui;
use crate::global::shared::DragGuard;
use core::settings::logic::state::ModsSettings;
use super::tabs::toggle_ui;

pub fn show(ui_container: &mut egui::Ui, settings: &mut ModsSettings, drag_guard: &mut DragGuard) -> bool {
    let context = ui_container.ctx().clone();
    let mut refresh_needed = false;

    egui::ScrollArea::vertical()
        .id_salt("mods_settings_scroll")
        .auto_shrink([false, true])
        .show(ui_container, |scroll_ui| {
            scroll_ui.heading("Export");
            scroll_ui.add_space(5.0);

            let manage_pem_button = egui::Button::new("Manage PEM")
                .fill(egui::Color32::from_rgb(40, 90, 160));

            if scroll_ui.add_sized([180.0, 30.0], manage_pem_button).clicked() {
                crate::features::settings::pem::open(&context);
            }

            scroll_ui.add_space(10.0);

            scroll_ui.horizontal(|ui| {
                let label_response = ui.label("Replace on Update Patch");
                let tooltip_text = "Replace the original input file instead of creating an updated copy in the exports folder";
                label_response.on_hover_text(tooltip_text);

                let toggle_response = toggle_ui(ui, &mut settings.replace_on_update).on_hover_text(tooltip_text);
                if toggle_response.changed() { refresh_needed = true; }
            });

            scroll_ui.add_space(20.0);
        });

    crate::features::settings::pem::show(&context, drag_guard);

    refresh_needed
}