use artwork_ejas::{calculate_ergonomic_assignment, Day, Matrix, Pass, PassWrapper};
use eframe::egui;
use std::collections::HashMap;
use std::fs;

// --- IMPORTANT: Bring your backend structs and functions into scope here ---
// use your_crate_name::{Matrix, PassWrapper, Pass, Day, calculate_ergonomic_assignment};
// --------------------------------------------------------------------------

#[derive(PartialEq, Clone, Copy, Debug)]
enum OperatorStatus {
    Available,
    Absent,
    Training,
    Loaned,
    Supervision,
}

// Data structure to hold everything returned/calculated from the algorithm
struct CalculationResults {
    assignments: Vec<(String, String)>,
    pref_score: String,
    lead_score: String,
    exte_score: String,
    hist_ergo_score: String,
    total_score: String,
    solver_time: String,
    missing_operations: Vec<String>,
    free_operators: Vec<String>,
}

struct FactoryApp {
    // Core data
    operators: Vec<String>,
    operations: Vec<String>,

    // State
    statuses: HashMap<String, OperatorStatus>,
    forced_assignments: HashMap<String, String>,
    team_leader: Option<String>,

    // Result
    calculation_results: Option<CalculationResults>,
}

impl Default for FactoryApp {
    fn default() -> Self {
        let operators = (b'A'..=b'K')
            .map(|c| (c as char).to_string())
            .collect::<Vec<_>>();

        let operations = (1..=8).map(|n| format!("O{}", n)).collect::<Vec<_>>();

        let mut statuses = HashMap::new();
        for op in &operators {
            statuses.insert(op.clone(), OperatorStatus::Available);
        }

        Self {
            operators,
            operations,
            statuses,
            forced_assignments: HashMap::new(),
            team_leader: None,
            calculation_results: None,
        }
    }
}

