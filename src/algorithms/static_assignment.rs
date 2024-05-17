use std::collections::HashMap;

use nanoid::nanoid;
use z3::{
    ast::{Bool, Int},
    *,
};

pub fn calculate_static_assignment(
    anonymize: bool,
    employees: &Vec<String>,
    jobs: &Vec<String>,
    competence_map: &Vec<(String, Vec<String>)>,
    preference_map: &Vec<(String, Vec<String>)>,
    // ) -> (SatResult, String) {
) -> (Vec<(String, String)>, usize) {
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
        let at_most_one_job_per_employee = ast::Bool::pb_eq(
            &ctx,
            employee_constraints
                .iter()
                .map(|x| (x, 1))
                .collect::<Vec<(&ast::Bool, i32)>>()
                .as_slice(),
            1,
        );
        // preferred but not required to be satisfied (i.e. at most one job but it could be none)
        optimizer.assert_soft(&at_most_one_job_per_employee, 1, None);
    }

    // Constraints: Each job is assigned to at most one employee
    for j in 0..jobs.len() {
        let job_constraints: Vec<_> = (0..employees.len()).map(|i| x[i][j].clone()).collect();
        let at_most_one_employee_per_job = ast::Bool::pb_eq(
            &ctx,
            job_constraints
                .iter()
                .map(|x| (x, 1))
                .collect::<Vec<(&ast::Bool, i32)>>()
                .as_slice(),
            1,
        );
        // preferred but not required to be satisfied (i.e. at most one employee but it could be none)
        optimizer.assert_soft(&at_most_one_employee_per_job, 1, None);
    }

    // Constraints: Only assign jobs that employees are competent to perform
    for i in 0..employees.len() {
        for j in 0..jobs.len() {
            optimizer.assert_soft(
                &Bool::implies(&x[i][j], &Bool::from_bool(&ctx, c_matrix[i][j])),
                1,
                None,
            );
        }
    }

    // Constraints: Each job must be covered by at least one competent worker
    for j in 0..jobs.len() {
        // let mut job_covered = Bool::from_bool(&ctx, false);
        let mut should_be_covered = vec![];
        for i in 0..employees.len() {
            if c_matrix[i][j] {
                should_be_covered.push(x[i][j].clone());
            }
        }
        let job_covered = ast::Bool::or(
            &ctx,
            should_be_covered
                .iter()
                .map(|x| x)
                .collect::<Vec<&ast::Bool>>()
                .as_slice(),
        );
        optimizer.assert(&job_covered);
    }

    // Objective: Maximize preferences
    let mut preference_score = Vec::new();
    for i in 0..employees.len() {
        for (rank, &j) in p_matrix[i].iter().enumerate() {
            let score = (jobs.len() - rank) as i32;
            preference_score.push((Bool::implies(&x[i][j], &Bool::from_bool(&ctx, true)), score));
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
    optimizer.maximize(&Int::add(&ctx, &preference_score_sum));

    // Check satisfiability and print the solution
    match optimizer.check(&[]) {
        SatResult::Sat => {
            let model = optimizer.get_model().unwrap();
            let mut assignment = Vec::new();
            let mut total_score = 0;

            for i in 0..employees.len() {
                for j in 0..jobs.len() {
                    if model.eval(&x[i][j], true).unwrap().as_bool().unwrap() {
                        assignment.push((employees[i].clone(), jobs[j].clone()));
                        total_score += jobs.len()
                            - match p_matrix[i].iter().position(|&z| z == j) {
                                Some(exists) => exists, //take current preference score
                                None => jobs.len(),     //take maximum preference score
                            };
                    }
                }
            }

            println!("Solution found");
            (assignment, total_score)
        }
        SatResult::Unsat => {
            println!("No solution found");
            (Vec::new(), 0)
        }
        _ => {
            println!("Solver failed");
            (Vec::new(), 0)
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

// fn build_preference_matrix(
//     preference_map: &Vec<(String, Vec<String>)>,
//     competence_matrix: &Vec<Vec<bool>>,
//     job_list: &Vec<String>,
// ) -> Vec<Vec<usize>> {
//     // Create a hashmap to map job names to their indices
//     let job_index: HashMap<&String, usize> = job_list
//         .iter()
//         .enumerate()
//         .map(|(i, job)| (job, i))
//         .collect();

//     // Initialize the preference matrix with default high values (e.g., job_list.len() which is worse than the worst preference)
//     let mut preference_matrix =
//         vec![vec![job_list.len() - 1; job_list.len()]; preference_map.len()];

//     // Fill the preference matrix and filter out preferences for jobs that the worker is not competent to perform
//     for (worker_index, (_, preferences)) in preference_map.iter().enumerate() {
//         for (rank, job) in preferences.iter().enumerate() {
//             if let Some(&job_idx) = job_index.get(job) {
//                 if competence_matrix[worker_index][job_idx] {
//                     preference_matrix[worker_index][job_idx] = rank;
//                 }
//             }
//         }
//     }

//     preference_matrix
// }

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
    fn test_static_assignment() {
        // Number of employees and jobs
        let employees = vec!["a", "b", "c"].iter().map(|x| x.to_string()).collect();
        let jobs = vec!["0", "1", "2"].iter().map(|x| x.to_string()).collect();

        let competences = vec![
            (
                "a".to_string(),
                vec!["0", "2"].iter().map(|x| x.to_string()).collect(),
            ), // employee a can perform jobs 0 and 2
            (
                "b".to_string(),
                vec!["0", "1"].iter().map(|x| x.to_string()).collect(),
            ), // employee b can perform jobs 0 and 1
            (
                "c".to_string(),
                vec!["1"].iter().map(|x| x.to_string()).collect(),
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

        let s = calculate_static_assignment(false, &employees, &jobs, &competences, &preferences);
        println!("Optimal assignment: {:?}", s.0);
        println!("Total preference score: {}", s.1);
        assert_eq!(
            s.0,
            [
                ("a".to_string(), "2".to_string()),
                ("b".to_string(), "0".to_string()),
                ("c".to_string(), "1".to_string())
            ]
        );
        assert_eq!(s.1, 4);
    }

    #[test]
    fn test_static_assignment_random() {
        fn generate_random_data() -> (
            Vec<String>,
            Vec<String>,
            Vec<(String, Vec<String>)>,
            Vec<(String, Vec<String>)>,
        ) {
            let employees = vec!["Alice", "Bob", "Carol", "David", "Eve"]
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
        let s = calculate_static_assignment(false, &r.0, &r.1, &r.2, &r.3);
        println!("Optimal assignment: {:?}", s.0);
        println!("Total preference score: {}", s.1);
    }
}
