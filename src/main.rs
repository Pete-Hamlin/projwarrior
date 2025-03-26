#![recursion_limit = "1024"]
mod config;
mod db;
mod parser;
mod project;
mod table;

use clap::Parser;
use config::{Cli, GtdConfig, get_config};
use db::{check_db, delete_project, get_projects, insert_projects, update_project_status};
use parser::{Task, get_task_list};
use project::{Project, State};
use std::fs::remove_file;
use std::usize;
use table::{project_details_table, project_list_table};
use uuid::Uuid;

fn main() {
    let args = Cli::parse();
    let cfg = get_config(&args);
    match check_db(&cfg) {
        Ok(_) => (),
        Err(error) => println!("Error connecting to the database: {:?}", error),
    };
    if let Some(command) = args.command.as_deref() {
        match command {
            "init" => init_projects(&cfg),
            "list" => list_projects(&cfg, &args),
            "count" => count_projects(&cfg, &args),
            "add" => add_project(&cfg, &args),
            "reset" => reset_projects(&cfg),
            _ => parse_subcommand(&cfg, &args),
        }
    } else {
        list_projects(&cfg, &args)
    }
}

fn parse_subcommand(cfg: &GtdConfig, args: &Cli) {
    // If we have subcommands, command should be a project ID, which is an an integer
    let projects = get_projects(&cfg, None, None).expect("Failed to retrieve project list");
    let id: usize = args
        .command
        .clone()
        .expect("ID incorrect format, check gtd --help for correct syntax")
        .parse::<usize>()
        .unwrap()
        + 1;
    if id >= projects.len() {
        println!("No project found with ID {:?}", id.to_string());
        return;
    }
    if let Some(subcommand) = args.subcommand.as_deref() {
        match subcommand {
            "show" => show_project(cfg, id),
            "done" => mark_project_done(cfg, id),
            "incubate" => mark_project_incubate(cfg, id),
            "start" => mark_project_pending(cfg, id),
            "delete" => delete_project_item(cfg, id),
            _ => println!("Subcommand {subcommand} not found"),
        }
    } else {
        show_project(cfg, id);
    }
}

fn init_projects(cfg: &GtdConfig) -> () {
    let tasks = get_task_list(&cfg).expect("Failed to get task list");
    let mut name_list: Vec<String> = vec![];
    tasks.into_iter().for_each(|task| {
        let project_name = task.project.clone().unwrap();
        if !name_list.contains(&project_name) {
            name_list.push(project_name);
        }
    });
    let projects: Vec<Project> = name_list
        .into_iter()
        .map(|name| Project {
            name,
            uuid: Uuid::new_v4(),
            ..Default::default()
        })
        .collect();
    match insert_projects(cfg, &projects) {
        Ok(_p) => println!("Successfully initialized new project list"),
        Err(e) => println!("Failed to write project list: {:?}", e),
    }
}

fn list_projects(cfg: &GtdConfig, args: &Cli) {
    let tasks = get_task_list(&cfg).expect("Failed to get task list");
    let filter = match args.subcommand.as_deref() {
        Some("all") => None,
        Some("incubate") => Some(&State::Incubate),
        Some("done") => Some(&State::Complete),
        _ => Some(&State::Pending),
    };
    let projects = get_projects(&cfg, filter, None).expect("Failed to retrieve project list");
    project_list_table(cfg, &tasks, &projects);
}

fn count_projects(cfg: &GtdConfig, args: &Cli) {
    let tasks = get_task_list(&cfg).expect("Failed to get task list");
    let filter = match args.subcommand.as_deref() {
        Some("all") => None,
        Some("incubate") => Some(&State::Incubate),
        Some("done") => Some(&State::Complete),
        _ => Some(&State::Pending),
    };
    let projects = get_projects(&cfg, filter, None).expect("Failed to retrieve project list");

    if !cfg.short {
        let count = projects.into_iter().count();
        println!("{:?}", count)
    } else {
        let count = projects
            .into_iter()
            .filter(|p| p.get_tasks(&tasks) == 0)
            .count();
        println!("{:?}", count)
    }
}

fn add_project(cfg: &GtdConfig, args: &Cli) -> () {
    if let Some(subcommand) = args.subcommand.as_deref() {
        let project = vec![Project {
            name: subcommand.to_string(),
            uuid: Uuid::new_v4(),
            ..Default::default()
        }];
        match insert_projects(cfg, &project) {
            Ok(_p) => println!("Successfully processed project"),
            Err(e) => println!("Failed to add project {:?}", e),
        }
    } else {
        println!("No task specified - run `proj --help` for guidance on running this command")
    }
}

fn reset_projects(cfg: &GtdConfig) {
    match remove_file(&cfg.storage_path) {
        Ok(_p) => println!("Successfully removed project list"),
        Err(e) => println!("Failed to remove project list: {:?}", e),
    }
}

fn show_project(cfg: &GtdConfig, project_id: usize) {
    let projects = get_projects(&cfg, None, None).expect("Failed to retrieve project list");
    let tasks = get_task_list(&cfg).expect("Failed to get task list");
    let project = &projects[project_id];
    let project_tasks: Vec<Task> = tasks
        .iter()
        .filter(|t| t.project.as_deref() == Some(&project.name))
        .cloned()
        .collect();
    project_details_table(cfg, project, &project_tasks);
}

fn mark_project_done(cfg: &GtdConfig, project_id: usize) -> () {
    let projects = get_projects(&cfg, None, None).expect("Failed to retrieve project list");
    let project = projects.get(project_id).unwrap();
    match update_project_status(cfg, &State::Complete, &project.uuid) {
        Ok(_) => println!("Marked project {:?} as done!", project.name),
        Err(e) => println!("Unable to process project, error: {e:?}"),
    };
}

fn mark_project_incubate(cfg: &GtdConfig, project_id: usize) -> () {
    let projects = get_projects(&cfg, None, None).expect("Failed to retrieve project list");
    let project = projects.get(project_id).unwrap();
    match update_project_status(cfg, &State::Incubate, &project.uuid) {
        Ok(_) => println!("Incubated project {:?}!", project.name),
        Err(e) => println!("Unable to process project, error: {e:?}"),
    };
}

fn mark_project_pending(cfg: &GtdConfig, project_id: usize) -> () {
    let projects = get_projects(&cfg, None, None).expect("Failed to retrieve project list");
    let project = projects.get(project_id).unwrap();
    match update_project_status(cfg, &State::Pending, &project.uuid) {
        Ok(_) => println!("Marked project {:?} as pending!", project.name),
        Err(e) => println!("Unable to process project, error: {e:?}"),
    };
}

fn delete_project_item(cfg: &GtdConfig, proj_id: usize) -> () {
    let projects = get_projects(&cfg, None, None).expect("Failed to retrieve project list");
    let project = projects.get(proj_id).unwrap();
    match delete_project(cfg, &project.uuid) {
        Ok(p) => println!("Successfully removed project {:?}", p),
        Err(e) => println!("Failed to remove project {:?}", e),
    }
}
