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

fn run(request: SolveRequest) -> SolveResponse {
    let params = request.params;
    let timeout = params.timeout_ms.or(Some(DEFAULT_TIMEOUT_MS));

    let solution = calculate_ergonomic_assignment_with_timeout(
        &request.station,
        request.history,
        params.offset,
        params.omega,
        params.alpha,
        params.beta,
        params.tau,
        params.gamma,
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
