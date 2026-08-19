//! Roster and request validation shared by the UI and the server.
//!
//! The UI runs these to disable the solve button and explain why; the
//! server runs them again because it cannot trust its client.

use crate::api::SolveRequest;
use crate::structs::{Role, Station};

/// A single reason a roster or request cannot be solved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Problem {
    /// Blocks solving.
    Error(String),
    /// Worth showing, but the solve will still run.
    Warning(String),
}

impl Problem {
    pub fn message(&self) -> &str {
        match self {
            Problem::Error(m) | Problem::Warning(m) => m,
        }
    }

    pub fn is_error(&self) -> bool {
        matches!(self, Problem::Error(_))
    }
}

/// Check a station is well-formed enough to solve.
pub fn validate_station(station: &Station) -> Vec<Problem> {
    let mut problems = Vec::new();

    if station.ergo_score.is_empty() {
        problems.push(Problem::Error("Add at least one job.".into()));
    }
    if station.people.is_empty() {
        problems.push(Problem::Error("Add at least one employee.".into()));
    }

    let leaders: Vec<&str> = station
        .people
        .iter()
        .filter(|e| e.role == Role::TeamLeader)
        .map(|e| e.name.as_str())
        .collect();
    match leaders.len() {
        1 => {}
        0 => problems.push(Problem::Error(
            "Exactly one employee must have the TeamLeader role; none do.".into(),
        )),
        n => problems.push(Problem::Error(format!(
            "Exactly one employee must have the TeamLeader role; {n} do ({}).",
            leaders.join(", ")
        ))),
    }

    let mut seen: Vec<&str> = Vec::new();
    for employee in &station.people {
        if employee.name.trim().is_empty() {
            problems.push(Problem::Error("An employee has a blank name.".into()));
        } else if seen.contains(&employee.name.as_str()) {
            problems.push(Problem::Error(format!(
                "Duplicate employee name '{}'.",
                employee.name
            )));
        } else {
            seen.push(&employee.name);
        }

        for job in &employee.competences {
            if !station.ergo_score.contains_key(job) {
                problems.push(Problem::Error(format!(
                    "'{}' has competence '{job}', which is not a job at this station.",
                    employee.name
                )));
            }
        }

        for job in &employee.preferences {
            if !station.ergo_score.contains_key(job) {
                problems.push(Problem::Error(format!(
                    "'{}' prefers '{job}', which is not a job at this station.",
                    employee.name
                )));
            } else if !employee.competences.contains(job) {
                // Not fatal: the model simply never gets to award the reward.
                problems.push(Problem::Warning(format!(
                    "'{}' prefers '{job}' but is not competent for it.",
                    employee.name
                )));
            }
        }
    }

    problems
}

/// Check a whole request, including the parts that depend on history.
pub fn validate_request(request: &SolveRequest) -> Vec<Problem> {
    let mut problems = validate_station(&request.station);

    let names: Vec<&str> = request
        .station
        .people
        .iter()
        .map(|e| e.name.as_str())
        .collect();

    // tau counts days of history; asking for more than exists silently
    // changes what the fairness term means.
    if request.params.tau as usize > request.history.len() {
        problems.push(Problem::Warning(format!(
            "tau is {} but only {} day(s) of history were supplied; \
             the fairness term will use everything available.",
            request.params.tau,
            request.history.len()
        )));
    }

    for (group, members) in [
        ("loaned", &request.loaned),
        ("absent", &request.absent),
        ("training", &request.training),
        ("supervision", &request.supervision),
    ] {
        for member in members.iter() {
            if !names.contains(&member.as_str()) {
                problems.push(Problem::Error(format!(
                    "'{member}' is marked {group} but is not on this station's roster."
                )));
            }
        }
    }

    for (employee, job) in &request.forced_assignments {
        if !names.contains(&employee.as_str()) {
            problems.push(Problem::Error(format!(
                "'{employee}' has a forced assignment but is not on this station's roster."
            )));
        }
        if !request.station.ergo_score.contains_key(job) {
            problems.push(Problem::Error(format!(
                "'{employee}' is forced onto '{job}', which is not a job at this station."
            )));
        }
    }

    problems
}

/// Convenience: the blocking problems only.
pub fn errors(problems: &[Problem]) -> Vec<&str> {
    problems
        .iter()
        .filter(|p| p.is_error())
        .map(|p| p.message())
        .collect()
}
