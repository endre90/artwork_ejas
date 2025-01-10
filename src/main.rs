use rusqlite::{params, Connection, Result};
use inquire::{ui::{Attributes, Color, RenderConfig, StyleSheet, Styled}, MultiSelect, Password, Select, Text};

// should have option to view history
// need a help option probably
// save history up till 6 months
// encrypted database? probably no need to do that for now, neither anonymization
// we can do some analytics based on the collected information, and show that the algorithm will actually perform better
// start with a random historical distribution of 2 weeks, just noise, in order to predisct sho should do what
// and then use the historic data every day to schedule fair task allocation
// we probably don't need a long horizon, just the passes for the day
// ensure that database is not lost, automatic backup somewhere, or send it somewhere?
// quick way of selecting who is here today so that we can get a quick job allocation
// overview for the next passes over the day (4 passes per day)
// probably don't need to know who is doing what tomorrow, just for the day
// but need to able to change the allocation if somebody leaves during the day
// keep track of dates and times

fn main() -> Result<()> {
    // Connect to the SQLite database (or create it if it doesn't exist)
    let conn = Connection::open("people.db")?;

    // Create the necessary tables if they don't exist
    setup_database(&conn)?;

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

fn custom_renrer_config() -> RenderConfig<'static> {
    let mut config = RenderConfig::default();
    config.selected_option = Some(StyleSheet::new().with_fg(Color::DarkGreen));
    config.answer = StyleSheet::new().with_fg(Color::DarkGreen).with_attr(Attributes::BOLD);
    config.highlighted_option_prefix = Styled::new(" ->").with_fg(Color::DarkGreen).with_attr(Attributes::BOLD);
    config
}