impl eframe::App for FactoryApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 1. TOP PANEL: Title
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add_space(10.0);
            ui.heading(egui::RichText::new("Ergonomic Assigner").size(30.0));
            ui.add_space(10.0);
        });

        // 2. LEFT PANEL: Operator Configuration
        egui::SidePanel::left("left_panel")
            .default_width(450.0)
            .resizable(true)
            .show(ctx, |ui| {
                ui.add_space(10.0);
                ui.heading(egui::RichText::new("Operator Configuration").size(22.0));
                ui.add_space(10.0);

                egui::ScrollArea::vertical()
                    .id_source("ops_scroll")
                    .show(ui, |ui| {
                        for operator in &self.operators {
                            ui.group(|ui| {
                                ui.vertical(|ui| {
                                    ui.horizontal(|ui| {
                                        // Operator Name
                                        ui.label(
                                            egui::RichText::new(format!("Operator {}", operator))
                                                .size(20.0)
                                                .strong(),
                                        );
                                        ui.add_space(20.0);

                                        // Team Leader Toggle
                                        let mut is_tl = self.team_leader.as_ref() == Some(operator);
                                        if ui
                                            .add(egui::SelectableLabel::new(
                                                is_tl,
                                                egui::RichText::new("TL").size(16.0),
                                            ))
                                            .clicked()
                                        {
                                            if is_tl {
                                                self.team_leader = None;
                                            } else {
                                                self.team_leader = Some(operator.clone());
                                            }
                                        }
                                    });

                                    ui.add_space(5.0);

                                    ui.horizontal(|ui| {
                                        // Status Dropdown
                                        ui.label(egui::RichText::new("Status:").size(16.0));
                                        let mut current_status = self.statuses[operator];
                                        egui::ComboBox::from_id_source(format!(
                                            "{}_status",
                                            operator
                                        ))
                                        .width(150.0)
                                        .selected_text(
                                            egui::RichText::new(format!("{:?}", current_status))
                                                .size(16.0),
                                        )
                                        .show_ui(
                                            ui,
                                            |ui| {
                                                ui.selectable_value(
                                                    &mut current_status,
                                                    OperatorStatus::Available,
                                                    egui::RichText::new("Available").size(16.0),
                                                );
                                                ui.selectable_value(
                                                    &mut current_status,
                                                    OperatorStatus::Absent,
                                                    egui::RichText::new("Absent (E)").size(16.0),
                                                );
                                                ui.selectable_value(
                                                    &mut current_status,
                                                    OperatorStatus::Training,
                                                    egui::RichText::new("Training (T)").size(16.0),
                                                );
                                                ui.selectable_value(
                                                    &mut current_status,
                                                    OperatorStatus::Loaned,
                                                    egui::RichText::new("Loaned (L)").size(16.0),
                                                );
                                                ui.selectable_value(
                                                    &mut current_status,
                                                    OperatorStatus::Supervision,
                                                    egui::RichText::new("Supervision (S)")
                                                        .size(16.0),
                                                );
                                            },
                                        );

                                        if current_status != self.statuses[operator] {
                                            self.statuses.insert(operator.clone(), current_status);
                                            if current_status != OperatorStatus::Available {
                                                self.forced_assignments.remove(operator);
                                            }
                                        }
                                    });

                                    // Forced Assignment (Only show if Available)
                                    if self.statuses[operator] == OperatorStatus::Available {
                                        ui.add_space(5.0);
                                        ui.horizontal(|ui| {
                                            ui.label(egui::RichText::new("Force Op:").size(16.0));
                                            let mut current_forced = self
                                                .forced_assignments
                                                .get(operator)
                                                .cloned()
                                                .unwrap_or_else(|| "None".to_string());

                                            egui::ComboBox::from_id_source(format!(
                                                "{}_force",
                                                operator
                                            ))
                                            .width(100.0)
                                            .selected_text(
                                                egui::RichText::new(&current_forced).size(16.0),
                                            )
                                            .show_ui(
                                                ui,
                                                |ui| {
                                                    ui.selectable_value(
                                                        &mut current_forced,
                                                        "None".to_string(),
                                                        egui::RichText::new("None").size(16.0),
                                                    );
                                                    ui.separator();
                                                    for op in &self.operations {
                                                        ui.selectable_value(
                                                            &mut current_forced,
                                                            op.clone(),
                                                            egui::RichText::new(op).size(16.0),
                                                        );
                                                    }
                                                },
                                            );

                                            if current_forced == "None" {
                                                self.forced_assignments.remove(operator);
                                            } else {
                                                self.forced_assignments
                                                    .insert(operator.clone(), current_forced);
                                            }
                                        });
                                    }
                                });
                            });
                            ui.add_space(10.0);
                        }
                    });
            });

        // 3. CENTRAL PANEL: Button and Results (Takes up remaining space)
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(10.0);

            // Calculate Button
            let calculate_btn = egui::Button::new(
                egui::RichText::new("Calculate Assignments")
                    .size(22.0)
                    .strong(),
            )
            .fill(egui::Color32::from_rgb(45, 120, 200))
            .min_size(egui::vec2(0.0, 50.0));

            if ui
                .add_sized([ui.available_width(), 50.0], calculate_btn)
                .clicked()
            {
                self.run_algorithm();
            }

            ui.add_space(20.0);

            // Results Display
            if let Some(results) = &self.calculation_results {
                ui.horizontal(|ui| {
                    // --- COLUMN 2: The Assignment Grid ---
                    ui.allocate_ui_with_layout(
                        egui::vec2(ui.available_width() * 0.5, ui.available_height()),
                        egui::Layout::top_down(egui::Align::LEFT),
                        |ui| {
                            ui.heading(egui::RichText::new("Assignments:").size(24.0));
                            ui.add_space(10.0);

                            // Render the Grid directly so it takes up exactly the space it needs
                            egui::Grid::new("results_grid")
                                .striped(true)
                                .spacing([40.0, 12.0])
                                .show(ui, |ui| {
                                    ui.label(egui::RichText::new("Operator").size(18.0).strong());
                                    ui.label(egui::RichText::new("Task").size(18.0).strong());
                                    ui.end_row();

                                    for (op, task) in &results.assignments {
                                        ui.label(egui::RichText::new(op).size(18.0));
                                        ui.label(egui::RichText::new(task).size(18.0));
                                        ui.end_row();
                                    }
                                });
                        },
                    );

                    ui.add(egui::Separator::default().vertical());
                    ui.add_space(10.0);

                    // --- COLUMN 3: Metrics, Missing Ops, Free Operators ---
                    ui.vertical(|ui| {
                        ui.heading(egui::RichText::new("Algorithm Metrics:").size(24.0));
                        ui.add_space(10.0);

                        // Scores
                        ui.label(
                            egui::RichText::new(format!("Total Score: {}", results.total_score))
                                .size(18.0)
                                .strong(),
                        );
                        ui.label(
                            egui::RichText::new(format!(
                                "Preference Score: {}",
                                results.pref_score
                            ))
                            .size(16.0),
                        );
                        ui.label(
                            egui::RichText::new(format!(
                                "Ergo History Penalty: {}",
                                results.hist_ergo_score
                            ))
                            .size(16.0),
                        );
                        ui.label(
                            egui::RichText::new(format!(
                                "Leadership Penalty: {}",
                                results.lead_score
                            ))
                            .size(16.0),
                        );
                        ui.label(
                            egui::RichText::new(format!(
                                "External Penalty: {}",
                                results.exte_score
                            ))
                            .size(16.0),
                        );
                        ui.label(
                            egui::RichText::new(format!("Solver Time: {}", results.solver_time))
                                .size(16.0),
                        );

                        ui.add_space(20.0);
                        ui.separator();
                        ui.add_space(10.0);

                        // Missing Operations (Need External)
                        ui.label(
                            egui::RichText::new("Needs External Operator:")
                                .size(18.0)
                                .strong()
                                .color(egui::Color32::from_rgb(220, 80, 80)),
                        );
                        if results.missing_operations.is_empty() {
                            ui.label(egui::RichText::new("None - All covered").italics());
                        } else {
                            for op in &results.missing_operations {
                                ui.label(
                                    egui::RichText::new(format!("• Operation {}", op)).size(16.0),
                                );
                            }
                        }

                        ui.add_space(20.0);

                        // Free Operators (Can be loaned)
                        ui.label(
                            egui::RichText::new("Free Internal Operators:")
                                .size(18.0)
                                .strong()
                                .color(egui::Color32::from_rgb(80, 200, 80)),
                        );
                        if results.free_operators.is_empty() {
                            ui.label(egui::RichText::new("None - All assigned").italics());
                        } else {
                            for op in &results.free_operators {
                                ui.label(
                                    egui::RichText::new(format!("• Operator {}", op)).size(16.0),
                                );
                            }
                        }
                    });
                });
            } else {
                ui.label(
                    egui::RichText::new("Click Calculate to see assignments.")
                        .size(16.0)
                        .italics(),
                );
            }
        });
    }
}

