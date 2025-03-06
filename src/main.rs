#![recursion_limit = "1024"]
mod config;
mod parser;
mod project;
mod table;

use clap::Parser;
use config::{Cli, GtdConfig, get_config};
use parser::{Task, get_task_list};
use project::{Project, get_projects, write_project_list};
use std::error::Error;
use std::fs::remove_file;
use table::{project_details_table, project_list_table};

fn main() {
    let args = Cli::parse();
    let cfg = get_config(&args);
    let tasks = get_task_list(&cfg).expect("Failed to get task list");
    let mut projects = get_projects(&cfg).expect("Failed to retrieve project list");

    if let Some(command) = args.command.as_deref() {
        match command {
            "init" => init_projects(&cfg, &tasks),
            "list" => list_projects(&cfg, &tasks, &projects),
            "count" => count_projects(&cfg, &tasks, &projects),
            "add" => add_project(&cfg, &args, &mut projects),
            "reset" => reset_projects(&cfg),
            _ => parse_subcommand(&cfg, &args, &tasks, &mut projects),
        }
    } else {
        list_projects(&cfg, &tasks, &projects)
    }
}

fn parse_subcommand(cfg: &GtdConfig, args: &Cli, tasks: &[Task], projects: &mut Vec<Project>) {
    // If we have subcommands, command should be a project ID, which is an an integer
    let id: usize = args
        .command
        .clone()
        .expect("ID incorrect format, check gtd --help for correct syntax")
        .parse::<usize>()
        .unwrap();
    if id >= projects.len() {
        println!("No project found with ID {:?}", id.to_string());
        return;
    }
    if let Some(subcommand) = args.subcommand.as_deref() {
        match subcommand {
            // "done" => mark_project_done(id, projects),
            "delete" => delete_item(cfg, id, projects),
            "show" => show_project(cfg, id, projects, tasks),
            _ => println!("Subcommand {} not found", subcommand),
        }
    } else {
        show_project(cfg, id, projects, tasks);
    }
}

fn init_projects(cfg: &GtdConfig, tasks: &[Task]) -> () {
    let mut name_list: Vec<String> = vec![];
    tasks.into_iter().for_each(|task| {
        let project_name = task.project.clone().unwrap();
        if !name_list.contains(&project_name) {
            name_list.push(project_name);
        }
    });
    let projects: Vec<Project> = name_list.into_iter().map(|name| Project { name }).collect();
    match write_project_list(cfg, &projects) {
        Ok(_p) => println!("Successfully initialized new project list"),
        Err(e) => println!("Failed to write project list: {:?}", e),
    }
}

fn list_projects(cfg: &GtdConfig, tasks: &[Task], projects: &[Project]) {
    project_list_table(cfg, tasks, &projects);
}

fn count_projects(cfg: &GtdConfig, tasks: &[Task], projects: &[Project]) {
    if !cfg.short {
        let count = projects.into_iter().count();
        println!("{:?}", count)
    } else {
        let count = projects
            .into_iter()
            .filter(|p| p.get_tasks(tasks) == 0)
            .count();
        println!("{:?}", count)
    }
}

fn reset_projects(cfg: &GtdConfig) {
    remove_file(&cfg.storage_path).expect("Error removing config file.");
    let projects: Vec<Project> = vec![];
    match write_project_list(cfg, &projects) {
        Ok(_p) => println!("Successfully reset project list"),
        Err(e) => println!("Failed to write project list: {:?}", e),
    }
}

fn show_project(cfg: &GtdConfig, project_id: usize, projects: &[Project], tasks: &[Task]) {
    let project = &projects[project_id];
    let project_tasks: Vec<Task> = tasks
        .iter()
        .filter(|t| t.project.as_deref() == Some(&project.name))
        .cloned()
        .collect();
    project_details_table(cfg, project, &project_tasks);
}

fn add_project(cfg: &GtdConfig, args: &Cli, projects: &mut Vec<Project>) -> () {
    if let Some(subcommand) = args.subcommand.as_deref() {
        projects.push(Project {
            name: subcommand.to_string(),
        });
        match write_project_list(cfg, projects) {
            Ok(_p) => println!("Successfully processed project"),
            Err(e) => println!("Failed to add project {:?}", e),
        }
    } else {
        println!("No task specified - run `proj --help` for guidance on running this command")
    }
}

// fn mark_project_done(proj_id: usize, projects: &mut Vec<Project>) -> () {}

fn delete_item(cfg: &GtdConfig, proj_id: usize, projects: &mut Vec<Project>) -> () {
    match remove_project_item(cfg, proj_id, projects) {
        Ok(p) => println!("Successfully removed project {:?}", p),
        Err(e) => println!("Failed to remove project {:?}", e),
    }
}

fn remove_project_item(
    cfg: &GtdConfig,
    project_id: usize,
    projects: &mut Vec<Project>,
) -> Result<String, Box<dyn Error>> {
    let project = projects.remove(project_id);
    write_project_list(cfg, projects)?;
    Ok(project.name)
}
