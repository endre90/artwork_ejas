//! The weight presets implement the three strategies from the paper, and each
//! weight is a threshold that has to outweigh a specific competing term of the
//! objective. Those thresholds scale with the station, so the tests below check
//! the *formulas* rather than pinning constants, then check the behaviour they
//! produce against the bundled VCE and GTO rosters.
//!
//! Note that the strategies only separate on a constrained roster. VCE can
//! satisfy everyone with no repeats, no leader and no externals, so all three
//! land on the same assignment there; GTO is where they diverge.

use artwork_ejas::calculate_ergonomic_assignment_with_timeout;
use ejas_core::api::{PresetInputs, SolveStatus, StationDims, WeightPreset, PREFERENCE_SCALE};
use ejas_core::structs::{Day, DayWrapper, Matrix, PassWrapper, Role, Station};

fn repo_path(rel: &str) -> String {
    format!("{}/../../{rel}", env!("CARGO_MANIFEST_DIR"))
}

fn station(file: &str, id: &str) -> Station {
    let text = std::fs::read_to_string(repo_path(file)).expect("matrix file");
    let matrix: Matrix = serde_json::from_str(&text).expect("matrix parses");
    matrix.stations[id].clone()
}

fn leader_of(station: &Station) -> String {
    station
        .people
        .iter()
        .find(|e| e.role == Role::TeamLeader)
        .expect("a team leader")
        .name
        .clone()
}

fn day_history(file: &str) -> Vec<Day> {
    let text = std::fs::read_to_string(repo_path(file)).expect("history file");
    serde_json::from_str::<Vec<DayWrapper>>(&text)
        .expect("day history parses")
        .into_iter()
        .map(|w| w.day)
        .collect()
}

fn pass_history(file: &str, leader: &str) -> Vec<Day> {
    let text = std::fs::read_to_string(repo_path(file)).expect("history file");
    serde_json::from_str::<Vec<PassWrapper>>(&text)
        .expect("pass history parses")
        .into_iter()
        .map(|w| Day {
            date: w.pass.date,
            station: w.pass.station,
            assignments: w.pass.assignments,
            leader: leader.to_owned(),
        })
        .collect()
}

struct Scores {
    preference: i64,
    /// Ergonomically weighted when `use_ergo_multiplier` is on, and a raw
    /// repeat count when it is off - so this is only comparable between runs
    /// that agree on that flag.
    historical: i64,
    leader: i64,
    external: i64,
}

fn solve(station: &Station, history: &[Day], preset: WeightPreset) -> Scores {
    let p = preset.params(StationDims::of(station), PresetInputs::default());
    let solution = calculate_ergonomic_assignment_with_timeout(
        station,
        history.to_vec(),
        p.offset,
        p.omega,
        p.alpha,
        p.beta,
        p.tau,
        p.gamma,
        p.use_ergo_multiplier,
        &[],
        &[],
        &[],
        &[],
        &[],
        p.timeout_ms,
    );
    assert_eq!(
        solution.status,
        SolveStatus::Sat,
        "{} should find an assignment",
        preset.label()
    );
    Scores {
        // Compare the unweighted terms: the weighted ones scale with the
        // weight itself, so they would "improve" even for a preset that
        // changed nothing about the actual assignment.
        preference: solution.preference_reward_score,
        historical: solution.historical_penalty_score,
        leader: solution.leader_penalty_score,
        external: solution.external_penalty_score,
    }
}

/// As [`solve`], but with one weight bundle overridden, to isolate the effect
/// of a single field.
fn solve_with(station: &Station, history: &[Day], p: ejas_core::api::SolverParams) -> Scores {
    let solution = calculate_ergonomic_assignment_with_timeout(
        station,
        history.to_vec(),
        p.offset,
        p.omega,
        p.alpha,
        p.beta,
        p.tau,
        p.gamma,
        p.use_ergo_multiplier,
        &[],
        &[],
        &[],
        &[],
        &[],
        p.timeout_ms,
    );
    assert_eq!(
        solution.status,
        SolveStatus::Sat,
        "should find an assignment"
    );
    Scores {
        preference: solution.preference_reward_score,
        historical: solution.historical_penalty_score,
        leader: solution.leader_penalty_score,
        external: solution.external_penalty_score,
    }
}

fn vce() -> (Station, Vec<Day>) {
    (
        station("data/factory/VCE_matrix.json", "CE"),
        day_history("data/factory/VCE_history.json"),
    )
}

fn gto() -> (Station, Vec<Day>) {
    let station = station("data/factory/GTO_matrix.json", "GTO");
    let leader = leader_of(&station);
    let history = pass_history("data/factory/GTO_nov_history.json", &leader);
    (station, history)
}

/// A worked station: N = 4 people, M = 3 jobs, ergonomic scores 1..5.
fn dims() -> StationDims {
    StationDims {
        n: 4,
        m: 3,
        e_min: 1,
        e_max: 5,
    }
}