impl FactoryApp {
    fn run_algorithm(&mut self) {
        // 1. Extract lists for your algorithm based on UI state
        let mut absent = Vec::new();
        let mut training = Vec::new();
        let mut loaned = Vec::new();
        let mut supervision = Vec::new();

        for (operator, status) in &self.statuses {
            match status {
                OperatorStatus::Absent => absent.push(operator.clone()),
                OperatorStatus::Training => training.push(operator.clone()),
                OperatorStatus::Loaned => loaned.push(operator.clone()),
                OperatorStatus::Supervision => supervision.push(operator.clone()),
                OperatorStatus::Available => {}
            }
        }

        // 2. Format forced assignments into tuples
        let forced_assignments_vec: Vec<(String, String)> = self
            .forced_assignments
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        // 3. Load Data Files (Matrix and History)
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());

        let matrix_path = format!("{}/data/factory/GTO_matrix.json", manifest_dir);
        let matrix_content = fs::read_to_string(matrix_path).expect("Failed to read matrix.json");
        // Ensure Matrix is imported to parse this
        let matrix: Matrix =
            serde_json::from_str(&matrix_content).expect("Failed to parse matrix JSON");

        let history_path = format!("{}/data/factory/GTO_nov_history.json", manifest_dir);
        let history_content =
            fs::read_to_string(history_path).expect("Failed to read history JSON");
        // Ensure PassWrapper/Pass is imported to parse this
        let history_wrapper: Vec<PassWrapper> =
            serde_json::from_str(&history_content).expect("Failed to parse history");
        let full_passes: Vec<Pass> = history_wrapper.into_iter().map(|pw| pw.pass).collect();

