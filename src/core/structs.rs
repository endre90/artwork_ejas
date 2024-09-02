use serde::{Serialize, Deserialize};

#[derive(Debug, Clone)]
pub struct Date {
    pub year: u32,
    pub month: u8,
    pub day: u8,
}

#[derive(Debug, Clone)]
pub struct Day {
    pub date: Date,
    pub station: String,
    pub assignments: Vec<(String, String)> // (employee, job)
}

#[derive(Serialize, Deserialize)]
pub struct CompetenceEntry {
    pub employee: String,
    pub competences: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct PreferenceEntry {
    pub employee: String,
    pub preferences: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct StaticAssignmentData {
    pub employees: Vec<String>,
    pub jobs: Vec<String>,
    pub competence_map: Vec<CompetenceEntry>,
    pub preference_map: Vec<PreferenceEntry>,
}