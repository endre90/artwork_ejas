use std::collections::HashMap;

// use crate::*;

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