        // 4. Algorithm Parameters
        let offset = 2000;
        let omega = 1;
        let alpha = 192;
        let beta = 384;
        let tau: usize = 8;
        let gamma = 24;
        let station_id = "GTO";

        let station_data = matrix
            .stations
            .get(station_id)
            .expect("Station GTO not found in matrix");

        let leader_name = self.team_leader.clone().unwrap_or_default();

        // 5. Slice the history window
        let start_idx = if full_passes.len() > tau {
            full_passes.len() - tau
        } else {
            0
        };
        let history_window_passes = full_passes[start_idx..].to_vec();

        let history_window_days: Vec<Day> = history_window_passes
            .iter()
            .map(|p| Day {
                date: p.date.clone(),
                station: p.station.clone(),
                leader: leader_name.clone(),
                assignments: p.assignments.clone(),
            })
            .collect();

        // 6. Run the algorithm
        let s = calculate_ergonomic_assignment(
            station_data,
            history_window_days,
            offset,
            omega,
            alpha,
            beta,
            tau as u32,
            gamma,
            &forced_assignments_vec,
            &loaned,
            &absent,
            &training,
            &supervision,
        );

        // 7. Reconstruct the full assignment for the UI
        let mut sorted_assignment = s.internal_assignments.to_vec();

        loaned
            .iter()
            .for_each(|x| sorted_assignment.push((x.to_string(), "L".to_string())));
        absent
            .iter()
            .for_each(|x| sorted_assignment.push((x.to_string(), "E".to_string())));
        training
            .iter()
            .for_each(|x| sorted_assignment.push((x.to_string(), "T".to_string())));
        supervision
            .iter()
            .for_each(|x| sorted_assignment.push((x.to_string(), "S".to_string())));

        if !leader_name.is_empty() {
            let is_absent = absent.contains(&leader_name);
            let is_training = training.contains(&leader_name);
            let is_loaned = loaned.contains(&leader_name);
            let is_supervision = supervision.contains(&leader_name);
            let is_on_operation = s
                .internal_assignments
                .iter()
                .any(|(emp, _)| emp == &leader_name);

            if !is_absent && !is_training && !is_loaned && !is_supervision && !is_on_operation {
                sorted_assignment.push((leader_name.clone(), "TL".to_string()));
            }
        }

        // 8. Calculate Missing Operations and Free Operators
        let mut missing_operations = self.operations.clone();
        for (_, task) in &s.internal_assignments {
            missing_operations.retain(|op| op != task);
        }

        let assigned_operator_names: Vec<String> =
            sorted_assignment.iter().map(|(op, _)| op.clone()).collect();
        let mut free_operators = Vec::new();
        for op in &self.operators {
            if !assigned_operator_names.contains(op) {
                free_operators.push(op.clone());
            }
        }

        sorted_assignment.sort_by(|(e1, _), (e2, _)| e1.cmp(e2));

        // 9. Update UI State with struct
        self.calculation_results = Some(CalculationResults {
            assignments: sorted_assignment,
            pref_score: format!("{:.2}", s.weighted_preference_reward_score),
            lead_score: format!("{:.2}", s.weighted_leader_penalty_score),
            exte_score: format!("{:.2}", s.weighted_external_penalty_score),
            hist_ergo_score: format!("{:.2}", s.weighted_historical_penalty_score),
            total_score: format!("{:.2}", s.objective_score),
            solver_time: format!("{:?}", s.solving_time),
            missing_operations,
            free_operators,
        });
    }
}

fn main() -> eframe::Result<()> {
    // Increased window sizes to comfortably fit the 3 columns
    let mut native_options = eframe::NativeOptions::default();
    native_options.viewport.inner_size = Some(egui::vec2(1000.0, 1200.0));
    native_options.viewport.min_inner_size = Some(egui::vec2(700.0, 900.0));

    eframe::run_native(
        "Ergonomic Assigner v2",
        native_options,
        Box::new(|_cc| Box::<FactoryApp>::default()),
    )
}

