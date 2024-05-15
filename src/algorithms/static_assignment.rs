use std::collections::HashMap;

use nanoid::nanoid;
use z3::{
    ast::{Bool, Int},
    *,
};

pub fn calculate_static_assignment(
    employees: &Vec<String>,
    jobs: &Vec<String>,
    competence_map: &Vec<(String, Vec<String>)>,
    preference_map: &Vec<(String, Vec<String>)>,
) -> (SatResult, String) {
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

    // Just checking if it the anonymizatgion went as planned
    // println!("employees: {:#?}", anon_employees_map);
    // println!("jobs: {:#?}", anon_jobs_map);
    println!("competences: {:#?}", competence_map);
    println!("anon_competences: {:#?}", anon_competence_map);
    // println!("preferences: {:#?}", preference_map);
    // println!("anon_preferences: {:#?}", anon_preferences);
    println!("competence_matrix: {:#?}", competence_matrix);
    println!("anon_competence_matrix: {:#?}", anon_competence_matrix); // check this might not be correct...

    // Create the Z3 context and optimizer
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let optimize = Optimize::new(&ctx);

    // Check if a solution exists and return the satisfiable assignments
    let solution = optimize.check(&[]);
    let model = optimize.get_model();
    let string_model = model.unwrap().to_string();
    (solution, string_model)
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
    for (worker_index, (_, competences)) in competence_map.iter().enumerate() {
        for competence in competences {
            if let Some(&job_index) = job_index.get(competence) {
                competence_matrix[worker_index][job_index] = true;
            }
        }
    }

    competence_matrix
}

// 1. anonymize
// 2. put into matrices
