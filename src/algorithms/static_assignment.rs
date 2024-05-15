use std::collections::HashMap;

use nanoid::nanoid;
use z3::{
    ast::{Bool, Int},
    *,
};

pub fn calculate_static_assignment(
    employees: &Vec<String>,
    jobs: &Vec<String>,
    competences: &Vec<(String, Vec<String>)>,
    // preferences: &Vec<(String, Vec<String>)>,
) -> (SatResult, String) {
    let anon_employees_map = employees
        .iter()
        .map(|e| (e.to_owned(), nanoid!()))
        .collect::<HashMap<String, String>>();

    let anon_jobs_map = jobs
        .iter()
        .map(|e| (e.to_owned(), nanoid!()))
        .collect::<HashMap<String, String>>();

    let anon_competences = competences.iter().map(|(employee, competences)| {
        let anon_employee = anon_employees_map.get(employee).unwrap().to_owned();
        let anon_competences = competences
            .iter()
            .map(|comp| anon_jobs_map.get(comp).unwrap().to_owned())
            .collect::<Vec<String>>();
        (anon_employee, anon_competences)
    }).collect::<Vec<(String, Vec<String>)>>();

    println!("employees: {:#?}", anon_employees_map);
    println!("jobs: {:#?}", anon_jobs_map);
    println!("anon_competences: {:#?}", anon_competences);

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

// 1. anonymize
// 2. put into matrices
