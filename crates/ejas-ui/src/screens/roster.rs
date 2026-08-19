//! Screen 2: build or edit the station — jobs, then people.
//!
//! This is both the setup wizard and the editor for a loaded file; there is
//! no useful difference between the two once the data is in memory.

use ejas_core::structs::{Employee, Role};
use ejas_core::validate;

use crate::app::EjasApp;
use crate::fileio;
use crate::load;
use crate::state::Screen;

pub fn show(app: &mut EjasApp, ui: &mut egui::Ui) {
    egui::Panel::left("jobs_panel")
        .default_size(320.0)
        .resizable(true)
        .show(ui, |ui| {
            egui::ScrollArea::vertical()
                .id_salt("jobs_scroll")
                .show(ui, |ui| jobs(app, ui));
        });

    egui::Panel::bottom("roster_actions").show(ui, |ui| actions(app, ui));

    egui::CentralPanel::default().show(ui, |ui| {
        egui::ScrollArea::vertical()
            .id_salt("people_scroll")
            .show(ui, |ui| people(app, ui));
    });
}

fn jobs(app: &mut EjasApp, ui: &mut egui::Ui) {
    super::section(ui, "Jobs", |ui| {
        ui.label(
            egui::RichText::new(
                "Each job has an ergonomic score: higher means physically harder.",
            )
            .weak(),
        );
    });
    ui.add_space(6.0);

    let mut remove: Option<String> = None;
    for job in app.problem.jobs() {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(&job).monospace().size(16.0));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button("✖").on_hover_text("Remove job").clicked() {
                    remove = Some(job.clone());
                }
                if let Some(score) = app.problem.station.ergo_score.get_mut(&job) {
                    ui.add(egui::DragValue::new(score).range(1..=10).speed(0.1));
                    ui.label("ergo");
                }
            });
        });
    }

    if let Some(job) = remove {
        app.problem.station.ergo_score.remove(&job);
        // A job that no longer exists must not linger in anyone's lists.
        for person in &mut app.problem.station.people {
            person.competences.retain(|c| c != &job);
            person.preferences.retain(|p| p != &job);
        }
        app.daily.forced_assignments.retain(|_, v| v != &job);
        app.outcome = None;
    }

    ui.add_space(8.0);
    ui.horizontal(|ui| {
        let response = ui.add(
            egui::TextEdit::singleline(&mut app.new_job)
                .hint_text("New job id, e.g. O1")
                .desired_width(140.0),
        );
        let submitted =
            response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
        if ui.button("Add job").clicked() || submitted {
            add_job(app);
        }
    });
}

fn add_job(app: &mut EjasApp) {
    let id = app.new_job.trim().to_owned();
    if id.is_empty() {
        return;
    }
    if app.problem.station.ergo_score.contains_key(&id) {
        app.error = Some(format!("Job '{id}' already exists."));
        return;
    }
    app.problem.station.ergo_score.insert(id, 1);
    app.new_job.clear();
    app.error = None;
    app.outcome = None;
}

fn people(app: &mut EjasApp, ui: &mut egui::Ui) {
    let jobs = app.problem.jobs();

    super::section(ui, "Employees", |ui| {
        if jobs.is_empty() {
            ui.label(
                egui::RichText::new("Add at least one job before adding employees.").weak(),
            );
        }
    });

    let mut remove: Option<usize> = None;
    for index in 0..app.problem.station.people.len() {
        ui.push_id(index, |ui| {
            ui.group(|ui| {
                employee_row(app, ui, index, &jobs, &mut remove);
            });
        });
        ui.add_space(4.0);
    }

    if let Some(index) = remove {
        app.problem.station.people.remove(index);
        app.daily.reconcile(&app.problem.station);
        app.outcome = None;
    }

    ui.add_space(8.0);
    ui.horizontal(|ui| {
        let response = ui.add(
            egui::TextEdit::singleline(&mut app.new_employee)
                .hint_text("New employee name")
                .desired_width(200.0),
        );
        let submitted =
            response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
        if ui.add_enabled(!jobs.is_empty(), egui::Button::new("Add employee")).clicked()
            || (submitted && !jobs.is_empty())
        {
            add_employee(app);
        }
    });

    problems(app, ui);
}

fn add_employee(app: &mut EjasApp) {
    let name = app.new_employee.trim().to_owned();
    if name.is_empty() {
        return;
    }
    if app.problem.station.people.iter().any(|e| e.name == name) {
        app.error = Some(format!("'{name}' is already on the roster."));
        return;
    }
    app.problem.station.people.push(Employee {
        name,
        role: Role::Operator,
        competences: Vec::new(),
        preferences: Vec::new(),
    });
    app.new_employee.clear();
    app.error = None;
    app.daily.reconcile(&app.problem.station);
    app.outcome = None;
}

