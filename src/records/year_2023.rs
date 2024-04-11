use crate::*;

pub fn records() -> Vec<Day> {
    let mut records = vec![];
    records.push(Day {
        date: crate::Date {
            year: 2023,
            month: 8,
            day: 29,
        },
        station: "station_4".to_string(),
        pass_1: Some(Assignment {
            bbm: Operator::Gio,
            drag: Operator::Tasha,
            slap: Operator::Sarah,
            kkm: Operator::Jessica,
            fuse: Operator::Alice,
            tank: Operator::Patrik,
            batt: Operator::Agnes,
            topp: Operator::Vanessa,
        }),
        pass_2: Some(Assignment {
            bbm: Operator::Tasha,
            drag: Operator::Sarah,
            slap: Operator::Gio,
            kkm: Operator::Alice,
            fuse: Operator::Jessica,
            tank: Operator::Kenny,
            batt: Operator::Agnes,
            topp: Operator::Vanessa,
        }),
        pass_3: None,
        pass_4: None,
        team_leader: Operator::Patrik
    });
    records
}