#[test]
fn balanced_matches_the_specified_thresholds() {
    let inputs = PresetInputs {
        d_limit: 2,
        k: 2,
        tau: 8,
    };
    let p = WeightPreset::Balanced.params(dims(), inputs);

    // omega is the anchor.
    assert_eq!(p.omega, 1);
    // alpha > N * M * omega = 12
    assert_eq!(p.alpha, 13);
    assert!(p.alpha as u64 > 4 * 3 * u64::from(p.omega));
    // beta = K * alpha
    assert_eq!(p.beta, inputs.k * p.alpha);
    // gamma > omega * M * N / (d_limit * E_min) = 12 / 2 = 6
    assert_eq!(p.gamma, 7);
    // Balanced keeps the ergonomic multiplier.
    assert!(p.use_ergo_multiplier);
    assert_eq!(p.tau, inputs.tau);
}

#[test]
fn safety_first_matches_the_specified_thresholds() {
    let inputs = PresetInputs::default();
    let p = WeightPreset::SafetyFirst.params(dims(), inputs);
    let scale = u64::from(PREFERENCE_SCALE);

    // omega = 0.01 and gamma = 1, expressed as integers scaled by 100.
    assert_eq!(p.omega, 1);
    assert_eq!(u64::from(p.gamma), scale);
    // alpha > N * tau * E_max, in the same scaled units: 100 * 4 * 8 * 5.
    let bound = scale * 4 * u64::from(inputs.tau) * 5;
    assert_eq!(u64::from(p.alpha), bound + 1);
    assert_eq!(p.beta, inputs.k * p.alpha);
    assert!(p.use_ergo_multiplier);
}

#[test]
fn happiness_first_drops_the_multiplier_and_reuses_balanced_penalties() {
    let inputs = PresetInputs::default();
    let balanced = WeightPreset::Balanced.params(dims(), inputs);
    let p = WeightPreset::HappinessFirst.params(dims(), inputs);

    // "The alpha and beta weights remain identical to the balanced strategy."
    assert_eq!((p.alpha, p.beta), (balanced.alpha, balanced.beta));
    assert_eq!(p.omega, 1);
    // gamma > omega * M * N / (d_limit * E_max) = 12 / 10 = 1
    assert_eq!(p.gamma, 2);
    // The defining difference: no per-job ergonomic scaling.
    assert!(!p.use_ergo_multiplier);
}

#[test]
fn weights_scale_with_the_station_rather_than_being_constants() {
    // The whole point of deriving them: a threshold that dominates
    // preferences in a small station must still dominate in a large one.
    let small = StationDims {
        n: 4,
        m: 3,
        e_min: 1,
        e_max: 5,
    };
    let large = StationDims {
        n: 40,
        m: 30,
        e_min: 1,
        e_max: 5,
    };
    let inputs = PresetInputs::default();

    for preset in WeightPreset::ALL {
        let s = preset.params(small, inputs);
        let l = preset.params(large, inputs);
        assert!(
            l.alpha > s.alpha,
            "{} did not scale alpha with the station ({} vs {})",
            preset.label(),
            l.alpha,
            s.alpha
        );
        assert!(
            l.beta > s.beta,
            "{} did not scale beta with the station",
            preset.label()
        );
    }
}

#[test]
fn alpha_outweighs_the_largest_preference_swing() {
    // Team leaders are a last resort: using one must cost more than any
    // preference gain it could buy, or the solver will trade them off.
    let d = dims();
    let inputs = PresetInputs::default();
    for preset in [WeightPreset::Balanced, WeightPreset::HappinessFirst] {
        let p = preset.params(d, inputs);
        let max_preference_swing = u64::from(p.omega) * u64::from(d.m) * u64::from(d.n);
        assert!(
            u64::from(p.alpha) > max_preference_swing,
            "{}: alpha {} does not clear the {} preference swing",
            preset.label(),
            p.alpha,
            max_preference_swing
        );
    }
}

#[test]
fn beta_always_exceeds_alpha_so_externals_are_the_later_resort() {
    let inputs = PresetInputs::default();
    for preset in WeightPreset::ALL {
        let p = preset.params(dims(), inputs);
        assert!(
            p.beta > p.alpha,
            "{}: external penalty {} must exceed the leader penalty {}",
            preset.label(),
            p.beta,
            p.alpha
        );
    }
}

#[test]
fn degenerate_stations_do_not_divide_by_zero() {
    // A zero ergonomic score, or an empty station, reaches the denominators.
    let d = StationDims::of(&Station {
        ergo_score: std::collections::HashMap::new(),
        people: Vec::new(),
    });
    for preset in WeightPreset::ALL {
        let p = preset.params(
            d,
            PresetInputs {
                d_limit: 0,
                k: 0,
                tau: 0,
            },
        );
        assert!(p.gamma >= 1, "{} produced gamma 0", preset.label());
    }
}

