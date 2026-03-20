use std::time::{Duration, Instant};

use ast::Ast;
use z3::{
    ast::{Bool, Int},
    *,
};

use crate::*;

/// Note: The returned internal assignments now include the day index.
#[derive(Debug)]
pub struct CompleteAssignmentSolution {
    /// (day, employee, job)
    pub internal_assignments: Vec<(usize, String, String)>,
    /// (day, job)
    pub external_assignments: Vec<(usize, String)>,
    pub objective_score: i64,
    pub preference_reward_score: i64,
    pub weighted_preference_reward_score: i64,
    pub leader_penalty_score: i64,
    pub weighted_leader_penalty_score: i64,
    pub external_penalty_score: i64,
    pub weighted_external_penalty_score: i64,
    pub historical_penalty_score: i64,
    pub weighted_historical_penalty_score: i64,
    pub ergonomics_reward_score: i64,
    pub weighted_ergonomics_reward_score: i64,
    pub c_matrix: Vec<Vec<bool>>,  // competence matrix (returned for reference)
    pub p_matrix: Vec<Vec<usize>>, // preference matrix (returned for reference)
    pub h_matrix: Vec<Vec<u32>>,   // historical count matrix from the past
    pub solving_time: Duration,
}

/// This version of `calculate_ergonomic_assignment` now accepts a `horizon` parameter
/// (number of days in the planning period) and extends the model with time-indexed variables.
pub fn calculate_complete_assignment(
    station: &Station,
    history: Vec<Day>,
    horizon: usize, // number of days to plan ahead
    offset: u32,    // constant added to the objective (for aesthetics)
    omega: u32,     // weight for the preference reward
    alpha: u32,     // penalty weight for using the team leader
    beta: u32,      // penalty weight for using an external operator
    tau: u32,       // number of days to consider in the historical data
    gamma: u32,     // penalty weight for repeated (historical) employee–job assignments
    delta: u32,     // weight for ergonomic reward
    theta: u32,     // weight controlling how history reduces ergonomic benefit
) -> CompleteAssignmentSolution {
    // Prepare lists of jobs and employees (and their competence/preferences)
    let mut jobs = vec![];
    let mut employees = vec![];
    let mut competences: Vec<(String, Vec<String>)> = vec![];
    let mut preferences: Vec<(String, Vec<String>)> = vec![];

    // Sort jobs by name (or any other criterion)
    let mut sorted = station
        .ergo_score
        .keys()
        .map(|x| x.to_owned())
        .collect::<Vec<String>>();
    sorted.sort();
    for op in sorted {
        jobs.push(op);
    }

    for person in &station.people {
        employees.push(person.name.clone());
        competences.push((person.name.clone(), person.competences.clone()));
        preferences.push((person.name.clone(), person.preferences.clone()));
    }

    // Build matrices from the competence and preference information.
    let c_matrix = build_competence_matrix(&competences, &jobs);
    let p_matrix = build_preference_matrix(&preferences, &jobs);

    // Create the Z3 context and optimizer.
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let optimizer = Optimize::new(&ctx);

    // ======================================================
    // 1. DECISION VARIABLES: Time-indexed assignments
    // ======================================================

    // Create a 3D array for internal assignments:
    // x[i][j][t] is true if employee i is assigned to job j on day t.
    let x: Vec<Vec<Vec<Bool>>> = (0..employees.len())
        .map(|i| {
            (0..jobs.len())
                .map(|j| {
                    (0..horizon)
                        .map(|t| {
                            Bool::new_const(&ctx, format!("x_{}_{}_{}", i, j, t))
                        })
                        .collect()
                })
                .collect()
        })
        .collect();

    // Create a 2D array for external assignments:
    // e[j][t] is true if job j is covered by an external operator on day t.
    let e: Vec<Vec<Bool>> = (0..jobs.len())
        .map(|j| {
            (0..horizon)
                .map(|t| {
                    Bool::new_const(&ctx, format!("e_{}_{}", j, t))
                })
                .collect()
        })
        .collect();

    // ======================================================
    // 2. CONSTRAINTS
    // ======================================================

    // 2.1. Each employee is assigned at most one job per day.
    for i in 0..employees.len() {
        for t in 0..horizon {
            let employee_day_vars: Vec<_> = (0..jobs.len())
                .map(|j| x[i][j][t].clone())
                .collect();
            let at_most_one_job = ast::Bool::pb_le(
                &ctx,
                &employee_day_vars
                    .iter()
                    .map(|var| (var, 1))
                    .collect::<Vec<(&ast::Bool, i32)>>()
                    .as_slice(),
                1,
            );
            optimizer.assert(&at_most_one_job);
        }
    }

    // 2.2. Each job is covered exactly once per day (either by one internal assignment or externally).
    for j in 0..jobs.len() {
        for t in 0..horizon {
            let mut vars_for_job = vec![e[j][t].clone()];
            for i in 0..employees.len() {
                vars_for_job.push(x[i][j][t].clone());
            }
            let exactly_one = ast::Bool::pb_eq(
                &ctx,
                &vars_for_job
                    .iter()
                    .map(|var| (var, 1))
                    .collect::<Vec<(&ast::Bool, i32)>>()
                    .as_slice(),
                1,
            );
            optimizer.assert(&exactly_one);
        }
    }

    // 2.3. Competence coverage: Each job on each day must be covered either by an external operator
    //      or by at least one internal employee who is competent.
    for j in 0..jobs.len() {
        for t in 0..horizon {
            let mut coverage_vars: Vec<ast::Bool> = vec![];
            for i in 0..employees.len() {
                if c_matrix[i][j] {
                    coverage_vars.push(x[i][j][t].clone());
                }
            }
            coverage_vars.push(e[j][t].clone());
            let job_covered = ast::Bool::pb_ge(
                &ctx,
                &coverage_vars
                    .iter()
                    .map(|b| (b, 1))
                    .collect::<Vec<(&ast::Bool, i32)>>()
                    .as_slice(),
                1,
            );
            optimizer.assert(&job_covered);
        }
    }

    // 2.4. Only assign an internal employee to a job if they are competent.
    for i in 0..employees.len() {
        for j in 0..jobs.len() {
            for t in 0..horizon {
                let comp_constraint = Bool::implies(
                    &x[i][j][t],
                    &Bool::from_bool(&ctx, c_matrix[i][j]),
                );
                optimizer.assert(&comp_constraint);
            }
        }
    }

    // 2.5. Historical Count Variables: Track cumulative assignments (for fairness and ergonomics)
    // Create a 3D array h[i][j][t] for t = 0,1,...,horizon.
    let mut h: Vec<Vec<Vec<Int>>> = Vec::new();
    for i in 0..employees.len() {
        let mut row = Vec::new();
        for j in 0..jobs.len() {
            let mut h_for_pair = Vec::new();
            for t in 0..=horizon {
                let var = Int::new_const(&ctx, format!("h_{}_{}_{}", i, j, t));
                h_for_pair.push(var);
            }
            row.push(h_for_pair);
        }
        h.push(row);
    }

    // Build the initial historical count matrix from past data.
    let h_matrix_initial = build_historical_count_matrix(history, tau as usize, &employees, &jobs);

    // Initialize h[i][j][0] with the historical counts.
    for i in 0..employees.len() {
        for j in 0..jobs.len() {
            let init_val = Int::from_i64(&ctx, h_matrix_initial[i][j] as i64);
            optimizer.assert(&h[i][j][0]._eq(&init_val));
        }
    }

    // For each day t = 1,...,horizon, update the historical count:
    // h[i][j][t] = h[i][j][t-1] + (if x[i][j][t-1] then 1 else 0)
    for i in 0..employees.len() {
        for j in 0..jobs.len() {
            for t in 1..=horizon {
                let x_int = x[i][j][t - 1].ite(&Int::from_i64(&ctx, 1), &Int::from_i64(&ctx, 0));
                optimizer.assert(&h[i][j][t]._eq(&(h[i][j][t - 1].clone() + x_int)));
            }
        }
    }

    // ======================================================
    // 3. OBJECTIVE FUNCTION TERMS (summed over time)
    // ======================================================

    // 3.1. Preference Reward Term:
    // For each day, if employee i is assigned to job j then add (jobs.len() - p_matrix[i][j]) to the reward.
    let mut preference_terms = Vec::new();
    for t in 0..horizon {
        for i in 0..employees.len() {
            for j in 0..jobs.len() {
                let rank = p_matrix[i][j];
                let score = (jobs.len() - rank) as i32;
                let term = x[i][j][t].ite(
                    &Int::from_i64(&ctx, score as i64),
                    &Int::from_i64(&ctx, 0),
                );
                preference_terms.push(term);
            }
        }
    }
    let preference_reward = Int::add(&ctx, &preference_terms);
    let omega_z3 = Int::from_i64(&ctx, omega as i64);
    let weighted_preference_reward = omega_z3 * preference_reward.clone();

    // 3.2. Leader Penalty Term:
    // Penalize assignments made by the team leader. (Assumes the first person with Role::TeamLeader.)
    let mut leader_penalty = Int::from_i64(&ctx, 0);
    for (index, person) in station.people.iter().enumerate() {
        if let Role::TeamLeader = person.role {
            let mut leader_terms = Vec::new();
            for t in 0..horizon {
                for j in 0..jobs.len() {
                    let term = x[index][j][t].ite(
                        &Int::from_i64(&ctx, 1),
                        &Int::from_i64(&ctx, 0),
                    );
                    leader_terms.push(term);
                }
            }
            leader_penalty = Int::add(&ctx, &leader_terms);
            break;
        }
    }
    let alpha_z3 = Int::from_i64(&ctx, alpha as i64);
    let weighted_leader_penalty = alpha_z3 * leader_penalty.clone();

    // 3.3. External Assignment Penalty Term:
    // For each day and job, add a penalty if the job is assigned externally.
    let mut external_terms = Vec::new();
    for t in 0..horizon {
        for j in 0..jobs.len() {
            let term = e[j][t].ite(
                &Int::from_i64(&ctx, 1),
                &Int::from_i64(&ctx, 0),
            );
            external_terms.push(term);
        }
    }
    let external_penalty = Int::add(&ctx, &external_terms);
    let beta_z3 = Int::from_i64(&ctx, beta as i64);
    let weighted_external_penalty = beta_z3 * external_penalty.clone();

    // 3.4. Historical Fairness Penalty Term:
    // For each day, penalize assignments that have been made frequently in the past.
    let mut historical_terms = Vec::new();
    for t in 0..horizon {
        for i in 0..employees.len() {
            for j in 0..jobs.len() {
                let term = h[i][j][t].clone() * x[i][j][t].ite(
                    &Int::from_i64(&ctx, 1),
                    &Int::from_i64(&ctx, 0),
                );
                historical_terms.push(term);
            }
        }
    }
    let historical_penalty = Int::add(&ctx, &historical_terms);
    let gamma_z3 = Int::from_i64(&ctx, gamma as i64);
    let weighted_historical_penalty = gamma_z3 * historical_penalty.clone();

    // 3.5. Ergonomic Reward Term:
    // For each assignment on day t, reward according to the job's ergonomic score,
    // reduced by historical usage. (Here we use a simplified formulation.)
    let ergo_scores: Vec<i32> = jobs
        .iter()
        .map(|job| station.ergo_score.get(job).copied().unwrap_or(1) as i32)
        .collect();
    let mut ergonomic_terms = Vec::new();
    for t in 0..horizon {
        for i in 0..employees.len() {
            for j in 0..jobs.len() {
                let ergo_score_j = ergo_scores[j];
                // In a more refined model you might compute:
                //   effective_reward = round((tau * ergo_score_j) / (1 + theta * h[i][j][t]))
                // Here we use a simplified version:
                let numerator = (tau as i64) * (ergo_score_j as i64);
                // (For simplicity, we assume denominator 1; see note below.)
                let ergonomic_reward_value = numerator;
                let term = x[i][j][t].ite(
                    &Int::from_i64(&ctx, ergonomic_reward_value),
                    &Int::from_i64(&ctx, 0),
                );
                ergonomic_terms.push(term);
            }
        }
    }
    let ergonomics_reward = Int::add(&ctx, &ergonomic_terms);
    let delta_z3 = Int::from_i64(&ctx, delta as i64);
    let weighted_ergonomics_reward = delta_z3 * ergonomics_reward.clone();

    // 3.6. Final Objective:
    // Maximize the weighted sum of the preference reward and ergonomics reward,
    // while subtracting the leader, external, and historical penalties, plus an offset.
    let offset_z3 = Int::from_i64(&ctx, offset as i64);
    let objective = offset_z3 + weighted_preference_reward.clone()
        - weighted_leader_penalty.clone()
        - weighted_external_penalty.clone()
        - weighted_historical_penalty.clone()
        + weighted_ergonomics_reward.clone();
    optimizer.maximize(&objective);

    // Optionally track the objective value.
    let objective_tracker = Int::new_const(&ctx, "objective");
    optimizer.assert(&objective_tracker._eq(&objective));

    // ======================================================
    // 4. SOLVE THE MODEL
    // ======================================================
    let start_time = Instant::now();
    let result = optimizer.check(&[]);
    let solving_time = start_time.elapsed();

    match result {
        SatResult::Sat => {
            let model = optimizer.get_model().expect("Model should be available when Sat");

            // Helper: extract an i64 value from an Int AST.
            fn get_objective_value(model: &z3::Model, obj_ast: &Int, label: &str) -> i64 {
                match model.get_const_interp(obj_ast) {
                    Some(val) => val.as_i64().unwrap_or_else(|| {
                        println!("{} has an interpretation that isn't an i64", label);
                        0
                    }),
                    None => {
                        println!("{} has no interpretation in the model", label);
                        0
                    }
                }
            }

            let objective_score = get_objective_value(&model, &objective_tracker, "objective");
            let preference_reward_score =
                get_objective_value(&model, &preference_reward, "preference_reward");
            let weighted_preference_reward_score =
                get_objective_value(&model, &weighted_preference_reward, "weighted_preference_reward");
            let leader_penalty_score =
                get_objective_value(&model, &leader_penalty, "leader_penalty");
            let weighted_leader_penalty_score =
                get_objective_value(&model, &weighted_leader_penalty, "weighted_leader_penalty");
            let external_penalty_score =
                get_objective_value(&model, &external_penalty, "external_penalty");
            let weighted_external_penalty_score =
                get_objective_value(&model, &weighted_external_penalty, "weighted_external_penalty");
            let historical_penalty_score =
                get_objective_value(&model, &historical_penalty, "historical_penalty");
            let weighted_historical_penalty_score =
                get_objective_value(&model, &weighted_historical_penalty, "weighted_historical_penalty");
            let ergonomics_reward_score =
                get_objective_value(&model, &ergonomics_reward, "ergonomics_reward");
            let weighted_ergonomics_reward_score =
                get_objective_value(&model, &weighted_ergonomics_reward, "weighted_ergonomics_reward");

            // ======================================================
            // 5. EXTRACT THE SOLUTION
            // ======================================================
            let mut internal_assignments = Vec::new();
            for t in 0..horizon {
                for i in 0..employees.len() {
                    for j in 0..jobs.len() {
                        if let Some(true) = model.eval(&x[i][j][t], true).unwrap().as_bool() {
                            internal_assignments.push((t, employees[i].clone(), jobs[j].clone()));
                        }
                    }
                }
            }

            let mut external_assignments = Vec::new();
            for t in 0..horizon {
                for j in 0..jobs.len() {
                    if let Some(true) = model.eval(&e[j][t], true).unwrap().as_bool() {
                        external_assignments.push((t, jobs[j].clone()));
                    }
                }
            }

            log::info!(target: "employee_job_assignment", "Solution found");

            CompleteAssignmentSolution {
                internal_assignments,
                external_assignments,
                objective_score,
                preference_reward_score,
                weighted_preference_reward_score,
                leader_penalty_score,
                weighted_leader_penalty_score,
                external_penalty_score,
                weighted_external_penalty_score,
                historical_penalty_score,
                weighted_historical_penalty_score,
                ergonomics_reward_score,
                weighted_ergonomics_reward_score,
                c_matrix,
                p_matrix,
                h_matrix: h_matrix_initial, // returning the initial historical counts
                solving_time,
            }
        }
        SatResult::Unsat => {
            log::warn!(target: "employee_job_assignment", "No solution found");
            CompleteAssignmentSolution {
                internal_assignments: Vec::new(),
                external_assignments: Vec::new(),
                objective_score: 0,
                preference_reward_score: 0,
                weighted_preference_reward_score: 0,
                leader_penalty_score: 0,
                weighted_leader_penalty_score: 0,
                external_penalty_score: 0,
                weighted_external_penalty_score: 0,
                historical_penalty_score: 0,
                weighted_historical_penalty_score: 0,
                ergonomics_reward_score: 0,
                weighted_ergonomics_reward_score: 0,
                c_matrix,
                p_matrix,
                h_matrix: h_matrix_initial,
                solving_time,
            }
        }
        _ => {
            log::error!(target: "employee_job_assignment", "Solver returned an unknown or failed state");
            CompleteAssignmentSolution {
                internal_assignments: Vec::new(),
                external_assignments: Vec::new(),
                objective_score: 0,
                preference_reward_score: 0,
                weighted_preference_reward_score: 0,
                leader_penalty_score: 0,
                weighted_leader_penalty_score: 0,
                external_penalty_score: 0,
                weighted_external_penalty_score: 0,
                historical_penalty_score: 0,
                weighted_historical_penalty_score: 0,
                ergonomics_reward_score: 0,
                weighted_ergonomics_reward_score: 0,
                c_matrix,
                p_matrix,
                h_matrix: h_matrix_initial,
                solving_time,
            }
        }
    }
}