fn setup_database(conn: &Connection) -> Result<()> {
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
    .with_render_config(custom_renrer_config())
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
    let password = Password::new("Enter password:").without_confirmation().prompt().unwrap();

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
            "Assign Competences to Operator",
            "Overview of Operators and Competences",  // New Option
            "Exit"
        ])
        .prompt()
        .unwrap();

        match choice {
            "Add a new Operator" => add_operator(conn)?,
            "Remove an Operator" => remove_operator(conn)?,
            "Add a new Manager" => add_manager(conn)?,
            "Add new Competence" => add_competence(conn)?,
            "Assign Competences to Operator" => assign_competences_to_operator(conn)?,
            "Overview of Operators and Competences" => overview_operators_competences(conn)?,  // New functionality
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

fn add_operator(conn: &Connection) -> Result<()> {
    // Prompt the user for the operator's name or to cancel by pressing Enter with empty input
    let username = Text::new("Enter new Operator's name:")
    .with_help_message("press Enter without typing anything to cancel")
        .prompt()
        .unwrap();

    // Check if the input is empty (indicating the user wants to cancel)
    if username.trim().is_empty() {
        println!("Operation canceled.");
        return Ok(());
    }

    // Insert operator into 'users' table with role 'Operator'
    conn.execute(
        "INSERT INTO users (username, role) VALUES (?1, 'Operator')",
        params![username],
    )?;

    println!("Operator added successfully.");
    Ok(())
}

fn remove_operator(conn: &Connection) -> Result<()> {
    // Fetch the list of operators
    let mut stmt = conn.prepare("SELECT username FROM users WHERE role = 'Operator'")?;
    let operators = stmt.query_map([], |row| Ok(row.get::<_, String>(0)?))?;

    let operator_list: Vec<String> = operators.collect::<Result<Vec<_>, _>>()?;

    // If no operators exist, return immediately
    if operator_list.is_empty() {
        println!("No operators found.");
        return Ok(());
    }

    // Add "Cancel" option to the list of operators
    let mut operator_list_with_cancel = operator_list.clone();
    operator_list_with_cancel.push("Cancel".to_string());

    // Select an operator to remove or select "Cancel"
    let selected_operator = Select::new("Select the Operator to remove (or select 'Cancel' to go back):", operator_list_with_cancel)
        .prompt()
        .unwrap();

    // Check if the user selected "Cancel"
    if selected_operator == "Cancel" {
        println!("Operation canceled.");
        return Ok(());
    }

    // Remove the operator from the 'users' table
    conn.execute(
        "DELETE FROM users WHERE username = ?1 AND role = 'Operator'",
        params![selected_operator],
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

// Function to assign competences to an operator
fn assign_competences_to_operator(conn: &Connection) -> Result<()> {
    // Fetch the list of operators
    let mut stmt = conn.prepare("SELECT username FROM users WHERE role = 'Operator'")?;
    let operators = stmt.query_map([], |row| Ok(row.get::<_, String>(0)?))?;

    let operator_list: Vec<String> = operators.collect::<Result<Vec<_>, _>>()?;
    if operator_list.is_empty() {
        println!("No operators found.");
        return Ok(());
    }

    // Select an operator
    let selected_operator = Select::new("Select an operator:", operator_list)
        .prompt()
        .unwrap();

    // Fetch the list of competences
    let mut stmt = conn.prepare("SELECT name FROM competences")?;
    let competences = stmt.query_map([], |row| Ok(row.get::<_, String>(0)?))?;

    let competence_list: Vec<String> = competences.collect::<Result<Vec<_>, _>>()?;
    if competence_list.is_empty() {
        println!("No competences found.");
        return Ok(());
    }

    // Use MultiSelect to select competences
    let selected_competences = MultiSelect::new(
        "Select competences (use space to select, enter to confirm):",
        competence_list,
    )
    .prompt()
    .unwrap();

    // Get operator's ID
    let operator_id: i32 = conn.query_row(
        "SELECT id FROM users WHERE username = ?1 AND role = 'Operator'",
        params![selected_operator],
        |row| row.get(0),
    )?;

    // Insert selected competences for the operator
    for competence in selected_competences {
        let competence_id: i32 = conn.query_row(
            "SELECT id FROM competences WHERE name = ?1",
            params![competence],
            |row| row.get(0),
        )?;

        conn.execute(
            "INSERT OR IGNORE INTO person_competences (person_id, competence_id) VALUES (?1, ?2)",
            params![operator_id, competence_id],
        )?;
    }

    println!("Competences assigned to operator successfully.");
    Ok(())
}

// Function to display overview of operators and their competences
fn overview_operators_competences(conn: &Connection) -> Result<()> {
    // Fetch all operators from the database
    let mut stmt = conn.prepare("SELECT id, username FROM users WHERE role = 'Operator'")?;
    let operators = stmt.query_map([], |row| {
        Ok(Operator {
            id: row.get(0)?,
            username: row.get(1)?,
        })
    })?;

    let operator_list: Vec<Operator> = operators.collect::<Result<Vec<_>, _>>()?;
    if operator_list.is_empty() {
        println!("No operators found.");
        return Ok(());
    }

    // Loop through the list of operators and display their competences
    let operator_usernames: Vec<String> = operator_list.iter().map(|op| op.username.clone()).collect();
    
    loop {
        // Select an operator to view their competences
        let selected_operator = Select::new("Select an operator to view their competences:", operator_usernames.clone())
            .prompt()
            .unwrap();

        // Find the operator in the list and fetch their competences
        let operator = operator_list.iter().find(|op| op.username == selected_operator).unwrap();
        let competences = get_competences_for_operator(conn, operator.id)?;

        // Display the competences in a tree-like format
        println!("Operator: {}", operator.username);
        if competences.is_empty() {
            println!("  No competences assigned.");
        } else {
            println!("  Competences:");
            for competence in competences {
                println!("    - {}", competence);
            }
        }

        // Ask if the managing user wants to view another operator's competences
        let view_another = Select::new("Do you want to view another operator's competences?", vec!["Yes", "No"])
            .prompt()
            .unwrap();

        if view_another == "No" {
            break;
        }
    }

    Ok(())
}

// Helper function to get competences for a specific operator
fn get_competences_for_operator(conn: &Connection, operator_id: i32) -> Result<Vec<String>> {
    let mut stmt = conn.prepare(
        "SELECT competences.name
         FROM competences
         JOIN person_competences ON competences.id = person_competences.competence_id
         WHERE person_competences.person_id = ?1"
    )?;

    let competences = stmt.query_map(params![operator_id], |row| Ok(row.get::<_, String>(0)?))?;

    competences.collect::<Result<Vec<_>, _>>()
}

// Struct to represent an operator
struct Operator {
    id: i32,
    username: String,
}
