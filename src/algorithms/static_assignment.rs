use std::collections::HashMap;

use ast::Ast;
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
) -> (
    Vec<(String, String)>,
    usize,
    Vec<Vec<bool>>,
    Vec<Vec<usize>>,
) {
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
        // If coverage_per_job[j] is the list of booleans for job j,
        // we need the sum of those booleans to be >= 1.
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
            preference_score.push((Bool::and(&ctx, vec![&x[i][j], &Bool::from_bool(&ctx, true)].as_slice()), score));
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
    let pref_score = Int::new_const(&ctx, "pref_score");
    optimizer.assert(&pref_score._eq(&Int::add(&ctx, &preference_score_sum)));
    optimizer.maximize(&Int::add(&ctx, &preference_score_sum));

    // Check satisfiability and print the solution
    match optimizer.check(&[]) {
        SatResult::Sat => {
            let model = optimizer.get_model().unwrap();
            let mut assignment = Vec::new();
            // let mut total_score = 0;

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
                        // total_score += p_matrix[i].len()
                        //     - match p_matrix[i].iter().position(|&z| z == j) {
                        //         Some(exists) => exists, //take current preference score
                        //         None => jobs.len(),     //take maximum preference score
                        //     };
                    }
                }
            }

            log::info!(target: "employee_job_assignment", "Solution found");
            (
                assignment,
                pref_score as usize,
                competence_matrix,
                preference_matrix,
            )
        }
        SatResult::Unsat => {
            log::warn!(target: "employee_job_assignment", "No solution found");
            (Vec::new(), 0, competence_matrix, preference_matrix)
        }
        _ => {
            log::error!(target: "employee_job_assignment", "Solver failed");
            (Vec::new(), 0, competence_matrix, preference_matrix)
        }
    }
}

pub fn build_competence_matrix(
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

pub fn build_preference_matrix(
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
    use rand::seq::{IteratorRandom, SliceRandom};
    use rand::thread_rng;
    use serde::Deserialize;

    #[test]
    fn test_static_matrix() -> Result<(), Box<dyn std::error::Error>> {
        let manifest_dir =
            std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is not set");
        let path = format!("{}/data/matrix.json", manifest_dir);

        #[derive(Debug, Deserialize)]
        struct Matrix {
            pub stations: std::collections::HashMap<String, Station>,
        }

        #[derive(Debug, Deserialize)]
        struct Station {
            pub ergo_score: std::collections::HashMap<String, u8>,
            pub people: Vec<Person>,
        }

        #[derive(Debug, Deserialize)]
        struct Person {
            pub name: String,
            pub role: String,
            pub competences: Vec<String>,
            pub preferences: Vec<String>,
        }

        let json_content = fs::read_to_string(path)?;
        let matrix: Matrix = serde_json::from_str(&json_content)?;
        if let Some(station_1) = matrix.stations.get("S1") {
            let mut jobs = vec![];
            for op in station_1
                .ergo_score
                .keys() // this is random but score should still be the same
                .map(|x| x.to_owned())
                .collect::<Vec<String>>()
            {
                jobs.push(op);
            }
            let mut employees = vec![];
            let mut competences: Vec<(String, Vec<String>)> = vec![];
            let mut preferences: Vec<(String, Vec<String>)> = vec![];
            for person in &station_1.people {
                employees.push(person.name.clone());
                competences.push((person.name.clone(), person.competences.clone()));
                preferences.push((person.name.clone(), person.preferences.clone()));
            }

            // println!("Employees: {:?}", employees);
            // println!("Jobs: {:?}", jobs);
            // println!("Competences: {:?}", competences);
            // println!("Preferences: {:?}", preferences);

            // let competence_matrix = build_competence_matrix(&competences, &jobs);
            // let preferrence_matrix = build_preference_matrix(&preferences, &jobs);

            // println!("C_matrix: {:?}", competence_matrix);
            // println!("P_matrix: {:?}", preferrence_matrix);

            let s =
                calculate_static_assignment(false, &employees, &jobs, &competences, &preferences);
            println!("Optimal assignment: {:?}", s.0);
            println!("Total preference score: {}", s.1);
        }

        Ok(())
    }

    #[test]
    fn test_static_assignment() {
        // Number of employees and jobs
        let employees = vec!["A", "B", "C"].iter().map(|x| x.to_string()).collect();
        let jobs = vec!["O3", "O1", "O2"]
            .iter()
            .map(|x| x.to_string())
            .collect();

        let competences: Vec<(String, Vec<String>)> = vec![
            (
                "A".to_string(),
                vec!["O1", "O2"].iter().map(|x| x.to_string()).collect(),
            ), // employee a can perform jobs 0 and 2
            (
                "B".to_string(),
                vec!["O2", "O3"].iter().map(|x| x.to_string()).collect(),
            ), // employee b can perform jobs 0 and 1
            (
                "C".to_string(),
                vec!["O1", "O3"].iter().map(|x| x.to_string()).collect(),
            ), // employee c can perform jobs 1 and 2
        ];

        let preferences = vec![
            (
                "A".to_string(),
                vec!["O1"].iter().map(|x| x.to_string()).collect(),
            ), // employee a prefers job 2, then 0, then 1
            (
                "B".to_string(),
                vec!["O2"].iter().map(|x| x.to_string()).collect(),
            ), // employee b prefers job 2, then 1, then 0
            (
                "C".to_string(),
                vec!["O3"].iter().map(|x| x.to_string()).collect(),
            ), // employee c prefers job 1, then 2, then 0
        ];

        println!("Employees: {:?}", employees);
        println!("Jobs: {:?}", jobs);
        println!("Competences: {:?}", competences);
        println!("Preferences: {:?}", preferences);

        let competence_matrix = build_competence_matrix(&competences, &jobs);
        let preferrence_matrix = build_preference_matrix(&preferences, &jobs);

        println!("C_matrix: {:?}", competence_matrix);
        println!("P_matrix: {:?}", preferrence_matrix);

        let s = calculate_static_assignment(false, &employees, &jobs, &competences, &preferences);
        println!("Optimal assignment: {:?}", s.0);
        println!("Total preference score: {}", s.1);
        // assert_eq!(
        //     s.0,
        //     [
        //         ("a".to_string(), "2".to_string()),
        //         ("b".to_string(), "0".to_string()),
        //         ("c".to_string(), "1".to_string())
        //     ]
        // );
        // assert_eq!(s.1, 4);
    }

    #[test]
    fn test_static_assignment_random() {
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
        let s = calculate_static_assignment(false, &r.0.clone(), &r.1, &r.2, &r.3);
        println!("Optimal assignment: {:?}", s.0);
        println!("Total preference score: {}", s.1);
    }
}
