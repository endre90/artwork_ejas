//! Screen 3: today's statuses and weights, the solve, and the result.

use ejas_core::api::{SolveStatus, StationDims, WeightPreset};
use ejas_core::validate;

use crate::app::EjasApp;
use crate::fileio;
use crate::state::{OperatorStatus, WeightMode};

pub fn show(app: &mut EjasApp, ui: &mut egui::Ui) {
    egui::Panel::left("operators_panel")
        .default_size(430.0)
        .resizable(true)
        .show(ui, |ui| {
            egui::ScrollArea::vertical()
                .id_salt("operators_scroll")
                .show(ui, |ui| {
                    leader(app, ui);
                    ui.add_space(8.0);
                    ui.separator();
                    operators(app, ui);
                });
        });

    egui::CentralPanel::default().show(ui, |ui| {
        egui::ScrollArea::vertical()
            .id_salt("results_scroll")
            .show(ui, |ui| {
                weights(app, ui);
                ui.add_space(10.0);
                ui.separator();
                solve_button(app, ui);
                ui.add_space(10.0);
                results(app, ui);
            });
    });
}

/// Today's team leader.
///
/// This is the only place it is set. Leaders rotate, so it belongs to the day
/// rather than to the roster, and the server takes this over whatever role the
/// loaded matrix file happens to carry.
fn leader(app: &mut EjasApp, ui: &mut egui::Ui) {
    super::section(ui, "Team leader today", |ui| {
        ui.label(
            egui::RichText::new(
                "Leaders rotate, so this is chosen per day rather than on the roster.",
            )
            .weak(),
        );
    });
    ui.add_space(6.0);

    let names: Vec<String> = app
        .problem
        .station
        .people
        .iter()
        .map(|e| e.name.clone())
        .collect();

    let mut selected = app.daily.team_leader.clone();
    egui::ComboBox::from_id_salt("todays_leader")
        .selected_text(selected.clone().unwrap_or_else(|| "— choose —".to_owned()))
        .width(220.0)
        .show_ui(ui, |ui| {
            ui.selectable_value(&mut selected, None, "— nobody —");
            for name in &names {
                ui.selectable_value(&mut selected, Some(name.clone()), name);
            }
        });
    if selected != app.daily.team_leader {
        app.daily.team_leader = selected;
        app.outcome = None;
    }

    // The same check that gates the solve button, shown where it is fixable.
    for problem in validate::validate_leader(&app.problem.station, app.daily.team_leader.as_deref())
    {
        ui.colored_label(
            ui.visuals().error_fg_color,
            format!("⚠ {}", problem.message()),
        );
    }
}

fn operators(app: &mut EjasApp, ui: &mut egui::Ui) {
    super::section(ui, "Today's operators", |ui| {
        ui.label(
            egui::RichText::new(
                "Mark anyone not available for normal work, and force an assignment \
                 where you have to.",
            )
            .weak(),
        );
    });
    ui.add_space(6.0);

    let jobs = app.problem.jobs();
    let names: Vec<String> = app
        .problem
        .station
        .people
        .iter()
        .map(|e| e.name.clone())
        .collect();

    for name in names {
        ui.push_id(&name, |ui| {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(&name).size(18.0).strong());

                    let is_leader = app.daily.team_leader.as_deref() == Some(name.as_str());
                    if ui
                        .selectable_label(is_leader, "TL")
                        .on_hover_text("Make this person today's team leader")
                        .clicked()
                    {
                        app.daily.team_leader =
                            if is_leader { None } else { Some(name.clone()) };
                        app.outcome = None;
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let mut status = app.daily.status(&name);
                        egui::ComboBox::from_id_salt(("status", &name))
                            .selected_text(status.label())
                            .show_ui(ui, |ui| {
                                for option in OperatorStatus::ALL {
                                    ui.selectable_value(&mut status, option, option.label());
                                }
                            });
                        if status != app.daily.status(&name) {
                            app.daily.statuses.insert(name.clone(), status);
                            if status != OperatorStatus::Available {
                                // A forced job is meaningless for someone who
                                // is not working today.
                                app.daily.forced_assignments.remove(&name);
                            }
                            app.outcome = None;
                        }
                    });
                });

                if app.daily.status(&name) == OperatorStatus::Available {
                    forced_row(app, ui, &name, &jobs);
                }
            });
        });
        ui.add_space(3.0);
    }
}

