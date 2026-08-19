//! The application shell: state, screen dispatch and the solve round-trip.

use ejas_core::api::{SolveRequest, SolveResponse};
use ejas_core::validate;

use crate::client::{default_base_url, PendingSolve};
use crate::fileio::FilePicker;
use crate::screens;
use crate::state::{DailyInputs, Problem, Screen, Weights};

/// What the results panel is currently showing.
pub struct Outcome {
    pub response: SolveResponse,
    /// Operator -> job, including the pseudo-jobs for non-working operators.
    pub rows: Vec<(String, String)>,
    pub uncovered_jobs: Vec<String>,
    pub free_operators: Vec<String>,
}

pub struct EjasApp {
    pub screen: Screen,
    pub problem: Problem,
    /// Every station from the loaded matrix, so a multi-station file stays
    /// switchable without reloading.
    pub all_stations: std::collections::HashMap<String, ejas_core::structs::Station>,
    pub daily: DailyInputs,
    pub weights: Weights,

    pub base_url: String,
    pub pending: PendingSolve,
    pub outcome: Option<Outcome>,

    pub picker: FilePicker,
    /// What the next picked file should be interpreted as.
    pub picker_intent: PickerIntent,

    pub error: Option<String>,
    pub notice: Option<String>,

    /// Text-field buffers for the roster editor.
    pub new_job: String,
    pub new_employee: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PickerIntent {
    Matrix,
    History,
}

impl Default for EjasApp {
    fn default() -> Self {
        Self {
            screen: Screen::DataSource,
            problem: Problem::empty(),
            all_stations: std::collections::HashMap::new(),
            daily: DailyInputs::default(),
            weights: Weights::default(),
            base_url: default_base_url(),
            pending: PendingSolve::default(),
            outcome: None,
            picker: FilePicker::default(),
            picker_intent: PickerIntent::Matrix,
            error: None,
            notice: None,
            new_job: String::new(),
            new_employee: String::new(),
        }
    }
}

impl EjasApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Slightly larger text than the default: this is read across a desk
        // on a factory floor, not at arm's length.
        cc.egui_ctx.all_styles_mut(|style| {
            for (_, font) in style.text_styles.iter_mut() {
                font.size *= 1.15;
            }
        });
        Self::default()
    }

    /// Adopt a freshly loaded or built station.
    pub fn set_station(&mut self, station_id: String, station: ejas_core::structs::Station) {
        self.problem.station_id = station_id;
        self.problem.station = station;
        self.daily.reconcile(&self.problem.station);
        // Default today's leader to whoever holds the role on the roster.
        if self.daily.team_leader.is_none() {
            self.daily.team_leader = self
                .problem
                .station
                .people
                .iter()
                .find(|e| e.role == ejas_core::structs::Role::TeamLeader)
                .map(|e| e.name.clone());
        }
        self.outcome = None;
    }

    /// Everything that currently blocks solving.
    pub fn blocking_problems(&self) -> Vec<String> {
        validate::validate_station(&self.problem.station)
            .iter()
            .filter(|p| p.is_error())
            .map(|p| p.message().to_owned())
            .collect()
    }

    pub fn build_request(&self) -> SolveRequest {
        let buckets = self.daily.buckets();
        let mut forced: Vec<(String, String)> = self
            .daily
            .forced_assignments
            .iter()
            .filter(|(name, _)| self.daily.status(name) == crate::state::OperatorStatus::Available)
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        forced.sort();

        SolveRequest {
            station: self.problem.station.clone(),
            history: self.problem.history.clone(),
            params: self.weights.params(),
            leader: self.daily.team_leader.clone(),
            forced_assignments: forced,
            loaned: buckets.loaned,
            absent: buckets.absent,
            training: buckets.training,
            supervision: buckets.supervision,
        }
    }

    pub fn start_solve(&mut self, ctx: &egui::Context) {
        self.error = None;
        self.notice = None;
        let request = self.build_request();
        self.pending.start(ctx, &self.base_url, &request);
    }

    /// Turn a raw solver response into what the results panel shows.
    ///
    /// The pseudo-jobs for loaned/absent/training/supervision operators, and
    /// the `TL` marker, are added here rather than server-side: the solver
    /// never models them, they exist only to make the grid complete.
    fn absorb(&mut self, response: SolveResponse) {
        let mut rows = response.internal_assignments.clone();

        for (name, status) in &self.daily.statuses {
            if let Some(job) = status.pseudo_job() {
                rows.push((name.clone(), job.to_owned()));
            }
        }

        if let Some(leader) = &self.daily.team_leader {
            if !rows.iter().any(|(name, _)| name == leader) {
                rows.push((leader.clone(), "TL".to_owned()));
            }
        }

        rows.sort();

        let assigned_jobs: Vec<&str> = response
            .internal_assignments
            .iter()
            .map(|(_, job)| job.as_str())
            .collect();
        let uncovered_jobs: Vec<String> = self
            .problem
            .jobs()
            .into_iter()
            .filter(|job| !assigned_jobs.contains(&job.as_str()))
            .collect();

        let placed: Vec<&str> = rows.iter().map(|(name, _)| name.as_str()).collect();
        let free_operators: Vec<String> = self
            .problem
            .station
            .people
            .iter()
            .map(|e| e.name.clone())
            .filter(|name| !placed.contains(&name.as_str()))
            .collect();

        self.outcome = Some(Outcome {
            response,
            rows,
            uncovered_jobs,
            free_operators,
        });
    }

    fn poll(&mut self, ctx: &egui::Context) {
        if let Some(result) = self.pending.take() {
            match result {
                Ok(response) => self.absorb(response),
                Err(message) => {
                    self.error = Some(message);
                    self.outcome = None;
                }
            }
        }

        self.picker.accept_dropped(ctx);
        if let Some(file) = self.picker.take() {
            let text = String::from_utf8_lossy(&file.bytes).into_owned();
            match self.picker_intent {
                PickerIntent::Matrix => self.absorb_matrix_file(&file.name, &text),
                PickerIntent::History => self.absorb_history_file(&file.name, &text),
            }
        }
    }

    fn absorb_matrix_file(&mut self, name: &str, text: &str) {
        match crate::load::parse_matrix(text) {
            Ok(matrix) => {
                let mut ids: Vec<&String> = matrix.stations.keys().collect();
                ids.sort();
                match ids.first() {
                    Some(id) => {
                        let id = (*id).clone();
                        let station = matrix.stations[&id].clone();
                        let count = matrix.stations.len();
                        self.set_station(id.clone(), station);
                        self.error = None;
                        self.notice = Some(if count > 1 {
                            format!(
                                "Loaded {name}: {count} stations, showing '{id}'. \
                                 Pick another on the Roster screen."
                            )
                        } else {
                            format!("Loaded {name} (station '{id}').")
                        });
                        // Keep the full matrix so other stations stay reachable.
                        self.all_stations = matrix.stations;
                        self.screen = Screen::Roster;
                    }
                    None => self.error = Some(format!("{name} contains no stations.")),
                }
            }
            Err(e) => self.error = Some(format!("{name}: {e}")),
        }
    }

    fn absorb_history_file(&mut self, name: &str, text: &str) {
        let leader = self.daily.team_leader.clone().unwrap_or_default();
        match crate::load::parse_history(text, &leader) {
            Ok(history) => {
                self.notice = Some(format!("Loaded {} day(s) of history from {name}.", history.len()));
                self.problem.history = history;
                self.problem.history_source = Some(name.to_owned());
                self.error = None;
            }
            Err(e) => self.error = Some(format!("{name}: {e}")),
        }
    }
}

impl eframe::App for EjasApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.poll(ui.ctx());

        egui::Panel::top("top").show(ui, |ui| {
            screens::top_bar(self, ui);
        });

        egui::Panel::bottom("status").show(ui, |ui| {
            screens::status_bar(self, ui);
        });

        match self.screen {
            Screen::DataSource => screens::data_source::show(self, ui),
            Screen::Roster => screens::roster::show(self, ui),
            Screen::Daily => screens::daily::show(self, ui),
        }
    }
}
