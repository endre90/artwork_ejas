//! The `/api/solve` handler.

use axum::http::StatusCode;
use axum::Json;

use artwork_ejas::calculate_ergonomic_assignment_with_timeout;
use ejas_core::api::{SolveRequest, SolveResponse, DEFAULT_TIMEOUT_MS};
use ejas_core::validate;

/// Solve one day's assignment.
///
/// Z3 is blocking and CPU-bound, so the solve runs on the blocking pool
/// rather than occupying an async worker for its whole duration.
pub async fn solve(
    Json(request): Json<SolveRequest>,
) -> Result<Json<SolveResponse>, (StatusCode, Json<ErrorBody>)> {
    // The UI validates too, but a server cannot trust its client.
    let problems = validate::validate_request(&request);
    let errors = validate::errors(&problems);
    if !errors.is_empty() {
        return Err(bad_request(errors.join(" ")));
    }

    let response = tokio::task::spawn_blocking(move || run(request))
        .await
        .map_err(|e| {
            // A panic inside the solver must not take the server down with it.
            tracing::error!("solver task failed: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorBody {
                    error: "the solver crashed; see server logs".into(),
                }),
            )
        })?;

    Ok(Json(response))
}

fn run(mut request: SolveRequest) -> SolveResponse {
    let params = request.params;
    let timeout = params.timeout_ms.or(Some(DEFAULT_TIMEOUT_MS));

    // Who leads changes from day to day, so the request is authoritative and
    // the roster's own roles are only a stale default. The solver reads the
    // role off the station, so stamp today's choice onto it before solving -
    // without this, `leader` travels with every request and is then ignored.
    request.station.set_todays_leader(request.leader.as_deref());

    let solution = calculate_ergonomic_assignment_with_timeout(
        &request.station,
        request.history,
        params.offset,
        params.omega,
        params.alpha,
        params.beta,
        params.tau,
        params.gamma,
        params.use_ergo_multiplier,
        &request.forced_assignments,
        &request.loaned,
        &request.absent,
        &request.training,
        &request.supervision,
        timeout,
    );

    SolveResponse {
        status: solution.status,
        internal_assignments: solution.internal_assignments,
        external_assignments: solution.external_assignments,
        objective_score: solution.objective_score,
        preference_reward_score: solution.preference_reward_score,
        weighted_preference_reward_score: solution.weighted_preference_reward_score,
        leader_penalty_score: solution.leader_penalty_score,
        weighted_leader_penalty_score: solution.weighted_leader_penalty_score,
        external_penalty_score: solution.external_penalty_score,
        weighted_external_penalty_score: solution.weighted_external_penalty_score,
        historical_penalty_score: solution.historical_penalty_score,
        weighted_historical_penalty_score: solution.weighted_historical_penalty_score,
        c_matrix: solution.c_matrix,
        p_matrix: solution.p_matrix,
        h_matrix: solution.h_matrix,
        solving_time_ms: solution.solving_time.as_millis() as u64,
        offset: solution.offset,
    }
}

#[derive(serde::Serialize)]
pub struct ErrorBody {
    pub error: String,
}

fn bad_request(message: String) -> (StatusCode, Json<ErrorBody>) {
    (StatusCode::BAD_REQUEST, Json(ErrorBody { error: message }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ejas_core::api::{PresetInputs, SolverParams, StationDims, WeightPreset};
    use ejas_core::structs::{Matrix, Role};

    fn gto_request(leader: Option<&str>) -> SolveRequest {
        let path = format!(
            "{}/../../data/factory/GTO_matrix.json",
            env!("CARGO_MANIFEST_DIR")
        );
        let text = std::fs::read_to_string(path).expect("matrix file");
        let matrix: Matrix = serde_json::from_str(&text).expect("matrix parses");
        let station = matrix.stations["GTO"].clone();
        let params =
            WeightPreset::SafetyFirst.params(StationDims::of(&station), PresetInputs::default());
        SolveRequest {
            station,
            history: Vec::new(),
            params: SolverParams {
                timeout_ms: Some(10_000),
                ..params
            },
            leader: leader.map(str::to_owned),
            forced_assignments: Vec::new(),
            loaned: Vec::new(),
            absent: Vec::new(),
            training: Vec::new(),
            supervision: Vec::new(),
        }
    }

    /// The handler must apply `leader` before solving. It previously did not:
    /// the field travelled with every request and the solver went on reading
    /// the role off the roster, so the Today tab's choice did nothing.
    #[test]
    fn run_honours_todays_leader_over_the_roster() {
        let request = gto_request(None);
        let from_file = request
            .station
            .people
            .iter()
            .find(|e| e.role == Role::TeamLeader)
            .expect("GTO designates a leader")
            .name
            .clone();
        let someone_else = request
            .station
            .people
            .iter()
            .map(|e| e.name.clone())
            .find(|n| *n != from_file)
            .expect("another operator");

        let response = run(gto_request(Some(&someone_else)));
        let assigned: Vec<&str> = response
            .internal_assignments
            .iter()
            .map(|(name, _)| name.as_str())
            .collect();

        assert!(
            !assigned.contains(&someone_else.as_str()),
            "'{someone_else}' was named leader but still got work: {assigned:?}"
        );
        assert!(
            assigned.contains(&from_file.as_str()),
            "'{from_file}' only leads according to the file and should be free \
             to work: {assigned:?}"
        );
    }
}
