use artwork_ejas::*;
use inquire::ui::{Attributes, Color, RenderConfig, StyleSheet, Styled};
use inquire::Select;
use std::time::Duration;
use std::{fs, thread};
use rusqlite::{params, Connection, Result};
use inquire::{Text, Confirm, MultiSelect};

fn main() -> Result<()> {
    // Connect to the SQLite database (or create it if it doesn't exist)
    let conn = Connection::open("people.db")?;

    // Create the 'people' table if it doesn't exist
    conn.execute(
        "CREATE TABLE IF NOT EXISTS people (
                  id INTEGER PRIMARY KEY,
                  name TEXT NOT NULL,
                  job TEXT NOT NULL
                  )",
        [],
    )?;

    // Create the 'competences' table if it doesn't exist
    conn.execute(
        "CREATE TABLE IF NOT EXISTS competences (
                  id INTEGER PRIMARY KEY,
                  name TEXT NOT NULL UNIQUE
                  )",
        [],
    )?;

    // Create the 'person_competences' table to link people with competences
    conn.execute(
        "CREATE TABLE IF NOT EXISTS person_competences (
                  person_id INTEGER,
                  competence_id INTEGER,
                  FOREIGN KEY(person_id) REFERENCES people(id),
                  FOREIGN KEY(competence_id) REFERENCES competences(id),
                  PRIMARY KEY(person_id, competence_id)
                  )",
        [],
    )?;

    // Predefined list of competences
    let competences = vec![
        "Programming",
        "Project Management",
        "Design",
        "Data Analysis",
        "Communication",
        "Leadership",
    ];

    // Insert predefined competences into the 'competences' table
    for competence in &competences {
        conn.execute(
            "INSERT OR IGNORE INTO competences (name) VALUES (?1)",
            params![competence],
        )?;
    }

    loop {
        // Use `inquire` to ask the user for a name and job
        let name = Text::new("Enter the person's name:")
            .prompt()
            .unwrap();
        let job = Text::new("Enter the person's job:")
            .prompt()
            .unwrap();

        // Insert the data into the 'people' table
        conn.execute(
            "INSERT INTO people (name, job) VALUES (?1, ?2)",
            params![name, job],
        )?;

        // Get the ID of the newly inserted person
        let person_id: i32 = conn.last_insert_rowid() as i32;

        // Use `inquire` to select competences from the predefined list
        let selected_competences = MultiSelect::new(
            "Select competences (use space to select, enter to confirm):",
            competences.clone(),
        )
        .prompt()
        .unwrap();

        // Insert the selected competences into the junction table
        for competence in selected_competences {
            let competence_id: i32 = conn.query_row(
                "SELECT id FROM competences WHERE name = ?1",
                params![competence],
                |row| row.get(0),
            )?;
            conn.execute(
                "INSERT INTO person_competences (person_id, competence_id) VALUES (?1, ?2)",
                params![person_id, competence_id],
            )?;
        }

        // Ask if the user wants to add another person
        let add_another = Confirm::new("Do you want to add another person?")
            .prompt()
            .unwrap();

        if !add_another {
            break;
        }
    }

    // Query and display all rows in the 'people' table with their competences
    let mut stmt = conn.prepare(
        "SELECT people.id, people.name, people.job, GROUP_CONCAT(competences.name, ', ') as competences
         FROM people
         LEFT JOIN person_competences ON people.id = person_competences.person_id
         LEFT JOIN competences ON person_competences.competence_id = competences.id
         GROUP BY people.id, people.name, people.job",
    )?;
    let person_iter = stmt.query_map([], |row| {
        Ok(PersonWithCompetences {
            id: row.get(0)?,
            name: row.get(1)?,
            job: row.get(2)?,
            competences: row.get(3)?,
        })
    })?;

    println!("People stored in the database with their competences:");
    for person in person_iter {
        println!("{:?}", person?);
    }

    Ok(())
}

// Struct to hold person data with competences
#[derive(Debug)]
struct PersonWithCompetences {
    id: i32,
    name: String,
    job: String,
    competences: Option<String>,  // Competences as a comma-separated string
}


// fn main() -> Result<()> {
//     // Connect to the SQLite database (or create it if it doesn't exist)
//     let conn = Connection::open("people.db")?;

//     // Create the 'people' table if it doesn't exist
//     conn.execute(
//         "CREATE TABLE IF NOT EXISTS people (
//                   id INTEGER PRIMARY KEY,
//                   name TEXT NOT NULL,
//                   job TEXT NOT NULL
//                   )",
//         [],
//     )?;

//     loop {
//         // Use `inquire` to ask the user for a name and job
//         let name = Text::new("Enter the person's name:")
//             .prompt()
//             .unwrap();
//         let job = Text::new("Enter the person's job:")
//             .prompt()
//             .unwrap();

//         // Insert the data into the 'people' table
//         conn.execute(
//             "INSERT INTO people (name, job) VALUES (?1, ?2)",
//             params![name, job],
//         )?;

