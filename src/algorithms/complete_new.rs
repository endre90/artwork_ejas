use std::collections::hash_map;

use ast::Ast;
use z3::{
    ast::{Bool, Int},
    *,
};

use crate::*;

pub fn calculate_complete_assignment(
    station: &Station,
    history: Vec<Day>,
    horizon: usize, // For how many days to plan ahead (the planning horizon (1 means only assignment for today))
    offset: u32,    // Add to objective to get a positive integer result (just for aesthetics)
    omega: u32,     // How strongly preference considerations influence the objective function
    alpha: u32,     // How strongly to discourage leader usage
    beta: u32,      // How strongly to discourage external operator usage
    tau: u32, // Number of days to consider in the historical data (from last day to last day - tau)
    gamma: u32, // how strongly to penalize assigning the same employee–job pair that was frequently assigned in the past tau days
    delta: u32, // Ergonomics weight
    theta: u32, // Weight controlling how the historical count reduces the ergonomics benefit of a job for a given employee.
                // A higher theta means historical assignment counts erode the ergonomics score more severely,
                // thereby discouraging repeated allocations of the same job to a single employee from an ergonomics standpoint
) -> (
    Vec<(String, String, usize)>,
    Vec<(String, usize)>,
    usize,
    Vec<Vec<bool>>,
    Vec<Vec<usize>>,
) {
    let mut jobs = vec![];
    let mut employees = vec![];
    let mut competences: Vec<(String, Vec<String>)> = vec![];
    let mut preferences: Vec<(String, Vec<String>)> = vec![];
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

    let c_matrix = build_competence_matrix(&competences, &jobs);
    let p_matrix = build_preference_matrix(&preferences, &jobs);

    // Create the Z3 context and optimizer
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let optimizer = Optimize::new(&ctx);

    let x: Vec<Vec<Vec<Bool>>> = (0..employees.len())
        .map(|i| {
            (0..jobs.len())
                .map(|j| {
                    (0..horizon)
                        .map(|t| Bool::new_const(&ctx, format!("x_{}_{}_{}", i, j, t)))
                        .collect()
                })
                .collect()
        })
        .collect();

    // Create boolean variables for external assignments
    let e: Vec<Vec<Bool>> = (0..jobs.len())
        .map(|j| {
            (0..horizon)
                .map(|t| Bool::new_const(&ctx, format!("e_{}_{}", j, t)))
                .collect()
        })
        .collect();

    // Constraints: Each employee is assigned at most one job per day
    for i in 0..employees.len() {
        for t in 0..horizon as usize {
            let employee_day_constraints: Vec<_> =
                (0..jobs.len()).map(|j| x[i][j][t].clone()).collect();

            let at_most_one_job_per_employee_per_day = ast::Bool::pb_le(
                &ctx,
                &employee_day_constraints
                    .iter()
                    .map(|x| (x, 1))
                    .collect::<Vec<(&ast::Bool, i32)>>()
                    .as_slice(),
                1,
            );

            optimizer.assert(&at_most_one_job_per_employee_per_day);
        }
    }

    // Constraints: Each job is assigned exactly to either one internal employee or one external employee per day
    for j in 0..jobs.len() {
        for t in 0..horizon as usize {
            let mut job_constraints = vec![];
            job_constraints.push(e[j][t].clone());
            for i in 0..employees.len() {
                job_constraints.push(x[i][j][t].clone());
            }

            let exactly_one_employee_per_job = ast::Bool::pb_eq(
                &ctx,
                job_constraints
                    .iter()
                    .map(|x| (x, 1))
                    .collect::<Vec<(&ast::Bool, i32)>>()
                    .as_slice(),
                1,
            );

            optimizer.assert(&exactly_one_employee_per_job);
        }
    }

    // Constraints: Each job must be covered by at least one competent worker or one external worker per day
    for j in 0..jobs.len() {
        for t in 0..horizon as usize {
            let mut coverage_per_job_day: Vec<ast::Bool> = Vec::new();

            // Add competent internal workers for job j on day t
            for i in 0..employees.len() {
                if c_matrix[i][j] {
                    coverage_per_job_day.push(x[i][j][t].clone());
                }
            }

            // Add external worker option for job j on day t
            coverage_per_job_day.push(e[j][t].clone());

            // Create the coverage constraint: sum >= 1
            let job_covered = ast::Bool::pb_ge(
                &ctx,
                coverage_per_job_day
                    .iter()
                    .map(|b| (b, 1))
                    .collect::<Vec<(&ast::Bool, i32)>>()
                    .as_slice(),
                1,
            );

            // Assert the constraint in the optimizer
            optimizer.assert(&job_covered);
        }
    }

    // Constraints: Only assign jobs to employees are competent to perform them
    for i in 0..employees.len() {
        for j in 0..jobs.len() {
            for t in 0..horizon as usize {
                optimizer.assert(&Bool::implies(
                    &x[i][j][t],
                    &Bool::from_bool(&ctx, c_matrix[i][j]),
                ));
            }
        }
    }

    // Objective: Maximize preferences
    let mut preference_score = Vec::new();
    for i in 0..employees.len() {
        for j in 0..jobs.len() {
            let rank = p_matrix[i][j];
            let score = (jobs.len() - rank) as i32;
            for t in 0..horizon as usize {
                preference_score.push((
                    Bool::and(
                        &ctx,
                        vec![&x[i][j][t], &Bool::from_bool(&ctx, true)].as_slice(),
                    ),
                    score,
                ));
            }
        }
    }

    let preference_score_sum: Vec<_> = preference_score
        .iter()
        .map(|(b, s)| {
            b.ite(
                &z3::ast::Int::from_i64(&ctx, *s as i64),
                &z3::ast::Int::from_i64(&ctx, 0),
            )
        })
        .collect();

    // Create an Int expression that sums up all preference contributions
    let preference_sum_expr =
        Int::from_i64(&ctx, omega as i64) * Int::add(&ctx, &preference_score_sum);

    // Find a leader index
    let mut leader_index: Option<usize> = None;
    for (i, person) in station.people.iter().enumerate() {
        match person.role {
            Role::TeamLeader => {
                leader_index = Some(i);
                break;
            }
            _ => (),
        }
    }

    // Convert beta to an Int once
    let beta_int = Int::from_i64(&ctx, beta as i64);

    // Prepare integers for 'true' (1) and 'false' (0)
    let one = Int::from_i64(&ctx, 1);
    let zero = Int::from_i64(&ctx, 0);

    // Count how many external assignments occur across all jobs and days
    let mut external_count = Int::from_i64(&ctx, 0);

    for j in 0..jobs.len() {
        for t in 0..horizon as usize {
            let is_external = e[j][t].ite(&one, &zero);
            external_count = external_count + is_external;
        }
    }

    // Multiply the total count of external assignments by the penalty factor `beta`
    let external_penalty_sum = beta_int * external_count;

    // // Penalty for using external employees
    // let external_penalty_sum: Int = (Int::from_i64(&ctx, beta as i64))
    //     * e.iter()
    //         .map(|e_j| e_j.ite(&Int::from_i64(&ctx, 1), &Int::from_i64(&ctx, 0)))
    //         .fold(Int::from_i64(&ctx, 0), |acc, x| acc + x);

    // // Convert beta into a Z3 Int
    // let beta_int = Int::from_i64(&ctx, beta as i64);

    // // Precompute 1 and 0
    // let one = Int::from_i64(&ctx, 1);
    // let zero = Int::from_i64(&ctx, 0);

    // // Summation over jobs and days, referencing e by index
    // let sum_ext = (0..jobs.len())
    //     .map(|j| {
    //         (0..horizon)
    //             .map(|t| {
    //                 // For each e[j][t] Boolean, convert to an Int (1 if true, 0 otherwise)
    //                 e[j][t].ite(&one, &zero)
    //             })
    //             .fold(Int::from_i64(&ctx, 0), |acc, x| acc + x)
    //     })
    //     .fold(Int::from_i64(&ctx, 0), |acc, x| acc + x);

    // // Multiply by beta
    // let external_penalty_sum = beta_int * sum_ext;

    // 1) Create a 3D array of Int expressions for historical usage
    let mut h = Vec::new();
    for i in 0..employees.len() {
        h.push(Vec::new());
        for j in 0..jobs.len() {
            h[i].push(Vec::new());
            for t in 0..=horizon {
                // Create a new Int variable for h[i][j][t]
                let var_name = format!("h_{}_{}_{}", i, j, t);
                let var = Int::new_const(&ctx, var_name);
                h[i][j].push(var);
            }
        }
    }

    // Penalty for assigning assigning the same employee–job pair that was frequently assigned in the past $\tau$ days
    let h_matrix: Vec<Vec<u32>> =
        build_historical_count_matrix(history.clone(), tau as usize, &employees, &jobs);

    // 2) Add constraints:
    //    - h[i][j][0] = h_matrix[i][j] (the initial “historical” usage)
    //    - For each t >= 1, h[i][j][t] = h[i][j][t-1] + x[i][j][t-1]
    for i in 0..employees.len() {
        for j in 0..jobs.len() {
            // Initialize day 0
            let initial_val = Int::from_i64(&ctx, h_matrix[i][j].into());
            optimizer.assert(&h[i][j][0]._eq(&initial_val));

            // For each subsequent day
            for t in 1..=horizon as usize {
                // x[i][j][t-1] is Bool, so convert it to Int with ite
                let x_int = x[i][j][t - 1].ite(&Int::from_i64(&ctx, 1), &Int::from_i64(&ctx, 0));

                // h[i][j][t] = h[i][j][t-1] + x_int
                let prev_val = &h[i][j][t - 1];
                optimizer.assert(&h[i][j][t]._eq(&(prev_val + x_int)));
            }
        }
    }

    // Penalty for assigning the same employee–job pair that was frequently assigned in the past $\tau$ days
    let mut h_fairness_terms = Vec::new();
    for i in 0..employees.len() {
        for j in 0..jobs.len() {
            for t in 0..horizon as usize {
                // x[i][j][t] is Bool, so convert it to Int with ite
                let x_int = x[i][j][t].ite(&Int::from_i64(&ctx, 1), &Int::from_i64(&ctx, 0));
                let penalty_expr = h[i][j][t].clone() * x_int;
                h_fairness_terms.push(penalty_expr);
            }
        }
    }
    let sum_of_h_fairness = z3::ast::Int::add(&ctx, &h_fairness_terms);
    let h_fairness_penalty_sum = z3::ast::Int::from_i64(&ctx, gamma as i64) * sum_of_h_fairness;

    // Variable to track the total preference score
    let pref_score = Int::new_const(&ctx, "pref_score");

    // Build the ergonomic scores for jobs
    let ergo_scores: Vec<i32> = jobs
        .iter()
        .map(|job| station.ergo_score.get(job).copied().unwrap_or(1) as i32) // Default to 1 if not found
        .collect();

    // Ergonomic reward terms
    let mut ergonomic_terms = Vec::new();

    // Compute the effective ergonomics matrix
    for t in 0..horizon as usize {
        let h_matrix =
            build_historical_count_matrix(history.clone(), tau as usize, &employees, &jobs);
        let e_eff_matrix: Vec<Vec<(i32, i32)>> = h_matrix
            .iter()
            .enumerate()
            .map(|(_, row)| {
                row.iter()
                    .enumerate()
                    .map(|(j, &h_ij)| {
                        let e_j = ergo_scores[j];
                        (e_j, (1 + theta * h_ij) as i32)
                    })
                    .collect()
            })
            .collect();

        for i in 0..employees.len() {
            for j in 0..jobs.len() {
                let e_eff = e_eff_matrix[i][j];
                let ergonomic_reward = x[i][j][t].ite(
                    &z3::ast::Int::from_i64(&ctx, (e_eff.0 as f64 / e_eff.1 as f64).round() as i64),
                    &z3::ast::Int::from_i64(&ctx, 0),
                );
                ergonomic_terms.push(ergonomic_reward);
            }
        }
    }
    // Total ergonomic reward
    let ergonomic_reward_sum =
        Int::from_i64(&ctx, delta as i64) * z3::ast::Int::add(&ctx, &ergonomic_terms);

    // Build objective that includes a penalty if the leader is assigned
    if let Some(l_i) = leader_index {
        let mut leader_penalties = Vec::new();
        for j in 0..jobs.len() {
            for t in 0..horizon as usize {
                let penalty_if_leader =
                    x[l_i][j][t].ite(&Int::from_i64(&ctx, 1), &Int::from_i64(&ctx, 0));
                leader_penalties.push(penalty_if_leader);
            }
        }
        let leader_penalty_sum =
            Int::from_i64(&ctx, alpha as i64) * Int::add(&ctx, &leader_penalties);

        let final_objective = Int::sub(&ctx, &[preference_sum_expr, leader_penalty_sum]);
        let final_objective = Int::sub(&ctx, &[final_objective, external_penalty_sum]);
        let final_objective = Int::sub(&ctx, &[final_objective, h_fairness_penalty_sum]);
        let final_objective = Int::add(&ctx, &[final_objective, ergonomic_reward_sum]);
        let final_objective =
            Int::add(&ctx, &[final_objective, Int::from_i64(&ctx, offset as i64)]);
        optimizer.assert(&pref_score._eq(&final_objective));
        optimizer.maximize(&pref_score);
    } else {
        // If no leader, just use the original preference sum
        let final_objective = Int::sub(&ctx, &[preference_sum_expr, external_penalty_sum]);
        let final_objective = Int::sub(&ctx, &[final_objective, h_fairness_penalty_sum]);
        let final_objective = Int::add(&ctx, &[final_objective, ergonomic_reward_sum]);
        let final_objective =
            Int::add(&ctx, &[final_objective, Int::from_i64(&ctx, offset as i64)]);
        optimizer.assert(&pref_score._eq(&final_objective));
        optimizer.maximize(&pref_score);
    }

    // Check satisfiability and print the solution
    match optimizer.check(&[]) {
        SatResult::Sat => {
            let model = optimizer.get_model().unwrap();
            let mut assignment = Vec::new();

            let pref_score_interp = model.get_const_interp(&pref_score);
            let pref_score = match pref_score_interp {
                Some(x) => x.as_i64().unwrap(),
                None => {
                    println!("pref_score has no interpretation");
                    0
                }
            };

            for i in 0..employees.len() {
                for j in 0..jobs.len() {
                    for t in 0..horizon as usize {
                        if model.eval(&x[i][j][t], true).unwrap().as_bool().unwrap() {
                            assignment.push((employees[i].clone(), jobs[j].clone(), t));
                        }
                    }
                }
            }

            let mut external_assignments = Vec::new();
            for j in 0..jobs.len() {
                for t in 0..horizon as usize {
                    if model.eval(&e[j][t], true).unwrap().as_bool().unwrap() {
                        external_assignments.push((jobs[j].clone(), t));
                    }
                }
            }

            log::info!(target: "employee_job_assignment", "Solution found");
            (
                assignment,
                external_assignments,
                pref_score as usize,
                c_matrix,
                p_matrix,
            )
        }
        SatResult::Unsat => {
            log::warn!(target: "employee_job_assignment", "No solution found");
            (Vec::new(), Vec::new(), 0, c_matrix, p_matrix)
        }
        _ => {
            log::error!(target: "employee_job_assignment", "Solver failed");
            (Vec::new(), Vec::new(), 0, c_matrix, p_matrix)
        }
    }
}

