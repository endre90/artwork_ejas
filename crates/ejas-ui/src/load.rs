//! Parsing the JSON the user hands us.

use ejas_core::structs::{Day, DayWrapper, Matrix, PassWrapper};

/// Bundled examples, so the app can demonstrate itself with no file at hand.
pub const EXAMPLES: &[(&str, &str)] = &[
    (
        "VCE (16 operators, 12 jobs)",
        include_str!("../../../data/factory/VCE_matrix.json"),
    ),
    (
        "GTO (11 operators, 8 jobs)",
        include_str!("../../../data/factory/GTO_matrix.json"),
    ),
];

pub fn parse_matrix(text: &str) -> Result<Matrix, String> {
    serde_json::from_str(text).map_err(|e| format!("not a valid matrix file: {e}"))
}

/// Read a history file.
///
/// Two shapes exist in the wild: day-based (`{"day": ...}`, carries a leader)
/// and pass-based (`{"pass": ...}`, does not). Try both rather than making the
/// user know which one they have.
pub fn parse_history(text: &str, leader: &str) -> Result<Vec<Day>, String> {
    if let Ok(days) = serde_json::from_str::<Vec<DayWrapper>>(text) {
        return Ok(days.into_iter().map(|w| w.day).collect());
    }

    match serde_json::from_str::<Vec<PassWrapper>>(text) {
        Ok(passes) => Ok(passes
            .into_iter()
            .map(|w| Day {
                date: w.pass.date,
                station: w.pass.station,
                assignments: w.pass.assignments,
                // Pass records carry no leader; use the one selected in the UI.
                leader: leader.to_owned(),
            })
            .collect()),
        Err(e) => Err(format!(
            "not a valid history file (tried both day- and pass-based formats): {e}"
        )),
    }
}

/// Serialise a station back out in the on-disk matrix schema.
pub fn matrix_json(station_id: &str, station: &ejas_core::structs::Station) -> String {
    let mut stations = std::collections::HashMap::new();
    stations.insert(station_id.to_owned(), station.clone());
    serde_json::to_string_pretty(&Matrix { stations })
        .unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}"))
}