// // Enum now includes Supervision
// #[derive(PartialEq, Clone, Copy, Debug)]
// enum OperatorStatus {
//     Available,
//     Absent,
//     Training,
//     Loaned,
//     Supervision,
// }

// struct FactoryApp {
//     // Core data
//     operators: Vec<String>,
//     operations: Vec<String>,

//     // State
//     statuses: HashMap<String, OperatorStatus>,
//     forced_assignments: HashMap<String, String>,
//     team_leader: Option<String>,
//     // Track Supervision status separately or via enum.
//     // Given 'S' is listed next to 'T', 'L', 'E' (statuses),
//     // it makes sense to treat it as an exclusive status for simplicity in this mock.

//     // Result
//     calculated_assignments: Option<Vec<(String, String)>>,
// }

// impl Default for FactoryApp {
//     fn default() -> Self {
//         let operators = (b'A'..=b'K')
//             .map(|c| (c as char).to_string())
//             .collect::<Vec<_>>();

//         let operations = (1..=8).map(|n| format!("O{}", n)).collect::<Vec<_>>();

//         let mut statuses = HashMap::new();
//         for op in &operators {
//             statuses.insert(op.clone(), OperatorStatus::Available);
//         }

//         Self {
//             operators,
//             operations,
//             statuses,
//             forced_assignments: HashMap::new(),
//             team_leader: None,
//             calculated_assignments: None,
//         }
//     }
// }

// impl eframe::App for FactoryApp {
//     fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

//         // 1. TOP PANEL: Title
//         egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
//             ui.add_space(10.0);
//             ui.heading(egui::RichText::new("Ergonomic Assigner").size(30.0));
//             ui.add_space(10.0);
//         });

//         // 2. LEFT PANEL: Operator Configuration
//         egui::SidePanel::left("left_panel")
//             .default_width(450.0) // Give it enough default width
//             .resizable(true)      // Allow the user to drag the split
//             .show(ctx, |ui| {
//                 ui.add_space(10.0);
//                 ui.heading(egui::RichText::new("Operator Configuration").size(22.0));
//                 ui.add_space(10.0);

//                 egui::ScrollArea::vertical().id_source("ops_scroll").show(ui, |ui| {
//                     for operator in &self.operators {
//                         ui.group(|ui| {
//                             ui.vertical(|ui| {
//                                 ui.horizontal(|ui| {
//                                     // Operator Name
//                                     ui.label(egui::RichText::new(format!("Operator {}", operator)).size(20.0).strong());
//                                     ui.add_space(20.0);

//                                     // Team Leader Toggle
//                                     let mut is_tl = self.team_leader.as_ref() == Some(operator);
//                                     if ui.add(egui::SelectableLabel::new(is_tl, egui::RichText::new("TL").size(16.0))).clicked() {
//                                         if is_tl {
//                                             self.team_leader = None;
//                                         } else {
//                                             self.team_leader = Some(operator.clone());
//                                         }
//                                     }
//                                 });

//                                 ui.add_space(5.0);

//                                 ui.horizontal(|ui| {
//                                     // Status Dropdown
//                                     ui.label(egui::RichText::new("Status:").size(16.0));
//                                     let mut current_status = self.statuses[operator];
//                                     egui::ComboBox::from_id_source(format!("{}_status", operator))
//                                         .width(150.0)
//                                         .selected_text(egui::RichText::new(format!("{:?}", current_status)).size(16.0))
//                                         .show_ui(ui, |ui| {
//                                             ui.selectable_value(&mut current_status, OperatorStatus::Available, egui::RichText::new("Available").size(16.0));
//                                             ui.selectable_value(&mut current_status, OperatorStatus::Absent, egui::RichText::new("Absent (E)").size(16.0));
//                                             ui.selectable_value(&mut current_status, OperatorStatus::Training, egui::RichText::new("Training (T)").size(16.0));
//                                             ui.selectable_value(&mut current_status, OperatorStatus::Loaned, egui::RichText::new("Loaned (L)").size(16.0));
//                                             ui.selectable_value(&mut current_status, OperatorStatus::Supervision, egui::RichText::new("Supervision (S)").size(16.0));
//                                         });

