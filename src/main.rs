// use eframe::egui;
// use std::collections::HashMap;

// // Enum to track the availability of an operator
// #[derive(PartialEq, Clone, Copy, Debug)]
// enum OperatorStatus {
//     Available,
//     Absent,
//     Training,
//     Loaned,
// }

// struct FactoryApp {
//     // Core data
//     operators: Vec<String>,
//     operations: Vec<String>,

//     // State
//     statuses: HashMap<String, OperatorStatus>,
//     forced_assignments: HashMap<String, String>,
//     team_leader: Option<String>,

//     // Result
//     calculated_assignments: Option<Vec<(String, String)>>,
// }

// impl Default for FactoryApp {
//     fn default() -> Self {
//         // Initialize operators A through K
//         let operators = (b'A'..=b'K')
//             .map(|c| (c as char).to_string())
//             .collect::<Vec<_>>();

//         // Initialize operations O1 through O8
//         let operations = (1..=8)
//             .map(|n| format!("O{}", n))
//             .collect::<Vec<_>>();

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
//         // Use a central panel with a clean background
//         egui::CentralPanel::default().show(ctx, |ui| {
//             ui.heading("Ergonomic Assigner");
//             ui.add_space(10.0);

//             // Scrollable area for the operator list
//             egui::ScrollArea::vertical().show(ui, |ui| {
//                 for operator in &self.operators {
//                     ui.group(|ui| {
//                         ui.horizontal(|ui| {
//                             // 1. Operator Name
//                             ui.label(egui::RichText::new(operator).size(18.0).strong());
//                             ui.add_space(10.0);

//                             // 2. Status Dropdown
//                             let mut current_status = self.statuses[operator];
//                             egui::ComboBox::from_id_source(format!("{}_status", operator))
//                                 .selected_text(format!("{:?}", current_status))
//                                 .show_ui(ui, |ui| {
//                                     ui.selectable_value(&mut current_status, OperatorStatus::Available, "Available");
//                                     ui.selectable_value(&mut current_status, OperatorStatus::Absent, "Absent (E)");
//                                     ui.selectable_value(&mut current_status, OperatorStatus::Training, "Training (T)");
//                                     ui.selectable_value(&mut current_status, OperatorStatus::Loaned, "Loaned (L)");
//                                 });

//                             if current_status != self.statuses[operator] {
//                                 self.statuses.insert(operator.clone(), current_status);
//                                 // If they are no longer available, remove any forced assignments
//                                 if current_status != OperatorStatus::Available {
//                                     self.forced_assignments.remove(operator);
//                                 }
//                             }

//                             // 3. Team Leader Toggle
//                             let mut is_tl = self.team_leader.as_ref() == Some(operator);
//                             if ui.toggle_value(&mut is_tl, "TL").clicked() {
//                                 if is_tl {
//                                     self.team_leader = Some(operator.clone());
//                                 } else if self.team_leader.as_ref() == Some(operator) {
//                                     self.team_leader = None;
//                                 }
//                             }
//                         });

//                         // 4. Forced Assignment (Only show if Available)
//                         if self.statuses[operator] == OperatorStatus::Available {
//                             ui.horizontal(|ui| {
//                                 ui.label("Force Operation:");
//                                 let mut current_forced = self.forced_assignments
//                                     .get(operator)
//                                     .cloned()
//                                     .unwrap_or_else(|| "None".to_string());

//                                 egui::ComboBox::from_id_source(format!("{}_force", operator))
//                                     .selected_text(&current_forced)
//                                     .show_ui(ui, |ui| {
//                                         ui.selectable_value(&mut current_forced, "None".to_string(), "None");
//                                         ui.separator();
//                                         for op in &self.operations {
//                                             ui.selectable_value(&mut current_forced, op.clone(), op.clone());
//                                         }
//                                     });

//                                 if current_forced == "None" {
//                                     self.forced_assignments.remove(operator);
//                                 } else {
//                                     self.forced_assignments.insert(operator.clone(), current_forced);
//                                 }
//                             });
//                         }
//                     });
//                     ui.add_space(5.0);
//                 }
//             });

//             ui.add_space(10.0);
//             ui.separator();
//             ui.add_space(10.0);

//             // Calculate Button
//             let calculate_btn = egui::Button::new(egui::RichText::new("Calculate Assignments").size(20.0))
//                 .fill(egui::Color32::from_rgb(45, 120, 200)); // Nice blue button

