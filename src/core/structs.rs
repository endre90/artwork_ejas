use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct Matrix {
    pub stations: std::collections::HashMap<String, Station>,
}

#[derive(Debug, Deserialize)]
pub struct Station {
    pub ergo_score: std::collections::HashMap<String, u8>,
    pub people: Vec<Employee>,
}

#[derive(Debug, Deserialize)]
pub enum Role {
    TeamLeader,
    Operator
}

#[derive(Debug, Deserialize)]
pub struct Employee {
    pub name: String,
    pub role: Role,
    pub competences: Vec<String>,
    pub preferences: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Date {
    pub year: u32,
    pub month: u8,
    pub day: u8,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Day {
    pub date: Date,
    pub station: String,
    pub assignments: Vec<(String, String)>, // (employee, job)
    pub leader: String
}

#[derive(Debug, Clone, Deserialize)]
pub struct Pass {
    pub date: Date,
    pub pass: u8,
    pub station: String,
    pub assignments: Vec<(String, String)>, // (employee, job)
}

#[derive(Debug, Deserialize)]
pub struct DayWrapper {
    pub day: Day,
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