#[test]
fn safety_first_treats_the_leader_and_externals_as_true_last_resorts() {
    // alpha > N * tau * E_max puts both above the largest ergonomic penalty
    // the station can accumulate, so neither is ever worth spending.
    let (station, history) = gto();
    let balanced = solve(&station, &history, WeightPreset::Balanced);
    let safety = solve(&station, &history, WeightPreset::SafetyFirst);

    assert_eq!(safety.leader, 0, "safety-first put the leader on a job");
    assert_eq!(safety.external, 0, "safety-first called in an external");
    // On GTO, balanced does spend the leader - that is the divergence.
    assert!(
        balanced.leader > safety.leader,
        "balanced used the leader {} times, safety-first {}",
        balanced.leader,
        safety.leader
    );
}

#[test]
fn safety_first_accepts_strain_rather_than_spending_the_leader() {
    // A direct consequence of sizing alpha above the whole ergonomic range:
    // when the only way to clear a hard repeat is to put the leader on the
    // job, safety-first declines and takes the strain instead. Balanced, whose
    // alpha only has to clear the preference swing, takes the leader.
    //
    // This pins a real tension in the specification rather than endorsing it:
    // the strategy is described as minimising strain, yet on GTO it ends up
    // with more strain than Balanced. If that is not the intent, `alpha` for
    // SafetyFirst is the term to revisit.
    let (station, history) = gto();
    let balanced = solve(&station, &history, WeightPreset::Balanced);
    let safety = solve(&station, &history, WeightPreset::SafetyFirst);

    // Both keep the multiplier on, so the two numbers are comparable.
    assert!(
        safety.historical > balanced.historical,
        "expected safety-first to absorb strain balanced avoided ({} vs {})",
        safety.historical,
        balanced.historical
    );
}

#[test]
fn safety_first_does_not_buy_preferences() {
    // Preferences are demoted to tie-breakers between equally safe
    // assignments, so safety-first must never come out ahead on them.
    for (station, history) in [vce(), gto()] {
        let balanced = solve(&station, &history, WeightPreset::Balanced);
        let safety = solve(&station, &history, WeightPreset::SafetyFirst);
        assert!(
            safety.preference <= balanced.preference,
            "safety-first scored {} preferences, balanced {}",
            safety.preference,
            balanced.preference
        );
    }
}

#[test]
fn happiness_first_will_not_spend_the_leader_on_ergonomics() {
    // With the multiplier gone, a repeat on a physically hard job is just a
    // repeat, and no longer worth the team leader that Balanced spends on it.
    let (station, history) = gto();
    let balanced = solve(&station, &history, WeightPreset::Balanced);
    let happiness = solve(&station, &history, WeightPreset::HappinessFirst);

    assert!(
        balanced.leader > happiness.leader,
        "balanced used the leader {} times, happiness-first {}",
        balanced.leader,
        happiness.leader
    );
    assert_eq!(happiness.external, 0);
}

#[test]
fn the_ergonomic_multiplier_is_what_changes_the_assignment() {
    // Hold every weight fixed and flip only the flag, so the difference
    // cannot be attributed to Happiness-first's smaller gamma.
    let (station, history) = gto();
    let dims = StationDims::of(&station);
    let mut with = WeightPreset::Balanced.params(dims, PresetInputs::default());
    with.use_ergo_multiplier = true;
    let mut without = with;
    without.use_ergo_multiplier = false;

    let weighted = solve_with(&station, &history, with);
    let unweighted = solve_with(&station, &history, without);

    assert!(
        weighted.leader > unweighted.leader,
        "the multiplier made no difference to the assignment ({} vs {} leader uses)",
        weighted.leader,
        unweighted.leader
    );
}

#[test]
fn an_unconstrained_roster_leaves_the_strategies_indistinguishable() {
    // VCE can satisfy everyone at once. No strategy should damage that: the
    // thresholds exist to break ties, not to introduce cost where none exists.
    let (station, history) = vce();
    let balanced = solve(&station, &history, WeightPreset::Balanced);

    for preset in WeightPreset::ALL {
        let s = solve(&station, &history, preset);
        assert_eq!(
            (s.preference, s.leader, s.external),
            (balanced.preference, balanced.leader, balanced.external),
            "{} diverged on an unconstrained roster",
            preset.label()
        );
        assert_eq!(
            s.historical,
            0,
            "{} accepted a repeat on VCE",
            preset.label()
        );
    }
}

#[test]
fn the_derived_offset_keeps_the_reported_score_positive() {
    // `offset` exists only so the score reads as a positive integer, and it is
    // now derived rather than a flat 2000 - which it has to be, since
    // Safety-first's alpha alone can run into the tens of thousands.
    for (station, history) in [vce(), gto()] {
        for preset in WeightPreset::ALL {
            let p = preset.params(StationDims::of(&station), PresetInputs::default());
            let solution = calculate_ergonomic_assignment_with_timeout(
                &station,
                history.clone(),
                p.offset,
                p.omega,
                p.alpha,
                p.beta,
                p.tau,
                p.gamma,
                p.use_ergo_multiplier,
                &[],
                &[],
                &[],
                &[],
                &[],
                p.timeout_ms,
            );
            assert_eq!(solution.status, SolveStatus::Sat);
            assert!(
                solution.objective_score >= 0,
                "{} reported a negative score {} (offset {})",
                preset.label(),
                solution.objective_score,
                p.offset
            );
        }
    }
}
