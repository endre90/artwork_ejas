use std::{cmp::{max, min}, collections::{HashMap, HashSet, VecDeque}, time::{Duration, Instant}};

use serde::{Deserialize, Serialize};
use crate::*;

/// Represents one rectangle in (omega, alpha) space.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rect2D {
    pub omega_min: u32,
    pub omega_max: u32,
    pub alpha_min: u32,
    pub alpha_max: u32,
}

/// Holds all rectangles (disjoint regions) for one particular output key.
#[derive(Debug, Serialize, Deserialize)]
pub struct GroupedRectangles {
    pub rects: Vec<Rect2D>,
}

/// Dummy "Station" type; replace with your real type
// pub struct Station;

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

/// We'll store final data for JSON as a list of these items.
/// The user can then pick a short "key_id" to name each unique output
/// rather than the full text, but we still store the full "output_key" if desired.
#[derive(Debug, Serialize, Deserialize)]
pub struct JsonOutputItem {
    pub key_id: String,
    pub output_key: OutputKey,
    pub rects: Vec<Rect2D>,
}

/// An item we will serialize to JSON for Python:
/// - `key_id`: A short label like "K1" or "Z2"
/// - `output_key`: The full data for the key (if you want to store it)
/// - `points`: All (omega, alpha) pairs that produce this key
#[derive(Debug, Serialize, Deserialize)]
pub struct JsonOutputItemPoints {
    pub key_id: String,
    pub output_key: OutputKey,
    pub points: Vec<(u32, u32)>,
}

/// The main function to get deterministic results, returning a vector of
/// `(OutputKey, GroupedRectangles)` **in sorted order**.
pub fn run_and_group(
    station: &Station,
    offset: u32,
    omega_min: u32,
    omega_max: u32,
    alpha_min: u32,
    alpha_max: u32,
) -> Vec<(OutputKey, GroupedRectangles)> {
    // 1) Collect all (OutputKey, omega, alpha) into a Vec
    let mut all_data: Vec<(OutputKey, u32, u32)> = Vec::new();

    for omega in omega_min..=omega_max {
        for alpha in alpha_min..=alpha_max {
            let solution = calculate_static_assignment(station, offset, omega, alpha);

            // If you need stable ordering inside the OutputKey,
            // make sure to sort the pairs themselves.
            let mut key = solution.internal_assignments;
            key.sort(); // ensures consistent ordering of (String, String)

            all_data.push((key, omega, alpha));
        }
    }

    // 2) Sort by (OutputKey, then omega, then alpha).
    all_data.sort_by(|(k1, o1, a1), (k2, o2, a2)| {
        match k1.cmp(k2) {
            std::cmp::Ordering::Equal => match o1.cmp(o2) {
                std::cmp::Ordering::Equal => a1.cmp(a2),
                other => other,
            },
            other => other,
        }
    });

    // 3) Group by OutputKey and find disjoint rectangles for each
    let mut result: Vec<(OutputKey, GroupedRectangles)> = Vec::new();

    // Helper to flush each group
    fn push_group(
        result: &mut Vec<(OutputKey, GroupedRectangles)>,
        current_key: OutputKey,
        current_points: &[(u32, u32)],
    ) {
        let rects = find_disjoint_bounding_boxes(current_points);
        result.push((current_key, GroupedRectangles { rects }));
    }

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
                    // flush the old group
                    let old_key = std::mem::take(k);
                    push_group(&mut result, old_key, &current_points);

                    // start the new group
                    *k = key;
                    current_points.clear();
                    current_points.push((om, al));
                }
            }
        }
    }
    // flush final group
    if let Some(k) = current_key {
        push_group(&mut result, k, &current_points);
    }

    // Sort final bounding boxes for consistency (optional)
    for (_, group) in &mut result {
        group.rects.sort_by(|r1, r2| {
            (r1.omega_min, r1.alpha_min).cmp(&(r2.omega_min, r2.alpha_min))
        });
    }

    result
}