//                                     if current_status != self.statuses[operator] {
//                                         self.statuses.insert(operator.clone(), current_status);
//                                         if current_status != OperatorStatus::Available {
//                                             self.forced_assignments.remove(operator);
//                                         }
//                                     }
//                                 });

//                                 // Forced Assignment (Only show if Available)
//                                 if self.statuses[operator] == OperatorStatus::Available {
//                                     ui.add_space(5.0);
//                                     ui.horizontal(|ui| {
//                                         ui.label(egui::RichText::new("Force Op:").size(16.0));
//                                         let mut current_forced = self.forced_assignments
//                                             .get(operator)
//                                             .cloned()
//                                             .unwrap_or_else(|| "None".to_string());

//                                         egui::ComboBox::from_id_source(format!("{}_force", operator))
//                                             .width(100.0)
//                                             .selected_text(egui::RichText::new(&current_forced).size(16.0))
//                                             .show_ui(ui, |ui| {
//                                                 ui.selectable_value(&mut current_forced, "None".to_string(), egui::RichText::new("None").size(16.0));
//                                                 ui.separator();
//                                                 for op in &self.operations {
//                                                     ui.selectable_value(&mut current_forced, op.clone(), egui::RichText::new(op).size(16.0));
//                                                 }
//                                             });

//                                         if current_forced == "None" {
//                                             self.forced_assignments.remove(operator);
//                                         } else {
//                                             self.forced_assignments.insert(operator.clone(), current_forced);
//                                         }
//                                     });
//                                 }
//                             });
//                         });
//                         ui.add_space(10.0);
//                     }
//                 });
//             });

//         // 3. CENTRAL PANEL: Button and Results (Takes up remaining space)
//         egui::CentralPanel::default().show(ctx, |ui| {
//             ui.add_space(10.0);

//             // Calculate Button
//             let calculate_btn = egui::Button::new(egui::RichText::new("Calculate Assignments").size(22.0).strong())
//                 .fill(egui::Color32::from_rgb(45, 120, 200))
//                 .min_size(egui::vec2(0.0, 50.0));

//             if ui.add_sized([ui.available_width(), 50.0], calculate_btn).clicked() {
//                 self.run_algorithm();
//             }

//             ui.add_space(20.0);

//             // Results Display
//             if let Some(assignments) = &self.calculated_assignments {
//                 ui.heading(egui::RichText::new("Results:").size(24.0));
//                 ui.add_space(10.0);

//                 egui::ScrollArea::vertical().id_source("results_scroll").show(ui, |ui| {
//                     egui::Grid::new("results_grid")
//                         .striped(true)
//                         .spacing([40.0, 12.0])
//                         .show(ui, |ui| {
//                             ui.label(egui::RichText::new("Operator").size(18.0).strong());
//                             ui.label(egui::RichText::new("Task/Operation").size(18.0).strong());
//                             ui.end_row();

//                             for (op, task) in assignments {
//                                 ui.label(egui::RichText::new(op).size(18.0));
//                                 ui.label(egui::RichText::new(task).size(18.0));
//                                 ui.end_row();
//                             }
//                         });
//                 });
//             } else {
//                 ui.label(egui::RichText::new("Click Calculate to see assignments.").size(16.0).italics());
//             }
//         });
//     }
// }

// impl FactoryApp {
//     fn run_algorithm(&mut self) {
//         // 1. Extract lists for your algorithm based on UI state
//         let mut absent = Vec::new();
//         let mut training = Vec::new();
//         let mut loaned = Vec::new();
//         let mut supervision = Vec::new();

//         for (operator, status) in &self.statuses {
//             match status {
//                 OperatorStatus::Absent => absent.push(operator.clone()),
//                 OperatorStatus::Training => training.push(operator.clone()),
//                 OperatorStatus::Loaned => loaned.push(operator.clone()),
//                 OperatorStatus::Supervision => supervision.push(operator.clone()),
//                 OperatorStatus::Available => {}
//             }
//         }

//         // 2. Format forced assignments into tuples
//         let forced_assignments_vec: Vec<(String, String)> = self.forced_assignments
//             .iter()
//             .map(|(k, v)| (k.clone(), v.clone()))
//             .collect();

