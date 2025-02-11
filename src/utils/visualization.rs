use std::{
    cmp::{max, min},
    collections::{HashMap, HashSet, VecDeque},
    time::{Duration, Instant},
};

use crate::*;
use serde::{Deserialize, Serialize};

/// Example of what your assignment function might return.
pub struct Solution {
    pub internal_assignments: Vec<(String, String)>,
    pub objective_score: i64,
}

/// If your real key is more complex than a `Vec<(String, String)>`,
/// define a struct that implements `Ord` + `Eq` + `Hash` + `Serialize`.
/// Here we demonstrate just `Vec<(String, String)>` as the "OutputKey".
pub type OutputKey = Vec<(String, String)>;

// -------------------- Deterministic Grouping Code --------------------

/// An item we will serialize to JSON for Python:
/// - `key_id`: A short label like "K1" or "Z2"
/// - `output_key`: The full data for the key (if you want to store it)
/// - `points`: All (omega, alpha) pairs that produce this key
#[derive(Debug, Serialize, Deserialize)]
pub struct JsonOutputItemPointsStatic {
    pub key_id: String,
    pub output_key: OutputKey,
    pub points: Vec<(u32, u32)>,
}

/// An item we will serialize to JSON for Python:
/// - `key_id`: A short label like "K1" or "Z2"
/// - `output_key`: The full data for the key (if you want to store it)
/// - `points`: All (omega, alpha, beta) tuples that produce this key
#[derive(Debug, Serialize, Deserialize)]
pub struct JsonOutputItemPointsExternal {
    pub key_id: String,
    pub output_key: OutputKey,
    pub points: Vec<(u32, u32, u32)>,
}

/// An item we will serialize to JSON for Python:
/// - `key_id`: A short label like "K1" or "Z2"
/// - `output_key`: The full data for the key (if you want to store it)
/// - `points`: All (omega, alpha, beta) tuples that produce this key
#[derive(Debug, Serialize, Deserialize)]
pub struct JsonOutputItemPointsHistoric {
    pub key_id: String,
    pub output_key: OutputKey,
    pub points: Vec<(u32, u32, u32, u32)>,
}

/// An item we will serialize to JSON for Python:
/// - `key_id`: A short label like "K1" or "Z2"
/// - `output_key`: The full data for the key (if you want to store it)
/// - `points`: All (omega, alpha, beta) tuples that produce this key
#[derive(Debug, Serialize, Deserialize)]
pub struct JsonOutputItemPointsErgonomic {
    pub key_id: String,
    pub output_key: OutputKey,
    pub points: Vec<(u32, u32, u32, u32, u32, u32)>,
}

pub fn write_to_file_points_static(
    solutions: &Vec<(OutputKey, Vec<(u32, u32)>)>,
) -> std::io::Result<()> {
    let mut json_items: Vec<JsonOutputItemPointsStatic> = Vec::new();

    for (i, (key, points)) in solutions.into_iter().enumerate() {
        let key_id = format!("K{}", i + 1);
        let item = JsonOutputItemPointsStatic {
            key_id,
            output_key: key.clone(),
            points: points.clone(),
        };
        json_items.push(item);
    }

    // Serialize to JSON
    let json_str = serde_json::to_string_pretty(&json_items).expect("Failed to serialize to JSON");

    // Write to file
    std::fs::write("/home/endre/Desktop/points_static.json", json_str)?;

    Ok(())
}

pub fn write_to_file_points_external(
    solutions: &Vec<(OutputKey, Vec<(u32, u32, u32)>)>,
) -> std::io::Result<()> {
    let mut json_items: Vec<JsonOutputItemPointsExternal> = Vec::new();

    for (i, (key, points)) in solutions.into_iter().enumerate() {
        let key_id = format!("K{}", i + 1);
        let item = JsonOutputItemPointsExternal {
            key_id,
            output_key: key.clone(),
            points: points.clone(),
        };
        json_items.push(item);
    }

    // Serialize to JSON
    let json_str = serde_json::to_string_pretty(&json_items).expect("Failed to serialize to JSON");

    // Write to file
    std::fs::write("/home/endre/Desktop/points_external.json", json_str)?;

    Ok(())
}

