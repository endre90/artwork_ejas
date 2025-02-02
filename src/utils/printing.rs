use crate::*;
// Print all (employee -> job) assignments
pub fn pretty_print_internal_assignments(station: &Station, assignment: &[(String, String)]) {
    // Sort job names (for consistency if you want to show them in some order here)
    let mut sorted_jobs: Vec<String> = station.ergo_score.keys().cloned().collect();
    sorted_jobs.sort();

    // Collect employees in the order they appear in station.people
    // (If you want them sorted, you can sort here as well)
    // let employees: Vec<String> = station.people.iter().map(|p| p.name.clone()).collect();

    println!("=== INTERNAL ASSIGNMENTS ===");
    if assignment.is_empty() {
        println!("No assignments found.");
    } else {
        // Sort assignment for deterministic order
        let mut sorted_assignment = assignment.to_vec();
        sorted_assignment.sort_by(|(e1, _), (e2, _)| e1.cmp(e2));
        for (employee, job) in &sorted_assignment {
            println!("    {} -> {}", employee, job);
        }
    }
    println!();
}

pub fn pretty_print_external_assignments(station: &Station, assignment: &[String]) {
    // Sort job names (for consistency if you want to show them in some order here)
    let mut sorted_jobs: Vec<String> = station.ergo_score.keys().cloned().collect();
    sorted_jobs.sort();

    println!("=== EXTERNAL ASSIGNMENTS ===");
    if assignment.is_empty() {
        println!("No assignments found.");
    } else {
        for i in 0..assignment.len() {
            println!("    X{} -> {}", i, assignment[i]);
        }
    }
    println!();
}

// Print competence matrix (c_matrix):
// rows = employees, columns = *sorted* jobs.
// c_matrix[i][j] = true means i-th employee can perform j-th job.
pub fn pretty_print_competence_matrix(station: &Station) {

    let mut jobs = vec![];
    let mut employees = vec![];
    let mut competences: Vec<(String, Vec<String>)> = vec![];
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
    }

    let c_matrix = build_competence_matrix(&competences, &jobs);

    // Employees in their natural order
    let employees: Vec<String> = station.people.iter().map(|p| p.name.clone()).collect();

    println!("=== COMPETENCE MATRIX ===");
    // println!("Rows = employees, Columns = sorted jobs:\n");

    // Print header row
    print!("{:>5}", ""); 
    for job in &jobs {
        print!("{:>5}", job);
    }
    println!();

    // Print each row: employee name + T/F columns
    for (i, employee) in employees.iter().enumerate() {
        print!("{:>5}", employee);

        // Guard: only print if row `i` exists
        if i < c_matrix.len() {
            let row = &c_matrix[i];
            for j in 0..jobs.len() {
                // If we're within bounds of the row, print T/F
                let val = if j < row.len() && row[j] {
                    "1"
                } else {
                    "0"
                };
                print!("{:>5}", val);
            }
        }
        println!();
    }
    println!();
}

// Print preference matrix (p_matrix):
// rows = employees, columns = *sorted* jobs.
// p_matrix[i][j] = numeric rank/preference for j-th job.
pub fn pretty_print_preference_matrix(station: &Station) {

    let mut jobs = vec![];
    let mut employees = vec![];
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
        preferences.push((person.name.clone(), person.preferences.clone()));
    }

    let p_matrix = build_preference_matrix(&preferences, &jobs);

    // Employees in their natural order
    let employees: Vec<String> = station.people.iter().map(|p| p.name.clone()).collect();

    println!("=== PREFERENCE MATRIX ===");
    // println!("Rows = employees, Columns = sorted jobs:\n");

    // Print header row
    print!("{:>5}", "");
    for job in &jobs {
        print!("{:>5}", job);
    }
    println!();

    // Each row for each employee
    for (i, employee) in employees.iter().enumerate() {
        print!("{:>5}", employee);

        // Guard: only print if row `i` exists
        if i < p_matrix.len() {
            let row = &p_matrix[i];
            for j in 0..jobs.len() {
                // If out of bounds, use a fallback
                let rank = if j < row.len() {
                    row[j]
                } else {
                    9999 // placeholder
                };
                print!("{:>5}", rank);
            }
        }
        println!();
    }
    println!();
}