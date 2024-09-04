use artwork_ejas::*;
use inquire::ui::{Attributes, Color, RenderConfig, StyleSheet, Styled};
use std::time::Duration;
use std::{fs, thread};
use rusqlite::{params, Connection, Result};
use inquire::{Text, Confirm, Password, MultiSelect, Select};

// 4.9.2024.
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

    // Create the 'users' table to store managing users and operators
    conn.execute(
        "CREATE TABLE IF NOT EXISTS users (
                  id INTEGER PRIMARY KEY,
                  username TEXT NOT NULL UNIQUE,
                  password TEXT,   -- Only managing users need a password
                  role TEXT NOT NULL  -- 'Managing' or 'Operator'
                  )",
        [],
    )?;

    // Check if any managers exist
    let managing_users_count: i32 = conn.query_row(
        "SELECT COUNT(*) FROM users WHERE role = 'Managing'",
        [],
        |row| row.get(0),
    )?;

    if managing_users_count == 0 {
        // No managing users exist, enter setup mode
        println!("No managing users found. Entering setup mode to create the first manager.");
        setup_first_manager(&conn)?;
    } else {
        // Normal login process
        login(&conn)?;
    }

    Ok(())
}

// Function to handle first-time setup (create first manager)
fn setup_first_manager(conn: &Connection) -> Result<()> {
    println!("Create the first managing user:");
    
    let username = Text::new("Enter username for the first manager:").prompt().unwrap();
    let password = Password::new("Enter password for the first manager:").prompt().unwrap();

    // Insert the first managing user into the 'users' table
    conn.execute(
        "INSERT INTO users (username, password, role) VALUES (?1, ?2, 'Managing')",
        params![username, password],
    )?;

    println!("First managing user created successfully.");
    Ok(())
}

// Function to handle login and routing based on user role
fn login(conn: &Connection) -> Result<()> {
    // Select whether to log in as Managing or Operator
    let role = Select::new("Select your role:", vec!["Managing", "Operator"])
        .prompt()
        .unwrap();

    match role {
        "Managing" => managing_login(conn),
        "Operator" => operator_login(conn),
        _ => Ok(()),
    }
}

// Managing user login function
fn managing_login(conn: &Connection) -> Result<()> {
    let username = Text::new("Enter username:").prompt().unwrap();
    let password = Password::new("Enter password:").prompt().unwrap();

    // Validate managing user
    let valid_user = conn.query_row(
        "SELECT COUNT(*) FROM users WHERE username = ?1 AND password = ?2 AND role = 'Managing'",
        params![username, password],
        |row| row.get::<_, i32>(0),
    )?;

    if valid_user == 1 {
        println!("Managing user logged in.");
        managing_menu(conn)
    } else {
        println!("Invalid username or password.");
        Ok(())
    }
}

// Operator login function
fn operator_login(conn: &Connection) -> Result<()> {
    let username = Text::new("Enter your name:").prompt().unwrap();

    // Validate if the user is an operator
    let valid_user = conn.query_row(
        "SELECT COUNT(*) FROM users WHERE username = ?1 AND role = 'Operator'",
        params![username],
        |row| row.get::<_, i32>(0),
    )?;

    if valid_user == 1 {
        println!("Operator logged in.");
        operator_menu(conn, username)
    } else {
        println!("Invalid operator name.");
        Ok(())
    }
}

// Menu for managing users
fn managing_menu(conn: &Connection) -> Result<()> {
    loop {
        let choice = Select::new("Managing menu:", vec![
            "Add a new Operator",
            "Remove an Operator",
            "Add a new Manager",
            "Add new Competence",
            "Exit"
        ])
        .prompt()
        .unwrap();

        match choice {
            "Add a new Operator" => add_operator(conn)?,
            "Remove an Operator" => remove_operator(conn)?,
            "Add a new Manager" => add_manager(conn)?,
            "Add new Competence" => add_competence(conn)?,
            "Exit" => break,
            _ => (),
        }
    }

    Ok(())
}

// Menu for operators
fn operator_menu(conn: &Connection, username: String) -> Result<()> {
    loop {
        let choice = Select::new("Operator menu:", vec![
            "Change your job preference",
            "Exit"
        ])
        .prompt()
        .unwrap();

        match choice {
            "Change your job preference" => change_job_preference(conn, &username)?,
            "Exit" => break,
            _ => (),
        }
    }

    Ok(())
}

// Function to add a new operator
fn add_operator(conn: &Connection) -> Result<()> {
    let username = Text::new("Enter new Operator's name:").prompt().unwrap();

    // Insert operator into 'users' table with role 'Operator'
    conn.execute(
        "INSERT INTO users (username, role) VALUES (?1, 'Operator')",
        params![username],
    )?;

    println!("Operator added successfully.");
    Ok(())
}

