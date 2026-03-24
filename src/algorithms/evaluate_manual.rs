use crate::{build_historical_count_matrix, build_preference_matrix, Day, Role, Station};

#[derive(Debug, Clone)]
pub struct ManualScoringResult {
    pub objective_score: i64,
    pub preference_reward_score: i64,
    pub weighted_preference_reward_score: i64,
    pub leader_penalty_score: i64,
    pub weighted_leader_penalty_score: i64,
    pub external_penalty_score: i64,
    pub weighted_external_penalty_score: i64,
    pub historical_penalty_score: i64,
    pub weighted_historical_penalty_score: i64,
    pub offset: u32,
}

pub fn evaluate_manual_assignment(
    station: &Station,
    history: Vec<Day>,
    manual_assignments: &[(String, String)],
    offset: u32,
    omega: u32,
    alpha: u32,
    beta: u32,
    tau: u32,
    gamma: u32,
) -> ManualScoringResult {
    // 1. Setup Jobs and Employees exactly as the solver does
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

    // Build the matrices
    let p_matrix = build_preference_matrix(&preferences, &jobs);
    let h_matrix = build_historical_count_matrix(history, tau as usize, &employees, &jobs);

    // 2. Preference Reward Term
    let mut preference_reward_score = 0;
    for (emp_name, job_name) in manual_assignments {
        if let (Some(i), Some(j)) = (
            employees.iter().position(|e| e == emp_name),
            jobs.iter().position(|job| job == job_name),
        ) {
            let rank = p_matrix[i][j];
            let score = (jobs.len() - rank) as i64;
            preference_reward_score += score;
        }
    }
    let weighted_preference_reward_score = (omega as i64) * preference_reward_score;

    // 3. Team Leader Penalty Term
    let mut leader_penalty_score = 0;

    let leader_name = station
        .people
        .iter()
        .find(|p| matches!(p.role, Role::TeamLeader))
        .map(|p| p.name.clone());

    if let Some(lname) = leader_name {
        for (emp_name, job_name) in manual_assignments {
            // Only penalize if the leader is assigned to a valid station job
            if emp_name == &lname && jobs.contains(job_name) {
                leader_penalty_score += 1;
            }
        }
    }
    let weighted_leader_penalty_score = (alpha as i64) * leader_penalty_score;

    // 4. External Penalty Term
    // In Z3, e_j is true if no internal employee takes job j.
    // Here, we count how many station jobs are completely missing from the manual_assignments.
    let mut covered_jobs = 0;
    for job in &jobs {
        if manual_assignments
            .iter()
            .any(|(_, assigned_job)| assigned_job == job)
        {
            covered_jobs += 1;
        }
    }
    let external_penalty_score = (jobs.len() - covered_jobs) as i64;
    let weighted_external_penalty_score = (beta as i64) * external_penalty_score;

    // 5. Combined Historical & Ergonomic Penalty Term
    let max_ergo = jobs
        .iter()
        .map(|job| station.ergo_score.get(job).unwrap_or(&1).to_owned() as i64)
        .max()
        .unwrap_or(1);

    let mut historical_penalty_score = 0;
    for (emp_name, job_name) in manual_assignments {
        if let (Some(i), Some(j)) = (
            employees.iter().position(|e| e == emp_name),
            jobs.iter().position(|job| job == job_name),
        ) {
            let hist_count = h_matrix[i][j] as i64;
            let e_j = station.ergo_score.get(&jobs[j]).unwrap_or(&1).to_owned() as i64;
            let ergo_multiplier = max_ergo - e_j + 1;

            let penalty_val = hist_count * ergo_multiplier;
            historical_penalty_score += penalty_val;
        }
    }
    let weighted_historical_penalty_score = (gamma as i64) * historical_penalty_score;

    // 6. Total Objective Math
    let objective_score = (offset as i64) + weighted_preference_reward_score
        - weighted_leader_penalty_score
        - weighted_external_penalty_score
        - weighted_historical_penalty_score;

    ManualScoringResult {
        objective_score,
        preference_reward_score,
        weighted_preference_reward_score,
        leader_penalty_score,
        weighted_leader_penalty_score,
        external_penalty_score,
        weighted_external_penalty_score,
        historical_penalty_score,
        weighted_historical_penalty_score,
        offset,
    }
}