pub fn write_to_file_points_historic(
    solutions: &Vec<(OutputKey, Vec<(u32, u32, u32, u32)>)>,
) -> std::io::Result<()> {
    let mut json_items: Vec<JsonOutputItemPointsHistoric> = Vec::new();

    for (i, (key, points)) in solutions.into_iter().enumerate() {
        let key_id = format!("K{}", i + 1);
        let item = JsonOutputItemPointsHistoric {
            key_id,
            output_key: key.clone(),
            points: points.clone(),
        };
        json_items.push(item);
    }

    // Serialize to JSON
    let json_str = serde_json::to_string_pretty(&json_items).expect("Failed to serialize to JSON");

    // Write to file
    std::fs::write("/home/endre/Desktop/points_historic.json", json_str)?;

    Ok(())
}

pub fn write_to_file_points_ergonomic(
    solutions: &Vec<(OutputKey, Vec<(u32, u32, u32, u32, u32, u32)>)>,
) -> std::io::Result<()> {
    let mut json_items: Vec<JsonOutputItemPointsErgonomic> = Vec::new();

    for (i, (key, points)) in solutions.into_iter().enumerate() {
        let key_id = format!("K{}", i + 1);
        let item = JsonOutputItemPointsErgonomic {
            key_id,
            output_key: key.clone(),
            points: points.clone(),
        };
        json_items.push(item);
    }

    // Serialize to JSON
    let json_str = serde_json::to_string_pretty(&json_items).expect("Failed to serialize to JSON");

    // Write to file
    std::fs::write("/home/endre/Desktop/points_ergonomic.json", json_str)?;

    Ok(())
}

/// Gathers all (omega, alpha) → OutputKey in a stable order,
/// groups them by OutputKey, and returns a Vec of (OutputKey, Vec of points).
pub fn run_and_group_points_static(
    station: &Station,
    offset: u32,
    omega_min: u32,
    omega_max: u32,
    alpha_min: u32,
    alpha_max: u32,
) -> Vec<(OutputKey, Vec<(u32, u32)>)> {
    let mut all_data: Vec<(OutputKey, u32, u32)> = Vec::new();

    // 1) Collect
    for omega in omega_min..=omega_max {
        for alpha in alpha_min..=alpha_max {
            let sol = calculate_static_assignment(station, offset, omega, alpha);

            // Sort the pairs if needed to ensure stable ordering
            let mut key = sol.internal_assignments;
            key.sort();

            all_data.push((key, omega, alpha));
        }
    }

    // 2) Sort by (key, then omega, then alpha)
    all_data.sort_by(|(k1, o1, a1), (k2, o2, a2)| match k1.cmp(k2) {
        std::cmp::Ordering::Equal => match o1.cmp(o2) {
            std::cmp::Ordering::Equal => a1.cmp(a2),
            other => other,
        },
        other => other,
    });

    // 3) Group
    let mut result = Vec::new();
    let mut current_key: Option<OutputKey> = None;
    let mut current_points: Vec<(u32, u32)> = Vec::new();

    for (key, om, al) in all_data {
        match &mut current_key {
            None => {
                current_key = Some(key);
                current_points.clear();
                current_points.push((om, al));
            }
            Some(k) => {
                if *k == key {
                    current_points.push((om, al));
                } else {
                    // flush old group
                    let old_key = std::mem::take(k);
                    result.push((old_key, current_points.clone()));

                    // start new group
                    *k = key;
                    current_points.clear();
                    current_points.push((om, al));
                }
            }
        }
    }
    // flush final group
    if let Some(k) = current_key {
        result.push((k, current_points.clone()));
    }

    result
}