// Function to remove an operator
fn remove_operator(conn: &Connection) -> Result<()> {
    let username = Text::new("Enter the name of the Operator to remove:").prompt().unwrap();

    // Remove the operator from the 'users' table
    conn.execute(
        "DELETE FROM users WHERE username = ?1 AND role = 'Operator'",
        params![username],
    )?;

    println!("Operator removed successfully.");
    Ok(())
}

// Function to add a new manager
fn add_manager(conn: &Connection) -> Result<()> {
    let username = Text::new("Enter new Manager's username:").prompt().unwrap();
    let password = Password::new("Enter new Manager's password:").prompt().unwrap();

    // Insert new manager into 'users' table with role 'Managing'
    conn.execute(
        "INSERT INTO users (username, password, role) VALUES (?1, ?2, 'Managing')",
        params![username, password],
    )?;

    println!("Manager added successfully.");
    Ok(())
}

// Function to add a new competence
fn add_competence(conn: &Connection) -> Result<()> {
    let competence = Text::new("Enter new competence:").prompt().unwrap();

    // Insert new competence into the 'competences' table
    conn.execute(
        "INSERT OR IGNORE INTO competences (name) VALUES (?1)",
        params![competence],
    )?;

    println!("Competence added successfully.");
    Ok(())
}

// Function for an operator to change their job preference
fn change_job_preference(conn: &Connection, username: &str) -> Result<()> {
    let new_job = Text::new("Enter your new job preference:").prompt().unwrap();

    // Update the job preference for the operator
    conn.execute(
        "UPDATE people SET job = ?1 WHERE name = ?2",
        params![new_job, username],
    )?;

    println!("Job preference updated.");
    Ok(())
}






// Working version
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

//     // Create the 'competences' table if it doesn't exist
//     conn.execute(
//         "CREATE TABLE IF NOT EXISTS competences (
//                   id INTEGER PRIMARY KEY,
//                   name TEXT NOT NULL UNIQUE
//                   )",
//         [],
//     )?;

//     // Create the 'person_competences' table to link people with competences
//     conn.execute(
//         "CREATE TABLE IF NOT EXISTS person_competences (
//                   person_id INTEGER,
//                   competence_id INTEGER,
//                   FOREIGN KEY(person_id) REFERENCES people(id),
//                   FOREIGN KEY(competence_id) REFERENCES competences(id),
//                   PRIMARY KEY(person_id, competence_id)
//                   )",
//         [],
//     )?;

//     // Predefined list of competences
//     let competences = vec![
//         "Programming",
//         "Project Management",
//         "Design",
//         "Data Analysis",
//         "Communication",
//         "Leadership",
//     ];

//     // Insert predefined competences into the 'competences' table
//     for competence in &competences {
//         conn.execute(
//             "INSERT OR IGNORE INTO competences (name) VALUES (?1)",
//             params![competence],
//         )?;
//     }

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

//         // Get the ID of the newly inserted person
//         let person_id: i32 = conn.last_insert_rowid() as i32;

//         // Use `inquire` to select competences from the predefined list
//         let selected_competences = MultiSelect::new(
//             "Select competences (use space to select, enter to confirm):",
//             competences.clone(),
//         )
//         .prompt()
//         .unwrap();

//         // Insert the selected competences into the junction table
//         for competence in selected_competences {
//             let competence_id: i32 = conn.query_row(
//                 "SELECT id FROM competences WHERE name = ?1",
//                 params![competence],
//                 |row| row.get(0),
//             )?;
//             conn.execute(
//                 "INSERT INTO person_competences (person_id, competence_id) VALUES (?1, ?2)",
//                 params![person_id, competence_id],
//             )?;
//         }

//         // Ask if the user wants to add another person
//         let add_another = Confirm::new("Do you want to add another person?")
//             .prompt()
//             .unwrap();

//         if !add_another {
//             break;
//         }
//     }

//     // Query and display all rows in the 'people' table with their competences
//     let mut stmt = conn.prepare(
//         "SELECT people.id, people.name, people.job, GROUP_CONCAT(competences.name, ', ') as competences
//          FROM people
//          LEFT JOIN person_competences ON people.id = person_competences.person_id
//          LEFT JOIN competences ON person_competences.competence_id = competences.id
//          GROUP BY people.id, people.name, people.job",
//     )?;
//     let person_iter = stmt.query_map([], |row| {
//         Ok(PersonWithCompetences {
//             id: row.get(0)?,
//             name: row.get(1)?,
//             job: row.get(2)?,
//             competences: row.get(3)?,
//         })
//     })?;

//     println!("People stored in the database with their competences:");
//     for person in person_iter {
//         println!("{:?}", person?);
//     }

//     Ok(())
// }

// // Struct to hold person data with competences
// #[derive(Debug)]
// struct PersonWithCompetences {
//     id: i32,
//     name: String,
//     job: String,
//     competences: Option<String>,  // Competences as a comma-separated string
// }


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