//         // Ask if the user wants to add another person
//         let add_another = Confirm::new("Do you want to add another person?")
//             .prompt()
//             .unwrap();

//         if !add_another {
//             break;
//         }
//     }

//     // Query and display all rows in the 'people' table
//     let mut stmt = conn.prepare("SELECT id, name, job FROM people")?;
//     let person_iter = stmt.query_map([], |row| {
//         Ok(Person {
//             id: row.get(0)?,
//             name: row.get(1)?,
//             job: row.get(2)?,
//         })
//     })?;

//     println!("People stored in the database:");
//     for person in person_iter {
//         println!("{:?}", person?);
//     }

//     Ok(())
// }

// // Struct to hold person data
// #[derive(Debug)]
// struct Person {
//     id: i32,
//     name: String,
//     job: String,
// }




// fn main() {
//     println!("");

//     let algorithms = vec!["Static", "External", "Horizon", "Historic"];
//     let yes_no = vec!["Yes", "No"];

//     let default: RenderConfig = RenderConfig::empty();
//     let prompt_prefix = Styled::new("?")
//         .with_fg(Color::DarkGreen)
//         .with_attr(Attributes::BOLD);
//     let prompt_choice = Styled::new(" ->")
//         .with_fg(Color::DarkGreen)
//         .with_attr(Attributes::BOLD);
//     let mine = default.with_prompt_prefix(prompt_prefix);
//     let mine = mine.with_highlighted_option_prefix(prompt_choice);
//     let new_style_sheet = StyleSheet::empty();
//     let mine = mine.with_answer(
//         new_style_sheet
//             .with_fg(Color::DarkGreen)
//             .with_attr(Attributes::BOLD),
//     );

//     let selected_algorithm = Select::new("Select the algorithm:", algorithms.iter().map(|opt| opt.to_string()).collect())
//         .with_page_size(4)
//         .with_help_message("Use arrow keys to select an option, press Enter to select.] \n[For information about algorithms, look in the README.")
//         .with_render_config(mine)
//         .prompt()
//         .unwrap();

//     // let selected_anonymize = Select::new(
//     //     "Do you want to anonymize data?",
//     //     yes_no.iter().map(|opt| opt.to_string()).collect(),
//     // )
//     // .with_page_size(2)
//     // .with_render_config(mine)
//     // .with_help_message("Use arrow keys to select an option, press Enter to select.")
//     // .with_render_config(mine)
//     // .prompt()
//     // .unwrap();

//     let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is not set");
//     let path = format!("{}/data", manifest_dir);
//     let databases = list_frames_in_dir(&path);

//     let selected_path = match databases {
//         Ok(datas) => Select::new(
//             "Choose a database to process:",
//             datas.iter().map(|opt| opt.to_string()).collect(),
//         )
//         .with_page_size(4)
//         .with_render_config(mine)
//         .with_help_message("Use arrow keys to select an option, press Enter to select.")
//         .with_render_config(mine)
//         .prompt()
//         .unwrap(),
//         Err(e) => panic!("{}", e),
//     };

//     let selected_show_matrices = Select::new(
//         "Show matrices?",
//         yes_no.iter().map(|opt| opt.to_string()).collect(),
//     )
//     .with_page_size(2)
//     .with_render_config(mine)
//     .with_help_message("Use arrow keys to select an option, press Enter to select.")
//     .with_render_config(mine)
//     .prompt()
//     .unwrap();

//     let data = match fs::read_to_string(selected_path) {
//         Ok(json_string) => match serde_json::from_str::<StaticAssignmentData>(&json_string) {
//             Ok(data) => data,
//             Err(e) => panic!("{}", e),
//         },
//         Err(e) => panic!("{}", e),
//     };

//     let result = calculate_static_assignment(
//         false,
//         &data.employees,
//         &data.jobs,
//         &data
//             .competence_map
//             .iter()
//             .map(|map| (map.employee.clone(), map.competences.clone()))
//             .collect(),
//         &data
//             .preference_map
//             .iter()
//             .map(|map| (map.employee.clone(), map.preferences.clone()))
//             .collect(),
    
//     );

//     println!("");
//     println!("  Total preference score: {}", result.1);
//     println!("");
//     println!("  Optimal employee job assignment: ");
//     for (employee, job) in result.0.clone() {
//         println!("  {:<8} -> {}", employee, job);
//     }
//     println!("");

//     match selected_show_matrices.as_str() {
//         "Yes" => {
//             println!("  Competence matrix:");
//             for i in 0..result.2.len() {
//                 println!("  {:<8}  : {:?}", result.0[i].0, result.2[i].iter().map(|x| match x {
//                     true => 1,
//                     false => 0
//                 }).collect::<Vec<i32>>())
//             }
//             println!("");
//             println!("  Preference matrix:");
//             for i in 0..result.2.len() {
//                 println!("  {:<8}  : {:?}", result.0[i].0, result.3[i])
//             }
//         },
//         _ => {}
//     }

// }
