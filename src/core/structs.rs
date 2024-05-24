use super::enums::Operator;

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

// #[derive(Debug, Clone)]
// pub struct Day {
//     pub date: Date,
//     pub station: String,
//     pub pass_1: Option<Assignment>,
//     pub pass_2: Option<Assignment>,
//     pub pass_3: Option<Assignment>,
//     pub pass_4: Option<Assignment>,
//     pub team_leader: Operator
// }

// #[derive(Debug, Clone)]
// pub struct AnonymousDay {
//     pub date: Date,
//     pub pass_1: Option<AnonymousAssignment>,
//     pub pass_2: Option<AnonymousAssignment>,
//     pub pass_3: Option<AnonymousAssignment>,
//     pub pass_4: Option<AnonymousAssignment>,
//     pub team_leader: 
// }

#[derive(Debug, Clone)]
pub struct Assignment {
    pub bbm: Operator,
    pub drag: Operator,
    pub slap: Operator,
    pub kkm: Operator,
    pub fuse: Operator,
    pub tank: Operator,
    pub batt: Operator,
    pub topp: Operator
}

// pub struct AnonymousAssignment {

// }