fn employee_row(
    app: &mut EjasApp,
    ui: &mut egui::Ui,
    index: usize,
    jobs: &[String],
    remove: &mut Option<usize>,
) {
    let mut dirty = false;
    let person = &mut app.problem.station.people[index];

    ui.horizontal(|ui| {
        ui.add(
            egui::TextEdit::singleline(&mut person.name)
                .desired_width(140.0)
                .font(egui::TextStyle::Heading),
        );

        egui::ComboBox::from_id_salt(("role", index))
            .selected_text(match person.role {
                Role::TeamLeader => "Team leader",
                Role::Operator => "Operator",
            })
            .show_ui(ui, |ui| {
                dirty |= ui
                    .selectable_value(&mut person.role, Role::Operator, "Operator")
                    .changed();
                dirty |= ui
                    .selectable_value(&mut person.role, Role::TeamLeader, "Team leader")
                    .changed();
            });

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.small_button("✖").on_hover_text("Remove employee").clicked() {
                *remove = Some(index);
            }
        });
    });

    ui.add_space(4.0);
    ui.label(egui::RichText::new("Competences").weak());
    ui.horizontal_wrapped(|ui| {
        for job in jobs {
            let mut has = person.competences.contains(job);
            if ui.checkbox(&mut has, job).changed() {
                if has {
                    person.competences.push(job.clone());
                } else {
                    person.competences.retain(|c| c != job);
                    // Preferring a job you cannot do is meaningless.
                    person.preferences.retain(|p| p != job);
                }
                dirty = true;
            }
        }
    });

    ui.add_space(4.0);
    ui.label(
        egui::RichText::new("Preferences (most wanted first — order matters)").weak(),
    );

    let mut swap: Option<(usize, usize)> = None;
    let mut drop_pref: Option<usize> = None;
    for (rank, job) in person.preferences.clone().iter().enumerate() {
        ui.horizontal(|ui| {
            ui.label(format!("{}.", rank + 1));
            ui.label(egui::RichText::new(job).monospace());
            if ui.add_enabled(rank > 0, egui::Button::new("▲").small()).clicked() {
                swap = Some((rank, rank - 1));
            }
            if ui
                .add_enabled(
                    rank + 1 < person.preferences.len(),
                    egui::Button::new("▼").small(),
                )
                .clicked()
            {
                swap = Some((rank, rank + 1));
            }
            if ui.small_button("✖").clicked() {
                drop_pref = Some(rank);
            }
        });
    }
    if let Some((a, b)) = swap {
        person.preferences.swap(a, b);
        dirty = true;
    }
    if let Some(rank) = drop_pref {
        person.preferences.remove(rank);
        dirty = true;
    }

    // Only competences that are not already preferred can be added.
    let addable: Vec<String> = person
        .competences
        .iter()
        .filter(|c| !person.preferences.contains(c))
        .cloned()
        .collect();
    if !addable.is_empty() {
        ui.horizontal(|ui| {
            ui.label("Add preference:");
            egui::ComboBox::from_id_salt(("add_pref", index))
                .selected_text("choose…")
                .show_ui(ui, |ui| {
                    for job in &addable {
                        if ui.selectable_label(false, job).clicked() {
                            person.preferences.push(job.clone());
                            dirty = true;
                        }
                    }
                });
        });
    }

    if dirty {
        app.outcome = None;
    }
}

fn problems(app: &mut EjasApp, ui: &mut egui::Ui) {
    let found = validate::validate_station(&app.problem.station);
    if found.is_empty() {
        return;
    }
    ui.add_space(12.0);
    ui.separator();
    super::section(ui, "Checks", |ui| {
        for problem in &found {
            let (icon, color) = if problem.is_error() {
                ("⛔", ui.visuals().error_fg_color)
            } else {
                ("⚠", ui.visuals().warn_fg_color)
            };
            ui.colored_label(color, format!("{icon} {}", problem.message()));
        }
    });
}

fn actions(app: &mut EjasApp, ui: &mut egui::Ui) {
    ui.add_space(6.0);
    ui.horizontal(|ui| {
        // A multi-station file stays switchable without reloading.
        if app.all_stations.len() > 1 {
            let mut ids: Vec<String> = app.all_stations.keys().cloned().collect();
            ids.sort();
            let mut selected = app.problem.station_id.clone();
            egui::ComboBox::from_id_salt("station_pick")
                .selected_text(&selected)
                .show_ui(ui, |ui| {
                    for id in &ids {
                        ui.selectable_value(&mut selected, id.clone(), id);
                    }
                });
            if selected != app.problem.station_id {
                let station = app.all_stations[&selected].clone();
                app.set_station(selected, station);
            }
            ui.separator();
        } else {
            ui.label("Station id:");
            ui.add(
                egui::TextEdit::singleline(&mut app.problem.station_id).desired_width(120.0),
            );
            ui.separator();
        }

        if ui.button("Export matrix JSON").clicked() {
            let json = load::matrix_json(&app.problem.station_id, &app.problem.station);
            fileio::save_json(&format!("{}_matrix.json", app.problem.station_id), json);
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let blockers = app.blocking_problems();
            let ready = blockers.is_empty();
            let button = egui::Button::new(egui::RichText::new("Continue to Today →").size(17.0));
            let response = ui.add_enabled(ready, button);
            if response.clicked() {
                app.daily.reconcile(&app.problem.station);
                app.screen = Screen::Daily;
            }
            if !ready {
                response.on_hover_text(blockers.join("\n"));
            }
        });
    });
    ui.add_space(6.0);
}
