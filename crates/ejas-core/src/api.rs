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
    /// Whether the historical penalty is scaled per job by the ergonomic
    /// multiplier `E_max - E_j + 1`, which makes repeating a physically hard
    /// job hurt more than repeating an easy one. The Happiness-first strategy
    /// drops it, leaving `gamma` as a pure boredom penalty on repetition.
    #[serde(default = "yes")]
    pub use_ergo_multiplier: bool,
    /// Wall-clock budget for the solver. `None` means the server default.
    pub timeout_ms: Option<u32>,
}

fn yes() -> bool {
    true
}

pub const DEFAULT_TIMEOUT_MS: u32 = 30_000;

/// Safety-first anchors the ergonomic penalty at `gamma = 1` and drops
/// preferences to `omega = 0.01`. The weights are integers, so that ratio is
/// expressed by scaling the whole objective by this factor - which leaves the
/// argmax untouched, since scaling every term scales the objective uniformly.
pub const PREFERENCE_SCALE: u32 = 100;

impl Default for SolverParams {
    /// Only reached when a client omits `params` entirely; the UI always
    /// derives weights from the station actually being solved. The nominal
    /// dimensions below are a placeholder, not a recommendation.
    fn default() -> Self {
        WeightPreset::Balanced.params(StationDims::NOMINAL, PresetInputs::default())
    }
}

/// The station dimensions the strategy formulas are written in terms of.
///
/// `n` and `m` bound the largest preference reward the solver can reach
/// (`omega * m * n`, every operator on their first choice), and the ergonomic
/// scores bound the largest historical penalty it can reach.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StationDims {
    /// `N` - operators in the station, the team leader included.
    pub n: u32,
    /// `M` - jobs in the station.
    pub m: u32,
    /// `E_min` - worst ergonomic score in the station.
    pub e_min: u32,
    /// `E_max` - best ergonomic score in the station.
    pub e_max: u32,
}

impl StationDims {
    /// Stand-in dimensions for [`SolverParams::default`].
    pub const NOMINAL: Self = Self {
        n: 12,
        m: 8,
        e_min: 1,
        e_max: 5,
    };

    /// Read the dimensions off a station. Jobs are the `ergo_score` keys, the
    /// same way the solver derives them.
    ///
    /// Every field is floored at 1: a station with no jobs, nobody in it, or a
    /// zero ergonomic score would otherwise divide by zero below.
    pub fn of(station: &Station) -> Self {
        let scores = || station.ergo_score.values().map(|e| u32::from(*e));
        Self {
            n: (station.people.len() as u32).max(1),
            m: (station.ergo_score.len() as u32).max(1),
            e_min: scores().min().unwrap_or(1).max(1),
            e_max: scores().max().unwrap_or(1).max(1),
        }
    }
}

/// The policy choices that feed the strategy formulas but are not weights
/// themselves.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PresetInputs {
    /// `d_limit` - consecutive days on one job after which rotating off it is
    /// mandatory rather than merely preferred.
    pub d_limit: u32,
    /// `K` - how many times more undesirable an external operator is than
    /// putting the team leader on a job. Fixes `beta = K * alpha`.
    pub k: u32,
    /// `tau` - days of history the fairness term looks back over. Safety-first
    /// sizes `alpha` from it, since the ergonomic penalty it must outweigh
    /// accumulates over the whole window.
    pub tau: u32,
}

impl Default for PresetInputs {
    fn default() -> Self {
        Self {
            d_limit: 2,
            k: 2,
            tau: 8,
        }
    }
}

/// Named bundles of [`SolverParams`], so operators can pick an intent
/// instead of hand-tuning six numbers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeightPreset {
    Balanced,
    SafetyFirst,
    HappinessFirst,
}

impl WeightPreset {
    pub const ALL: [WeightPreset; 3] = [
        WeightPreset::Balanced,
        WeightPreset::SafetyFirst,
        WeightPreset::HappinessFirst,
    ];