#[cfg(test)]
mod tests {

    use std::fs;

    use crate::*;

    #[test]
    fn test_ergonomic() -> Result<(), Box<dyn std::error::Error>> {
        let station = "S0".to_string();
        let example = "E1".to_string();
        let manifest_dir =
            std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is not set");
        let matrix_path = format!("{}/data/{}_matrix.json", manifest_dir, station);
        let matrix_content = fs::read_to_string(matrix_path)?;
        let matrix: Matrix = serde_json::from_str(&matrix_content)?;

        let history_path = format!("{}/data/{}_{}_history.json", manifest_dir, station, example);
        let history_content = fs::read_to_string(history_path)?;
        let history_wrapper: Vec<DayWrapper> = serde_json::from_str(&history_content)?;
        let history: Vec<Day> = history_wrapper.into_iter().map(|dw| dw.day).collect();

        let horizon: usize = 3; // For how many days to plan ahead (the planning horizon (1 means only assignment for today))
        let offset: u32 = 10000; // Add to objective to get a positive integer result (just for aesthetics)
        let omega: u32 = 1000; // How strongly preference considerations influence the objective function
        let alpha: u32 = 10; // How strongly to discourage leader usage
        let beta: u32 = 1000; // How strongly to discourage external operator usage
        let tau: u32 = 10; // Number of days to consider in the historical data (from last day to last day - tau)
        let gamma: u32 = 1; // how strongly to penalize assigning the same employee–job pair that was frequently assigned in the past tau days
        let delta: u32 = 0; // Ergonomics weight
        let theta: u32 = 0; // Weight controlling how the historical count reduces the ergonomics benefit of a job for a given employee.

        if let Some(station_1) = matrix.stations.get("S0") {
            let s = calculate_complete_assignment(
                station_1, history, horizon, offset, omega, alpha, beta, tau, gamma, delta, theta,
            );

            for day in 0..horizon as usize {
                println!("Day {}:", day);
                for (employee, job, d) in &s.0 {
                    if *d == day {
                        println!("  {} -> {}", employee, job);
                    }
                }
            }

            println!("Optimal internal assignment: {:?}", s.0);
            println!("Necessary external assignment: {:?}", s.1);
            println!("Total preference score: {:?}", s.2);
        }

        Ok(())
    }
}
