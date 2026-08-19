//! Screen 1: where does the roster come from?

use ejas_core::structs::Station;

use crate::app::{EjasApp, PickerIntent};
use crate::load;
use crate::state::Screen;

pub fn show(app: &mut EjasApp, ui: &mut egui::Ui) {
    egui::CentralPanel::default().show(ui, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add_space(10.0);
            ui.label(
                egui::RichText::new("Choose how to get your competences and preferences in.")
                    .size(16.0),
            );
            ui.add_space(16.0);

            ui.columns(3, |columns| {
                load_card(app, &mut columns[0]);
                wizard_card(app, &mut columns[1]);
                example_card(app, &mut columns[2]);
            });

            ui.add_space(20.0);
            ui.separator();
            history_section(app, ui);
            server_section(app, ui);
        });
    });
}

fn load_card(app: &mut EjasApp, ui: &mut egui::Ui) {
    ui.group(|ui| {
        ui.set_min_height(160.0);
        ui.vertical(|ui| {
            ui.label(egui::RichText::new("Load a matrix file").strong().size(18.0));
            ui.add_space(6.0);
            ui.label(
                "A JSON file describing the station's jobs, ergonomic scores, employees, \
                 competences and preferences.",
            );
            ui.add_space(10.0);
            if ui.button("Choose file…").clicked() {
                app.picker_intent = PickerIntent::Matrix;
                app.picker.open(ui.ctx());
            }
            ui.add_space(4.0);
            ui.label(egui::RichText::new("…or drag one onto this window.").weak());
        });
    });
}

fn wizard_card(app: &mut EjasApp, ui: &mut egui::Ui) {
    ui.group(|ui| {
        ui.set_min_height(160.0);
        ui.vertical(|ui| {
            ui.label(egui::RichText::new("Set up from scratch").strong().size(18.0));
            ui.add_space(6.0);
            ui.label(
                "Define the jobs, then add employees with their competences and \
                 preferences. You can export the result as a matrix file to reuse later.",
            );
            ui.add_space(10.0);
            if ui.button("Start setup").clicked() {
                app.all_stations.clear();
                app.set_station("Station".to_owned(), Station {
                    ergo_score: Default::default(),
                    people: Vec::new(),
                });
                app.error = None;
                app.notice = Some("Add your jobs first, then the employees.".to_owned());
                app.screen = Screen::Roster;
            }
        });
    });
}

fn example_card(app: &mut EjasApp, ui: &mut egui::Ui) {
    ui.group(|ui| {
        ui.set_min_height(160.0);
        ui.vertical(|ui| {
            ui.label(egui::RichText::new("Try an example").strong().size(18.0));
            ui.add_space(6.0);
            ui.label("Real anonymised rosters, useful for seeing what the tool does.");
            ui.add_space(10.0);
            for (name, text) in load::EXAMPLES {
                if ui.button(*name).clicked() {
                    match load::parse_matrix(text) {
                        Ok(matrix) => {
                            let mut ids: Vec<&String> = matrix.stations.keys().collect();
                            ids.sort();
                            if let Some(id) = ids.first().cloned().cloned() {
                                let station = matrix.stations[&id].clone();
                                app.set_station(id, station);
                                app.all_stations = matrix.stations;
                                app.error = None;
                                app.notice = Some(format!("Loaded the {name} example."));
                                app.screen = Screen::Roster;
                            }
                        }
                        // Examples are compiled in, so this is a build problem.
                        Err(e) => app.error = Some(format!("bundled example is broken: {e}")),
                    }
                }
            }
        });
    });
}

fn history_section(app: &mut EjasApp, ui: &mut egui::Ui) {
    super::section(ui, "Assignment history (optional)", |ui| {
        ui.label(
            "History is what makes the fairness term work: it stops the same person \
             getting the same job day after day. Both the day-based and pass-based \
             file formats are accepted.",
        );
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            if ui.button("Load history…").clicked() {
                app.picker_intent = PickerIntent::History;
                app.picker.open(ui.ctx());
            }
            if !app.problem.history.is_empty() {
                let source = app
                    .problem
                    .history_source
                    .clone()
                    .unwrap_or_else(|| "session".to_owned());
                ui.label(format!(
                    "{} day(s) from {source}",
                    app.problem.history.len()
                ));
                if ui.button("Clear").clicked() {
                    app.problem.history.clear();
                    app.problem.history_source = None;
                }
            }
        });
    });
}

fn server_section(app: &mut EjasApp, ui: &mut egui::Ui) {
    // On the web the solver is the same origin and there is nothing to
    // configure; natively it is worth showing where requests are going.
    if cfg!(target_arch = "wasm32") {
        return;
    }
    super::section(ui, "Solver server", |ui| {
        ui.horizontal(|ui| {
            ui.label("URL:");
            ui.text_edit_singleline(&mut app.base_url);
            ui.label(egui::RichText::new("(override with EJAS_SERVER)").weak());
        });
    });
}
