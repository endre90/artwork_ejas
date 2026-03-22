use std::time::{Duration, Instant};

use ast::Ast;
use z3::{
    ast::{Bool, Int},
    *,
};

use crate::*;

#[derive(Debug)]
pub struct ErgonomicAssignmentSolution {
    pub internal_assignments: Vec<(String, String)>, // (employee, job) pairs
    pub external_assignments: Vec<String>,           // jobs
    pub objective_score: i64,                        // final value of the objective function
    pub preference_reward_score: i64,
    pub weighted_preference_reward_score: i64,
    pub leader_penalty_score: i64,
    pub weighted_leader_penalty_score: i64,
    pub external_penalty_score: i64,
    pub weighted_external_penalty_score: i64,
    pub historical_penalty_score: i64,
    pub weighted_historical_penalty_score: i64,
    // pub ergonomics_penalty_score: i64,
    // pub weighted_ergonomics_penalty_score: i64,
    // pub ergonomics_reward_score: i64,
    // pub weighted_ergonomics_reward_score: i64,
    pub c_matrix: Vec<Vec<bool>>,  // competence matrix passed back
    pub p_matrix: Vec<Vec<usize>>, // preference matrix passed back
    pub h_matrix: Vec<Vec<u32>>,   // historic matrix passed back
    pub solving_time: Duration,    // how long the solver took
}

