use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Matrix {
    pub stations: std::collections::HashMap<String, Station>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Station {
    pub ergo_score: std::collections::HashMap<String, u8>,
    pub people: Vec<Employee>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Role {
    TeamLeader,
    Operator,
}

impl Station {
    /// Stamp today's team leader onto the roster's roles.
    ///
    /// Who leads changes from day to day, so it is chosen per solve rather than
    /// stored on the roster; a role carried by a loaded matrix file is only a
    /// stale default. The solvers all read the leader off `people`, so the
    /// daily choice has to be written here to have any effect.
    ///
    /// Normalises unconditionally, so exactly the named person holds the role
    /// and nobody does when `leader` is `None` or unknown - rather than
    /// quietly falling back to whatever the file designated.
    pub fn set_todays_leader(&mut self, leader: Option<&str>) {
        for person in &mut self.people {
            person.role = match leader {
                Some(name) if person.name == name => Role::TeamLeader,
                _ => Role::Operator,
            };
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Employee {
    pub name: String,
    pub role: Role,
    pub competences: Vec<String>,
    pub preferences: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Date {
    pub year: u32,
    pub month: u8,
    pub day: u8,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Day {
    pub date: Date,
    pub station: String,
    pub assignments: Vec<(String, String)>, // (employee, job)
    pub leader: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pass {
    pub date: Date,
    pub pass: u8,
    pub station: String,
    pub assignments: Vec<(String, String)>, // (employee, job)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DayWrapper {
    pub day: Day,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PassWrapper {
    pub pass: Pass,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompetenceEntry {
    pub employee: String,
    pub competences: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PreferenceEntry {
    pub employee: String,
    pub preferences: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StaticAssignmentData {
    pub employees: Vec<String>,
    pub jobs: Vec<String>,
    pub competence_map: Vec<CompetenceEntry>,
    pub preference_map: Vec<PreferenceEntry>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn station(names: &[&str]) -> Station {
        Station {
            ergo_score: std::collections::HashMap::new(),
            people: names
                .iter()
                .map(|n| Employee {
                    name: (*n).to_owned(),
                    // Everyone starts as the leader, so a test that ends with
                    // one leader proves the others were actively cleared.
                    role: Role::TeamLeader,
                    competences: Vec::new(),
                    preferences: Vec::new(),
                })
                .collect(),
        }
    }

    fn leaders(station: &Station) -> Vec<&str> {
        station
            .people
            .iter()
            .filter(|e| e.role == Role::TeamLeader)
            .map(|e| e.name.as_str())
            .collect()
    }

    #[test]
    fn names_exactly_one_leader() {
        let mut s = station(&["ada", "bo", "cy"]);
        s.set_todays_leader(Some("bo"));
        assert_eq!(leaders(&s), ["bo"]);
    }

    #[test]
    fn clears_a_stale_role_from_the_roster_file() {
        // The whole point: yesterday's leader must not keep the role.
        let mut s = station(&["ada", "bo"]);
        s.set_todays_leader(Some("ada"));
        s.set_todays_leader(Some("bo"));
        assert_eq!(leaders(&s), ["bo"]);
    }

    #[test]
    fn nobody_leads_when_unset_or_unknown() {
        let mut s = station(&["ada", "bo"]);
        s.set_todays_leader(None);
        assert!(leaders(&s).is_empty());

        let mut s = station(&["ada", "bo"]);
        s.set_todays_leader(Some("nobody-by-that-name"));
        assert!(leaders(&s).is_empty());
    }
}
