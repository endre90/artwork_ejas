//! The weight presets are a product promise: each one must actually move the
//! assignment in the direction its name claims. They are plain data, so it is
//! easy to retune them and accidentally make one a no-op — these tests pin the
//! behaviour to the bundled datasets.

use artwork_ejas::calculate_ergonomic_assignment_with_timeout;
use ejas_core::api::{SolveStatus, WeightPreset};
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
    historical: i64,
}

fn solve(station: &Station, history: &[Day], preset: WeightPreset) -> Scores {
    let p = preset.params();
    let solution = calculate_ergonomic_assignment_with_timeout(
        station,
        history.to_vec(),
        p.offset,
        p.omega,
        p.alpha,
        p.beta,
        p.tau,
        p.gamma,
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
    }
}

#[test]
fn balanced_preset_still_matches_the_original_hardcoded_weights() {
    // These are the values `src/main.rs` used before the split. Historic
    // results are only comparable if Balanced keeps reproducing them.
    let p = WeightPreset::Balanced.params();
    assert_eq!(
        (p.offset, p.omega, p.alpha, p.beta, p.tau, p.gamma),
        (2000, 1, 192, 384, 8, 24)
    );
}

#[test]
fn every_preset_changes_exactly_one_weight_from_balanced() {
    let b = WeightPreset::Balanced.params();
    for preset in WeightPreset::ALL {
        let p = preset.params();
        assert_eq!(p.offset, b.offset, "{} changed offset", preset.label());
        assert_eq!(p.alpha, b.alpha, "{} changed alpha", preset.label());
        assert_eq!(p.beta, b.beta, "{} changed beta", preset.label());
        // Widening tau counts more history and therefore *raises* the repeat
        // count, which would make a fairness preset look like it backfired.
        assert_eq!(p.tau, b.tau, "{} changed tau", preset.label());
    }
}

#[test]
fn preference_first_beats_balanced_on_preferences() {
    let station = station("data/factory/VCE_matrix.json", "CE");
    let history = day_history("data/factory/VCE_history.json");

    let balanced = solve(&station, &history, WeightPreset::Balanced);
    let preferred = solve(&station, &history, WeightPreset::PreferenceFirst);

    assert!(
        preferred.preference >= balanced.preference,
        "preference-first scored {} preferences, balanced scored {}",
        preferred.preference,
        balanced.preference
    );
    // On VCE, where people actually state preferences, it must do strictly
    // better - otherwise the preset is decorative.
    assert!(
        preferred.preference > balanced.preference,
        "preference-first made no difference on VCE ({} vs {})",
        preferred.preference,
        balanced.preference
    );
}

#[test]
fn fairness_first_beats_balanced_on_repeats() {
    let station = station("data/factory/GTO_matrix.json", "GTO");
    let leader = leader_of(&station);
    let history = pass_history("data/factory/GTO_nov_history.json", &leader);

    let balanced = solve(&station, &history, WeightPreset::Balanced);
    let fair = solve(&station, &history, WeightPreset::FairnessFirst);

    assert!(
        fair.historical <= balanced.historical,
        "fairness-first repeated {} pairings, balanced repeated {}",
        fair.historical,
        balanced.historical
    );
    // GTO is the dataset with repeats to remove; it must strictly improve.
    assert!(
        fair.historical < balanced.historical,
        "fairness-first made no difference on GTO ({} vs {})",
        fair.historical,
        balanced.historical
    );
}