pub fn calculate_ergonomic_assignment(
    station: &Station,
    history: Vec<Day>,
    offset: u32, // Add to objective to get a positive integer result (just for aesthetics)
    omega: u32,  // How strongly preference considerations influence the objective function
    alpha: u32,  // How strongly to discourage leader usage
    beta: u32,   // How strongly to discourage external operator usage
    tau: u32,    // Number of days to consider in the historical data (from tau to today)
    gamma: u32, // how strongly to penalize assigning the same employee–job pair that was frequently assigned in the past tau days
    delta: u32, // Ergonomics weight
    theta: u32, // Weight controlling how the historical count reduces the ergonomics benefit of a job for a given employee.
    forced_assignments: &[(String, String)],
    loaned_employees: &[String],
    absent_employees: &[String],
    special_training_employees: &[String],
) -> ErgonomicAssignmentSolution {
    let mut jobs = vec![];
    let mut employees = vec![];
    let mut competences: Vec<(String, Vec<String>)> = vec![];
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
        competences.push((person.name.clone(), person.competences.clone()));
        preferences.push((person.name.clone(), person.preferences.clone()));
    }

    let c_matrix = build_competence_matrix(&competences, &jobs);
    let p_matrix = build_preference_matrix(&preferences, &jobs);

    // Create the Z3 context and optimizer
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let optimizer = Optimize::new(&ctx);

    // let mut params = z3::Params::new(&ctx);
    // params.set_u32("timeout", 60000); // 5000 milliseconds = 5 seconds
    // optimizer.set_params(&params);

    // We define x_ij as a boolean variable indicating whether employee i
    // is assigned to job j, i.e. x_ij ∈ {true, false}.
    // Create a container for all xᵢⱼ variables
    let mut x = Vec::new();

    // Loop over each employee index i
    for i in 0..employees.len() {
        // For each employee, create a row of boolean variables
        let mut row = Vec::new();

        // Loop over each job index j
        for j in 0..jobs.len() {
            // Create a boolean variable xᵢⱼ with a unique name
            let var_name = format!("x_{}_{}", i, j);
            let x_ij = Bool::new_const(&ctx, var_name);

            // Add the variable to the current row
            row.push(x_ij);
        }

        // Add this row to the main container
        x.push(row);
    }

    // For each job j, we define e_j as a boolean variable indicating whether job j
    // is assigned to an external employee, i.e., e_j ∈ {true, false}.

    // Create a container for the external assignment variables
    let mut e = Vec::new();

    // Loop over each job index j
    for j in 0..jobs.len() {
        // Create a unique name for the boolean variable e_j
        let var_name = format!("e_{}", j);

        // Create the boolean variable e_j
        let e_j = Bool::new_const(&ctx, var_name);

        // Store e_j in the container
        e.push(e_j);
    }

    // Constraint 1:
    // For each employee i, we want: Σ(x_ij) ≤ 1
    // This means an employee can have at most one job.

    // For each employee i:
    for i in 0..employees.len() {
        // Gather boolean variables x_ij for all jobs j
        let mut employee_constraints = Vec::new();
        for j in 0..jobs.len() {
            employee_constraints.push(x[i][j].clone());
        }

        // Build a "sum ≤ 1" constraint from these variables
        let at_most_one_job_per_employee = ast::Bool::pb_le(
            &ctx,
            employee_constraints
                .iter()
                .map(|x_ij| (x_ij, 1)) // each x_ij contributes '1' to the sum
                .collect::<Vec<(&ast::Bool, i32)>>()
                .as_slice(),
            1, // sum of x_ij ≤ 1
        );

        // Assert this constraint in the optimizer
        optimizer.assert(&at_most_one_job_per_employee);
    }

    // Constraint 2:
    // We want exactly one of either e_j or x_ij for each job j to be true:
    //   e_j + Σi(x_ij) = 1
    // This means job j is assigned to exactly one employee:
    // either an external contractor (e_j) or exactly one internal employee.

    for j in 0..jobs.len() {
        // Gather all internal-assignment variables x_ij for job j
        let mut job_constraints = Vec::new();
        for i in 0..employees.len() {
            job_constraints.push(x[i][j].clone());
        }

        // Combine the external variable e_j and the internal variables x_ij
        let mut all_vars_for_job = Vec::new();
        all_vars_for_job.push(e[j].clone());
        all_vars_for_job.extend(job_constraints);

        // Create a "sum = 1" constraint on these boolean variables
        let at_most_one_employee_per_job = ast::Bool::pb_eq(
            &ctx,
            all_vars_for_job
                .iter()
                .map(|var| (var, 1)) // each var contributes '1' if true
                .collect::<Vec<(&ast::Bool, i32)>>()
                .as_slice(),
            1, // sum of e_j + x_ij = 1
        );

        // Assert this constraint in the optimizer
        optimizer.assert(&at_most_one_employee_per_job);
    }

    // Constraint 3:
    // For each job j, we want: Σ(x_ij for all competent i) + e_j ≥ 1
    // This means each job j is either covered by at least one competent internal employee
    // or is assigned to an external worker (e_j).

    // Prepare a container to store all variables that can cover each job j
    let mut coverage_per_job: Vec<Vec<ast::Bool>> = vec![Vec::new(); jobs.len()];

    // Populate a list of x_ij variables for each job j, but only if employee i is competent
    for i in 0..employees.len() {
        for j in 0..jobs.len() {
            if c_matrix[i][j] {
                // Add x_ij to the coverage list for job j
                coverage_per_job[j].push(x[i][j].clone());
            }
        }
    }

    // For each job j, create a constraint that ensures at least one competent employee is assigned
    for j in 0..jobs.len() {
        coverage_per_job[j].push(e[j].clone()); // Add the external worker option
                                                // Build the "sum ≥ 1" constraint from the collected x_ij and e_j variables
        let job_covered = ast::Bool::pb_ge(
            &ctx,
            coverage_per_job[j]
                .iter()
                .map(|b| (b, 1)) // each x_ij contributes '1' if it's true
                .collect::<Vec<(&ast::Bool, i32)>>()
                .as_slice(),
            1, // sum of ≥ 1
        );

        // Assert this constraint in the optimizer
        optimizer.assert(&job_covered);
    }

    // Constraint 4:
    // We impose x_ij implies c_ij for each employee i and job j.
    // This means if x_ij = true (employee i is assigned to job j),
    // then c_ij must also be true (employee i is competent for job j).

    for i in 0..employees.len() {
        for j in 0..jobs.len() {
            // Build the implication: x_ij implies c_ij
            let implies_competent = Bool::implies(&x[i][j], &Bool::from_bool(&ctx, c_matrix[i][j]));

            // Assert this implication in the optimizer
            optimizer.assert(&implies_competent);
        }
    }

    // Preference reward term:
    // We want to maximize overall preferences. For each employee i and job j,
    // p_matrix[i][j] gives a rank, where a lower rank = higher preference.
    // We convert this rank into a "score" using (jobs.len() - rank).
    // When x_ij is true, we add that score; otherwise we add 0.
    // The wheighed sum of these scores (omega * preference_obj)
    // is the preference term in our objective.

    let mut preference_score = Vec::new();

    // Calculate a score for each (i, j) pair and store (boolean variable, score)
    for i in 0..employees.len() {
        for j in 0..jobs.len() {
            // rank = p_matrix[i][j], lower means more preferred
            let rank = p_matrix[i][j];

            // Convert rank into a score: higher for lower rank
            let score = (jobs.len() - rank) as i32;

            // Store the pair: (assignment variable, preference score)
            preference_score.push((x[i][j].clone(), score));
        }
    }

    // Convert each (variable, score) into an expression: x_ij ? score : 0
    let mut preference_score_sum = Vec::new();
    for (assignment_var, score) in &preference_score {
        let score_expr = assignment_var.ite(
            &z3::ast::Int::from_i64(&ctx, *score as i64), // if x_ij = true, add score
            &z3::ast::Int::from_i64(&ctx, 0),             // if x_ij = false, add 0
        );
        preference_score_sum.push(score_expr);
    }

    // Sum up all these expressions to form the total preference objective
    let preference_reward = Int::add(&ctx, &preference_score_sum);

    // Track the value of the preference objective for measurement purposes
    let preference_reward_tracker = Int::new_const(&ctx, "preference_reward");
    optimizer.assert(&preference_reward._eq(&preference_reward_tracker));

    // Add weight to the preference objective
    let omega_z3 = Int::from_i64(&ctx, omega as i64);
    let weighted_preference_reward = omega_z3 * preference_reward.clone();

    // Track the value of the weighted preference objective for measurement purposes
    let weighted_preference_reward_tracker = Int::new_const(&ctx, "weighted_preference_reward");
    optimizer.assert(&weighted_preference_reward._eq(&weighted_preference_reward_tracker));

    // Team leader penalty term:
    // 1. Identify the first TeamLeader in `station.people`.
    // 2. Count how many jobs are assigned to that leader using x[index][job] (1 if true, 0 if false).
    // 3. Store this count in an Int expression (`leader_penalty`).
    // 4. Create a separate variable (`leader_penalty_tracker`) just to track that count.
    // 5. Multiply the count by a weight `alpha` to form a weighted objective.
    // 6. Track that weighted objective for measurement.

    let mut leader_penalty = Int::from_i64(&ctx, 0); // Will remain 0 if leader doesn't exist

    // Look for a person with the TeamLeader role
    for (index, person) in station.people.iter().enumerate() {
        if let Role::TeamLeader = person.role {
            // For this leader, sum how many jobs they are assigned
            let mut leader_penalties = Vec::new();

            // Loop over all jobs
            for j in 0..jobs.len() {
                // If x[index][j] is true, we add 1; otherwise add 0
                let penalty_if_leader =
                    x[index][j].ite(&Int::from_i64(&ctx, 1), &Int::from_i64(&ctx, 0));
                leader_penalties.push(penalty_if_leader);
            }

            // `leader_penalty` is the sum of all leader assignments
            leader_penalty = Int::add(&ctx, &leader_penalties);

            // Stop after finding the first TeamLeader
            break;
        }
    }

    // Track the value of the leader objective for measurement purposes
    let leader_penalty_tracker = Int::new_const(&ctx, "leader_penalty");
    optimizer.assert(&leader_penalty._eq(&leader_penalty_tracker));

    // Add weight to the leader objective
    let alpha_z3 = Int::from_i64(&ctx, alpha as i64);
    let weighted_leader_penalty = alpha_z3 * leader_penalty.clone();

    // Track the value of the weighted leader objective for measurement purposes
    let weighted_leader_penalty_tracker = Int::new_const(&ctx, "weighted_leader_penalty");
    optimizer.assert(&weighted_leader_penalty._eq(&weighted_leader_penalty_tracker));

    // External assignment penalty term:
    // For each job j, e_j is a boolean variable indicating if job j is assigned externally.
    // We impose a penalty of 1 for each external assignment. Summing these gives the total
    // count of externally assigned jobs. Then we multiply that sum by beta, a weight
    // controlling how strongly we penalize external assignments.

    // Build a list of "penalty expressions" for each job's external variable e_j
    let mut external_penalty_expressions = Vec::new();
    for e_j in &e {
        // If e_j is true, add 1; if e_j is false, add 0
        let penalty_expr = e_j.ite(&Int::from_i64(&ctx, 1), &Int::from_i64(&ctx, 0));
        external_penalty_expressions.push(penalty_expr);
    }

    // Sum all these "1 or 0" penalty expressions
    let external_penalty = Int::add(&ctx, &external_penalty_expressions);

    // Track the external penalty for measurement purposes
    let external_penalty_tracker = Int::new_const(&ctx, "external_penalty");
    optimizer.assert(&external_penalty._eq(&external_penalty_tracker));

    // Multiply the total external penalty by beta
    let beta_z3 = Int::from_i64(&ctx, beta as i64);
    let weighted_external_penalty = beta_z3 * external_penalty;

    // Track the weighted external penalty for measurement purposes
    let weighted_external_penalty_tracker = Int::new_const(&ctx, "weighted_external_penalty");
    optimizer.assert(&weighted_external_penalty_tracker._eq(&weighted_external_penalty));

    //     If we assume a higher ergonomic score (ej​) means a job is physically easier or better, we want to multiply the historical penalty by a larger number when the job has a bad score.

    // We can find the maximum ergonomic score in the station (Emax​) and invert the score to create a multiplier:
    // ErgoMultiplierj​=Emax​−ej​+1

    //     If a job has a great score (e.g., 5 out of 5): The multiplier is (5−5+1)=1. The history penalty applies normally.

    //     If a job has a terrible score (e.g., 1 out of 5): The multiplier is (5−1+1)=5. The solver will heavily penalize repeating this job.

    // The new penalty term for an assignment becomes a simple, pre-calculated constant:
    // Penaltyij​=hij​×ErgoMultiplierj​
    // Historic + Ergonomic penalty term (Combined):
    // We penalize repeating a job, but we multiply that penalty based on how bad the job's ergonomics are.




    // Working and fast:
    let h_matrix = build_historical_count_matrix(history, tau as usize, &employees, &jobs);

    // Find the max ergonomic score in the station to create the inverse multiplier
    let max_ergo = jobs
        .iter()
        .map(|job| station.ergo_score.get(job).unwrap_or(&1).to_owned() as i64)
        .max()
        .unwrap_or(1);

    let mut h_fairness_terms = Vec::new();

    for i in 0..employees.len() {
        for j in 0..jobs.len() {
            // CRITICAL: Domain pruning. If not competent, don't build the AST node.
            if !c_matrix[i][j] {
                continue;
            }

            let hist_count = h_matrix[i][j] as i64;
            let e_j = station.ergo_score.get(&jobs[j]).unwrap_or(&1).to_owned() as i64;

            // Invert the ergo score so bad ergonomics multiply the penalty more heavily
            let ergo_multiplier = max_ergo - e_j + 1;

            // Calculate the final static penalty integer in Rust
            let penalty_val = hist_count * ergo_multiplier;

            if penalty_val > 0 {
                let penalty_expr = x[i][j].ite(
                    &z3::ast::Int::from_i64(&ctx, penalty_val),
                    &z3::ast::Int::from_i64(&ctx, 0),
                );
                h_fairness_terms.push(penalty_expr);
            }
        }
    }

    if h_fairness_terms.is_empty() {
        h_fairness_terms.push(z3::ast::Int::from_i64(&ctx, 0));
    }

    let historical_penalty = z3::ast::Int::add(&ctx, &h_fairness_terms);

    let historical_penalty_tracker = Int::new_const(&ctx, "historical_penalty");
    optimizer.assert(&historical_penalty._eq(&historical_penalty_tracker));

    // You can use the 'gamma' weight to control the overall strength of this combined penalty
    let gamma_z3 = z3::ast::Int::from_i64(&ctx, gamma as i64);
    let weighted_historical_penalty = gamma_z3 * historical_penalty;

    let weighted_historical_penalty_tracker = Int::new_const(&ctx, "weighted_historical_penalty");
    optimizer.assert(&weighted_historical_penalty_tracker._eq(&weighted_historical_penalty));

    // Separated H and E below
    // // ---------------------------------------------------------
    // // 1. PURE HISTORICAL PENALTY TERM (Gamma)
    // // ---------------------------------------------------------
    // let h_matrix = build_historical_count_matrix(history, tau as usize, &employees, &jobs);

    // let mut h_fairness_terms = Vec::new();
    // for i in 0..employees.len() {
    //     for j in 0..jobs.len() {
    //         if !c_matrix[i][j] {
    //             continue; // Domain pruning
    //         }

    //         let hist_count = h_matrix[i][j] as i64;

    //         if hist_count > 0 { // Zero-penalty pruning
    //             let penalty_expr = x[i][j].ite(
    //                 &z3::ast::Int::from_i64(&ctx, hist_count),
    //                 &z3::ast::Int::from_i64(&ctx, 0),
    //             );
    //             h_fairness_terms.push(penalty_expr);
    //         }
    //     }
    // }

    // if h_fairness_terms.is_empty() {
    //     h_fairness_terms.push(z3::ast::Int::from_i64(&ctx, 0));
    // }

    // let historical_penalty = z3::ast::Int::add(&ctx, &h_fairness_terms);
    // let historical_penalty_tracker = Int::new_const(&ctx, "historical_penalty");
    // optimizer.assert(&historical_penalty._eq(&historical_penalty_tracker));

    // let gamma_z3 = z3::ast::Int::from_i64(&ctx, gamma as i64);
    // let weighted_historical_penalty = gamma_z3 * historical_penalty;

    // let weighted_historical_penalty_tracker = Int::new_const(&ctx, "weighted_historical_penalty");
    // optimizer.assert(&weighted_historical_penalty_tracker._eq(&weighted_historical_penalty));


    // // ---------------------------------------------------------
    // // 2. COMBINED ERGONOMIC & HISTORY PENALTY TERM (Delta)
    // // ---------------------------------------------------------
    // let max_ergo = jobs
    //     .iter()
    //     .map(|job| station.ergo_score.get(job).unwrap_or(&1).to_owned() as i64)
    //     .max()
    //     .unwrap_or(1);

    // let mut ergonomic_terms = Vec::new();
    
    // for i in 0..employees.len() {
    //     for j in 0..jobs.len() {
    //         if !c_matrix[i][j] {
    //             continue; // Domain pruning
    //         }

    //         let e_j = station.ergo_score.get(&jobs[j]).unwrap_or(&1).to_owned() as i64;
    //         let hist_count = h_matrix[i][j] as i64;
            
    //         // H_ij * (E_max - E_j + 1)
    //         let ergo_penalty_val = hist_count * (max_ergo - e_j + 1);

    //         if ergo_penalty_val > 0 { // Zero-penalty pruning
    //             let ergo_expr = x[i][j].ite(
    //                 &z3::ast::Int::from_i64(&ctx, ergo_penalty_val),
    //                 &z3::ast::Int::from_i64(&ctx, 0),
    //             );
    //             ergonomic_terms.push(ergo_expr);
    //         }
    //     }
    // }

    // if ergonomic_terms.is_empty() {
    //     ergonomic_terms.push(z3::ast::Int::from_i64(&ctx, 0));
    // }

    // let ergonomics_penalty = z3::ast::Int::add(&ctx, &ergonomic_terms);
    // let ergonomics_penalty_tracker = Int::new_const(&ctx, "ergonomics_penalty");
    // optimizer.assert(&ergonomics_penalty._eq(&ergonomics_penalty_tracker));

    // let delta_z3 = z3::ast::Int::from_i64(&ctx, delta as i64);
    // let weighted_ergonomics_penalty = delta_z3 * ergonomics_penalty;

    // let weighted_ergonomics_penalty_tracker = Int::new_const(&ctx, "weighted_ergonomics_penalty");
    // optimizer.assert(&weighted_ergonomics_penalty_tracker._eq(&weighted_ergonomics_penalty));


    // Force specific (employee, job) pairings
    for (emp_name, job_name) in forced_assignments {
        // Find the matching indices
        if let (Some(i), Some(j)) = (
            employees.iter().position(|e| e == emp_name),
            jobs.iter().position(|job| job == job_name),
        ) {
            // Assert that this specific boolean MUST be true
            optimizer.assert(&x[i][j]._eq(&Bool::from_bool(&ctx, true)));
        } else {
            log::warn!(
                "Tried to force assignment for unknown employee or job: {} -> {}",
                emp_name,
                job_name
            );
        }
    }

    // Force loaned-out employees (they get NO jobs today)
    for emp_name in loaned_employees {
        if let Some(i) = employees.iter().position(|e| e == emp_name) {
            // Assert that EVERY job for this employee MUST be false
            for j in 0..jobs.len() {
                optimizer.assert(&x[i][j]._eq(&Bool::from_bool(&ctx, false)));
            }
        } else {
            log::warn!("Unknown employee: {}", emp_name);
        }
    }

    // Force absent employees (they get NO jobs today)
    for emp_name in absent_employees {
        if let Some(i) = employees.iter().position(|e| e == emp_name) {
            // Assert that EVERY job for this employee MUST be false
            for j in 0..jobs.len() {
                optimizer.assert(&x[i][j]._eq(&Bool::from_bool(&ctx, false)));
            }
        } else {
            log::warn!("Unknown employee: {}", emp_name);
        }
    }

    // Force special training employees (they get NO jobs today)
    for emp_name in special_training_employees {
        if let Some(i) = employees.iter().position(|e| e == emp_name) {
            // Assert that EVERY job for this employee MUST be false
            for j in 0..jobs.len() {
                optimizer.assert(&x[i][j]._eq(&Bool::from_bool(&ctx, false)));
            }
        } else {
            log::warn!("Unknown employee: {}", emp_name);
        }
    }

    // Add offset to the objective
    let offset_z3 = Int::from_i64(&ctx, offset as i64);

    // The objective os the weighted sum pf objective terms
    let objective = offset_z3 + weighted_preference_reward.clone()
        - weighted_leader_penalty.clone()
        - weighted_external_penalty.clone()
        - weighted_historical_penalty.clone();
        // - weighted_ergonomics_penalty.clone();
    // + weighted_ergonomics_reward.clone();

    // The goal is to maximize the total score (objective)
    optimizer.maximize(&objective);

    // Variable to track the total objective
    let objective_tracker = Int::new_const(&ctx, "objective");
    optimizer.assert(&objective_tracker._eq(&objective));

    // Start measuring calculation time
    let start_time = Instant::now();

    // Check consistency and produce optimal values
    let result = optimizer.check(&[]);

    // Measure time after solver finishes
    let solving_time = start_time.elapsed();

    match result {
        SatResult::Sat => {
            // Extract the model (assignment of variables)
            let model = optimizer
                .get_model()
                .expect("Model should be available when Sat");

            // Helper to get an i64 from an Int AST node in the model
            fn get_objective_value(model: &z3::Model, obj_ast: &Int, label: &str) -> i64 {
                match model.get_const_interp(obj_ast) {
                    Some(val) => val.as_i64().unwrap_or_else(|| {
                        println!("{} has an interpretation that isn't an i64", label);
                        0
                    }),
                    None => {
                        println!("{} has no interpretation in the model", label);
                        0
                    }
                }
            }

            let objective_score = get_objective_value(&model, &objective_tracker, "objective");
            let preference_reward_score =
                get_objective_value(&model, &preference_reward_tracker, "preference_reward");
            let weighted_preference_reward_score = get_objective_value(
                &model,
                &weighted_preference_reward_tracker,
                "weighted_preference_reward",
            );
            let leader_penalty_score =
                get_objective_value(&model, &leader_penalty_tracker, "leader_penalty");
            let weighted_leader_penalty_score = get_objective_value(
                &model,
                &weighted_leader_penalty_tracker,
                "weighted_leader_penalty",
            );

            let external_penalty_score =
                get_objective_value(&model, &external_penalty_tracker, "external_penalty");
            let weighted_external_penalty_score = get_objective_value(
                &model,
                &weighted_external_penalty_tracker,
                "weighted_external_penalty",
            );

            let historical_penalty_score =
                get_objective_value(&model, &historical_penalty_tracker, "historical_penalty");
            let weighted_historical_penalty_score = get_objective_value(
                &model,
                &weighted_historical_penalty_tracker,
                "weighted_historical_penalty",
            );

            // let ergonomics_reward_score =
            //     get_objective_value(&model, &ergonomics_reward_tracker, "ergonomics_reward");
            // let weighted_ergonomics_reward_score = get_objective_value(
            //     &model,
            //     &weighted_ergonomics_reward_tracker,
            //     "weighted_ergonomics_reward",
            // );

            // let ergonomics_penalty_score =
            //     get_objective_value(&model, &ergonomics_penalty_tracker, "ergonomics_penalty");
            // let weighted_ergonomics_penalty_score = get_objective_value(
            //     &model,
            //     &weighted_ergonomics_penalty_tracker,
            //     "weighted_ergonomics_penalty",
            // );

            // Build a list of (employee, job) assignments where x[i][j] = true
            let mut internal_assignments = Vec::new();
            for i in 0..employees.len() {
                for j in 0..jobs.len() {
                    if let Some(true) = model.eval(&x[i][j], true).unwrap().as_bool() {
                        internal_assignments.push((employees[i].clone(), jobs[j].clone()));
                    }
                }
            }

            // Build a list of job assignments where e[j] = true
            let mut external_assignments = Vec::new();
            for j in 0..jobs.len() {
                if let Some(true) = model.eval(&e[j], true).unwrap().as_bool() {
                    external_assignments.push(jobs[j].clone());
                }
            }

            // Log success
            log::info!(target: "employee_job_assignment", "Solution found");

            // Return everything in the AssignmentSolution struct
            ErgonomicAssignmentSolution {
                internal_assignments,
                external_assignments,
                objective_score,
                preference_reward_score,
                weighted_preference_reward_score,
                leader_penalty_score,
                weighted_leader_penalty_score,
                external_penalty_score,
                weighted_external_penalty_score,
                historical_penalty_score,
                weighted_historical_penalty_score,
                // ergonomics_penalty_score,
                // weighted_ergonomics_penalty_score,
                // ergonomics_reward_score,
                // weighted_ergonomics_reward_score,
                c_matrix,
                p_matrix,
                h_matrix,
                solving_time,
            }
        }
        SatResult::Unsat => {
            log::warn!(target: "employee_job_assignment", "No solution found");
            ErgonomicAssignmentSolution {
                internal_assignments: Vec::new(),
                external_assignments: Vec::new(),
                objective_score: 0,
                preference_reward_score: 0,
                weighted_preference_reward_score: 0,
                leader_penalty_score: 0,
                weighted_leader_penalty_score: 0,
                external_penalty_score: 0,
                weighted_external_penalty_score: 0,
                historical_penalty_score: 0,
                weighted_historical_penalty_score: 0,
                // ergonomics_penalty_score: 0,
                // weighted_ergonomics_penalty_score: 0,
                // ergonomics_reward_score: 0,
                // weighted_ergonomics_reward_score: 0,
                c_matrix,
                p_matrix,
                h_matrix,
                solving_time,
            }
        }
        _ => {
            log::error!(target: "employee_job_assignment", "Solver returned an unknown or failed state");
            ErgonomicAssignmentSolution {
                internal_assignments: Vec::new(),
                external_assignments: Vec::new(),
                objective_score: 0,
                preference_reward_score: 0,
                weighted_preference_reward_score: 0,
                leader_penalty_score: 0,
                weighted_leader_penalty_score: 0,
                external_penalty_score: 0,
                weighted_external_penalty_score: 0,
                historical_penalty_score: 0,
                weighted_historical_penalty_score: 0,
                // ergonomics_penalty_score: 0,
                // weighted_ergonomics_penalty_score: 0,
                // ergonomics_reward_score: 0,
                // weighted_ergonomics_reward_score: 0,
                c_matrix,
                p_matrix,
                h_matrix,
                solving_time,
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use std::fs;

    use crate::*;

    #[test]
    fn test_ergonomic() -> Result<(), Box<dyn std::error::Error>> {
        let manifest_dir =
            std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is not set");
        // let path = format!("{}/data/synthetic/{}_matrix_static.json", manifest_dir, s);
        let path = format!("{}/data/factory/VCE_matrix.json", manifest_dir);

        let history_path = format!("{}/data/factory/VCE_history.json", manifest_dir);
        let history_content = fs::read_to_string(history_path)?;
        let history_wrapper: Vec<DayWrapper> = serde_json::from_str(&history_content)?;
        let history: Vec<Day> = history_wrapper.into_iter().map(|dw| dw.day).collect();

        let json_content = fs::read_to_string(path)?;
        let matrix: Matrix = serde_json::from_str(&json_content)?;

        let offset = 10;
        let omega = 1;
        let alpha = 1;
        let beta = 1;
        let tau = 10;
        let gamma = 1;
        let theta = 1;
        let delta = 1;

        let forced_assignments = [
            // ("O".to_string(), "O3".to_string()),
            // ("P".to_string(), "O6".to_string()),
        ];

        if let Some(station) = matrix.stations.get("CE") {
            let s = calculate_ergonomic_assignment(
                station,
                history.clone(),
                offset,
                omega,
                alpha,
                beta,
                tau,
                gamma,
                delta,
                theta,
                &forced_assignments,
                &[],
                &[],
                &[],

                // &["O".to_string()],
                // &["A".to_string()], // A is absent for example...
                // &["P".to_string()]
            );
            pretty_print_internal_assignments(station, &s.internal_assignments);
            pretty_print_external_assignments(station, &s.external_assignments);
            pretty_print_competence_matrix(station);
            pretty_print_preference_matrix(station);
            pretty_print_historical_matrix(station, history.clone(), tau);
            println!("=== SCORING ===");
            println!("    Offs    : {}", offset);
            println!(
                "    Pref    : {}(omega) x {} = {}",
                omega, s.preference_reward_score, s.weighted_preference_reward_score
            );
            println!(
                "    Lead    : {}(alpha) x {} = {}",
                alpha, s.leader_penalty_score, s.weighted_leader_penalty_score
            );
            println!(
                "    Exte    : {}(beta) x {} = {}",
                beta, s.external_penalty_score, s.weighted_external_penalty_score
            );
            println!(
                "    Hist    : {}(gamma) x {} = {}",
                gamma, s.historical_penalty_score, s.weighted_historical_penalty_score
            );
            // println!(
            //     "    Ergo    : {}(delta) x {} = {}",
            //     delta, s.ergonomics_penalty_score, s.weighted_ergonomics_penalty_score
            // );
            // println!(
            //     "    Ergo    : {}(delta) x {} = {}",
            //     delta, s.ergonomics_reward_score, s.weighted_ergonomics_reward_score
            // );
            println!(
                "    Total   : {}(Offs) + {}(Pref) - {}(Lead) - {}(Exte) - {}(Hist/Ergo) = {}",
                offset,
                s.weighted_preference_reward_score,
                s.weighted_leader_penalty_score,
                s.weighted_external_penalty_score,
                s.weighted_historical_penalty_score,
                // s.weighted_ergonomics_penalty_score,
                s.objective_score
            );
            println!();
            println!("=== SOLVER TIME ===");
            println!("    {:?}", s.solving_time);

            // let offset = 10;
            // let tau = 10;
            // let omega_min = 0;
            // let omega_max = 10;
            // let alpha_min = 1000;
            // let alpha_max = 1000;
            // let beta_min = 1000;
            // let beta_max = 1000;
            // let gamma_min = 0;
            // let gamma_max = 10;
            // let delta_min = 0;
            // let delta_max = 10;
            // let theta_min = 0;
            // let theta_max = 10;

            // let points = run_and_group_points_ergonomic(
            //     station,
            //     history.clone(),
            //     offset,
            //     tau,
            //     omega_min,
            //     omega_max,
            //     alpha_min,
            //     alpha_max,
            //     beta_min,
            //     beta_max,
            //     gamma_min,
            //     gamma_max,
            //     delta_min,
            //     delta_max,
            //     theta_min,
            //     theta_max,
            // );
            // println!("{:?}", points.len());
            // for (k, v) in &points {
            //     println!("{:?}", k);
            //     // println!("{:?}", v);
            // }

            // let _ = write_to_file_points_ergonomic(&points);
        }

        Ok(())
    }
}