/// Gathers all (omega, alpha, beta) → OutputKey in a stable order,
/// groups them by OutputKey, and returns a Vec of (OutputKey, Vec of points).
pub fn run_and_group_points_external(
    station: &Station,
    offset: u32,
    omega_min: u32,
    omega_max: u32,
    alpha_min: u32,
    alpha_max: u32,
    beta_min: u32,
    beta_max: u32,
) -> Vec<(OutputKey, Vec<(u32, u32, u32)>)> {
    let mut all_data: Vec<(OutputKey, u32, u32, u32)> = Vec::new();

    // 1) Collect
    for omega in omega_min..=omega_max {
        for alpha in alpha_min..=alpha_max {
            for beta in beta_min..=beta_max {
                let sol = calculate_external_assignment(station, offset, omega, alpha, beta);

                // Sort the pairs if needed to ensure stable ordering
                let mut key = sol.internal_assignments;
                key.sort();

                all_data.push((key, omega, alpha, beta));
            }
        }
    }

    // Sort by (key, then omega, then alpha, then beta).
    all_data.sort_by(|(k1, o1, a1, b1), (k2, o2, a2, b2)| match k1.cmp(k2) {
        std::cmp::Ordering::Equal => match o1.cmp(o2) {
            std::cmp::Ordering::Equal => match a1.cmp(a2) {
                std::cmp::Ordering::Equal => b1.cmp(b2),
                other => other,
            },
            other => other,
        },
        other => other,
    });

    // 3) Group
    let mut result = Vec::new();
    let mut current_key: Option<OutputKey> = None;
    let mut current_points: Vec<(u32, u32, u32)> = Vec::new();

    for (key, om, al, be) in all_data {
        match &mut current_key {
            None => {
                current_key = Some(key);
                current_points.clear();
                current_points.push((om, al, be));
            }
            Some(k) => {
                if *k == key {
                    current_points.push((om, al, be));
                } else {
                    // flush old group
                    let old_key = std::mem::take(k);
                    result.push((old_key, current_points.clone()));

                    // start new group
                    *k = key;
                    current_points.clear();
                    current_points.push((om, al, be));
                }
            }
        }
    }
    // flush final group
    if let Some(k) = current_key {
        result.push((k, current_points.clone()));
    }

    result
}

/// Gathers all (omega, alpha, beta, gamma) → OutputKey in a stable order,
/// groups them by OutputKey, and returns a Vec of (OutputKey, Vec of points).
pub fn run_and_group_points_historic(
    station: &Station,
    history: Vec<Day>,
    offset: u32,
    tau: u32,
    omega_min: u32,
    omega_max: u32,
    alpha_min: u32,
    alpha_max: u32,
    beta_min: u32,
    beta_max: u32,
    gamma_min: u32,
    gamma_max: u32,
) -> Vec<(OutputKey, Vec<(u32, u32, u32, u32)>)> {
    let mut all_data: Vec<(OutputKey, u32, u32, u32, u32)> = Vec::new();

    // 1) Collect
    for omega in omega_min..=omega_max {
        for alpha in alpha_min..=alpha_max {
            // if alpha == 0 || alpha == 1 || alpha == 5 || alpha == 10 {
                for beta in beta_min..=beta_max {
                    for gamma in gamma_min..=gamma_max {
                        let sol = calculate_historic_assignment(
                            station,
                            history.clone(),
                            offset,
                            omega * 5,
                            alpha * 5,
                            beta,
                            tau,
                            gamma,
                        );

                        // Sort the pairs if needed to ensure stable ordering
                        let mut key = sol.internal_assignments;
                        key.sort();

                        all_data.push((key, omega, alpha, beta, gamma));
                    }
                }
            // }
        }
    }

    // Sort by (key, then omega, then alpha, then beta).
    all_data.sort_by(
        |(k1, o1, a1, b1, d1), (k2, o2, a2, b2, d2)| match k1.cmp(k2) {
            std::cmp::Ordering::Equal => match o1.cmp(o2) {
                std::cmp::Ordering::Equal => match a1.cmp(a2) {
                    std::cmp::Ordering::Equal => match b1.cmp(b2) {
                        std::cmp::Ordering::Equal => d1.cmp(d2),
                        other => other,
                    },
                    other => other,
                },
                other => other,
            },
            other => other,
        },
    );

    // 3) Group
    let mut result = Vec::new();
    let mut current_key: Option<OutputKey> = None;
    let mut current_points: Vec<(u32, u32, u32, u32)> = Vec::new();

    for (key, om, al, be, ga) in all_data {
        match &mut current_key {
            None => {
                current_key = Some(key);
                current_points.clear();
                current_points.push((om, al, be, ga));
            }
            Some(k) => {
                if *k == key {
                    current_points.push((om, al, be, ga));
                } else {
                    // flush old group
                    let old_key = std::mem::take(k);
                    result.push((old_key, current_points.clone()));

                    // start new group
                    *k = key;
                    current_points.clear();
                    current_points.push((om, al, be, ga));
                }
            }
        }
    }
    // flush final group
    if let Some(k) = current_key {
        result.push((k, current_points.clone()));
    }

    result
}