fn forced_row(app: &mut EjasApp, ui: &mut egui::Ui, name: &str, jobs: &[String]) {
    // Only jobs this person is competent for can sensibly be forced.
    let competent: Vec<String> = app
        .problem
        .station
        .people
        .iter()
        .find(|e| e.name == name)
        .map(|e| {
            jobs.iter()
                .filter(|j| e.competences.contains(j))
                .cloned()
                .collect()
        })
        .unwrap_or_default();

    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Force job:").weak());
        let current = app
            .daily
            .forced_assignments
            .get(name)
            .cloned()
            .unwrap_or_else(|| "—".to_owned());
        let mut selection = current.clone();

        egui::ComboBox::from_id_salt(("forced", name))
            .selected_text(&selection)
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut selection, "—".to_owned(), "—");
                for job in &competent {
                    ui.selectable_value(&mut selection, job.clone(), job);
                }
            });

        if selection != current {
            if selection == "—" {
                app.daily.forced_assignments.remove(name);
            } else {
                app.daily
                    .forced_assignments
                    .insert(name.to_owned(), selection);
            }
            app.outcome = None;
        }

        if competent.is_empty() {
            ui.label(egui::RichText::new("(no competences)").weak());
        }
    });
}

fn weights(app: &mut EjasApp, ui: &mut egui::Ui) {
    super::section(ui, "Weights", |ui| {
        ui.horizontal_wrapped(|ui| {
            for preset in WeightPreset::ALL {
                let selected = app.weights.mode == WeightMode::Preset(preset);
                if ui
                    .selectable_label(selected, preset.label())
                    .on_hover_text(preset.description())
                    .clicked()
                {
                    app.weights.mode = WeightMode::Preset(preset);
                    app.outcome = None;
                }
            }
            let manual = app.weights.mode == WeightMode::Manual;
            if ui
                .selectable_label(manual, "Manual")
                .on_hover_text("Set every weight yourself.")
                .clicked()
            {
                let station = app.problem.station.clone();
                app.weights.switch_to_manual(&station);
                app.outcome = None;
            }
        });
    });

    ui.add_space(6.0);

    match app.weights.mode {
        WeightMode::Preset(preset) => {
            ui.label(egui::RichText::new(preset.description()).weak());
            ui.add_space(4.0);
            // Show the numbers even for a preset: a label alone does not tell
            // anyone what the solver is actually going to do.
            let dims = StationDims::of(&app.problem.station);
            let p = preset.params(dims, app.weights.inputs);
            egui::Grid::new("preset_values")
                .num_columns(2)
                .spacing([18.0, 4.0])
                .show(ui, |ui| {
                    for (label, value) in rows(&p) {
                        ui.label(egui::RichText::new(label).weak());
                        ui.label(value);
                        ui.end_row();
                    }
                });
            ui.add_space(6.0);
            // The weights are computed, not chosen, so show what they were
            // computed from - otherwise the numbers above look arbitrary.
            ui.label(
                egui::RichText::new(format!(
                    "derived from N={} operators, M={} jobs, ergonomic scores {}..{}",
                    dims.n, dims.m, dims.e_min, dims.e_max
                ))
                .weak(),
            );
            ui.add_space(4.0);
            let mut inputs = app.weights.inputs;
            let mut changed = false;
            egui::Grid::new("preset_inputs")
                .num_columns(3)
                .spacing([14.0, 6.0])
                .show(ui, |ui| {
                    changed |= drag(ui, "dₗᵢₘ  rotate after", &mut inputs.d_limit, 1..=60,
                        "Consecutive days on one job after which rotating off it becomes mandatory.");
                    changed |= drag(ui, "K  external ÷ leader", &mut inputs.k, 1..=100,
                        "How many times more undesirable an external operator is than using the team leader. Sets β = K × α.");
                    changed |= drag(ui, "τ  history days", &mut inputs.tau, 1..=365,
                        "How many recent days the fairness term looks back over.");
                });
            if changed {
                app.weights.inputs = inputs;
                app.outcome = None;
            }
        }
        WeightMode::Manual => {
            let p = &mut app.weights.manual;
            let mut changed = false;
            egui::Grid::new("manual_values")
                .num_columns(3)
                .spacing([14.0, 6.0])
                .show(ui, |ui| {
                    changed |= drag(
                        ui,
                        "offset",
                        &mut p.offset,
                        0..=100_000,
                        "Added to the score so it reads as a positive number.",
                    );
                    changed |= drag(
                        ui,
                        "ω  preference",
                        &mut p.omega,
                        0..=1000,
                        "Reward for giving someone a job they asked for.",
                    );
                    changed |= drag(
                        ui,
                        "α  leader",
                        &mut p.alpha,
                        0..=10_000,
                        "Penalty for putting the team leader on a job.",
                    );
                    changed |= drag(
                        ui,
                        "β  external",
                        &mut p.beta,
                        0..=10_000,
                        "Penalty for needing an operator from outside the station.",
                    );
                    changed |= drag(
                        ui,
                        "τ  history days",
                        &mut p.tau,
                        0..=365,
                        "How many recent days the fairness term looks back over.",
                    );
                    changed |= drag(
                        ui,
                        "γ  repetition",
                        &mut p.gamma,
                        0..=10_000,
                        "Penalty for repeating a recent person-job pairing.",
                    );
                });
            changed |= ui
                .checkbox(
                    &mut p.use_ergo_multiplier,
                    "scale repetition by job ergonomics",
                )
                .on_hover_text(
                    "On: repeating a physically hard job is penalised more than \
                     repeating an easy one (× E_max - E_j + 1). Off: γ is a pure \
                     boredom penalty, as in Happiness-first.",
                )
                .changed();
            if changed {
                app.outcome = None;
            }

            let available = app.problem.history.len();
            if p.tau as usize > available {
                ui.colored_label(
                    ui.visuals().warn_fg_color,
                    format!(
                        "⚠ τ is {} but only {available} day(s) of history are loaded; \
                         the fairness term will use everything available.",
                        p.tau
                    ),
                );
            }
        }
    }
}

