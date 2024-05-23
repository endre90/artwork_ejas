use core::time;
use std::collections::HashMap;

use nanoid::nanoid;
use z3::{
    ast::{Bool, Int},
    *,
};

pub fn calculate_fair_horizon_assignment(
    anonymize: bool,
    employees: &Vec<String>,
    jobs: &Vec<String>,
    competence_map: &Vec<(String, Vec<String>)>,
    preference_map: &Vec<(String, Vec<String>)>,
    time_horizon_days: usize,
    k_pref_percentage_heuristic: usize,
    labmda_penalty: i64,
    beta_fairness_weight: i64,
    delta_tolerance_value: i64
    // ) -> (SatResult, String) {
) -> (Vec<(String, String, String)>, Vec<(String, String)>, usize) {
    let anon_employees_map = employees
        .iter()
        .map(|e| (e.to_owned(), nanoid!()))
        .collect::<HashMap<String, String>>();

    let anon_employees = anon_employees_map
        .iter()
        .map(|(_, val)| val.to_owned())
        .collect::<Vec<String>>();

    let anon_jobs_map = jobs
        .iter()
        .map(|e| (e.to_owned(), nanoid!()))
        .collect::<HashMap<String, String>>();

    let anon_jobs = anon_jobs_map
        .iter()
        .map(|(_, val)| val.to_owned())
        .collect::<Vec<String>>();

    let anon_competence_map = competence_map
        .iter()
        .map(|(employee, competences)| {
            let anon_employee = anon_employees_map.get(employee).unwrap().to_owned();
            let anon_competences = competences
                .iter()
                .map(|comp| anon_jobs_map.get(comp).unwrap().to_owned())
                .collect::<Vec<String>>();
            (anon_employee, anon_competences)
        })
        .collect::<Vec<(String, Vec<String>)>>();

    let anon_preference_map = preference_map
        .iter()
        .map(|(employee, preferences)| {
            let anon_employee = anon_employees_map.get(employee).unwrap().to_owned();
            let anon_preferences = preferences
                .iter()
                .map(|pref| anon_jobs_map.get(pref).unwrap().to_owned())
                .collect::<Vec<String>>();
            (anon_employee, anon_preferences)
        })
        .collect::<Vec<(String, Vec<String>)>>();

    let anon_competence_matrix = build_competence_matrix(&anon_competence_map, &anon_jobs);
    let competence_matrix = build_competence_matrix(competence_map, jobs);

    let anon_preference_matrix = build_preference_matrix(&anon_preference_map, &anon_jobs);
    let preference_matrix = build_preference_matrix(preference_map, jobs);

    let (employees, jobs, c_matrix, p_matrix) = match anonymize {
        false => (
            employees.clone(),
            jobs.clone(),
            competence_matrix.clone(),
            preference_matrix.clone(),
        ),
        true => (
            anon_employees,
            anon_jobs,
            anon_competence_matrix,
            anon_preference_matrix,
        ),
    };

    // Create the Z3 context and optimizer
    let mut cfg = Config::default();
    cfg.set_timeout_msec(5000);
    let ctx = Context::new(&cfg);
    let optimizer = Optimize::new(&ctx);

    // Calculate the number of top preferred jobs to consider
    let top_k_jobs = (k_pref_percentage_heuristic * jobs.len() / 100) as usize;

    // Create boolean variables for internal assignments
    let x: Vec<Vec<Vec<Bool>>> = (0..employees.len())
        .map(|i| {
            (0..jobs.len())
                .map(|j| {
                    (0..time_horizon_days)
                        .map(|t| Bool::new_const(&ctx, format!("x_{}_{}_{}", i, j, t)))
                        .collect()
                })
                .collect()
        })
        .collect();

    // Create boolean variables for external assignments
    let e: Vec<Vec<Bool>> = (0..jobs.len())
        .map(|j| {
            (0..time_horizon_days)
                .map(|t| Bool::new_const(&ctx, format!("e_{}", j)))
                .collect()
        })
        .collect();

    // Constraints: Each employee is assigned at most one job per time slot
    for i in 0..employees.len() {
        for t in 0..time_horizon_days {
            let employee_constraints: Vec<_> =
                (0..jobs.len()).map(|j| x[i][j][t].clone()).collect();
            let at_most_one_job_per_time_slot_per_employee = ast::Bool::pb_le(
                &ctx,
                employee_constraints
                    .iter()
                    .map(|x| (x, 1))
                    .collect::<Vec<(&ast::Bool, i32)>>()
                    .as_slice(),
                1,
            );
            optimizer.assert(&at_most_one_job_per_time_slot_per_employee);
        }
    }

    // Constraints: Each job is assigned to either one internal employee or one external employee per time slot
    for j in 0..jobs.len() {
        for t in 0..time_horizon_days {
            let job_constraints: Vec<_> =
                (0..employees.len()).map(|i| x[i][j][t].clone()).collect();
            let exactly_one_employee_per_job = ast::Bool::pb_eq(
                &ctx,
                vec![e[j][t].clone()]
                    .into_iter()
                    .chain(job_constraints.into_iter())
                    .collect::<Vec<_>>()
                    .iter()
                    .map(|x| (x, 1))
                    .collect::<Vec<(&ast::Bool, i32)>>()
                    .as_slice(),
                1,
            );

            optimizer.assert(&exactly_one_employee_per_job);
        }
    }

    // Constraints: Only assign jobs that employees are competent to perform them
    for i in 0..employees.len() {
        for j in 0..jobs.len() {
            for t in 0..time_horizon_days {
                optimizer.assert(&Bool::implies(
                    &x[i][j][t],
                    &Bool::from_bool(&ctx, c_matrix[i][j]),
                ));
            }
        }
    }

    // Constraints: Each job must be covered by at least one competent internal worker or an external worker per time slot
    for j in 0..jobs.len() {
        for t in 0..time_horizon_days {
            let mut should_be_covered = vec![];
            for i in 0..employees.len() {
                if c_matrix[i][j] {
                    should_be_covered.push(x[i][j][t].clone());
                }
            }
            should_be_covered.push(e[j][t].clone()); // Add the external worker option

            let job_covered = ast::Bool::pb_ge(
                &ctx,
                should_be_covered
                    .iter()
                    .map(|x| (x, 1))
                    .collect::<Vec<(&ast::Bool, i32)>>()
                    .as_slice(),
                1,
            );

            optimizer.assert(&job_covered);
        }
    }

    // Constraint: Fair Rotation of Highly Preferred Jobs
    for i in 0..employees.len() {
        let mut deviation_expr = Int::from_i64(&ctx, 0);
        for j in 0..jobs.len() {
            if p_matrix[i][j] <= top_k_jobs {
                for t in 0..time_horizon_days {
                    deviation_expr = &deviation_expr + &x[i][j][t].ite(&Int::from_i64(&ctx, 1), &Int::from_i64(&ctx, 0));
                }
            }
        }
        let ideal_share = Int::from_i64(&ctx, (time_horizon_days * top_k_jobs) as i64 / employees.len() as i64);
        let deviation = &deviation_expr - &ideal_share;
                let abs_deviation = deviation
                    .ge(&Int::from_i64(&ctx, 0))
                    .ite(&deviation, &(-&deviation));
        optimizer.assert(&abs_deviation.le(&Int::from_i64(&ctx, delta_tolerance_value as i64)));
    }

    // Objective Phi: Maximize preferences
    let mut preference_score = Vec::new();
    for i in 0..employees.len() {
        for (rank, &j) in p_matrix[i].iter().enumerate() {
            let score = (jobs.len() - rank) as i32;
            for t in 0..time_horizon_days {
                preference_score.push((
                    Bool::implies(&x[i][j][t], &Bool::from_bool(&ctx, true)),
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

    let total_preference_score = Int::add(&ctx, &preference_score_sum);

    // Objective Psi: Penalty for using external employees (accumulate penalty for each external employee occurrence)
    let penalty: Int = (0..time_horizon_days)
        .map(|t| {
            e.iter()
                .map(|e_j| e_j[t].ite(&Int::from_i64(&ctx, 1), &Int::from_i64(&ctx, 0)))
                .fold(Int::from_i64(&ctx, 0), |acc_a, x| acc_a + x)
        })
        .fold(Int::from_i64(&ctx, 0), |acc_b, y| acc_b + y);

    // Objective Delta: Minimize Repetition of Least Preferred Jobs
    let mut least_preferred_sum = Int::from_i64(&ctx, 0);
    for i in 0..employees.len() {
        for j in 0..jobs.len() {
            for t in 0..time_horizon_days {
                least_preferred_sum = least_preferred_sum
                    + Int::from_i64(&ctx, p_matrix[i][j] as i64)
                        * x[i][j][t].ite(&Int::from_i64(&ctx, 1), &Int::from_i64(&ctx, 0));
            }
        }
    }

    // Objective Gamma: Add later... : Promote fair distribution of top k% preferred jobs
    let fairness_deviation = Int::add(
        &ctx,
        &(0..employees.len())
            .map(|i| {
                let mut deviation_expr = Int::from_i64(&ctx, 0);
                for j in 0..jobs.len() {
                    if p_matrix[i][j] <= top_k_jobs {
                        for t in 0..time_horizon_days {
                            deviation_expr = &deviation_expr
                                + &x[i][j][t].ite(&Int::from_i64(&ctx, 1), &Int::from_i64(&ctx, 0));
                        }
                    }
                }
                let ideal_share = Int::from_i64(
                    &ctx,
                    (time_horizon_days * top_k_jobs) as i64 / employees.len() as i64,
                );
                let deviation = &deviation_expr - &ideal_share;
                let abs_deviation = deviation
                    .ge(&Int::from_i64(&ctx, 0))
                    .ite(&deviation, &(-&deviation));
                abs_deviation
            })
            .collect::<Vec<_>>(),
    );

    optimizer.maximize(
        &(&total_preference_score
            - &(penalty * Int::from_i64(&ctx, labmda_penalty))
            - &(beta_fairness_weight * least_preferred_sum)
            - &(beta_fairness_weight * fairness_deviation)),
    );

    // Check satisfiability and print the solution
    match optimizer.check(&[]) {
        SatResult::Sat => {
            let model = optimizer.get_model().unwrap();
            let mut assignment: Vec<(String, String, String)> = Vec::new();
            let mut total_score = 0;

            for i in 0..employees.len() {
                for j in 0..jobs.len() {
                    for t in 0..time_horizon_days {
                        if model.eval(&x[i][j][t], true).unwrap().as_bool().unwrap() {
                            assignment.push((employees[i].clone(), jobs[j].clone(), t.to_string()));
                            total_score += jobs.len()
                                - match p_matrix[i].iter().position(|&z| z == j) {
                                    Some(exists) => exists, //take current preference score
                                    None => jobs.len(),     //take maximum preference score
                                };
                        }
                    }
                }
            }

            let mut external_assignments: Vec<(String, String)> = Vec::new();
            for j in 0..jobs.len() {
                for t in 0..time_horizon_days {
                    if model.eval(&e[j][t], true).unwrap().as_bool().unwrap() {
                        external_assignments.push((jobs[j].clone(), t.to_string()));
                    }
                }
            }

            println!("Solution found");
            (assignment, external_assignments, total_score)
        }
        SatResult::Unsat => {
            println!("No solution found");
            (Vec::new(), Vec::new(), 0)
        }
        _ => {
            println!("Solver failed");
            (Vec::new(), Vec::new(), 0)
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

    use crate::*;
    use rand::seq::{IteratorRandom, SliceRandom};
    use rand::thread_rng;

    #[test]
    fn test_fair_horizon_assignment() {
        // Number of employees and jobs
        let employees = vec!["a", "b", "c"].iter().map(|x| x.to_string()).collect();
        let jobs = vec!["0", "1", "2", "3"]
            .iter()
            .map(|x| x.to_string())
            .collect();

        let competences = vec![
            (
                "a".to_string(),
                vec!["0", "2"].iter().map(|x| x.to_string()).collect(),
            ), // employee a can perform jobs 0 and 2
            (
                "b".to_string(),
                vec!["1"].iter().map(|x| x.to_string()).collect(),
            ), // employee b can perform jobs 0 and 1
            (
                "c".to_string(),
                vec!["0", "1"].iter().map(|x| x.to_string()).collect(),
            ), // employee c can perform jobs 1 and 2
        ];

        let preferences = vec![
            (
                "a".to_string(),
                vec!["2", "0", "1"].iter().map(|x| x.to_string()).collect(),
            ), // employee a prefers job 2, then 0, then 1
            (
                "b".to_string(),
                vec!["2", "1", "0"].iter().map(|x| x.to_string()).collect(),
            ), // employee b prefers job 2, then 1, then 0
            (
                "c".to_string(),
                vec!["1", "2", "0"].iter().map(|x| x.to_string()).collect(),
            ), // employee c prefers job 1, then 2, then 0
        ];

        let s = calculate_fair_horizon_assignment(
            false,
            &employees,
            &jobs,
            &competences,
            &preferences,
            5,
            50,
            100,
            50,
            50
        );
        println!("Optimal assignment: {:?}", s.0);
        println!("External assignment: {:?}", s.1);
        println!("Total preference score: {}", s.2);
        // assert_eq!(
        //     s.0,
        //     [
        //         ("a".to_string(), "2".to_string()),
        //         ("b".to_string(), "1".to_string()),
        //         ("c".to_string(), "0".to_string())
        //     ]
        // );
        // assert_eq!(s.1, ["3".to_string(), "4".to_string()]);
        // assert_eq!(s.2, 12);
    }

    #[test]
    fn test_fair_horizon_assignment_random() {
        fn generate_random_data() -> (
            Vec<String>,
            Vec<String>,
            Vec<(String, Vec<String>)>,
            Vec<(String, Vec<String>)>,
        ) {
            let employees = vec!["Megan", "Bob", "Carol", "David", "Eve"]
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>();

            let jobs = vec!["Job1", "Job2", "Job3", "Job4", "Job5"]
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>();

            let mut rng = thread_rng();

            let mut competences = vec![];
            for employee in &employees {
                let num_competences = (1..=jobs.len()).choose(&mut rng).unwrap();
                let employee_competences = jobs
                    .choose_multiple(&mut rng, num_competences)
                    .cloned()
                    .collect();
                competences.push((employee.clone(), employee_competences));
            }

            let mut preferences = vec![];
            for employee in &employees {
                let num_preferences = (1..=jobs.len()).choose(&mut rng).unwrap();
                let employee_preferences = jobs
                    .choose_multiple(&mut rng, num_preferences)
                    .cloned()
                    .collect();
                preferences.push((employee.clone(), employee_preferences));
            }

            (employees, jobs, competences, preferences)
        }

        let r = generate_random_data();
        let s = calculate_fair_horizon_assignment(false, &r.0, &r.1, &r.2, &r.3, 5, 50, 100, 10, 1);
        println!("Optimal assignment: {:?}", s.0);
        println!("External assignment: {:?}", s.1);
        println!("Total preference score: {}", s.2);
    }
}