    /// The single source of truth for preset weights.
    ///
    /// The weights are *derived* from the station rather than hardcoded,
    /// because every threshold below is a statement about outweighing some
    /// other term of the objective, and those terms scale with the station's
    /// size and ergonomic spread. A constant that dominates preferences in an
    /// 8-job station is a rounding error in a 40-job one.
    ///
    /// Three strategies, each anchoring a different term at 1:
    ///
    /// **Balanced** negotiates between stated preferences and compounding
    /// fatigue. Preferences are the anchor (`omega = 1`). To mandate rotation
    /// off a job after `d_limit` consecutive days, the ergonomic penalty must
    /// exceed the largest preference swing available:
    ///
    /// ```text
    /// gamma > omega * M * N / (d_limit * E_min)
    /// alpha > N * M * omega          beta = K * alpha
    /// ```
    ///
    /// **Safety-first** anchors the ergonomic penalty (`gamma = 1`) and
    /// reduces preferences to tie-breakers between equally safe assignments
    /// (`omega = 0.01`, applied as [`PREFERENCE_SCALE`]). The leader and
    /// external penalties must now clear the largest ergonomic penalty the
    /// station can accumulate over the whole history window:
    ///
    /// ```text
    /// alpha > N * tau * E_max        beta = K * alpha
    /// ```
    ///
    /// **Happiness-first** suits stations whose jobs are physically alike. It
    /// drops the ergonomic multiplier from the objective entirely, so `gamma`
    /// becomes a pure boredom penalty that still has to outgrow the reward for
    /// repeating a favourite job. `alpha` and `beta` are Balanced's:
    ///
    /// ```text
    /// gamma > omega * M * N / (d_limit * E_max)
    /// ```
    ///
    /// One caveat, kept faithful to the specification rather than silently
    /// "fixed": Happiness-first divides by `E_max` where Balanced divides by
    /// `E_min`. Since dropping the multiplier shrinks the historical penalty,
    /// a rotation guarantee on its own would call for a *larger* `gamma` here,
    /// not a smaller one - the two coincide only when `E_min == E_max`. The
    /// formulas are implemented as written; if the intent was to swap those
    /// two denominators, this is the line to change.
    pub fn params(self, dims: StationDims, inputs: PresetInputs) -> SolverParams {
        let (n, m) = (u64::from(dims.n), u64::from(dims.m));
        let (e_min, e_max) = (u64::from(dims.e_min), u64::from(dims.e_max));
        let d_limit = u64::from(inputs.d_limit.max(1));
        let tau = u64::from(inputs.tau.max(1));
        let k = u64::from(inputs.k);
        let scale = u64::from(PREFERENCE_SCALE);

        // The formulas are strict inequalities. The smallest integer strictly
        // above a quotient is that quotient floored, plus one.
        let above = |numerator: u64, denominator: u64| numerator / denominator.max(1) + 1;

        let (omega, alpha, gamma, use_ergo_multiplier) = match self {
            WeightPreset::Balanced => {
                let omega = 1;
                let alpha = above(n * m * omega, 1);
                (omega, alpha, above(omega * m * n, d_limit * e_min), true)
            }
            WeightPreset::SafetyFirst => {
                // omega = 0.01 and gamma = 1, both multiplied through by the
                // scale factor to stay in integers.
                let alpha = above(scale * n * tau * e_max, 1);
                (1, alpha, scale, true)
            }
            WeightPreset::HappinessFirst => {
                let omega = 1;
                let alpha = above(n * m * omega, 1);
                (omega, alpha, above(omega * m * n, d_limit * e_max), false)
            }
        };
        let beta = alpha.saturating_mul(k);

        // Keep the reported score positive: bound every penalty the solver can
        // incur. The leader and externals are capped by the job count, and the
        // historical term by a full window on the worst-ergonomics job.
        let worst_multiplier = if use_ergo_multiplier {
            e_max - e_min + 1
        } else {
            1
        };
        let offset = alpha
            .saturating_mul(m)
            .saturating_add(beta.saturating_mul(m))
            .saturating_add(gamma.saturating_mul(n * tau * worst_multiplier));

        SolverParams {
            offset: fits(offset),
            omega: fits(omega),
            alpha: fits(alpha),
            beta: fits(beta),
            tau: inputs.tau,
            gamma: fits(gamma),
            use_ergo_multiplier,
            timeout_ms: Some(DEFAULT_TIMEOUT_MS),
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            WeightPreset::Balanced => "Balanced",
            WeightPreset::SafetyFirst => "Safety-first",
            WeightPreset::HappinessFirst => "Happiness-first",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            WeightPreset::Balanced => {
                "Negotiates between stated preferences and compounding physical \
                 fatigue. Preferences are the anchor, and the ergonomic penalty \
                 is sized to force a rotation after d_limit consecutive days on \
                 the same job."
            }
            WeightPreset::SafetyFirst => {
                "Minimises physical strain: the ergonomic penalty becomes the \
                 anchor and preferences drop to tie-breakers between assignments \
                 that are equally safe."
            }
            WeightPreset::HappinessFirst => {
                "For stations whose jobs are physically alike. Drops the \
                 ergonomic multiplier entirely, leaving a pure boredom penalty \
                 that still rotates people off a favourite job."
            }
        }
    }
}

/// Saturate rather than wrap: the weights are only ever compared against each
/// other, so a pathological station is better served by a clamped weight than
/// by one that has silently wrapped around to nearly zero.
fn fits(value: u64) -> u32 {
    value.min(u64::from(u32::MAX)) as u32
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
    /// Today's team leader, and the authority on who leads: the roster's own
    /// `Role` values are a stale default that the server overwrites from this
    /// before solving. `None` means nobody leads, and is rejected by
    /// [`crate::validate::validate_leader`].
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