fn rows(p: &ejas_core::api::SolverParams) -> [(&'static str, String); 7] {
    [
        ("offset", p.offset.to_string()),
        ("ω  preference", p.omega.to_string()),
        ("α  leader", p.alpha.to_string()),
        ("β  external", p.beta.to_string()),
        ("τ  history days", p.tau.to_string()),
        ("γ  repetition", p.gamma.to_string()),
        (
            "ergonomic multiplier",
            if p.use_ergo_multiplier { "on" } else { "off" }.to_owned(),
        ),
    ]
}

fn drag(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut u32,
    range: std::ops::RangeInclusive<u32>,
    help: &str,
) -> bool {
    ui.label(label);
    let changed = ui.add(egui::DragValue::new(value).range(range)).changed();
    ui.label(egui::RichText::new(help).weak());
    ui.end_row();
    changed
}

fn solve_button(app: &mut EjasApp, ui: &mut egui::Ui) {
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        let busy = app.pending.is_in_flight();
        let blockers = app.blocking_problems();
        let ready = blockers.is_empty() && !busy;

        let button = egui::Button::new(
            egui::RichText::new(if busy { "Solving…" } else { "Calculate assignments" })
                .size(19.0),
        )
        .min_size(egui::vec2(240.0, 40.0));

        let response = ui.add_enabled(ready, button);
        if response.clicked() {
            app.start_solve(ui.ctx());
        }
        if !blockers.is_empty() {
            response.on_hover_text(blockers.join("\n"));
        }

        if busy {
            ui.spinner();
        }
    });
}

