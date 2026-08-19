//! Wire types shared by the web/desktop UI and the solver server.
//!
//! These are deliberately free of any native-only dependency so that
//! `ejas-ui` can be compiled to `wasm32-unknown-unknown`, where the Z3
//! solver itself cannot go.

use serde::{Deserialize, Serialize};

use crate::structs::{Day, Station};

/// Weights of the objective function
/// `offset + omega * preference - alpha * leader - beta * external - gamma * historical`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SolverParams {
    /// Added to the objective so the reported score is a positive integer.
    pub offset: u32,
    /// omega - how strongly stated preferences influence the objective.
    pub omega: u32,
    /// alpha - how strongly to discourage assigning the team leader to a job.
    pub alpha: u32,
    /// beta - how strongly to discourage requesting an external operator.
    pub beta: u32,
    /// tau - how many recent days of history to consider.
    pub tau: u32,
    /// gamma - how strongly to penalise repeating a recent employee-job pair.
    pub gamma: u32,
    /// Wall-clock budget for the solver. `None` means the server default.
    pub timeout_ms: Option<u32>,
}

/// The default weights, matching what the original desktop GUI hardcoded.
pub const DEFAULT_TIMEOUT_MS: u32 = 30_000;

impl Default for SolverParams {
    fn default() -> Self {
        WeightPreset::Balanced.params()
    }
}

/// Named bundles of [`SolverParams`], so operators can pick an intent
/// instead of hand-tuning six numbers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeightPreset {
    Balanced,
    PreferenceFirst,
    FairnessFirst,
}

impl WeightPreset {
    pub const ALL: [WeightPreset; 3] = [
        WeightPreset::Balanced,
        WeightPreset::PreferenceFirst,
        WeightPreset::FairnessFirst,
    ];

    /// The single source of truth for preset weights.
    ///
    /// `Balanced` reproduces the values the original GUI hardcoded, so
    /// historic results stay comparable. Each other preset changes exactly
    /// one weight, and each was checked against the bundled VCE and GTO data
    /// to confirm it actually moves the assignment:
    ///
    /// - `omega` 1 -> 8 raises VCE's preference reward from 85 to 96.
    /// - `gamma` 24 -> 96 drops GTO's repeat count from 2 to 0.
    ///
    /// `tau` deliberately stays at 8 everywhere: widening the window counts
    /// more history and so *raises* the repeat count rather than lowering it,
    /// which reads as the preset doing the opposite of what it claims.
    pub fn params(self) -> SolverParams {
        let (omega, alpha, beta, tau, gamma) = match self {
            WeightPreset::Balanced => (1, 192, 384, 8, 24),
            WeightPreset::PreferenceFirst => (8, 192, 384, 8, 24),
            WeightPreset::FairnessFirst => (1, 192, 384, 8, 96),
        };
        SolverParams {
            offset: 2000,
            omega,
            alpha,
            beta,
            tau,
            gamma,
            timeout_ms: Some(DEFAULT_TIMEOUT_MS),
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            WeightPreset::Balanced => "Balanced",
            WeightPreset::PreferenceFirst => "Preference-first",
            WeightPreset::FairnessFirst => "Fairness-first",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            WeightPreset::Balanced => {
                "Default trade-off between preferences, fairness and external operators."
            }
            WeightPreset::PreferenceFirst => {
                "Weights stated preferences eight times higher, accepting a few \
                 repeats to give people the jobs they asked for."
            }
            WeightPreset::FairnessFirst => {
                "Penalises repeating a recent person-job pairing four times harder, \
                 even if that means putting the team leader on a job."
            }
        }
    }
}

/// Everything the server needs for one solve. The server keeps no state
/// between requests, so the roster and history travel with each call.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SolveRequest {
    pub station: Station,
    #[serde(default)]
    pub history: Vec<Day>,
    #[serde(default)]
    pub params: SolverParams,
    #[serde(default)]
    pub leader: Option<String>,
    #[serde(default)]
    pub forced_assignments: Vec<(String, String)>,
    #[serde(default)]
    pub loaned: Vec<String>,
    #[serde(default)]
    pub absent: Vec<String>,
    #[serde(default)]
    pub training: Vec<String>,
    #[serde(default)]
    pub supervision: Vec<String>,
}

/// Whether the solver actually found an assignment.
///
/// The desktop code returned an all-zero solution for both `Unsat` and
/// `Unknown`, which is indistinguishable from a legitimate zero score.
/// Carrying the status explicitly lets the UI say what really happened.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SolveStatus {
    /// An optimal assignment was found.
    Sat,
    /// The constraints cannot all be satisfied.
    Unsat,
    /// The solver gave up, most likely on the timeout.
    Unknown,
}

impl SolveStatus {
    pub fn is_sat(self) -> bool {
        matches!(self, SolveStatus::Sat)
    }
}

/// Serialisable mirror of the solver's own solution struct.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SolveResponse {
    pub status: SolveStatus,
    pub internal_assignments: Vec<(String, String)>,
    pub external_assignments: Vec<String>,
    pub objective_score: i64,
    pub preference_reward_score: i64,
    pub weighted_preference_reward_score: i64,
    pub leader_penalty_score: i64,
    pub weighted_leader_penalty_score: i64,
    pub external_penalty_score: i64,
    pub weighted_external_penalty_score: i64,
    pub historical_penalty_score: i64,
    pub weighted_historical_penalty_score: i64,
    pub c_matrix: Vec<Vec<bool>>,
    pub p_matrix: Vec<Vec<usize>>,
    pub h_matrix: Vec<Vec<u32>>,
    /// `Duration` has no stable JSON form, so the wire carries milliseconds.
    pub solving_time_ms: u64,
    pub offset: u32,
}
