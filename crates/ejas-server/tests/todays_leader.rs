//! Today's team leader is chosen per solve, not stored on the roster.
//!
//! `SolveRequest::leader` used to travel with every request and then be
//! dropped on the floor: the solver reads the leader off `station.people`, so
//! whoever the matrix file designated led every day regardless of what the UI
//! showed. These tests pin the choice through to the assignment.

use artwork_ejas::calculate_ergonomic_assignment_with_timeout;
use ejas_core::api::{PresetInputs, SolveStatus, StationDims, WeightPreset};
use ejas_core::structs::{Day, Matrix, PassWrapper, Role, Station};
use ejas_core::validate;

fn repo_path(rel: &str) -> String {
    format!("{}/../../{rel}", env!("CARGO_MANIFEST_DIR"))
}

fn gto() -> (Station, Vec<Day>) {
    let text = std::fs::read_to_string(repo_path("data/factory/GTO_matrix.json")).expect("matrix");
    let matrix: Matrix = serde_json::from_str(&text).expect("matrix parses");
    let station = matrix.stations["GTO"].clone();

    let text =
        std::fs::read_to_string(repo_path("data/factory/GTO_nov_history.json")).expect("history");
    let history = serde_json::from_str::<Vec<PassWrapper>>(&text)
        .expect("history parses")
        .into_iter()
        .map(|w| Day {
            date: w.pass.date,
            station: w.pass.station,
            assignments: w.pass.assignments,
            leader: String::new(),
        })
        .collect();
    (station, history)
}

/// Solve with `leader` as today's team leader, returning who got work.
fn assigned_with_leader(
    station: &Station,
    history: &[Day],
    preset: WeightPreset,
    leader: Option<&str>,
) -> Vec<String> {
    let mut station = station.clone();
    station.set_todays_leader(leader);

    let p = preset.params(StationDims::of(&station), PresetInputs::default());
    let solution = calculate_ergonomic_assignment_with_timeout(
        &station,
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
    assert_eq!(solution.status, SolveStatus::Sat);
    let mut names: Vec<String> = solution
        .internal_assignments
        .into_iter()
        .map(|(name, _)| name)
        .collect();
    names.sort();
    names
}

#[test]
fn the_chosen_leader_is_the_one_left_off_the_board() {
    // Safety-first sizes alpha above the whole ergonomic range, so using the
    // leader is never worth it - whoever is named must end up unassigned.
    let (station, history) = gto();
    let candidates: Vec<String> = station.people.iter().map(|e| e.name.clone()).collect();

    for candidate in candidates.iter().take(4) {
        let assigned = assigned_with_leader(
            &station,
            &history,
            WeightPreset::SafetyFirst,
            Some(candidate),
        );
        assert!(
            !assigned.contains(candidate),
            "named '{candidate}' as today's leader but they were still assigned work: {assigned:?}"
        );
    }
}

#[test]
fn changing_todays_leader_changes_the_assignment() {
    // The regression this file exists for. Two different leaders on the same
    // roster and history must not produce the same board.
    let (station, history) = gto();
    let names: Vec<String> = station.people.iter().map(|e| e.name.clone()).collect();

    let first = assigned_with_leader(
        &station,
        &history,
        WeightPreset::SafetyFirst,
        Some(&names[0]),
    );
    let second = assigned_with_leader(
        &station,
        &history,
        WeightPreset::SafetyFirst,
        Some(&names[1]),
    );
    assert_ne!(
        first, second,
        "today's leader made no difference to who was assigned"
    );
}

#[test]
fn the_roster_file_no_longer_decides_who_leads() {
    // GTO ships with a TeamLeader role. Naming somebody else must override it,
    // leaving the file's leader available for normal work.
    let (station, history) = gto();
    let from_file = station
        .people
        .iter()
        .find(|e| e.role == Role::TeamLeader)
        .expect("GTO designates a leader")
        .name
        .clone();
    let someone_else = station
        .people
        .iter()
        .map(|e| e.name.clone())
        .find(|n| *n != from_file)
        .expect("another operator");

    let assigned = assigned_with_leader(
        &station,
        &history,
        WeightPreset::SafetyFirst,
        Some(&someone_else),
    );
    assert!(
        !assigned.contains(&someone_else),
        "the named leader was assigned work"
    );
    assert!(
        assigned.contains(&from_file),
        "the roster file's leader '{from_file}' should be available for work \
         once somebody else is leading, but was left off: {assigned:?}"
    );
}

#[test]
fn a_roster_without_a_leader_is_still_a_valid_roster() {
    // The role picker is gone from the roster tab, so a from-scratch roster
    // has nobody holding it. That must not block the roster screen.
    let (mut station, _) = gto();
    station.set_todays_leader(None);
    let problems = validate::validate_station(&station);
    let errors = validate::errors(&problems);
    assert!(
        errors.is_empty(),
        "roster rejected without a leader: {errors:?}"
    );
}

#[test]
fn but_solving_requires_one() {
    let (station, _) = gto();
    let errors: Vec<String> = validate::validate_leader(&station, None)
        .iter()
        .map(|p| p.message().to_owned())
        .collect();
    assert_eq!(
        errors.len(),
        1,
        "expected exactly one complaint: {errors:?}"
    );

    // And it has to be somebody who actually works here.
    let errors = validate::validate_leader(&station, Some("not-on-this-roster"));
    assert!(validate::errors(&errors).len() == 1);

    // A real name passes.
    let name = station.people[0].name.clone();
    assert!(validate::validate_leader(&station, Some(&name)).is_empty());
}