/// Finds disjoint connected components among the given points
/// in 2D (omega, alpha), returning one bounding box per component.
fn find_disjoint_bounding_boxes(points: &[(u32, u32)]) -> Vec<Rect2D> {
    use std::collections::BTreeSet;
    let btree: BTreeSet<(u32, u32)> = points.iter().copied().collect();
    let mut visited = BTreeSet::new();
    let mut rects = Vec::new();

    for &start in &btree {
        if visited.contains(&start) {
            continue;
        }
        // BFS from start
        let mut queue = VecDeque::new();
        queue.push_back(start);
        visited.insert(start);

        let (mut om_min, mut om_max) = (start.0, start.0);
        let (mut al_min, mut al_max) = (start.1, start.1);

        while let Some((om, al)) = queue.pop_front() {
            om_min = min(om_min, om);
            om_max = max(om_max, om);
            al_min = min(al_min, al);
            al_max = max(al_max, al);

            for neighbor in neighbors_4(om, al) {
                if btree.contains(&neighbor) && !visited.contains(&neighbor) {
                    visited.insert(neighbor);
                    queue.push_back(neighbor);
                }
            }
        }

        rects.push(Rect2D {
            omega_min: om_min,
            omega_max: om_max,
            alpha_min: al_min,
            alpha_max: al_max,
        });
    }

    rects
}

/// Return neighbors in a stable order (for BFS).
fn neighbors_4(om: u32, al: u32) -> Vec<(u32, u32)> {
    let mut v = Vec::new();
    // For example, up, left, right, down:
    if al > 0 {
        v.push((om, al - 1)); // up
    }
    if om > 0 {
        v.push((om - 1, al)); // left
    }
    v.push((om + 1, al));    // right
    v.push((om, al + 1));    // down
    v
}

// Write to a file
// std::fs::write("zones_points.json", json_str)?;

pub fn write_to_file(solutions: &Vec<(OutputKey, GroupedRectangles)>) -> std::io::Result<()> {
    // let station = Station;
    // collect data
    // let results = run_and_group(&station, 0, 0, 5, 0, 5);

    // Build a list of JsonOutputItem, giving each output a short name.
    let mut output_items = Vec::new();
    for (i, (output_key, grouped_rects)) in solutions.into_iter().enumerate() {
        let key_id = format!("K{}", i + 1); // or "ZONE-1", "ZONE-2", etc.
        let item = JsonOutputItem {
            key_id,
            output_key: output_key.to_owned(),
            rects: grouped_rects.rects.clone(),
        };
        output_items.push(item);
    }

    // Convert to pretty-printed JSON
    let json_str = serde_json::to_string_pretty(&output_items)
        .expect("Failed to serialize to JSON");

    // Write to file
    std::fs::write("/home/endre/Desktop/zones.json", json_str)?;

    Ok(())
}

pub fn write_to_file_points(solutions: &Vec<(OutputKey, Vec<(u32, u32)>)>) -> std::io::Result<()> {
    let mut json_items: Vec<JsonOutputItemPoints> = Vec::new();

for (i, (key, points)) in solutions.into_iter().enumerate() {
    let key_id = format!("K{}", i + 1);
    let item = JsonOutputItemPoints {
        key_id,
        output_key: key.clone(),
        points: points.clone(),
    };
    json_items.push(item);
}

// Serialize to JSON
let json_str = serde_json::to_string_pretty(&json_items)
    .expect("Failed to serialize to JSON");

    // Write to file
    std::fs::write("/home/endre/Desktop/points.json", json_str)?;

    Ok(())
}

/// Gathers all (omega, alpha) → OutputKey in a stable order,
/// groups them by OutputKey, and returns a Vec of (OutputKey, Vec of points).
pub fn run_and_group_points(
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
    all_data.sort_by(|(k1, o1, a1), (k2, o2, a2)| {
        match k1.cmp(k2) {
            std::cmp::Ordering::Equal => match o1.cmp(o2) {
                std::cmp::Ordering::Equal => a1.cmp(a2),
                other => other,
            },
            other => other,
        }
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


