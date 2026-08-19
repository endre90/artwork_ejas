//! The UI parses and re-emits the project's JSON formats. These are the
//! places where a schema slip would silently corrupt a customer's roster.

use ejas_core::structs::{Employee, Role, Station};
use ejas_ui::load;

fn repo(rel: &str) -> String {
    std::fs::read_to_string(format!("{}/../../{rel}", env!("CARGO_MANIFEST_DIR")))
        .unwrap_or_else(|e| panic!("reading {rel}: {e}"))
}

#[test]
fn bundled_examples_parse() {
    for (name, text) in load::EXAMPLES {
        let matrix = load::parse_matrix(text)
            .unwrap_or_else(|e| panic!("bundled example {name} does not parse: {e}"));
        assert!(!matrix.stations.is_empty(), "{name} has no stations");
        for (id, station) in &matrix.stations {
            assert!(!station.people.is_empty(), "{name}/{id} has no people");
            assert!(!station.ergo_score.is_empty(), "{name}/{id} has no jobs");
        }
    }
}

#[test]
fn vce_matrix_matches_the_documented_shape() {
    let matrix = load::parse_matrix(&repo("data/factory/VCE_matrix.json")).expect("parses");
    let station = &matrix.stations["CE"];
    assert_eq!(station.people.len(), 16);
    assert_eq!(station.ergo_score.len(), 12);
    assert_eq!(
        station
            .people
            .iter()
            .filter(|e| e.role == Role::TeamLeader)
            .count(),
        1
    );
}

#[test]
fn day_based_history_parses() {
    let days = load::parse_history(&repo("data/factory/VCE_history.json"), "K").expect("parses");
    assert!(!days.is_empty());
    // Day files carry their own leader; the fallback must not overwrite it.
    assert!(days.iter().all(|d| !d.leader.is_empty()));
}

#[test]
fn pass_based_history_parses_and_takes_the_supplied_leader() {
    let days =
        load::parse_history(&repo("data/factory/GTO_nov_history.json"), "A").expect("parses");
    assert!(!days.is_empty());
    // Pass records have no leader field, so every day gets the one we passed.
    assert!(days.iter().all(|d| d.leader == "A"));
}

#[test]
fn garbage_history_is_rejected_rather_than_silently_empty() {
    let err = load::parse_history("{\"not\": \"a history\"}", "A").unwrap_err();
    assert!(
        err.contains("day-") && err.contains("pass-"),
        "the error should say both formats were tried, got: {err}"
    );
}

#[test]
fn a_station_built_in_the_wizard_survives_export_and_reload() {
    // Exactly what the setup wizard produces: jobs, then people.
    let mut ergo_score = std::collections::HashMap::new();
    ergo_score.insert("O1".to_owned(), 1u8);
    ergo_score.insert("O2".to_owned(), 3u8);
    ergo_score.insert("O3".to_owned(), 2u8);

    let built = Station {
        ergo_score,
        people: vec![
            Employee {
                name: "Alice".into(),
                role: Role::TeamLeader,
                competences: vec!["O1".into()],
                preferences: vec![],
            },
            Employee {
                name: "Bob".into(),
                role: Role::Operator,
                competences: vec!["O1".into(), "O2".into(), "O3".into()],
                // Order is the preference rank, so it must survive verbatim.
                preferences: vec!["O3".into(), "O1".into()],
            },
        ],
    };

    let json = load::matrix_json("Line7", &built);
    let reloaded = load::parse_matrix(&json).expect("exported matrix re-parses");

    assert_eq!(reloaded.stations.len(), 1);
    let station = &reloaded.stations["Line7"];
    assert_eq!(station, &built, "export/reload changed the station");
    assert_eq!(
        station.people[1].preferences,
        vec!["O3".to_owned(), "O1".to_owned()],
        "preference order must be preserved"
    );
}