//             if ui.add_sized([ui.available_width(), 40.0], calculate_btn).clicked() {
//                 self.run_algorithm();
//             }

//             // Results Display
//             if let Some(assignments) = &self.calculated_assignments {
//                 ui.add_space(10.0);
//                 ui.heading("Results:");
//                 egui::ScrollArea::vertical().id_source("results_scroll").show(ui, |ui| {
//                     egui::Grid::new("results_grid")
//                         .striped(true)
//                         .spacing([40.0, 8.0])
//                         .show(ui, |ui| {
//                             ui.label(egui::RichText::new("Operator").strong());
//                             ui.label(egui::RichText::new("Task/Operation").strong());
//                             ui.end_row();

//                             for (op, task) in assignments {
//                                 ui.label(op);
//                                 ui.label(task);
//                                 ui.end_row();
//                             }
//                         });
//                 });
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

//         for (operator, status) in &self.statuses {
//             match status {
//                 OperatorStatus::Absent => absent.push(operator.clone()),
//                 OperatorStatus::Training => training.push(operator.clone()),
//                 OperatorStatus::Loaned => loaned.push(operator.clone()),
//                 OperatorStatus::Available => {}
//             }
//         }

//         // 2. Format forced assignments into tuples
//         let forced_assignments_vec: Vec<(String, String)> = self.forced_assignments
//             .iter()
//             .map(|(k, v)| (k.clone(), v.clone()))
//             .collect();

//         // 3. THIS IS WHERE YOU CALL YOUR BACKEND
//         //
//         // let s: ErgonomicAssignmentSolution = calculate_ergonomic_assignment(
//         //     station_data,
//         //     history_window_days, // You may need to load this from disk on app startup
//         //     2000, 1, 192, 384, 8, 24, // Algorithm params
//         //     &forced_assignments_vec,
//         //     &loaned,
//         //     &absent,
//         //     &training,
//         //     &vec![] // Supervision
//         // );
//         // let mut sorted_assignment = s.internal_assignments.to_vec();

//         // --- MOCK BACKEND LOGIC FOR DEMONSTRATION ---
//         let mut sorted_assignment = Vec::new();
//         for op in &self.operators {
//             if absent.contains(op) { sorted_assignment.push((op.clone(), "E".to_string())); }
//             else if training.contains(op) { sorted_assignment.push((op.clone(), "T".to_string())); }
//             else if loaned.contains(op) { sorted_assignment.push((op.clone(), "L".to_string())); }
//             else if Some(op) == self.team_leader.as_ref() { sorted_assignment.push((op.clone(), "TL".to_string())); }
//             else if let Some(forced) = self.forced_assignments.get(op) {
//                 sorted_assignment.push((op.clone(), forced.clone()));
//             } else {
//                 sorted_assignment.push((op.clone(), "Algorithm Pick".to_string())); // Mock assignment
//             }
//         }
//         // ---------------------------------------------

//         sorted_assignment.sort_by(|(e1, _), (e2, _)| e1.cmp(e2));

//         // 4. Update the UI state with results
//         self.calculated_assignments = Some(sorted_assignment);
//     }
// }

// fn main() -> eframe::Result<()> {
//     // Configure to look somewhat like a mobile device dimensions
//     let mut native_options = eframe::NativeOptions::default();
//     native_options.viewport.inner_size = Some(egui::vec2(400.0, 750.0));
//     native_options.viewport.min_inner_size = Some(egui::vec2(350.0, 500.0));

//     eframe::run_native(
//         "Station Assigner",
//         native_options,
//         Box::new(|_cc| Box::<FactoryApp>::default()),
//     )
// }

use artwork_ejas::{Day, Matrix, Pass, PassWrapper, calculate_ergonomic_assignment};
use eframe::egui;
use std::collections::HashMap;

// Enum now includes Supervision
#[derive(PartialEq, Clone, Copy, Debug)]
enum OperatorStatus {
    Available,
    Absent,
    Training,
    Loaned,
    Supervision,
}

struct FactoryApp {
    // Core data
    operators: Vec<String>,
    operations: Vec<String>,

    // State
    statuses: HashMap<String, OperatorStatus>,
    forced_assignments: HashMap<String, String>,
    team_leader: Option<String>,
    // Track Supervision status separately or via enum.
    // Given 'S' is listed next to 'T', 'L', 'E' (statuses),
    // it makes sense to treat it as an exclusive status for simplicity in this mock.