//         // 3. Load Data Files (Matrix and History)
//         // Note: For a production GUI, you might want to load these once in `FactoryApp::default()`
//         // to avoid reading from the disk on every button click, but this matches your test structure.
//         let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());

//         let matrix_path = format!("{}/data/factory/GTO_matrix.json", manifest_dir);
//         let matrix_content = std::fs::read_to_string(matrix_path).expect("Failed to read matrix.json");
//         let matrix: artwork_ejas::Matrix = serde_json::from_str(&matrix_content).expect("Failed to parse matrix JSON");

//         let history_path = format!("{}/data/factory/GTO_nov_history.json", manifest_dir);
//         let history_content = std::fs::read_to_string(history_path).expect("Failed to read history JSON");
//         let history_wrapper: Vec<PassWrapper> = serde_json::from_str(&history_content).expect("Failed to parse history");
//         let full_passes: Vec<Pass> = history_wrapper.into_iter().map(|pw| pw.pass).collect();

//         // 4. Algorithm Parameters
//         let offset = 2000;
//         let omega = 1;
//         let alpha = 192;
//         let beta = 384;
//         let tau: usize = 8;
//         let gamma = 24;
//         let station_id = "GTO";

//         let station_data = matrix.stations.get(station_id).expect("Station GTO not found in matrix");

//         // Use the team leader from the UI state (falling back to empty string if none selected)
//         let leader_name = self.team_leader.clone().unwrap_or_default();

//         // 5. Slice the history window (Take the last `tau` days from the loaded history)
//         let start_idx = if full_passes.len() > tau { full_passes.len() - tau } else { 0 };
//         let history_window_passes = full_passes[start_idx..].to_vec();

//         let history_window_days: Vec<Day> = history_window_passes.iter().map(|p| {
//             Day {
//                 date: p.date.clone(),
//                 station: p.station.clone(),
//                 leader: leader_name.clone(),
//                 assignments: p.assignments.clone(),
//             }
//         }).collect();

//         // 6. Run the algorithm
//         let s = calculate_ergonomic_assignment(
//             station_data,
//             history_window_days,
//             offset,
//             omega,
//             alpha,
//             beta,
//             tau as u32,
//             gamma,
//             &forced_assignments_vec,
//             &loaned,
//             &absent,
//             &training,
//             &supervision // Passed via your backend signature
//         );

//         // 7. Reconstruct the full assignment for the UI
//         let mut sorted_assignment = s.internal_assignments.to_vec();

//         loaned.iter().for_each(|x| sorted_assignment.push((x.to_string(), "L".to_string())));
//         absent.iter().for_each(|x| sorted_assignment.push((x.to_string(), "E".to_string())));
//         training.iter().for_each(|x| sorted_assignment.push((x.to_string(), "T".to_string())));
//         supervision.iter().for_each(|x| sorted_assignment.push((x.to_string(), "S".to_string())));

//         // Automatically assign the "TL" task if the leader is present and unassigned
//         if !leader_name.is_empty() {
//             let is_absent = absent.contains(&leader_name);
//             let is_training = training.contains(&leader_name);
//             let is_loaned = loaned.contains(&leader_name);
//             let is_supervision = supervision.contains(&leader_name);
//             let is_on_operation = s.internal_assignments.iter().any(|(emp, _)| emp == &leader_name);

//             if !is_absent && !is_training && !is_loaned && !is_supervision && !is_on_operation {
//                 sorted_assignment.push((leader_name.clone(), "TL".to_string()));
//             }
//         }

//         // 8. Sort alphabetically and update the UI state
//         sorted_assignment.sort_by(|(e1, _), (e2, _)| e1.cmp(e2));
//         self.calculated_assignments = Some(sorted_assignment);
//     }
// }

// fn main() -> eframe::Result<()> {
//     // Increased window size to accommodate larger elements and side-by-side layout
//     let mut native_options = eframe::NativeOptions::default();
//     native_options.viewport.inner_size = Some(egui::vec2(900.0, 700.0));
//     native_options.viewport.min_inner_size = Some(egui::vec2(600.0, 400.0));

//     eframe::run_native(
//         "Ergonomic Assigner v2",
//         native_options,
//         Box::new(|_cc| Box::<FactoryApp>::default()),
//     )
// }
