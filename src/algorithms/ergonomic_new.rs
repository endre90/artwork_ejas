use std::collections::HashMap;

use ast::Ast;
// use nanoid::nanoid;
use z3::{
    ast::{Bool, Int},
    *,
};

use crate::*;

pub fn calculate_ergonomic_assignment(
    station: &Station,
    history: Vec<Day>,
    alpha: u32, // How strongly to discourage leader usage
    beta: u32,  // How strongly to discourage external operator usage
    tau: u32, // Number of days to consider in the historical data (from last day to last day - tau)
    gamma: u32, // how strongly to penalize assigning the same employee–job pair that was frequently assigned in the past tau days
) -> (
    Vec<(String, String)>,
    Vec<String>,
    usize,
    Vec<Vec<bool>>,
    Vec<Vec<usize>>,
) {
    let mut jobs = vec![];
    let mut employees = vec![];
    let mut competences: Vec<(String, Vec<String>)> = vec![];
    let mut preferences: Vec<(String, Vec<String>)> = vec![];
    for op in station
        .ergo_score
        .keys()
        .map(|x| x.to_owned())
        .collect::<Vec<String>>()
    {
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

    // Create boolean variables for assignments
    let x: Vec<Vec<Bool>> = (0..employees.len())
        .map(|i| {
            (0..jobs.len())
                .map(|j| Bool::new_const(&ctx, format!("x_{}_{}", i, j)))
                .collect()
        })
        .collect();

    // Create boolean variables for external assignments
    let e: Vec<Bool> = (0..jobs.len())
        .map(|j| Bool::new_const(&ctx, format!("e_{}", j)))
        .collect();

    // Constraints: Each employee is assigned at most one job
    for i in 0..employees.len() {
        let employee_constraints: Vec<_> = (0..jobs.len()).map(|j| x[i][j].clone()).collect();
        let at_most_one_job_per_employee = ast::Bool::pb_le(
            &ctx,
            employee_constraints
                .iter()
                .map(|x| (x, 1))
                .collect::<Vec<(&ast::Bool, i32)>>()
                .as_slice(),
            1,
        );

        optimizer.assert(&at_most_one_job_per_employee);
    }

    // Constraints: Each job is assigned exactry to either one internal employee or one external employee
    for j in 0..jobs.len() {
        let job_constraints: Vec<_> = (0..employees.len()).map(|i| x[i][j].clone()).collect();
        let at_most_one_employee_per_job = ast::Bool::pb_eq(
            &ctx,
            vec![e[j].clone()]
                .into_iter()
                .chain(job_constraints.into_iter())
                .collect::<Vec<_>>()
                .iter()
                .map(|x| (x, 1))
                .collect::<Vec<(&ast::Bool, i32)>>()
                .as_slice(),
            1,
        );

        optimizer.assert(&at_most_one_employee_per_job);
    }

    // Constraints: Each job must be covered by at least one competent worker or one external worker
    let mut coverage_per_job: Vec<Vec<ast::Bool>> = vec![Vec::new(); jobs.len()];

    for i in 0..employees.len() {
        for j in 0..jobs.len() {
            if c_matrix[i][j] {
                coverage_per_job[j].push(x[i][j].clone());
            }
        }
    }

    for j in 0..jobs.len() {
        coverage_per_job[j].push(e[j].clone()); // Add the external worker option
        let job_covered = ast::Bool::pb_ge(
            &ctx,
            coverage_per_job[j]
                .iter()
                .map(|b| (b, 1))
                .collect::<Vec<(&ast::Bool, i32)>>()
                .as_slice(),
            1,
        );

        optimizer.assert(&job_covered);
    }

    // Constraints: Only assign jobs to employees are competent to perform them
    for i in 0..employees.len() {
        for j in 0..jobs.len() {
            optimizer.assert(&Bool::implies(
                &x[i][j],
                &Bool::from_bool(&ctx, c_matrix[i][j]),
            ));
        }
    }

    // Objective: Maximize preferences
    let mut preference_score = Vec::new();
    for i in 0..employees.len() {
        for j in 0..jobs.len() {
            let rank = p_matrix[i][j];
            let score = (jobs.len() - rank) as i32;
            preference_score.push((
                Bool::and(
                    &ctx,
                    vec![&x[i][j], &Bool::from_bool(&ctx, true)].as_slice(),
                ),
                score,
            ));
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
    let preference_sum_expr = Int::add(&ctx, &preference_score_sum);

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

    // Penalty for using external employees
    let external_penalty_sum: Int = (Int::from_i64(&ctx, beta as i64))
        * e.iter()
            .map(|e_j| e_j.ite(&Int::from_i64(&ctx, 1), &Int::from_i64(&ctx, 0)))
            .fold(Int::from_i64(&ctx, 0), |acc, x| acc + x);

    // Penalty for assigning assigning the same employee–job pair that was frequently assigned in the past $\tau$ days
    let h_matrix = build_historical_count_matrix(history, tau as usize, &employees, &jobs);
    let mut h_fairness_terms = Vec::new();
    for i in 0..employees.len() {
        for j in 0..jobs.len() {
            let hist_count = h_matrix[i][j] as i64;
            let penalty_expr = x[i][j].ite(
                &z3::ast::Int::from_i64(&ctx, hist_count),
                &z3::ast::Int::from_i64(&ctx, 0),
            );
            h_fairness_terms.push(penalty_expr);
        }
    }
    let sum_of_h_fairness = z3::ast::Int::add(&ctx, &h_fairness_terms);
    let h_fairness_penalty_sum = z3::ast::Int::from_i64(&ctx, gamma as i64) * sum_of_h_fairness;

    // Variable to track the total preference score
    let pref_score = Int::new_const(&ctx, "pref_score");

    // Build objective that includes a penalty if the leader is assigned
    if let Some(l_i) = leader_index {
        let mut leader_penalties = Vec::new();
        for j in 0..jobs.len() {
            let penalty_if_leader = x[l_i][j].ite(&Int::from_i64(&ctx, 1), &Int::from_i64(&ctx, 0));
            leader_penalties.push(penalty_if_leader);
        }
        let leader_penalty_sum =
            Int::from_i64(&ctx, alpha as i64) * Int::add(&ctx, &leader_penalties);

        let final_objective = Int::sub(&ctx, &[preference_sum_expr, leader_penalty_sum]);
        let final_objective = Int::sub(&ctx, &[final_objective, external_penalty_sum]);
        let final_objective = Int::sub(&ctx, &[final_objective, h_fairness_penalty_sum]);
        optimizer.assert(&pref_score._eq(&final_objective));
        optimizer.maximize(&final_objective);
    } else {
        // If no leader, just use the original preference sum
        let final_objective = Int::sub(&ctx, &[preference_sum_expr, external_penalty_sum]);
        let final_objective = Int::sub(&ctx, &[final_objective, h_fairness_penalty_sum]);
        optimizer.assert(&pref_score._eq(&final_objective));
        optimizer.maximize(&final_objective);
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
                    if model.eval(&x[i][j], true).unwrap().as_bool().unwrap() {
                        assignment.push((employees[i].clone(), jobs[j].clone()));
                    }
                }
            }

            let mut external_assignments = Vec::new();
            for j in 0..jobs.len() {
                if model.eval(&e[j], true).unwrap().as_bool().unwrap() {
                    external_assignments.push(jobs[j].clone());
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

fn build_competence_matrix(
    competence_map: &Vec<(String, Vec<String>)>,
    job_list: &Vec<String>,
) -> Vec<Vec<bool>> {
    // Create a hashmap to map job names to their indices
    let job_index: HashMap<&String, usize> = job_list
        .iter()
        .enumerate()
        .map(|(i, job)| (job, i))
        .collect();

    // Initialize the competence matrix with false values
    let mut competence_matrix = vec![vec![false; job_list.len()]; competence_map.len()];

    // Fill the competence matrix
    for (employee_index, (_, competences)) in competence_map.iter().enumerate() {
        for competence in competences {
            if let Some(&job_index) = job_index.get(competence) {
                competence_matrix[employee_index][job_index] = true;
            }
        }
    }

    competence_matrix
}

fn build_preference_matrix(
    preference_map: &Vec<(String, Vec<String>)>,
    job_list: &Vec<String>,
) -> Vec<Vec<usize>> {
    // Create a hashmap to map job names to their indices
    let job_index: HashMap<&String, usize> = job_list
        .iter()
        .enumerate()
        .map(|(i, job)| (job, i))
        .collect();

    // Initialize the preference matrix with default high values (e.g., job_list.len() which is worse than the worst preference)
    let mut preference_matrix =
        vec![vec![job_list.len() - 1; job_list.len()]; preference_map.len()];

    // Fill the preference matrix
    for (employee_index, (_, preferences)) in preference_map.iter().enumerate() {
        for (rank, job) in preferences.iter().enumerate() {
            if let Some(&job_idx) = job_index.get(job) {
                preference_matrix[employee_index][job_idx] = rank;
            }
        }
    }

    preference_matrix
}

// pub fn build_historical_count_matrix(
//     history_of_assignments: Vec<Day>,
//     period: usize,
//     employees: &Vec<String>,
//     jobs: &Vec<String>,
// ) -> Vec<Vec<u32>> {
//     // Create a mapping from employee/job names to indices
//     let employee_indices: HashMap<_, _> = employees
//         .iter()
//         .enumerate()
//         .map(|(i, name)| (name.clone(), i))
//         .collect();
//     let job_indices: HashMap<_, _> = jobs
//         .iter()
//         .enumerate()
//         .map(|(i, name)| (name.clone(), i))
//         .collect();

//     // Initialize the historic count matrix with zeros
//     let mut h_matrix = vec![vec![0; jobs.len()]; employees.len()];

//     // Iterate over the last `period` days in the history
//     let start_index = if history_of_assignments.len() > period {
//         history_of_assignments.len() - period
//     } else {
//         0
//     };

//     for day in &history_of_assignments[start_index..] {
//         for (employee, job) in &day.assignments {
//             if let (Some(&i), Some(&j)) = (employee_indices.get(employee), job_indices.get(job)) {
//                 h_matrix[i][j] += 1;
//             }
//         }
//     }

//     h_matrix
// }

pub fn build_historical_count_matrix_2(
    history_of_assignments: Vec<Day>,
    period: usize,
    employees: &Vec<String>,
    jobs: &Vec<String>,
) -> Vec<Vec<u32>> {
    // Initialize the historic count matrix with zeros
    let mut h_matrix = vec![vec![0; jobs.len()]; employees.len()];

    // Calculate how far back we go
    let start_index = history_of_assignments.len().saturating_sub(period);

    // For each day in the specified slice
    for day in &history_of_assignments[start_index..] {
        // `day.assignments` is already a Vec<(String, String)>
        // so iteration order is the insertion order in that vector.
        for (employee, job) in &day.assignments {
            // 1) Find the index of the employee in `employees`
            if let Some(i) = employees.iter().position(|e| e == employee) {
                // 2) Find the index of the job in `jobs`
                if let Some(j_pos) = jobs.iter().position(|job_name| job_name == job) {
                    // 3) Increment the counter
                    h_matrix[i][j_pos] += 1;
                }
            }
        }
    }

    h_matrix
}

#[cfg(test)]
mod tests {

    use std::fs;

    use crate::*;

    #[test]
    fn test_historic_matrix() -> Result<(), Box<dyn std::error::Error>> {
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

        let mut jobs = vec![];
        let mut employees = vec![];

        if let Some(station) = matrix.stations.get(&station) {
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
            }
        }

        let h_matrix = build_historical_count_matrix_2(history.clone(), 10, &employees, &jobs);
        // println!("       J  J  J  J  J");
        println!("Historic assignment count matrix:");
        for x in 0..h_matrix.len() {
            println!("{}:{:?}", employees[x], h_matrix[x])
        }

        if let Some(station_1) = matrix.stations.get("S0") {
            let s = calculate_historic_assignment(station_1, history, 0, 500, 10, 0);
            println!("Optimal internal assignment: {:?}", s.0);
            println!("Necessary external assignment: {:?}", s.1);
            println!("Total preference score: {:?}", s.2);
        }

        Ok(())
    }
}