#[cfg(test)]
mod tests {

    use std::fs;

    use crate::*;

    #[test]
    fn test_incremental() -> Result<(), Box<dyn std::error::Error>> {
        let manifest_dir =
            std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is not set");
        let s = "S2";
        let e = "E0";
        let path = format!("{}/data/synthetic/{}_matrix_static.json", manifest_dir, s);

        let history_path = format!("{}/data/synthetic/{}_{}_history.json", manifest_dir, s, e);
        let history_content = fs::read_to_string(history_path)?;
        let history_wrapper: Vec<DayWrapper> = serde_json::from_str(&history_content)?;
        let history: Vec<Day> = history_wrapper.into_iter().map(|dw| dw.day).collect();

        let json_content = fs::read_to_string(path)?;
        let matrix: Matrix = serde_json::from_str(&json_content)?;

        let horizon = 2;
        let offset = 10;
        let omega = 1;
        let alpha = 50;
        let beta = 100;
        let tau = 10;
        let gamma = 1;
        let theta = 1;
        let delta = 1;

        if let Some(station) = matrix.stations.get(s) {
            let solutions = calculate_complete_assignment(
                station,
                history.clone(),
                horizon,
                offset,
                omega,
                alpha,
                beta,
                tau,
                gamma,
                delta,
                theta,
            );

            for a in solutions.internal_assignments {
                println!("{:?}", a)
            }
        }

        Ok(())
    }
}
