//! Producing the on-disk result shape.
//!
//! The field names and nesting mirror `print_assignments_as_serde_json` in
//! the solver crate, so downloaded results stay readable by the existing
//! `scripts/*.py` plotting pipeline and by the history loader.

use ejas_core::structs::{Date, Day};

use crate::app::{EjasApp, Outcome};

/// Today's result as a `Day`, ready to append to the history.
pub fn as_day(app: &EjasApp, outcome: &Outcome) -> Day {
    Day {
        date: next_date(app),
        station: app.problem.station_id.clone(),
        assignments: outcome.rows.clone(),
        leader: app.daily.team_leader.clone().unwrap_or_default(),
    }
}

/// The result in the `{"day": {...}}` envelope the evaluation files use.
pub fn result_json(app: &EjasApp) -> Option<String> {
    let outcome = app.outcome.as_ref()?;
    let r = &outcome.response;
    let date = next_date(app);

    let value = serde_json::json!({
        "day": {
            "date": { "year": date.year, "month": date.month, "day": date.day },
            "offset": r.offset,
            "pref": r.weighted_preference_reward_score,
            "lead": r.weighted_leader_penalty_score,
            "exte": r.weighted_external_penalty_score,
            "hist_ergo": r.weighted_historical_penalty_score,
            "total": r.objective_score,
            "solver_time": format!("{}ms", r.solving_time_ms),
            "station": app.problem.station_id,
            "leader": app.daily.team_leader.clone().unwrap_or_default(),
            "assignments": outcome.rows,
        }
    });

    serde_json::to_string_pretty(&value).ok()
}

/// The date to stamp on today's result.
///
/// There is no clock in a wasm build worth relying on, and the history is
/// only ever ordered, never date-arithmetic'd, so continue the sequence from
/// the last known day rather than inventing a calendar.
fn next_date(app: &EjasApp) -> Date {
    match app.problem.history.last() {
        Some(last) => Date {
            year: last.date.year,
            month: last.date.month,
            day: last.date.day.saturating_add(1),
        },
        None => Date {
            year: 1,
            month: 1,
            day: 1,
        },
    }
}
