use artwork_ejas::*;
use inquire::ui::{Attributes, Color, RenderConfig, StyleSheet, Styled};
use inquire::{Confirm, Select};
use std::fs;

fn main() {
    println!("");

    let algorithms = vec!["Static", "External", "Horizon", "Historic"];
    let yes_no = vec!["Yes", "No"];

    let default: RenderConfig = RenderConfig::empty();
    let prompt_prefix = Styled::new("?")
        .with_fg(Color::DarkGreen)
        .with_attr(Attributes::BOLD);
    let prompt_choice = Styled::new(" ->")
        .with_fg(Color::DarkGreen)
        .with_attr(Attributes::BOLD);
    let mine = default.with_prompt_prefix(prompt_prefix);
    let mine = mine.with_highlighted_option_prefix(prompt_choice);
    let new_style_sheet = StyleSheet::empty();
    let mine = mine.with_answer(
        new_style_sheet
            .with_fg(Color::DarkGreen)
            .with_attr(Attributes::BOLD),
    );

    let selected_algorithm = Select::new("Select the algorithm:", algorithms.iter().map(|opt| opt.to_string()).collect())
        .with_page_size(4)
        .with_help_message("Use arrow keys to select an option, press Enter to select.] \n[For information about algorithms, look in the README.")
        .with_render_config(mine)
        .prompt()
        .unwrap();

    let selected_anonymize = Select::new(
        "Do you want to anonymize data?",
        yes_no.iter().map(|opt| opt.to_string()).collect(),
    )
    .with_page_size(2)
    .with_render_config(mine)
    .with_help_message("Use arrow keys to select an option, press Enter to select.")
    .with_render_config(mine)
    .prompt()
    .unwrap();

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is not set");
    let path = format!("{}/data", manifest_dir);
    let databases = list_frames_in_dir(&path);

    let selected_path = match databases {
        Ok(datas) => Select::new(
            "Choose a database to process:",
            datas.iter().map(|opt| opt.to_string()).collect(),
        )
        .with_page_size(4)
        .with_render_config(mine)
        .with_help_message("Use arrow keys to select an option, press Enter to select.")
        .with_render_config(mine)
        .prompt()
        .unwrap(),
        Err(e) => panic!(),
    };

    let selected_show_matrices = Select::new(
        "Show matrices?",
        yes_no.iter().map(|opt| opt.to_string()).collect(),
    )
    .with_page_size(2)
    .with_render_config(mine)
    .with_help_message("Use arrow keys to select an option, press Enter to select.")
    .with_render_config(mine)
    .prompt()
    .unwrap();

    let data = match fs::read_to_string(selected_path) {
        Ok(json_string) => match serde_json::from_str::<StaticAssignmentData>(&json_string) {
            Ok(data) => data,
            Err(e) => panic!(),
        },
        Err(_) => panic!(),
    };

    let result = calculate_static_assignment(
        match selected_anonymize.as_str() {
            "Yes" => true,
            _ => false,
        },
        &data.employees,
        &data.jobs,
        &data
            .competence_map
            .iter()
            .map(|map| (map.employee.clone(), map.competences.clone()))
            .collect(),
        &data
            .preference_map
            .iter()
            .map(|map| (map.employee.clone(), map.preferences.clone()))
            .collect(),
    
    );

    println!("");
    println!("  Total preference score: {}", result.1);
    println!("");
    println!("  Optimal employee job assignment: ");
    for (employee, job) in result.0.clone() {
        println!("  {:<8} -> {}", employee, job);
    }
    println!("");

    match selected_show_matrices.as_str() {
        "Yes" => {
            println!("  Competence matrix:");
            for i in 0..result.2.len() {
                println!("  {:<8}  : {:?}", result.0[i].0, result.2[i].iter().map(|x| match x {
                    true => 1,
                    false => 0
                }).collect::<Vec<i32>>())
            }
        },
        _ => {}
    }

}