#[cfg(test)]
mod tests {
    use crate::{algorithms::evaluate_manual::evaluate_manual_assignment, *};
    use std::{fs::{self, File}, io::Write, time::Instant}; // Assuming types like DayWrapper, Matrix, Day, etc. are here
        use serde_json::json;

    #[test]
    fn test_evaluate_manual() -> Result<(), Box<dyn std::error::Error>> {
        let manifest_dir =
            std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is not set");

        // Load the matrix
        let path = format!("{}/data/factory/VCE_matrix.json", manifest_dir);
        let json_content = fs::read_to_string(path)?;
        let matrix: Matrix = serde_json::from_str(&json_content)?;

        // Load the manual history data
        // Make sure this points to the file containing your manual assignment JSON
        let history_path = format!("{}/data/factory/evaluation/VCE_history_part_d.json", manifest_dir);
        // let history_path = format!("{}/data/factory/VCE_algo_strat_1_rolling_part_a.json", manifest_dir);
        let history_content = fs::read_to_string(history_path)?;
        let history_wrapper: Vec<DayWrapper> = serde_json::from_str(&history_content)?;
        let history: Vec<Day> = history_wrapper.into_iter().map(|dw| dw.day).collect();

        // Use the exact same weights as the solver
        let offset = 2000;
        let omega = 1;
        let alpha = 192;
        let beta = 384;
        let tau = 5;
        let gamma = 24;

        // Ensure we have at least one day to evaluate
        assert!(!history.is_empty(), "History file is empty!");

        if let Some(station) = matrix.stations.get("CE") {
            // Explicitly define the date you want to evaluate
            let target_year = 2026;
            let target_month = 2;
            let target_date = 27;

            // Find the index of that specific day in the history array
            let target_index = history
                .iter()
                .position(|d| {
                    d.date.year == target_year
                        && d.date.month == target_month
                        && d.date.day == target_date
                })
                .expect(&format!(
                    "Could not find date {:04}-{:02}-{:02} in history file",
                    target_year, target_month, target_date
                ));

            let target_day = &history[target_index];

            // The historical context is everything before the target day
            let previous_history: Vec<Day> = history.iter().take(target_index).cloned().collect();

            // Run the manual evaluation
            let manual_scores = evaluate_manual_assignment(
                station,
                previous_history,
                &target_day.assignments,
                offset,
                omega,
                alpha,
                beta,
                tau,
                gamma,
            );

            println!(
                "=== MANUAL SCORING FOR {:04}-{:02}-{:02} ===",
                target_day.date.year, target_day.date.month, target_day.date.day
            );
            println!("    Offs    : {}", manual_scores.offset);
            println!(
                "    Pref    : {}(omega) x {} = {}",
                omega,
                manual_scores.preference_reward_score,
                manual_scores.weighted_preference_reward_score
            );
            println!(
                "    Lead    : {}(alpha) x {} = {}",
                alpha,
                manual_scores.leader_penalty_score,
                manual_scores.weighted_leader_penalty_score
            );
            println!(
                "    Exte    : {}(beta) x {} = {}",
                beta,
                manual_scores.external_penalty_score,
                manual_scores.weighted_external_penalty_score
            );
            println!(
                "    Hist    : {}(gamma) x {} = {}",
                gamma,
                manual_scores.historical_penalty_score,
                manual_scores.weighted_historical_penalty_score
            );
            println!(
                "    Total   : {}(Offs) + {}(Pref) - {}(Lead) - {}(Exte) - {}(Hist/Ergo) = {}",
                manual_scores.offset,
                manual_scores.weighted_preference_reward_score,
                manual_scores.weighted_leader_penalty_score,
                manual_scores.weighted_external_penalty_score,
                manual_scores.weighted_historical_penalty_score,
                manual_scores.objective_score
            );
        } else {
            println!("Station 'CE' not found in matrix.");
        }

        Ok(())
    }

