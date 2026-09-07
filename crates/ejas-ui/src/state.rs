//! UI-side state: everything the user has built or loaded, plus how they
//! want it solved.

use std::collections::HashMap;

use ejas_core::api::{PresetInputs, SolverParams, StationDims, WeightPreset};
use ejas_core::structs::{Day, Station};

/// Which screen the user is on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    /// Pick where the roster comes from.
    DataSource,
    /// Build or edit jobs and employees.
    Roster,
    /// Set today's operator statuses and weights, then solve.
    Daily,
}

/// Operational status of one operator today.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperatorStatus {
    Available,
    Absent,
    Training,
    Loaned,
    Supervision,
}

impl OperatorStatus {
    pub const ALL: [OperatorStatus; 5] = [
        OperatorStatus::Available,
        OperatorStatus::Absent,
        OperatorStatus::Training,
        OperatorStatus::Loaned,
        OperatorStatus::Supervision,
    ];

    pub fn label(self) -> &'static str {
        match self {
            OperatorStatus::Available => "Available",
            OperatorStatus::Absent => "Absent",
            OperatorStatus::Training => "Training",
            OperatorStatus::Loaned => "Loaned",
            OperatorStatus::Supervision => "Supervision",
        }
    }

    /// The pseudo-job shown in the results grid for a non-working operator.
    /// The solver never sees these; they are a presentation concern.
    pub fn pseudo_job(self) -> Option<&'static str> {
        match self {
            OperatorStatus::Available => None,
            OperatorStatus::Loaned => Some("L"),
            OperatorStatus::Absent => Some("E"),
            OperatorStatus::Training => Some("T"),
            OperatorStatus::Supervision => Some("S"),
        }
    }
}

/// Whether weights come from a named preset or are being hand-tuned.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeightMode {
    Preset(WeightPreset),
    Manual,
}

/// Weight selection, keeping the manual values around so switching back and
/// forth does not lose the user's tuning.
#[derive(Debug, Clone)]
pub struct Weights {
    pub mode: WeightMode,
    /// Live values when `mode` is `Manual`. Seeded from the last preset used,
    /// so tuning starts from a working baseline rather than from zero.
    pub manual: SolverParams,
    /// `d_limit`, `K` and `tau`, which the preset formulas need but which are
    /// policy rather than weights. Editable so a station that rotates every
    /// day is not stuck with a preset built for rotating every second day.
    pub inputs: PresetInputs,
}

impl Default for Weights {
    fn default() -> Self {
        Self {
            mode: WeightMode::Preset(WeightPreset::Balanced),
            // Nominal until a station is loaded; `params` re-derives from the
            // real one on every call, so this is never what gets sent.
            manual: SolverParams::default(),
            inputs: PresetInputs::default(),
        }
    }
}

impl Weights {
    /// The params that will actually be sent.
    ///
    /// Presets are derived from the station, so this needs the roster: the
    /// thresholds are all statements about outweighing another term of the
    /// objective, and those terms scale with N, M and the ergonomic spread.
    pub fn params(&self, station: &Station) -> SolverParams {
        match self.mode {
            WeightMode::Preset(p) => p.params(StationDims::of(station), self.inputs),
            WeightMode::Manual => self.manual,
        }
    }

    /// Switch to manual, carrying the currently-shown numbers over.
    pub fn switch_to_manual(&mut self, station: &Station) {
        if self.mode != WeightMode::Manual {
            self.manual = self.params(station);
            self.mode = WeightMode::Manual;
        }
    }
}

/// Today's per-operator inputs.
#[derive(Debug, Default, Clone)]
pub struct DailyInputs {
    pub statuses: HashMap<String, OperatorStatus>,
    pub forced_assignments: HashMap<String, String>,
    /// Overrides the roster's TeamLeader for today, if set.
    pub team_leader: Option<String>,
}

impl DailyInputs {
    /// Drop entries for people who are no longer on the roster, and default
    /// everyone else to Available.
    pub fn reconcile(&mut self, station: &Station) {
        let names: Vec<&str> = station.people.iter().map(|e| e.name.as_str()).collect();
        self.statuses.retain(|k, _| names.contains(&k.as_str()));
        self.forced_assignments
            .retain(|k, v| names.contains(&k.as_str()) && station.ergo_score.contains_key(v));
        if let Some(leader) = &self.team_leader {
            if !names.contains(&leader.as_str()) {
                self.team_leader = None;
            }
        }
        for person in &station.people {
            self.statuses
                .entry(person.name.clone())
                .or_insert(OperatorStatus::Available);
        }
    }

    pub fn status(&self, name: &str) -> OperatorStatus {
        self.statuses
            .get(name)
            .copied()
            .unwrap_or(OperatorStatus::Available)
    }

    /// Split operators into the four status buckets the solver takes.
    pub fn buckets(&self) -> Buckets {
        let mut b = Buckets::default();
        for (name, status) in &self.statuses {
            match status {
                OperatorStatus::Absent => b.absent.push(name.clone()),
                OperatorStatus::Training => b.training.push(name.clone()),
                OperatorStatus::Loaned => b.loaned.push(name.clone()),
                OperatorStatus::Supervision => b.supervision.push(name.clone()),
                OperatorStatus::Available => {}
            }
        }
        // HashMap iteration order is arbitrary; sort so requests are stable
        // and two identical rosters produce byte-identical payloads.
        b.absent.sort();
        b.training.sort();
        b.loaned.sort();
        b.supervision.sort();
        b
    }
}

#[derive(Debug, Default, Clone)]
pub struct Buckets {
    pub absent: Vec<String>,
    pub training: Vec<String>,
    pub loaned: Vec<String>,
    pub supervision: Vec<String>,
}

/// The loaded problem: a station plus whatever history the user supplied.
#[derive(Debug, Clone)]
pub struct Problem {
    pub station_id: String,
    pub station: Station,
    pub history: Vec<Day>,
    /// Where the history came from, for display.
    pub history_source: Option<String>,
}

impl Problem {
    pub fn empty() -> Self {
        Self {
            station_id: "Station".to_owned(),
            station: Station {
                ergo_score: HashMap::new(),
                people: Vec::new(),
            },
            history: Vec::new(),
            history_source: None,
        }
    }

    /// Job ids in the order the solver uses them: lexically sorted, which is
    /// what `calculate_ergonomic_assignment` does with `ergo_score` keys.
    pub fn jobs(&self) -> Vec<String> {
        let mut jobs: Vec<String> = self.station.ergo_score.keys().cloned().collect();
        jobs.sort();
        jobs
    }
}
