use std::{cmp::{max, min}, collections::{HashMap, HashSet, VecDeque}, time::{Duration, Instant}};

use ast::Ast;
use z3::{
    ast::{Bool, Int},
    *,
};

use crate::*;

// #[derive(Debug)]
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct StaticAssignmentSolution {
    pub internal_assignments: Vec<(String, String)>, // (employee, job) pairs
    pub objective_score: i64,              // final value of the objective function
    pub preference_reward_score: i64,
    pub weighted_preference_reward_score: i64,
    pub leader_penalty_score: i64,
    pub weighted_leader_penalty_score: i64,
    pub c_matrix: Vec<Vec<bool>>,  // competence matrix passed back
    pub p_matrix: Vec<Vec<usize>>, // preference matrix passed back
    pub solving_time: Duration,    // how long the solver took
}

pub fn calculate_static_assignment(
    station: &Station,
    offset: u32, // Add to objective to get a positive integer result (just for aesthetics)
    omega: u32,  // How strongly preference considerations influence the objective function
    alpha: u32,  // How strongly to discourage leader usage
) -> StaticAssignmentSolution {
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
    // For each job j, we want: Σ(x_ij) ≥ 1
    // (summed only over those employees i who are competent for job j).
    // This ensures each job is covered by at least one competent worker.

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
        // Build the "sum ≥ 1" constraint from the collected x_ij variables
        let job_covered = ast::Bool::pb_ge(
            &ctx,
            coverage_per_job[j]
                .iter()
                .map(|b| (b, 1)) // each x_ij contributes '1' if it's true
                .collect::<Vec<(&ast::Bool, i32)>>()
                .as_slice(),
            1, // sum of x_ij ≥ 1
        );

        // Assert this constraint in the optimizer
        optimizer.assert(&job_covered);
    }

    // Constraint 3:
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

    // Preference objective term:
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
    let weighted_preference_reward_tracker =
        Int::new_const(&ctx, "weighted_preference_reward");
    optimizer.assert(&weighted_preference_reward._eq(&weighted_preference_reward_tracker));

    // Team leader objective term:
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

    // Add offset to the objective
    let offset_z3 = Int::from_i64(&ctx, offset as i64);

    // The objective os the weighted sum pf objective terms
    let objective =
        offset_z3 + weighted_preference_reward.clone() - weighted_leader_penalty.clone();
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
            let preference_reward_score = get_objective_value(
                &model,
                &preference_reward_tracker,
                "preference_reward",
            );
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

            // Build a list of (employee, job) assignments where x[i][j] = true
            let mut internal_assignments = Vec::new();
            for i in 0..employees.len() {
                for j in 0..jobs.len() {
                    if let Some(true) = model.eval(&x[i][j], true).unwrap().as_bool() {
                        internal_assignments.push((employees[i].clone(), jobs[j].clone()));
                    }
                }
            }

            // Log success
            log::info!(target: "employee_job_assignment", "Solution found");

            // Return everything in the AssignmentSolution struct
            StaticAssignmentSolution {
                internal_assignments,
                objective_score,
                preference_reward_score,
                weighted_preference_reward_score,
                leader_penalty_score,
                weighted_leader_penalty_score,
                c_matrix,
                p_matrix,
                solving_time,
            }
        }
        SatResult::Unsat => {
            log::warn!(target: "employee_job_assignment", "No solution found");
            StaticAssignmentSolution {
                internal_assignments: Vec::new(),
                objective_score: 0,
                preference_reward_score: 0,
                weighted_preference_reward_score: 0,
                leader_penalty_score: 0,
                weighted_leader_penalty_score: 0,
                c_matrix,
                p_matrix,
                solving_time,
            }
        }
        _ => {
            log::error!(target: "employee_job_assignment", "Solver returned an unknown or failed state");
            StaticAssignmentSolution {
                internal_assignments: Vec::new(),
                objective_score: 0,
                preference_reward_score: 0,
                weighted_preference_reward_score: 0,
                leader_penalty_score: 0,
                weighted_leader_penalty_score: 0,
                c_matrix,
                p_matrix,
                solving_time,
            }
        }
    }
}


/// Each unique output from your function. Here, we're just using
/// the internal assignments `(Vec<(String, String)>)` as the key.
/// If you also need to factor in a `score` or something else,
/// make a struct that is `#[derive(Eq, PartialEq, Hash)]`.
pub type OutputKey = Vec<(String, String)>;

/// A rectangle in the 2D input space (omega, alpha).
/// For instance, [omega_min..=omega_max] x [alpha_min..=alpha_max].
#[derive(Debug)]
pub struct Rect2D {
    pub omega_min: u32,
    pub omega_max: u32,
    pub alpha_min: u32,
    pub alpha_max: u32,
}

/// This will hold **all** the rectangles for a particular output.
/// Each rectangle is one connected component of `(omega, alpha)` points.
#[derive(Debug)]
pub struct GroupedRectangles {
    pub rects: Vec<Rect2D>,
}

/// The structure we'll use *internally* to accumulate raw points
/// for each unique output before post-processing.
#[derive(Debug)]
pub struct RawGroup {
    pub points: Vec<(u32, u32)>,
}

