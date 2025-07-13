#![recursion_limit = "1024"]
mod config;
mod db;
mod project;
mod table;
mod tasks;

use clap::Parser;
use config::{Cli, GtdConfig, get_config};
use db::DB;
use project::{Project, State};
use std::fs::remove_file;
use table::{project_details_table, project_list_table};
use task_hookrs::task::Task;
use tasks::get_task_list;
use uuid::Uuid;

fn main() {
    let args = Cli::parse();
    let cfg = get_config(&args);
    let mut db = DB::new(&cfg.storage_path).unwrap();
   match db.check() {
        Ok(_) => (),
        Err(error) => println!("Error connecting to the database: {:?}", error),
    };
    if let Some(command) = args.command.as_deref() {
        match command {
            "init" => init_projects(&cfg, &mut db),
            "list" => list_projects(&cfg, &db, &args),
            "count" => count_projects(&cfg, &db, &args),
            "add" => add_project(&mut db, &args),
            "reset" => reset_projects(&cfg),
            _ => parse_subcommand(&cfg, &db, &args),
        }
    } else {
        list_projects(&cfg, &db, &args)
    }
}

fn init_projects(cfg: &GtdConfig, db: &mut DB) -> () {
    let tasks = get_task_list(&cfg).expect("Failed to get task list");
    let mut name_list: Vec<String> = vec![];
    tasks.into_iter().for_each(|task| {
        let project_name = task.project().unwrap();
        if !name_list.contains(&project_name) {
            name_list.push(project_name.clone());
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
    match db.insert_projects(&projects) {
        Ok(_p) => println!("Successfully initialized new project list"),
        Err(e) => println!("Failed to write project list: {:?}", e),
    }
}

fn list_projects(cfg: &GtdConfig, db: &DB, args: &Cli) {
    let tasks = get_task_list(&cfg).expect("Failed to get task list");
    let filter = match args.subcommand.as_deref() {
        Some("all") => None,
        Some("incubate") => Some(&State::Incubate),
        Some("done") => Some(&State::Complete),
        _ => Some(&State::Pending),
    };
    let projects = db
        .get_projects(filter, None, None)
        .expect("Failed to retrieve project list");
    project_list_table(cfg, &tasks, &projects);
}

fn count_projects(cfg: &GtdConfig, db: &DB, args: &Cli) {
    let tasks = get_task_list(&cfg).expect("Failed to get task list");
    let filter = match args.subcommand.as_deref() {
        Some("all") => None,
        Some("incubate") => Some(&State::Incubate),
        Some("done") => Some(&State::Complete),
        _ => Some(&State::Pending),
    };
    let projects = db
        .get_projects(filter, None, None)
        .expect("Failed to retrieve project list");

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

fn add_project(db: &mut DB, args: &Cli) -> () {
    if let Some(subcommand) = args.subcommand.as_deref() {
        let project = vec![Project {
            name: subcommand.to_string(),
            uuid: Uuid::new_v4(),
            ..Default::default()
        }];
        match db.insert_projects(&project) {
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

fn parse_subcommand(cfg: &GtdConfig, db: &DB, args: &Cli) {
    // If we have subcommands, command should be a project ID, which is an an integer
    let id = match args
        .command
        .as_deref()
        .and_then(|cmd| cmd.parse::<u32>().ok())
    {
        Some(id) => id,
        None => {
            println!("Invalid value provided for ID");
            return;
        }
    };

    let project = match db.get_projects(None, None, Some(&id)) {
        Ok(p) => p.into_iter().nth(0).unwrap(),
        Err(_) => {
            println!("Failed to retrieve project from list");
            return;
        }
    };

    if let Some(subcommand) = args.subcommand.as_deref() {
        match subcommand {
            "show" => show_project(cfg, &project),
            "done" => mark_project_done(db, &project),
            "incubate" => mark_project_incubate(db, &project),
            "start" => mark_project_pending(db, &project),
            "delete" => delete_project_item(db, &project),
            _ => println!("Subcommand {subcommand} not found"),
        }
    } else {
        show_project(cfg, &project);
    }
}

fn show_project(cfg: &GtdConfig, project: &Project) {
    let tasks = get_task_list(&cfg).expect("Failed to get task list");
    let project_tasks: Vec<Task> = tasks
        .iter()
        .filter(|t| t.project() == Some(&project.name))
        .cloned()
        .collect();
    project_details_table(cfg, project, &project_tasks);
}

fn mark_project_done(db: &DB, project: &Project) -> () {
    match db.update_project_status(&State::Complete, &project.uuid) {
        Ok(_) => println!("Marked project {:?} as done!", project.name),
        Err(e) => println!("Unable to process project, error: {e:?}"),
    };
}

fn mark_project_incubate(db: &DB, project: &Project) -> () {
    match db.update_project_status(&State::Incubate, &project.uuid) {
        Ok(_) => println!("Incubated project {:?}!", project.name),
        Err(e) => println!("Unable to process project, error: {e:?}"),
    };
}

fn mark_project_pending(db: &DB, project: &Project) -> () {
    match db.update_project_status(&State::Pending, &project.uuid) {
        Ok(_) => println!("Marked project {:?} as pending!", project.name),
        Err(e) => println!("Unable to process project, error: {e:?}"),
    };
}

fn delete_project_item(db: &DB, project: &Project) -> () {
    match db.delete_project(&project.uuid) {
        Ok(_) => println!("Successfully removed project {:?}", project.name),
        Err(_) => println!("Failed to remove project {:?}", project.name),
    }
}