    #[test]
    fn test_evaluate_manual_to_file() -> Result<(), Box<dyn std::error::Error>> {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is not set");

        // 1. Load the Matrix
        let path = format!("{}/data/factory/VCE_matrix.json", manifest_dir);
        let json_content = fs::read_to_string(path)?;
        let matrix: Matrix = serde_json::from_str(&json_content)?;

        // 2. Load the Manual History
        let history_path = format!("{}/data/factory/VCE_history_part_d.json", manifest_dir);
        let history_content = fs::read_to_string(history_path)?;
        let history_wrapper: Vec<DayWrapper> = serde_json::from_str(&history_content)?;
        let full_history: Vec<Day> = history_wrapper.into_iter().map(|dw| dw.day).collect();

        // 3. Algorithm/Evaluation Parameters
        let offset = 2000;
        let omega = 1;
        let alpha = 192;
        let beta = 384;
        let tau = 5;
        let gamma = 24;

        let station_id = "CE";
        let station_data = matrix.stations.get(station_id).expect("Station CE not found in matrix");

        let mut all_days_output = Vec::new();

        // 4. Iterate day by day, starting from index 1 (to have at least 1 day of history)
        for i in 1..full_history.len() {
            let current_day = &full_history[i];
            let date = &current_day.date;
            let leader = &current_day.leader;

            // Extract the history window (up to tau days in the past)
            let start_idx = if i > tau as usize { i - (tau as usize) } else { 0 };
            let history_window = full_history[start_idx..i].to_vec();

            // Track how fast the manual evaluation runs just to keep the JSON schema consistent
            let start_time = Instant::now();

            // 5. Run the Manual Evaluation
            let manual_scores = evaluate_manual_assignment(
                station_data,
                history_window,
                &current_day.assignments,
                offset,
                omega,
                alpha,
                beta,
                tau,
                gamma,
            );

            let elapsed_time = start_time.elapsed();

            // 6. Sort assignments for consistent JSON output
            let mut sorted_assignment = current_day.assignments.clone();
            sorted_assignment.sort_by(|(e1, _), (e2, _)| e1.cmp(e2));

            // 7. Build the JSON output matching your algorithm output schema
            let day_output = json!({
                "day": {
                    "date": {
                        "year": date.year,
                        "month": date.month,
                        "day": date.day
                    },
                    "offset": manual_scores.offset,
                    "pref": manual_scores.weighted_preference_reward_score,
                    "lead": manual_scores.weighted_leader_penalty_score,
                    "exte": manual_scores.weighted_external_penalty_score,
                    "hist_ergo": manual_scores.weighted_historical_penalty_score,
                    "total": manual_scores.objective_score,
                    "solver_time": format!("{:?}", elapsed_time),
                    "station": station_id,
                    "leader": leader,
                    "assignments": sorted_assignment
                }
            });

            all_days_output.push(day_output);
        }

        // 8. Write the results to a file
        let output_file_path = format!("{}/data/factory/VCE_manual_part_d.json", manifest_dir);
        let mut file = File::create(&output_file_path)?;
        
        let pretty_json_array = serde_json::to_string_pretty(&all_days_output)?;
        file.write_all(pretty_json_array.as_bytes())?;

        println!("Successfully wrote manual evaluations to: {}", output_file_path);

        Ok(())
    }