/// Your main function that:
/// 1) Iterates over omega in [omega_min..=omega_max],
///    alpha in [alpha_min..=alpha_max].
/// 2) Calls `calculate_static_assignment(...)` to get an output.
/// 3) Collects all points that share that same output in a HashMap.
/// 4) Post-processes each group to find disjoint rectangles (connected components).
pub fn run_and_group(
    station: &Station,
    offset: u32,
    omega_min: u32,
    omega_max: u32,
    alpha_min: u32,
    alpha_max: u32,
) -> HashMap<OutputKey, GroupedRectangles> {
    // 1) Gather raw points for each unique output key
    let mut raw_map: HashMap<OutputKey, RawGroup> = HashMap::new();

    for omega in omega_min..=omega_max {
        for alpha in alpha_min..=alpha_max {
            let solution = calculate_static_assignment(station, offset, omega, alpha);

            // The "key" we use for grouping is the `internal_assignments`.
            // If you also want to factor in `objective_score` or other data,
            // create a custom struct that derives `Eq, Hash`.
            let output_key = solution.internal_assignments;

            let entry = raw_map.entry(output_key).or_insert_with(|| RawGroup {
                points: Vec::new(),
            });

            entry.points.push((omega, alpha));
        }
    }

    // 2) Post-process each group to find disjoint bounding boxes
    //    and build the final `HashMap<OutputKey, GroupedRectangles>`.
    let mut final_map: HashMap<OutputKey, GroupedRectangles> = HashMap::new();

    for (key, raw_group) in raw_map {
        let rects = find_disjoint_bounding_boxes(&raw_group.points);
        final_map.insert(key, GroupedRectangles { rects });
    }

    final_map
}

/// Given a list of points in 2D, find all connected components
/// (4-direction adjacency) and return each component's bounding box.
pub fn find_disjoint_bounding_boxes(points: &[(u32, u32)]) -> Vec<Rect2D> {
    // Put all points in a set for O(1) membership checks
    let set: HashSet<(u32, u32)> = points.iter().copied().collect();

    let mut visited: HashSet<(u32, u32)> = HashSet::new();
    let mut result = Vec::new();

    // For each point, if we haven't visited it yet, BFS to find its entire region
    for &start in &set {
        if visited.contains(&start) {
            continue;
        }

        // We'll track the bounding box for this region
        let (mut min_om, mut max_om) = (start.0, start.0);
        let (mut min_al, mut max_al) = (start.1, start.1);

        // BFS queue
        let mut queue = VecDeque::new();
        queue.push_back(start);
        visited.insert(start);

        while let Some((om, al)) = queue.pop_front() {
            // Update bounding box
            if om < min_om { min_om = om; }
            if om > max_om { max_om = om; }
            if al < min_al { min_al = al; }
            if al > max_al { max_al = al; }

            // Explore neighbors in 4 directions
            for (nom, nal) in neighbors_4(om, al) {
                if set.contains(&(nom, nal)) && !visited.contains(&(nom, nal)) {
                    visited.insert((nom, nal));
                    queue.push_back((nom, nal));
                }
            }
        }

        // One connected component => one bounding box
        result.push(Rect2D {
            omega_min: min_om,
            omega_max: max_om,
            alpha_min: min_al,
            alpha_max: max_al,
        });
    }

    result
}

/// Return the 4-direction neighbors of (omega, alpha),
/// skipping underflow for `u32 = 0`.
pub fn neighbors_4(om: u32, al: u32) -> Vec<(u32, u32)> {
    let mut result = Vec::with_capacity(4);
    
    // (om+1, al)
    result.push((om + 1, al));

    // (om-1, al) if om > 0
    if om > 0 {
        result.push((om - 1, al));
    }

    // (om, al+1)
    result.push((om, al + 1));

    // (om, al-1) if al > 0
    if al > 0 {
        result.push((om, al - 1));
    }

    result
}



#[cfg(test)]
mod tests {

    use std::fs;

    use crate::*;

    #[test]
    fn test_static() -> Result<(), Box<dyn std::error::Error>> {
        let manifest_dir =
            std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is not set");
        let s = "S1";
        // let e = "E0";
        let path = format!("{}/data/{}_matrix.json", manifest_dir, s);

        let json_content = fs::read_to_string(path)?;
        let matrix: Matrix = serde_json::from_str(&json_content)?;

        let offset = 10;
        let omega = 1;
        let alpha = 3;

        if let Some(station) = matrix.stations.get(s) {
            let s = calculate_static_assignment(station, offset, omega, alpha);
            pretty_print_internal_assignments(station, &s.internal_assignments);
            pretty_print_competence_matrix(station);
            pretty_print_preference_matrix(station);
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
                "    Total   : {}(Offs) + {}(Pref) + {}(Lead) = {}",
                offset,
                s.weighted_preference_reward_score,
                s.weighted_leader_penalty_score,
                s.objective_score
            );
            println!();
            println!("=== SOLVER TIME ===");
            println!("    {:?}", s.solving_time);

            let hashmap = run_and_group(station, offset, 1, 10, 1, 10);
            println!("{:?}", hashmap.len());
            for k in hashmap.keys() {
                println!("{:?}", k);
            }
            // println!("{:?}", hashmap.keys());
        }

        

        Ok(())
    }

}
