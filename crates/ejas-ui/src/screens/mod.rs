//! The three screens plus the chrome shared between them.

pub mod daily;
pub mod data_source;
pub mod roster;

use crate::app::EjasApp;
use crate::state::Screen;

/// Navigation and the current station name.
pub fn top_bar(app: &mut EjasApp, ui: &mut egui::Ui) {
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.heading("Ergonomic Assigner");
        ui.separator();

        let has_station = !app.problem.station.people.is_empty()
            || !app.problem.station.ergo_score.is_empty();

        ui.selectable_value(&mut app.screen, Screen::DataSource, "1. Data");
        ui.add_enabled_ui(has_station, |ui| {
            ui.selectable_value(&mut app.screen, Screen::Roster, "2. Roster");
            ui.selectable_value(&mut app.screen, Screen::Daily, "3. Today");
        });

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if has_station {
                ui.label(
                    egui::RichText::new(format!(
                        "{} · {} operators · {} jobs",
                        app.problem.station_id,
                        app.problem.station.people.len(),
                        app.problem.station.ergo_score.len()
                    ))
                    .weak(),
                );
            }
        });
    });
    ui.add_space(4.0);
}

/// Errors, notices and where the solver is.
pub fn status_bar(app: &mut EjasApp, ui: &mut egui::Ui) {
    ui.add_space(2.0);
    ui.horizontal(|ui| {
        if let Some(error) = app.error.clone() {
            ui.colored_label(ui.visuals().error_fg_color, format!("⚠ {error}"));
            if ui.small_button("dismiss").clicked() {
                app.error = None;
            }
        } else if let Some(notice) = app.notice.clone() {
            ui.label(egui::RichText::new(notice).weak());
            if ui.small_button("dismiss").clicked() {
                app.notice = None;
            }
        } else {
            let history = app.problem.history.len();
            ui.label(
                egui::RichText::new(if history == 0 {
                    "No history loaded — the fairness term has nothing to work with.".to_owned()
                } else {
                    format!("{history} day(s) of history loaded.")
                })
                .weak(),
            );
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(egui::RichText::new(server_label(app)).weak());
        });
    });
    ui.add_space(2.0);
}

fn server_label(app: &EjasApp) -> String {
    if app.base_url.is_empty() {
        "solver: same origin".to_owned()
    } else {
        format!("solver: {}", app.base_url)
    }
}

/// A labelled section with a little breathing room.
pub fn section(ui: &mut egui::Ui, title: &str, body: impl FnOnce(&mut egui::Ui)) {
    ui.add_space(8.0);
    ui.label(egui::RichText::new(title).strong().size(17.0));
    ui.add_space(4.0);
    body(ui);
}