    #[test]
    fn test_evaluate_manual_passes_to_file() -> Result<(), Box<dyn std::error::Error>> {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is not set");

        // 1. Load the Matrix
        let path = format!("{}/data/factory/GTO_matrix.json", manifest_dir); // Ensure correct matrix file
        let json_content = fs::read_to_string(path)?;
        let matrix: Matrix = serde_json::from_str(&json_content)?;

        // 2. Load the Manual Pass History
        let history_path = format!("{}/data/factory/GTO_nov_history.json", manifest_dir); // Ensure correct history file
        let history_content = fs::read_to_string(history_path)?;
        let history_wrapper: Vec<PassWrapper> = serde_json::from_str(&history_content)?;
        let full_passes: Vec<Pass> = history_wrapper.into_iter().map(|pw| pw.pass).collect();

        // 3. Algorithm/Evaluation Parameters
        let offset = 2000;
        let omega = 1;
        let alpha = 192;
        let beta = 384;
        let tau = 8; // 20 passes = 5 days
        let gamma = 24;

        let station_id = "GTO";
        let station_data = matrix.stations.get(station_id).expect("Station GTO not found in matrix");

        // 4. Identify the Team Leader dynamically from the matrix role
        let leader_name = station_data
            .people
            .iter()
            .find(|p| matches!(p.role, Role::TeamLeader))
            .map(|p| p.name.clone())
            .unwrap_or_default();

        let mut all_passes_output = Vec::new();

        // 5. Iterate pass by pass, starting from index 1
        for i in 1..full_passes.len() {
            let current_pass = &full_passes[i];
            
            // Extract the history window (up to tau passes in the past)
            let start_idx = if i > tau as usize { i - (tau as usize) } else { 0 };
            let history_window_passes = full_passes[start_idx..i].to_vec();

            // Map Pass to Day so the evaluator function can digest it
            let history_window_days: Vec<Day> = history_window_passes.iter().map(|p| {
                Day {
                    date: p.date.clone(), 
                    station: p.station.clone(),
                    leader: leader_name.clone(), 
                    assignments: p.assignments.clone(), 
                }
            }).collect();

            // Track how fast the manual evaluation runs
            let start_time = Instant::now();

            // 6. Run the Manual Evaluation
            let manual_scores = evaluate_manual_assignment(
                station_data,
                history_window_days,
                &current_pass.assignments, // Evaluate exactly what the human assigned
                offset,
                omega,
                alpha,
                beta,
                tau,
                gamma,
            );

            let elapsed_time = start_time.elapsed();

            // 7. Sort assignments for consistent JSON output
            let mut sorted_assignment = current_pass.assignments.clone();
            sorted_assignment.sort_by(|(e1, _), (e2, _)| e1.cmp(e2));

            // 8. Build the JSON output matching your pass schema
            let day_output = json!({
                "pass": {
                    "date": {
                        "year": current_pass.date.year,
                        "month": current_pass.date.month,
                        "day": current_pass.date.day
                    },
                    "station": station_id,
                    "pass": current_pass.pass,
                    "offset": manual_scores.offset,
                    "pref": manual_scores.weighted_preference_reward_score,
                    "lead": manual_scores.weighted_leader_penalty_score,
                    "exte": manual_scores.weighted_external_penalty_score,
                    "hist_ergo": manual_scores.weighted_historical_penalty_score,
                    "total": manual_scores.objective_score,
                    "solver_time": format!("{:?}", elapsed_time),
                    "assignments": sorted_assignment
                }
            });

            all_passes_output.push(day_output);
        }

        // 9. Write the results to a file
        let output_file_path = format!("{}/data/factory/GTO_november_manual_tau_8.json", manifest_dir);
        let mut file = File::create(&output_file_path)?;
        
        let pretty_json_array = serde_json::to_string_pretty(&all_passes_output)?;
        file.write_all(pretty_json_array.as_bytes())?;

        println!("Successfully wrote manual pass evaluations to: {}", output_file_path);

        Ok(())
    }
}
