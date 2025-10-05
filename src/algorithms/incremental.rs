use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

use ast::Ast;
use z3::{
    ast::{Bool, Int},
    *,
};

use crate::*;

pub fn calculate_incremental_assignment(
    station: &Station,
    history: Vec<Day>,
    horizon: u32, // For how many days to plan ahead (the planning horizon (1 means only assignment for today))
    offset: u32,  // Add to objective to get a positive integer result (just for aesthetics)
    omega: u32,   // How strongly preference considerations influence the objective function
    alpha: u32,   // How strongly to discourage leader usage
    beta: u32,    // How strongly to discourage external operator usage
    tau: u32,     // Number of days to consider in the historical data (from tau to today)
    gamma: u32, // how strongly to penalize assigning the same employee–job pair that was frequently assigned in the past tau days
    delta: u32, // Ergonomics weight
    theta: u32, // Weight controlling how the historical count reduces the ergonomics benefit of a job for a given employee.
) -> Vec<ErgonomicAssignmentSolution> {
    let mut schedule = vec![];
    let mut history: VecDeque<Day> = VecDeque::from(history.clone());
    
    for _ in 0..horizon {
        // println!("{:?}", history);
        // println!();
        pretty_print_historical_matrix(station, history
            .clone()
            .iter()
            .map(|x| x.clone())
            .collect::<Vec<Day>>(), tau);
        let next_day = calculate_ergonomic_assignment(
            station,
            history
                .clone()
                .iter()
                .map(|x| x.clone())
                .collect::<Vec<Day>>(),
            offset,
            omega,
            alpha,
            beta,
            tau,
            gamma,
            delta,
            theta,
        );
        history.push_back(Day {
            date: Date {
                year: 2025,
                month: 7,
                day: 9,
            },
            station: "Station1".to_string(),
            assignments: next_day.internal_assignments.clone(),
        });
        schedule.push(next_day);
    }

    schedule
}

#[cfg(test)]
mod tests {

    use std::fs;

    use crate::*;

    #[test]
    fn test_incremental() -> Result<(), Box<dyn std::error::Error>> {
        let manifest_dir =
            std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is not set");
        let s = "S2";
        let e = "E0";
        let path = format!("{}/data/{}_matrix_static.json", manifest_dir, s);

        let history_path = format!("{}/data/{}_{}_history.json", manifest_dir, s, e);
        let history_content = fs::read_to_string(history_path)?;
        let history_wrapper: Vec<DayWrapper> = serde_json::from_str(&history_content)?;
        let history: Vec<Day> = history_wrapper.into_iter().map(|dw| dw.day).collect();

        let json_content = fs::read_to_string(path)?;
        let matrix: Matrix = serde_json::from_str(&json_content)?;

        let horizon = 8;
        let offset = 10;
        let omega = 1;
        let alpha = 50;
        let beta = 100;
        let tau = 10;
        let gamma = 1;
        let theta = 1;
        let delta = 1;

        if let Some(station) = matrix.stations.get(s) {
            let solutions = calculate_incremental_assignment(
                station,
                history.clone(),
                horizon,
                offset,
                omega,
                alpha,
                beta,
                tau,
                gamma,
                delta,
                theta,
            );
            for s in solutions {
                pretty_print_internal_assignments(station, &s.internal_assignments);
                pretty_print_external_assignments(station, &s.external_assignments);
                // pretty_print_competence_matrix(station);
                // pretty_print_preference_matrix(station);
                // Because it is incrementa, this is now updated and printed in every step
                // pretty_print_historical_matrix(station, history.clone(), tau);
                // println!("=== SCORING ===");
                // println!("    Offs    : {}", offset);
                // println!(
                //     "    Pref    : {}(omega) x {} = {}",
                //     omega, s.preference_reward_score, s.weighted_preference_reward_score
                // );
                // println!(
                //     "    Lead    : {}(alpha) x {} = {}",
                //     alpha, s.leader_penalty_score, s.weighted_leader_penalty_score
                // );
                // println!(
                //     "    Exte    : {}(beta) x {} = {}",
                //     beta, s.external_penalty_score, s.weighted_external_penalty_score
                // );
                // println!(
                //     "    Hist    : {}(gamma) x {} = {}",
                //     gamma, s.historical_penalty_score, s.weighted_historical_penalty_score
                // );
                // println!(
                //     "    Ergo    : {}(delta) x {} = {}",
                //     delta, s.ergonomics_reward_score, s.weighted_ergonomics_reward_score
                // );
                // println!(
                //     "    Total   : {}(Offs) + {}(Pref) - {}(Lead) - {}(Exte) - {}(Hist) + {}(Ergo)= {}",
                //     offset,
                //     s.weighted_preference_reward_score,
                //     s.weighted_leader_penalty_score,
                //     s.weighted_external_penalty_score,
                //     s.weighted_historical_penalty_score,
                //     s.weighted_ergonomics_reward_score,
                //     s.objective_score
                // );
                // println!();
                // println!("=== SOLVER TIME ===");
                // println!("    {:?}", s.solving_time);
            }
        }

        Ok(())
    }
}
