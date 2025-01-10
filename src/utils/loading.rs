use std::{
    // collections::HashMap,
    fs::{self},
    // io::BufReader,
};

use crate::*;

pub fn list_frames_in_dir(path: &str) -> Result<Vec<String>, Box<dyn std::error::Error + Send>> {
    let mut scenario = vec![];
    match fs::read_dir(path) {
        Ok(dir) => dir.for_each(|file| match file {
            Ok(entry) => match entry.path().to_str() {
                Some(valid) => scenario.push(valid.to_string()),
                None => {
                    log::warn!(target: "employee_job_assignment", "Data path is not valid unicode.")
                }
            },
            Err(e) => log::warn!(target: "employee_job_assignment", "Reading entry failed with '{}'.", e),
        }),
        Err(e) => {
            log::error!(target: "employee_job_assignment",
                "Reading the data directory failed with: '{}'.",
                e
            );
            log::error!(target: "employee_job_assignment", "No data has been loaded.");
            return Err(Box::new(ErrorMsg::new(&format!(
                "Reading the data directory failed with: '{}'. 
                    No data has been loaded.",
                e
            ))));
        }
    }
    Ok(scenario)
}