fn results(app: &mut EjasApp, ui: &mut egui::Ui) {
    let Some(outcome) = &app.outcome else {
        ui.label(
            egui::RichText::new("Press Calculate to produce today's assignment.")
                .size(16.0)
                .italics(),
        );
        return;
    };

    match outcome.response.status {
        SolveStatus::Unsat => {
            ui.colored_label(
                ui.visuals().error_fg_color,
                "No assignment satisfies these constraints. Try relaxing a forced \
                 assignment, or check that enough competent operators are available.",
            );
            return;
        }
        SolveStatus::Unknown => {
            ui.colored_label(
                ui.visuals().warn_fg_color,
                "The solver ran out of time before proving an optimum. Raise the \
                 timeout or simplify the problem.",
            );
            return;
        }
        SolveStatus::Sat => {}
    }

    ui.columns(2, |columns| {
        assignments(outcome, &mut columns[0]);
        metrics(outcome, &mut columns[1]);
    });

    ui.add_space(12.0);
    ui.separator();
    export_row(app, ui);
}

fn assignments(outcome: &crate::app::Outcome, ui: &mut egui::Ui) {
    ui.label(egui::RichText::new("Assignments").size(20.0).strong());
    ui.add_space(6.0);
    egui::Grid::new("results_grid")
        .striped(true)
        .spacing([30.0, 8.0])
        .show(ui, |ui| {
            ui.label(egui::RichText::new("Operator").strong());
            ui.label(egui::RichText::new("Job").strong());
            ui.end_row();
            for (operator, job) in &outcome.rows {
                ui.label(operator);
                ui.label(job);
                ui.end_row();
            }
        });
    ui.add_space(6.0);
    ui.label(
        egui::RichText::new("L loaned · E absent · T training · S supervision · TL team leader")
            .weak()
            .small(),
    );
}

fn metrics(outcome: &crate::app::Outcome, ui: &mut egui::Ui) {
    let r = &outcome.response;
    ui.label(egui::RichText::new("Metrics").size(20.0).strong());
    ui.add_space(6.0);

    egui::Grid::new("metrics_grid")
        .num_columns(2)
        .spacing([18.0, 4.0])
        .show(ui, |ui| {
            ui.label(egui::RichText::new("Total score").strong());
            ui.label(egui::RichText::new(r.objective_score.to_string()).strong());
            ui.end_row();
            for (label, value) in [
                ("Preference reward", r.weighted_preference_reward_score),
                ("Leader penalty", r.weighted_leader_penalty_score),
                ("External penalty", r.weighted_external_penalty_score),
                ("History penalty", r.weighted_historical_penalty_score),
            ] {
                ui.label(label);
                ui.label(value.to_string());
                ui.end_row();
            }
            ui.label("Solver time");
            ui.label(format!("{} ms", r.solving_time_ms));
            ui.end_row();
        });

    ui.add_space(14.0);
    ui.label(
        egui::RichText::new("Needs external operator")
            .strong()
            .color(egui::Color32::from_rgb(220, 80, 80)),
    );
    if outcome.uncovered_jobs.is_empty() {
        ui.label(egui::RichText::new("None — all jobs covered").italics());
    } else {
        for job in &outcome.uncovered_jobs {
            ui.label(format!("• {job}"));
        }
    }

    ui.add_space(14.0);
    ui.label(
        egui::RichText::new("Free internal operators")
            .strong()
            .color(egui::Color32::from_rgb(80, 180, 80)),
    );
    if outcome.free_operators.is_empty() {
        ui.label(egui::RichText::new("None — everyone is placed").italics());
    } else {
        for operator in &outcome.free_operators {
            ui.label(format!("• {operator}"));
        }
    }
}

fn export_row(app: &mut EjasApp, ui: &mut egui::Ui) {
    ui.add_space(6.0);
    ui.horizontal(|ui| {
        if ui.button("Download result JSON").clicked() {
            if let Some(json) = crate::export::result_json(app) {
                fileio::save_json("assignment.json", json);
            }
        }

        if ui
            .button("Append to history")
            .on_hover_text(
                "Adds today's result to the loaded history, so the next solve \
                 rotates work away from the people who just did it.",
            )
            .clicked()
        {
            append_to_history(app);
        }
    });
}

fn append_to_history(app: &mut EjasApp) {
    let Some(outcome) = &app.outcome else { return };
    let day = crate::export::as_day(app, outcome);
    app.problem.history.push(day);
    app.problem.history_source = Some("session".to_owned());
    app.notice = Some(format!(
        "Appended. History now holds {} day(s).",
        app.problem.history.len()
    ));
    // The next solve should start from a clean slate.
    app.outcome = None;
}