/// Gathers all (omega, alpha, beta, gamma, delta) → OutputKey in a stable order,
/// groups them by OutputKey, and returns a Vec of (OutputKey, Vec of points).
pub fn run_and_group_points_ergonomic(
    station: &Station,
    history: Vec<Day>,
    offset: u32,
    tau: u32,
    omega_min: u32,
    omega_max: u32,
    alpha_min: u32,
    alpha_max: u32,
    beta_min: u32,
    beta_max: u32,
    gamma_min: u32,
    gamma_max: u32,
    delta_min: u32,
    delta_max: u32,
    theta_min: u32,
    theta_max: u32,
) -> Vec<(OutputKey, Vec<(u32, u32, u32, u32, u32, u32)>)> {
    let mut all_data: Vec<(OutputKey, u32, u32, u32, u32, u32, u32)> = Vec::new();

    // 1) Collect
    for omega in omega_min..=omega_max {
        for alpha in alpha_min..=alpha_max {
            for beta in beta_min..=beta_max {
                for gamma in gamma_min..=gamma_max {
                    for delta in delta_min..=delta_max {
                        for theta in theta_min..=theta_max {
                            let sol = calculate_ergonomic_assignment(
                                station,
                                history.clone(),
                                offset,
                                omega * 10,
                                alpha,
                                beta,
                                tau,
                                gamma * 4,
                                delta * 4,
                                theta,
                            );

                            // Sort the pairs if needed to ensure stable ordering
                            let mut key = sol.internal_assignments;
                            key.sort();

                            all_data.push((key, omega, alpha, beta, gamma, delta, theta));
                        }
                    }
                }
            }
        }
    }

    // Sort by (key, then omega, then alpha, then beta).
    all_data.sort_by(
        |(k1, o1, a1, b1, g1, d1, t1), (k2, o2, a2, b2, g2, d2, t2)| match k1.cmp(k2) {
            std::cmp::Ordering::Equal => match o1.cmp(o2) {
                std::cmp::Ordering::Equal => match a1.cmp(a2) {
                    std::cmp::Ordering::Equal => match b1.cmp(b2) {
                        std::cmp::Ordering::Equal => match g1.cmp(g2) {
                            std::cmp::Ordering::Equal => match d1.cmp(d2) {
                                std::cmp::Ordering::Equal => t1.cmp(t2),
                                other => other,
                            },
                            other => other,
                        },
                        other => other,
                    },
                    other => other,
                },
                other => other,
            },
            other => other,
        },
    );

    // 3) Group
    let mut result = Vec::new();
    let mut current_key: Option<OutputKey> = None;
    let mut current_points: Vec<(u32, u32, u32, u32, u32, u32)> = Vec::new();

    for (key, om, al, be, ga, de, th) in all_data {
        match &mut current_key {
            None => {
                current_key = Some(key);
                current_points.clear();
                current_points.push((om, al, be, ga, de, th));
            }
            Some(k) => {
                if *k == key {
                    current_points.push((om, al, be, ga, de, th));
                } else {
                    // flush old group
                    let old_key = std::mem::take(k);
                    result.push((old_key, current_points.clone()));

                    // start new group
                    *k = key;
                    current_points.clear();
                    current_points.push((om, al, be, ga, de, th));
                }
            }
        }
    }
    // flush final group
    if let Some(k) = current_key {
        result.push((k, current_points.clone()));
    }

    result
}
