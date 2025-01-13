use std::collections::HashMap;

use ast::Ast;
// use nanoid::nanoid;
use z3::{
    ast::{Bool, Int},
    *,
};

use crate::*;

pub fn calculate_static_assignment(
    station: &Station,
    alpha: u32, // How strongly to discourage leader usage
) -> (
    Vec<(String, String)>,
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

    // Constraints: Each job must be covered by at least one competent worker
    let mut coverage_per_job: Vec<Vec<ast::Bool>> = vec![Vec::new(); jobs.len()];

    for i in 0..employees.len() {
        for j in 0..jobs.len() {
            if c_matrix[i][j] {
                coverage_per_job[j].push(x[i][j].clone());
            }
        }
    }

    for j in 0..jobs.len() {
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

    // Variable to track the total preference score
    let pref_score = Int::new_const(&ctx, "pref_score");

    // Build objective that includes a penalty if the leader is assigned
    if let Some(l_i) = leader_index {
        let mut leader_penalties = Vec::new();
        for j in 0..jobs.len() {
            let penalty_if_leader =
                x[l_i][j].ite(&Int::from_i64(&ctx, alpha as i64), &Int::from_i64(&ctx, 0));
            leader_penalties.push(penalty_if_leader);
        }
        let leader_penalty_sum = Int::add(&ctx, &leader_penalties);

        // final_objective = preference_sum - sum_of_leader_penalties
        let final_objective = Int::sub(&ctx, &[preference_sum_expr, leader_penalty_sum]);
        optimizer.assert(&pref_score._eq(&final_objective));
        optimizer.maximize(&final_objective);
    } else {
        // If no leader, just use the original preference sum
        optimizer.assert(&pref_score._eq(&preference_sum_expr));
        optimizer.maximize(&preference_sum_expr);
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

            log::info!(target: "employee_job_assignment", "Solution found");
            (assignment, pref_score as usize, c_matrix, p_matrix)
        }
        SatResult::Unsat => {
            log::warn!(target: "employee_job_assignment", "No solution found");
            (Vec::new(), 0, c_matrix, p_matrix)
        }
        _ => {
            log::error!(target: "employee_job_assignment", "Solver failed");
            (Vec::new(), 0, c_matrix, p_matrix)
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

#[cfg(test)]
mod tests {

    use std::fs;

    use crate::*;

    #[test]
    fn test_static_matrix() -> Result<(), Box<dyn std::error::Error>> {
        let manifest_dir =
            std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is not set");
        let path = format!("{}/data/matrix.json", manifest_dir);

        let json_content = fs::read_to_string(path)?;
        let matrix: Matrix = serde_json::from_str(&json_content)?;
        if let Some(station_1) = matrix.stations.get("S0") {
            let s = calculate_static_assignment(station_1, 0);
            println!("Optimal assignment: {:?}", s.0);
            println!("Total preference score: {}", s.1);
        }

        Ok(())
    }
}
