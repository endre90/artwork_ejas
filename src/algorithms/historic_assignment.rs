use std::collections::HashMap;

use nanoid::nanoid;
use z3::{
    ast::{Bool, Int},
    *,
};

use crate::Day;

pub fn calculate_fair_historic_assignment(
    anonymize: bool,
    employees: &Vec<String>,
    jobs: &Vec<String>,
    competence_map: &Vec<(String, Vec<String>)>,
    preference_map: &Vec<(String, Vec<String>)>,
    history_of_assignments: Vec<Day>,
    history_lenght: usize,
    lambda_penalty: usize, // ) -> (SatResult, String) {
) -> (Vec<(String, String)>, Vec<String>, usize) {
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

    let h_matrix = calculate_historic_matrix(
        history_of_assignments,
        history_lenght,
        employees.clone(),
        jobs.clone(),
    );

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

    // Constraints: Each job is assigned to either one internal employee or one external employee
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

    // Constraints: Only assign jobs that employees are competent to perform
    for i in 0..employees.len() {
        for j in 0..jobs.len() {
            optimizer.assert(&Bool::implies(
                &x[i][j],
                &Bool::from_bool(&ctx, c_matrix[i][j]),
            ));
        }
    }

    // Constraints: Each job must be covered by at least one competent internal worker or an external worker
    for j in 0..jobs.len() {
        let mut should_be_covered = vec![];
        for i in 0..employees.len() {
            if c_matrix[i][j] {
                should_be_covered.push(x[i][j].clone());
            }
        }
        should_be_covered.push(e[j].clone()); // Add the external worker option

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

    // Parameters for fairness
    let alpha = 2; // Maximum allowed count for any job
    let mu = (jobs.len() as f64 / employees.len() as f64).ceil() as i32;
    let beta = 1; // Weight for fairness term

    // Objective: Maximize preferences and minimize the use of external employees while ensuring fairness
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

    // Penalty for using external employees
    let penalty: Int = e
        .iter()
        .map(|e_j| e_j.ite(&Int::from_i64(&ctx, 1), &Int::from_i64(&ctx, 0)))
        .fold(Int::from_i64(&ctx, 0), |acc, x| acc + x);

    // Fairness term in the objective function
    let mut fairness_term = Vec::new();
    for i in 0..employees.len() {
        for j in 0..jobs.len() {
            let historical_count = h_matrix[i][j]; // The historical count of assignments
            let total_assignments = Int::from_i64(&ctx, historical_count as i64)
                + x[i][j].ite(&Int::from_i64(&ctx, 1), &Int::from_i64(&ctx, 0));
            let deviation = total_assignments - Int::from_i64(&ctx, mu as i64);
            fairness_term.push(&deviation * &deviation);
        }
    }
    let fairness_term_sum = Int::add(&ctx, &fairness_term);

    // Objective function
    optimizer.maximize(
        &(&Int::add(&ctx, &preference_score_sum)
            - &(penalty * Int::from_i64(&ctx, lambda_penalty as i64))
            - &(fairness_term_sum * Int::from_i64(&ctx, beta as i64))),
    );

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
                                Some(exists) => exists, // take current preference score
                                None => jobs.len(),     // take maximum preference score
                            };
                    }
                }
            }

            let mut external_assignments = Vec::new();
            for j in 0..jobs.len() {
                if model.eval(&e[j], true).unwrap().as_bool().unwrap() {
                    external_assignments.push(jobs[j].clone());
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

fn calculate_historic_matrix(
    history_of_assignments: Vec<Day>,
    period: usize,
    employees: Vec<String>,
    jobs: Vec<String>,
) -> Vec<Vec<u32>> {
    // Create a mapping from employee/job names to indices
    let employee_indices: HashMap<_, _> = employees
        .iter()
        .enumerate()
        .map(|(i, name)| (name.clone(), i))
        .collect();
    let job_indices: HashMap<_, _> = jobs
        .iter()
        .enumerate()
        .map(|(i, name)| (name.clone(), i))
        .collect();

    // Initialize the historic count matrix with zeros
    let mut h_matrix = vec![vec![0; jobs.len()]; employees.len()];

    // Iterate over the last `period` days in the history
    let start_index = if history_of_assignments.len() > period {
        history_of_assignments.len() - period
    } else {
        0
    };

    for day in &history_of_assignments[start_index..] {
        for (employee, job) in &day.assignments {
            if let (Some(&i), Some(&j)) = (employee_indices.get(employee), job_indices.get(job)) {
                h_matrix[i][j] += 1;
            }
        }
    }

    h_matrix
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

    use crate::algorithms::historic_assignment::calculate_historic_matrix;
    use crate::*;
    use rand::seq::SliceRandom;
    use rand::Rng;

    fn generate_historical_data() -> Vec<Day> {
        let history_of_assignments = vec![
            Day {
                date: Date {
                    year: 2024,
                    month: 5,
                    day: 1,
                },
                station: "Station1".to_string(),
                assignments: vec![
                    ("Megan".to_string(), "Job1".to_string()),
                    ("Bob  ".to_string(), "Job2".to_string()),
                    ("Carol".to_string(), "Job3".to_string()),
                    ("David".to_string(), "Job4".to_string()),
                    ("Eve  ".to_string(), "Job5".to_string()),
                ],
            },
            Day {
                date: Date {
                    year: 2024,
                    month: 5,
                    day: 2,
                },
                station: "Station1".to_string(),
                assignments: vec![
                    ("Megan".to_string(), "Job2".to_string()),
                    ("Bob  ".to_string(), "Job3".to_string()),
                    ("Carol".to_string(), "Job4".to_string()),
                    ("David".to_string(), "Job5".to_string()),
                    ("Eve  ".to_string(), "Job1".to_string()),
                ],
            },
            Day {
                date: Date {
                    year: 2024,
                    month: 5,
                    day: 3,
                },
                station: "Station1".to_string(),
                assignments: vec![
                    ("Megan".to_string(), "Job3".to_string()),
                    ("Bob  ".to_string(), "Job4".to_string()),
                    ("Carol".to_string(), "Job5".to_string()),
                    ("David".to_string(), "Job1".to_string()),
                    ("Eve  ".to_string(), "Job2".to_string()),
                ],
            },
            Day {
                date: Date {
                    year: 2024,
                    month: 5,
                    day: 4,
                },
                station: "Station1".to_string(),
                assignments: vec![
                    ("Megan".to_string(), "Job4".to_string()),
                    ("Bob  ".to_string(), "Job5".to_string()),
                    ("Carol".to_string(), "Job1".to_string()),
                    ("David".to_string(), "Job2".to_string()),
                    ("Eve  ".to_string(), "Job3".to_string()),
                ],
            },
            Day {
                date: Date {
                    year: 2024,
                    month: 5,
                    day: 5,
                },
                station: "Station1".to_string(),
                assignments: vec![
                    ("Megan".to_string(), "Job5".to_string()),
                    ("Bob  ".to_string(), "Job1".to_string()),
                    ("Carol".to_string(), "Job2".to_string()),
                    ("David".to_string(), "Job3".to_string()),
                    ("Eve  ".to_string(), "Job4".to_string()),
                ],
            },
            Day {
                date: Date {
                    year: 2024,
                    month: 5,
                    day: 6,
                },
                station: "Station1".to_string(),
                assignments: vec![
                    ("Megan".to_string(), "Job1".to_string()),
                    ("Bob  ".to_string(), "Job3".to_string()),
                    ("Carol".to_string(), "Job4".to_string()),
                    ("David".to_string(), "Job2".to_string()),
                    ("Eve  ".to_string(), "Job5".to_string()),
                ],
            },
            Day {
                date: Date {
                    year: 2024,
                    month: 5,
                    day: 7,
                },
                station: "Station1".to_string(),
                assignments: vec![
                    ("Megan".to_string(), "Job2".to_string()),
                    ("Bob  ".to_string(), "Job4".to_string()),
                    ("Carol".to_string(), "Job5".to_string()),
                    ("David".to_string(), "Job3".to_string()),
                    ("Eve  ".to_string(), "Job1".to_string()),
                ],
            },
            Day {
                date: Date {
                    year: 2024,
                    month: 5,
                    day: 8,
                },
                station: "Station1".to_string(),
                assignments: vec![
                    ("Megan".to_string(), "Job3".to_string()),
                    ("Bob  ".to_string(), "Job5".to_string()),
                    ("Carol".to_string(), "Job1".to_string()),
                    ("David".to_string(), "Job4".to_string()),
                    ("Eve  ".to_string(), "Job2".to_string()),
                ],
            },
            Day {
                date: Date {
                    year: 2024,
                    month: 5,
                    day: 9,
                },
                station: "Station1".to_string(),
                assignments: vec![
                    ("Megan".to_string(), "Job4".to_string()),
                    ("Bob  ".to_string(), "Job1".to_string()),
                    ("Carol".to_string(), "Job2".to_string()),
                    ("David".to_string(), "Job5".to_string()),
                    ("Eve  ".to_string(), "Job3".to_string()),
                ],
            },
            Day {
                date: Date {
                    year: 2024,
                    month: 5,
                    day: 10,
                },
                station: "Station1".to_string(),
                assignments: vec![
                    ("Megan".to_string(), "Job5".to_string()),
                    ("Bob  ".to_string(), "Job2".to_string()),
                    ("Carol".to_string(), "Job3".to_string()),
                    ("David".to_string(), "Job1".to_string()),
                    ("Eve  ".to_string(), "Job4".to_string()),
                ],
            },
            Day {
                date: Date {
                    year: 2024,
                    month: 5,
                    day: 11,
                },
                station: "Station1".to_string(),
                assignments: vec![
                    ("Megan".to_string(), "Job1".to_string()),
                    ("Bob  ".to_string(), "Job3".to_string()),
                    ("Carol".to_string(), "Job4".to_string()),
                    ("David".to_string(), "Job2".to_string()),
                    ("Eve  ".to_string(), "Job5".to_string()),
                ],
            },
            Day {
                date: Date {
                    year: 2024,
                    month: 5,
                    day: 12,
                },
                station: "Station1".to_string(),
                assignments: vec![
                    ("Megan".to_string(), "Job2".to_string()),
                    ("Bob  ".to_string(), "Job4".to_string()),
                    ("Carol".to_string(), "Job5".to_string()),
                    ("David".to_string(), "Job3".to_string()),
                    ("Eve  ".to_string(), "Job1".to_string()),
                ],
            },
            Day {
                date: Date {
                    year: 2024,
                    month: 5,
                    day: 13,
                },
                station: "Station1".to_string(),
                assignments: vec![
                    ("Megan".to_string(), "Job3".to_string()),
                    ("Bob  ".to_string(), "Job5".to_string()),
                    ("Carol".to_string(), "Job1".to_string()),
                    ("David".to_string(), "Job4".to_string()),
                    ("Eve  ".to_string(), "Job2".to_string()),
                ],
            },
            Day {
                date: Date {
                    year: 2024,
                    month: 5,
                    day: 14,
                },
                station: "Station1".to_string(),
                assignments: vec![
                    ("Megan".to_string(), "Job4".to_string()),
                    ("Bob  ".to_string(), "Job1".to_string()),
                    ("Carol".to_string(), "Job2".to_string()),
                    ("David".to_string(), "Job5".to_string()),
                    ("Eve  ".to_string(), "Job3".to_string()),
                ],
            },
        ];

        history_of_assignments
    }

    fn generate_competences() -> Vec<(String, Vec<String>)> {
        vec![
            (
                "Megan".to_string(),
                vec!["Job1".to_string(), "Job2".to_string(), "Job3".to_string()],
            ),
            (
                "Bob  ".to_string(),
                vec!["Job2".to_string(), "Job3".to_string(), "Job4".to_string()],
            ),
            (
                "Carol".to_string(),
                vec!["Job3".to_string(), "Job4".to_string(), "Job5".to_string()],
            ),
            (
                "David".to_string(),
                vec!["Job1".to_string(), "Job4".to_string(), "Job5".to_string()],
            ),
            (
                "Eve  ".to_string(),
                vec!["Job1".to_string(), "Job2".to_string(), "Job5".to_string()],
            ),
        ]
    }

    fn generate_preferences() -> Vec<(String, Vec<String>)> {
        vec![
            (
                "Megan".to_string(),
                vec![
                    "Job3".to_string(),
                    "Job1".to_string(),
                    "Job2".to_string(),
                    "Job4".to_string(),
                    "Job5".to_string(),
                ],
            ),
            (
                "Bob  ".to_string(),
                vec![
                    "Job4".to_string(),
                    "Job2".to_string(),
                    "Job3".to_string(),
                    "Job1".to_string(),
                    "Job5".to_string(),
                ],
            ),
            (
                "Carol".to_string(),
                vec![
                    "Job5".to_string(),
                    "Job3".to_string(),
                    "Job4".to_string(),
                    "Job1".to_string(),
                    "Job2".to_string(),
                ],
            ),
            (
                "David".to_string(),
                vec![
                    "Job1".to_string(),
                    "Job5".to_string(),
                    "Job4".to_string(),
                    "Job3".to_string(),
                    "Job2".to_string(),
                ],
            ),
            (
                "Eve  ".to_string(),
                vec![
                    "Job2".to_string(),
                    "Job1".to_string(),
                    "Job5".to_string(),
                    "Job3".to_string(),
                    "Job4".to_string(),
                ],
            ),
        ]
    }

    fn generate_random_historical_data() -> Vec<Day> {
        let employees = vec!["Megan", "Bob  ", "Carol", "David", "Eve  "];
        let jobs = vec!["Job1", "Job2", "Job3", "Job4", "Job5"];
        let mut rng = rand::thread_rng();
        let mut history_of_assignments = Vec::new();
    
        for day in 1..=14 {
            let date = Date { year: 2024, month: 5, day };
            let station = format!("Station{}", (day % 3) + 1);
            
            let mut assignments = Vec::new();
            for _ in 0..5 {
                let employee = employees.choose(&mut rng).unwrap().to_string();
                let job = jobs.choose(&mut rng).unwrap().to_string();
                assignments.push((employee, job));
            }
    
            history_of_assignments.push(Day {
                date,
                station,
                assignments,
            });
        }
    
        history_of_assignments
    }
    
    fn generate_random_competences() -> Vec<(String, Vec<String>)> {
        let employees = vec!["Megan", "Bob  ", "Carol", "David", "Eve  "];
        let jobs = vec!["Job1", "Job2", "Job3", "Job4", "Job5"];
        let mut rng = rand::thread_rng();
        
        employees.into_iter().map(|employee| {
            let mut competences = jobs.clone();
            competences.shuffle(&mut rng);
            // Each employee is competent in a random number of jobs (between 2 and 5)
            let num_competences = rng.gen_range(2..=5);
            (employee.to_string(), competences.into_iter().take(num_competences).map(|c| c.to_string()).collect())
        }).collect()
    }
    
    fn generate_random_preferences() -> Vec<(String, Vec<String>)> {
        let employees = vec!["Megan", "Bob  ", "Carol", "David", "Eve  "];
        let jobs = vec!["Job1", "Job2", "Job3", "Job4", "Job5"];
        let mut rng = rand::thread_rng();
        
        employees.into_iter().map(|employee| {
            let mut preferences = jobs.clone();
            preferences.shuffle(&mut rng);
            (employee.to_string(), preferences.iter().map(|x| x.to_string()).collect())
        }).collect()
    }

    #[test]
    fn test_fair_historic_assignment() {
        fn generate_data() -> (
            Vec<String>,
            Vec<String>,
            Vec<(String, Vec<String>)>,
            Vec<(String, Vec<String>)>,
            Vec<Day>,
        ) {
            let employees = vec!["Megan", "Bob  ", "Carol", "David", "Eve  "]
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>();

            let jobs = vec!["Job1", "Job2", "Job3", "Job4", "Job5"]
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>();

            let competences = generate_competences();

            let preferences = generate_preferences();

            let historic_data = generate_historical_data();

            let h_matrix = calculate_historic_matrix(
                historic_data.clone(),
                10,
                employees.clone(),
                jobs.clone(),
            );
            println!("       J  J  J  J  J");
            println!("       1  2  3  4  5");
            for x in 0..h_matrix.len() {
                println!("{}:{:?}", employees[x], h_matrix[x])
            }

            (employees, jobs, competences, preferences, historic_data)
        }

        let r = generate_data();
        let s = calculate_fair_historic_assignment(false, &r.0, &r.1, &r.2, &r.3, r.4, 100, 10);

        println!("Optimal assignment: {:?}", s.0);
        println!("External assignment: {:?}", s.1);
        println!("Total preference score: {}", s.2);
    }

    #[test]
    fn test_fair_historic_assignment_random() {
        fn generate_data() -> (
            Vec<String>,
            Vec<String>,
            Vec<(String, Vec<String>)>,
            Vec<(String, Vec<String>)>,
            Vec<Day>,
        ) {
            let employees = vec!["Megan", "Bob  ", "Carol", "David", "Eve  "]
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>();

            let jobs = vec!["Job1", "Job2", "Job3", "Job4", "Job5"]
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>();

            let competences = generate_random_competences();

            let preferences = generate_random_preferences();

            let historic_data = generate_random_historical_data();

            let h_matrix = calculate_historic_matrix(
                historic_data.clone(),
                10,
                employees.clone(),
                jobs.clone(),
            );
            println!("       J  J  J  J  J");
            println!("       1  2  3  4  5");
            for x in 0..h_matrix.len() {
                println!("{}:{:?}", employees[x], h_matrix[x])
            }

            (employees, jobs, competences, preferences, historic_data)
        }

        let r = generate_data();
        let s = calculate_fair_historic_assignment(false, &r.0, &r.1, &r.2, &r.3, r.4, 100, 10);

        println!("Optimal assignment: {:?}", s.0);
        println!("External assignment: {:?}", s.1);
        println!("Total preference score: {}", s.2);
    }

    #[test]
    fn test_calculate_historic_matrix() {
        let history_of_assignments = vec![
            Day {
                date: Date {
                    year: 2024,
                    month: 5,
                    day: 1,
                },
                station: "Station1".to_string(),
                assignments: vec![
                    ("Alice".to_string(), "Job1".to_string()),
                    ("Bob  ".to_string(), "Job2".to_string()),
                ],
            },
            Day {
                date: Date {
                    year: 2024,
                    month: 5,
                    day: 2,
                },
                station: "Station1".to_string(),
                assignments: vec![
                    ("Alice".to_string(), "Job1".to_string()),
                    ("Bob  ".to_string(), "Job3".to_string()),
                ],
            },
            // Add more historical data as needed
        ];

        let employees = vec!["Alice".to_string(), "Bob  ".to_string()];
        let jobs = vec!["Job1".to_string(), "Job2".to_string(), "Job3".to_string()];
        let period = 10;

        let h_matrix = calculate_historic_matrix(history_of_assignments, period, employees, jobs);
        println!("{:?}", h_matrix);
    }

    #[test]
    fn test_calculate_historic_matrix_2() {
        fn generate_historical_data() -> Vec<Day> {
            let employees = vec![
                "Alice  ".to_string(),
                "Bob    ".to_string(),
                "Charlie".to_string(),
                "David  ".to_string(),
                "Eve    ".to_string(),
                "Frank  ".to_string(),
                "Grace  ".to_string(),
                "Heidi  ".to_string(),
                "Ivan   ".to_string(),
                "Judy   ".to_string(),
            ];

            let jobs = vec![
                "Job1".to_string(),
                "Job2".to_string(),
                "Job3".to_string(),
                "Job4".to_string(),
                "Job5".to_string(),
                "Job6".to_string(),
                "Job7".to_string(),
                "Job8".to_string(),
            ];

            let mut history_of_assignments = Vec::new();

            for day in 1..=14 {
                let date = Date {
                    year: 2024,
                    month: 5,
                    day,
                };
                let station = format!("Station{}", (day % 3) + 1);

                let mut assignments = Vec::new();
                for _ in 0..5 {
                    let employee = employees[rand::random::<usize>() % employees.len()].clone();
                    let job = jobs[rand::random::<usize>() % jobs.len()].clone();
                    assignments.push((employee, job));
                }

                history_of_assignments.push(Day {
                    date,
                    station,
                    assignments,
                });
            }

            history_of_assignments
        }

        let history_of_assignments = generate_historical_data();

        // Define employees and jobs
        let employees = vec![
            "Alice  ".to_string(),
            "Bob    ".to_string(),
            "Charlie".to_string(),
            "David  ".to_string(),
            "Eve    ".to_string(),
            "Frank  ".to_string(),
            "Grace  ".to_string(),
            "Heidi  ".to_string(),
            "Ivan   ".to_string(),
            "Judy   ".to_string(),
        ];

        let jobs = vec![
            "Job1".to_string(),
            "Job2".to_string(),
            "Job3".to_string(),
            "Job4".to_string(),
            "Job5".to_string(),
            "Job6".to_string(),
            "Job7".to_string(),
            "Job8".to_string(),
        ];

        let period = 10;

        let h_matrix = calculate_historic_matrix(
            history_of_assignments.clone(),
            period,
            employees.clone(),
            jobs.clone(),
        );
        println!("         J  J  J  J  J  J  J  J");
        // println!("         o  o  o  o  o  o  o  o");
        // println!("         b  b  b  b  b  b  b  b");
        println!("         1  2  3  4  5  6  7  8");
        // println!("        .. .. .. .. .. .. .. ..");
        for x in 0..h_matrix.len() {
            println!("{}:{:?}", employees[x], h_matrix[x])
        }
        // println!("{:?}", h_matrix);
    }
}