    // Result
    calculated_assignments: Option<Vec<(String, String)>>,
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
            calculated_assignments: None,
        }
    }
}

// impl eframe::App for FactoryApp {
//     fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
//         egui::CentralPanel::default().show(ctx, |ui| {
//             // Larger Heading
//             ui.heading(egui::RichText::new("Ergonomic Assigner").size(30.0));
//             ui.add_space(15.0);

//             // Use horizontal layout to split input and results
//             ui.horizontal(|ui| {
//                 // --- Left Column: Assignments ---
//                 ui.allocate_ui_with_layout(
//                     egui::vec2(ui.available_width() * 0.55, ui.available_height()),
//                     egui::Layout::top_down(egui::Align::LEFT),
//                     |ui| {
//                         ui.heading(egui::RichText::new("Operator Configuration").size(22.0));
//                         ui.add_space(10.0);

//                         egui::ScrollArea::vertical()
//                             .id_source("ops_scroll")
//                             .show(ui, |ui| {
//                                 for operator in &self.operators {
//                                     ui.group(|ui| {
//                                         // Use vertical layout inside group for more space
//                                         ui.vertical(|ui| {
//                                             ui.horizontal(|ui| {
//                                                 // 1. Larger Operator Name
//                                                 ui.label(
//                                                     egui::RichText::new(format!(
//                                                         "Operator {}",
//                                                         operator
//                                                     ))
//                                                     .size(20.0)
//                                                     .strong(),
//                                                 );
//                                                 ui.add_space(20.0);

//                                                 // 2. Team Leader Toggle
//                                                 let mut is_tl =
//                                                     self.team_leader.as_ref() == Some(operator);
//                                                 if ui
//                                                     .add(egui::SelectableLabel::new(
//                                                         is_tl,
//                                                         egui::RichText::new("TL").size(16.0),
//                                                     ))
//                                                     .clicked()
//                                                 {
//                                                     if is_tl {
//                                                         self.team_leader = None;
//                                                     } else {
//                                                         self.team_leader = Some(operator.clone());
//                                                     }
//                                                 }
//                                             });

//                                             ui.add_space(5.0);

//                                             ui.horizontal(|ui| {
//                                                 // 3. Larger Status Dropdown (now includes Supervision)
//                                                 ui.label(egui::RichText::new("Status:").size(16.0));
//                                                 let mut current_status = self.statuses[operator];
//                                                 let combo_reps = egui::ComboBox::from_id_source(
//                                                     format!("{}_status", operator),
//                                                 )
//                                                 .width(150.0)
//                                                 .selected_text(
//                                                     egui::RichText::new(format!(
//                                                         "{:?}",
//                                                         current_status
//                                                     ))
//                                                     .size(16.0),
//                                                 )
//                                                 .show_ui(ui, |ui| {
//                                                     // Use RichText directly on the selectable values instead of mutating the style
//                                                     ui.selectable_value(
//                                                         &mut current_status,
//                                                         OperatorStatus::Available,
//                                                         egui::RichText::new("Available").size(16.0),
//                                                     );
//                                                     ui.selectable_value(
//                                                         &mut current_status,
//                                                         OperatorStatus::Absent,
//                                                         egui::RichText::new("Absent (E)")
//                                                             .size(16.0),
//                                                     );
//                                                     ui.selectable_value(
//                                                         &mut current_status,
//                                                         OperatorStatus::Training,
//                                                         egui::RichText::new("Training (T)")
//                                                             .size(16.0),
//                                                     );
//                                                     ui.selectable_value(
//                                                         &mut current_status,
//                                                         OperatorStatus::Loaned,
//                                                         egui::RichText::new("Loaned (L)")
//                                                             .size(16.0),
//                                                     );
//                                                     ui.selectable_value(
//                                                         &mut current_status,
//                                                         OperatorStatus::Supervision,
//                                                         egui::RichText::new("Supervision (S)")
//                                                             .size(16.0),
//                                                     );
//                                                 });

//                                                 if current_status != self.statuses[operator] {
//                                                     self.statuses
//                                                         .insert(operator.clone(), current_status);
//                                                     if current_status != OperatorStatus::Available {
//                                                         self.forced_assignments.remove(operator);
//                                                     }
//                                                 }
//                                             });

//                                             // 4. Forced Assignment (Only show if Available)
//                                             if self.statuses[operator] == OperatorStatus::Available
//                                             {
//                                                 ui.add_space(5.0);
//                                                 ui.horizontal(|ui| {
//                                                     ui.label(
//                                                         egui::RichText::new("Force Op:").size(16.0),
//                                                     );
//                                                     let mut current_forced = self
//                                                         .forced_assignments
//                                                         .get(operator)
//                                                         .cloned()
//                                                         .unwrap_or_else(|| "None".to_string());

//                                                     egui::ComboBox::from_id_source(format!(
//                                                         "{}_force",
//                                                         operator
//                                                     ))
//                                                     .width(100.0)
//                                                     .selected_text(
//                                                         egui::RichText::new(&current_forced)
//                                                             .size(16.0),
//                                                     )
//                                                     .show_ui(ui, |ui| {
//                                                         // Again, use RichText for the options
//                                                         ui.selectable_value(
//                                                             &mut current_forced,
//                                                             "None".to_string(),
//                                                             egui::RichText::new("None").size(16.0),
//                                                         );
//                                                         ui.separator();
//                                                         for op in &self.operations {
//                                                             ui.selectable_value(
//                                                                 &mut current_forced,
//                                                                 op.clone(),
//                                                                 egui::RichText::new(op).size(16.0),
//                                                             );
//                                                         }
//                                                     });
//                                                     if current_forced == "None" {
//                                                         self.forced_assignments.remove(operator);
//                                                     } else {
//                                                         self.forced_assignments.insert(
//                                                             operator.clone(),
//                                                             current_forced,
//                                                         );
//                                                     }
//                                                 });
//                                             }
//                                         });
//                                     });
//                                     ui.add_space(10.0); // More space between operators
//                                 }
//                             });
//                     },
//                 );

//                 // --- Separator ---
//                 ui.add(egui::Separator::default().vertical());
//                 ui.add_space(10.0);

//                 // --- Right Column: Calculation & Results ---
//                 ui.vertical(|ui| {
//                     // Larger Calculate Button
//                     let calculate_btn = egui::Button::new(
//                         egui::RichText::new("Calculate Assignments")
//                             .size(22.0)
//                             .strong(),
//                     )
//                     .fill(egui::Color32::from_rgb(45, 120, 200))
//                     .min_size(egui::vec2(0.0, 50.0)); // Taller button

//                     if ui
//                         .add_sized([ui.available_width(), 50.0], calculate_btn)
//                         .clicked()
//                     {
//                         self.run_algorithm();
//                     }

//                     ui.add_space(20.0);

//                     // Results Display
//                     if let Some(assignments) = &self.calculated_assignments {
//                         ui.heading(egui::RichText::new("Results:").size(24.0));
//                         ui.add_space(10.0);
//                         egui::ScrollArea::vertical()
//                             .id_source("results_scroll")
//                             .show(ui, |ui| {
//                                 egui::Grid::new("results_grid")
//                                     .striped(true)
//                                     .spacing([20.0, 12.0]) // Increased grid spacing
//                                     .show(ui, |ui| {
//                                         ui.label(
//                                             egui::RichText::new("Operator").size(18.0).strong(),
//                                         );
//                                         ui.label(
//                                             egui::RichText::new("Task/Operation")
//                                                 .size(18.0)
//                                                 .strong(),
//                                         );
//                                         ui.end_row();

//                                         for (op, task) in assignments {
//                                             ui.label(egui::RichText::new(op).size(18.0));
//                                             ui.label(egui::RichText::new(task).size(18.0));
//                                             ui.end_row();
//                                         }
//                                     });
//                             });
//                     } else {
//                         ui.label(
//                             egui::RichText::new("Click Calculate to see assignments.")
//                                 .size(16.0)
//                                 .italics(),
//                         );
//                     }
//                 });
//             });
//         });
//     }
// }

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
            .default_width(450.0) // Give it enough default width
            .resizable(true)      // Allow the user to drag the split
            .show(ctx, |ui| {
                ui.add_space(10.0);
                ui.heading(egui::RichText::new("Operator Configuration").size(22.0));
                ui.add_space(10.0);

                egui::ScrollArea::vertical().id_source("ops_scroll").show(ui, |ui| {
                    for operator in &self.operators {
                        ui.group(|ui| {
                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    // Operator Name
                                    ui.label(egui::RichText::new(format!("Operator {}", operator)).size(20.0).strong());
                                    ui.add_space(20.0);
                                    
                                    // Team Leader Toggle
                                    let mut is_tl = self.team_leader.as_ref() == Some(operator);
                                    if ui.add(egui::SelectableLabel::new(is_tl, egui::RichText::new("TL").size(16.0))).clicked() {
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
                                    egui::ComboBox::from_id_source(format!("{}_status", operator))
                                        .width(150.0)
                                        .selected_text(egui::RichText::new(format!("{:?}", current_status)).size(16.0))
                                        .show_ui(ui, |ui| {
                                            ui.selectable_value(&mut current_status, OperatorStatus::Available, egui::RichText::new("Available").size(16.0));
                                            ui.selectable_value(&mut current_status, OperatorStatus::Absent, egui::RichText::new("Absent (E)").size(16.0));
                                            ui.selectable_value(&mut current_status, OperatorStatus::Training, egui::RichText::new("Training (T)").size(16.0));
                                            ui.selectable_value(&mut current_status, OperatorStatus::Loaned, egui::RichText::new("Loaned (L)").size(16.0));
                                            ui.selectable_value(&mut current_status, OperatorStatus::Supervision, egui::RichText::new("Supervision (S)").size(16.0));
                                        });
                                    
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
                                        let mut current_forced = self.forced_assignments
                                            .get(operator)
                                            .cloned()
                                            .unwrap_or_else(|| "None".to_string());

                                        egui::ComboBox::from_id_source(format!("{}_force", operator))
                                            .width(100.0)
                                            .selected_text(egui::RichText::new(&current_forced).size(16.0))
                                            .show_ui(ui, |ui| {
                                                ui.selectable_value(&mut current_forced, "None".to_string(), egui::RichText::new("None").size(16.0));
                                                ui.separator();
                                                for op in &self.operations {
                                                    ui.selectable_value(&mut current_forced, op.clone(), egui::RichText::new(op).size(16.0));
                                                }
                                            });

                                        if current_forced == "None" {
                                            self.forced_assignments.remove(operator);
                                        } else {
                                            self.forced_assignments.insert(operator.clone(), current_forced);
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
            let calculate_btn = egui::Button::new(egui::RichText::new("Calculate Assignments").size(22.0).strong())
                .fill(egui::Color32::from_rgb(45, 120, 200))
                .min_size(egui::vec2(0.0, 50.0));
            
            if ui.add_sized([ui.available_width(), 50.0], calculate_btn).clicked() {
                self.run_algorithm();
            }

            ui.add_space(20.0);

            // Results Display
            if let Some(assignments) = &self.calculated_assignments {
                ui.heading(egui::RichText::new("Results:").size(24.0));
                ui.add_space(10.0);
                
                egui::ScrollArea::vertical().id_source("results_scroll").show(ui, |ui| {
                    egui::Grid::new("results_grid")
                        .striped(true)
                        .spacing([40.0, 12.0])
                        .show(ui, |ui| {
                            ui.label(egui::RichText::new("Operator").size(18.0).strong());
                            ui.label(egui::RichText::new("Task/Operation").size(18.0).strong());
                            ui.end_row();

                            for (op, task) in assignments {
                                ui.label(egui::RichText::new(op).size(18.0));
                                ui.label(egui::RichText::new(task).size(18.0));
                                ui.end_row();
                            }
                        });
                });
            } else {
                ui.label(egui::RichText::new("Click Calculate to see assignments.").size(16.0).italics());
            }
        });
    }
}

// impl FactoryApp {
//     fn run_algorithm(&mut self) {
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

//         // --- MOCK BACKEND LOGIC FOR DEMONSTRATION ---
//         let mut sorted_assignment = Vec::new();
//         for op in &self.operators {
//             if absent.contains(op) {
//                 sorted_assignment.push((op.clone(), "E".to_string()));
//             } else if training.contains(op) {
//                 sorted_assignment.push((op.clone(), "T".to_string()));
//             } else if loaned.contains(op) {
//                 sorted_assignment.push((op.clone(), "L".to_string()));
//             } else if supervision.contains(op) {
//                 sorted_assignment.push((op.clone(), "S".to_string()));
//             } else if let Some(forced) = self.forced_assignments.get(op) {
//                 // Keep forced operation names (e.g., O1) as is, not just 'forced'
//                 sorted_assignment.push((op.clone(), forced.clone()));
//             } else {
//                 // Indicate Team Leader in final output if applicable
//                 let task = if Some(op) == self.team_leader.as_ref() {
//                     "TL / Auto Pick".to_string()
//                 } else {
//                     "Auto Pick".to_string()
//                 };
//                 sorted_assignment.push((op.clone(), task));
//             }
//         }
//         // ---------------------------------------------

//         sorted_assignment.sort_by(|(e1, _), (e2, _)| e1.cmp(e2));

//         // 4. Update the UI state with results
//         self.calculated_assignments = Some(sorted_assignment);
//     }
// }

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
        let forced_assignments_vec: Vec<(String, String)> = self.forced_assignments
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        // 3. Load Data Files (Matrix and History)
        // Note: For a production GUI, you might want to load these once in `FactoryApp::default()` 
        // to avoid reading from the disk on every button click, but this matches your test structure.
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
        
        let matrix_path = format!("{}/data/factory/GTO_matrix.json", manifest_dir);
        let matrix_content = std::fs::read_to_string(matrix_path).expect("Failed to read matrix.json");
        let matrix: artwork_ejas::Matrix = serde_json::from_str(&matrix_content).expect("Failed to parse matrix JSON");

        let history_path = format!("{}/data/factory/GTO_nov_history.json", manifest_dir);
        let history_content = std::fs::read_to_string(history_path).expect("Failed to read history JSON");
        let history_wrapper: Vec<PassWrapper> = serde_json::from_str(&history_content).expect("Failed to parse history");
        let full_passes: Vec<Pass> = history_wrapper.into_iter().map(|pw| pw.pass).collect();

        // 4. Algorithm Parameters
        let offset = 2000;
        let omega = 1;
        let alpha = 192;
        let beta = 384;
        let tau: usize = 8;
        let gamma = 24;
        let station_id = "GTO"; 

        let station_data = matrix.stations.get(station_id).expect("Station GTO not found in matrix");

        // Use the team leader from the UI state (falling back to empty string if none selected)
        let leader_name = self.team_leader.clone().unwrap_or_default();

        // 5. Slice the history window (Take the last `tau` days from the loaded history)
        let start_idx = if full_passes.len() > tau { full_passes.len() - tau } else { 0 };
        let history_window_passes = full_passes[start_idx..].to_vec();

        let history_window_days: Vec<Day> = history_window_passes.iter().map(|p| {
            Day {
                date: p.date.clone(), 
                station: p.station.clone(),
                leader: leader_name.clone(), 
                assignments: p.assignments.clone(), 
            }
        }).collect();

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
            &supervision // Passed via your backend signature
        );

        // 7. Reconstruct the full assignment for the UI
        let mut sorted_assignment = s.internal_assignments.to_vec();
        
        loaned.iter().for_each(|x| sorted_assignment.push((x.to_string(), "L".to_string())));
        absent.iter().for_each(|x| sorted_assignment.push((x.to_string(), "E".to_string())));
        training.iter().for_each(|x| sorted_assignment.push((x.to_string(), "T".to_string())));
        supervision.iter().for_each(|x| sorted_assignment.push((x.to_string(), "S".to_string())));
        
        // Automatically assign the "TL" task if the leader is present and unassigned
        if !leader_name.is_empty() {
            let is_absent = absent.contains(&leader_name);
            let is_training = training.contains(&leader_name);
            let is_loaned = loaned.contains(&leader_name);
            let is_supervision = supervision.contains(&leader_name);
            let is_on_operation = s.internal_assignments.iter().any(|(emp, _)| emp == &leader_name);

            if !is_absent && !is_training && !is_loaned && !is_supervision && !is_on_operation {
                sorted_assignment.push((leader_name.clone(), "TL".to_string()));
            }
        }

        // 8. Sort alphabetically and update the UI state
        sorted_assignment.sort_by(|(e1, _), (e2, _)| e1.cmp(e2));
        self.calculated_assignments = Some(sorted_assignment);
    }
}

fn main() -> eframe::Result<()> {
    // Increased window size to accommodate larger elements and side-by-side layout
    let mut native_options = eframe::NativeOptions::default();
    native_options.viewport.inner_size = Some(egui::vec2(900.0, 700.0));
    native_options.viewport.min_inner_size = Some(egui::vec2(600.0, 400.0));

    eframe::run_native(
        "Ergonomic Assigner v2",
        native_options,
        Box::new(|_cc| Box::<FactoryApp>::default()),
    )
}
