#![recursion_limit = "1024"]
mod cache;
mod config;
mod db;
mod error;
mod filters;
mod project;
mod projwarrior;
mod table;
mod tasks;

use clap::Parser;
use config::{Cli, ProjwarriorConfig, get_config};
use db::DB;
use futures::executor::block_on;
use project::{Project, State};
use std::{error::Error, fs::remove_file};
use table::{project_details_table, project_list_table};
use task_hookrs::task::Task;
use tasks::get_task_list;
use uuid::Uuid;

use crate::{
    config::{CommandType, FilterType},
    error::ProjChampionError,
    filters::ProjectFilter,
    projwarrior::Projchampion,
    table::Column,
};

fn main() -> Result<(), String> {
    let args = Cli::parse();

    let mut pc = match block_on(Projchampion::new(&args)) {
        Ok(pc) => pc,
        Err(e) => return Err(format!("Unable to initialize projchampion - {e:?}")),
    };

    match block_on(match_arg(&args, &mut pc)) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Unable to initialize projchampion - {e:?}")),
    }
}

async fn match_arg(args: &Cli, pc: &mut Projchampion) -> Result<(), ProjChampionError> {
    match args.command {
        Some(CommandType::Init) => pc.init_projects().await,
        // Some(CommandType::Reset) => reset_projects(&cfg),
        // Some(CommandType::List) => list_projects(&cfg, &db, &args.subcommand),
        // Some(CommandType::Count) => count_projects(&cfg, &db, &args),
        // Some(CommandType::Add) => add_project(&mut db, &args),
        // Some(CommandType::Query(arg)) => parse_filter(&cfg, &db, &arg, &args.subcommand),
        // Default behaviour - just display list and exit
        // None => list_projects(&cfg, &db, &args.subcommand),
        _ => pc.list_projects(&args.subcommand).await,
    }
}

fn reset_projects(cfg: &ProjwarriorConfig) -> Result<(), String> {
    match remove_file(&cfg.storage_path) {
        Ok(_p) => println!("Successfully removed project list"),
        Err(e) => return Err(format!("Failed to remove project list: {e:?}")),
    }
    Ok(())
}

fn list_projects(
    cfg: &ProjwarriorConfig,
    db: &DB,
    subcommand: &Option<String>,
) -> Result<(), String> {
    let tasks = match get_task_list(cfg) {
        Ok(tasks) => tasks,
        Err(e) => return Err(format!("Unable to retrieve task list - {e:?}")),
    };
    let state = match subcommand.as_deref() {
        Some("all") => None,
        Some("incubate") => Some(&State::Incubate),
        Some("done") => Some(&State::Complete),
        _ => Some(&State::Pending),
    };
    let filters = match state {
        Some(filter) => ProjectFilter::builder().state(filter).build(),
        None => ProjectFilter::builder().build(),
    };

    let projects = match db.get_projects(&filters) {
        Ok(proj_list) => proj_list,
        Err(e) => return Err(format!("Failed to retrieve project list - {e:?}")),
    };

    let columns = match state {
        Some(&State::Complete) => vec![Column::Uuid, Column::Name, Column::Tasks, Column::Entry],
        Some(_) => vec![Column::Id, Column::Name, Column::Tasks, Column::Entry],
        None => vec![
            Column::Id,
            Column::Uuid,
            Column::State,
            Column::Name,
            Column::Tasks,
            Column::Entry,
        ],
    };

    project_list_table(cfg, &tasks, &projects, &columns);
    Ok(())
}

fn count_projects(cfg: &ProjwarriorConfig, db: &DB, args: &Cli) -> Result<(), String> {
    let tasks = match get_task_list(cfg) {
        Ok(tasks) => tasks,
        Err(e) => return Err(format!("Unable to retrieve task list - {e:?}")),
    };
    let state = match args.subcommand.as_deref() {
        Some("all") => None,
        Some("incubate") => Some(&State::Incubate),
        Some("done") => Some(&State::Complete),
        _ => Some(&State::Pending),
    };
    let filters = match state {
        Some(filter) => ProjectFilter::builder().state(filter).build(),
        None => ProjectFilter::builder().build(),
    };

    let projects = match db.get_projects(&filters) {
        Ok(proj_list) => proj_list,
        Err(e) => return Err(format!("Failed to retrieve project list - {e:?}")),
    };

    if !cfg.short {
        let count = projects.len();
        println!("{:?}", count)
    } else {
        let count = projects
            .into_iter()
            .filter(|p| p.get_tasks(&tasks) == 0)
            .count();
        println!("{:?}", count)
    }
    Ok(())
}

fn add_project(db: &mut DB, args: &Cli) -> Result<(), String> {
    if let Some(subcommand) = args.subcommand.as_deref() {
        let project = vec![Project {
            name: subcommand.to_string(),
            uuid: Uuid::new_v4(),
            ..Default::default()
        }];
        match db.insert_projects(&project) {
            Ok(_p) => println!("Successfully added project {:?}", subcommand.to_string()),
            Err(e) => return Err(format!("Failed to add project {e:?}")),
        }
    } else {
        println!("No task specified - run `proj --help` for guidance on running this command")
    }
    Ok(())
}

fn parse_filter(
    cfg: &ProjwarriorConfig,
    db: &DB,
    filter_enum: &FilterType,
    subcommand: &Option<String>,
) -> Result<(), String> {
    let filters = match filter_enum {
        FilterType::ID(id) => ProjectFilter::builder().id(id),
        FilterType::Uuid(uuid) => ProjectFilter::builder().uuid(uuid),
        FilterType::Filter(string) => ProjectFilter::builder().name(string),
    };
    let query = filters.build();

    let projects = db.get_projects(&query).unwrap();

    if projects.len() > 1 {
        let tasks = get_task_list(cfg).expect("Failed to get task list");
        let projects = match db.get_projects(&query) {
            Ok(projects) => projects,
            Err(e) => return Err(format!("Unable to query project list - {e:?}")),
        };
        project_list_table(
            cfg,
            &tasks,
            &projects,
            &[Column::Id, Column::Name, Column::Tasks, Column::Entry],
        );
    } else {
        let project = match db.get_projects(&query) {
            Ok(p) => p.into_iter().next().unwrap(),
            Err(e) => return Err(format!("Unable to parse project entity - {e:?}")),
        };
        if let Some(subcommand) = subcommand.as_deref() {
            match subcommand {
                "show" => show_project(cfg, &project),
                "done" => mark_project_done(db, &project),
                "incubate" => mark_project_incubate(db, &project),
                "pending" => mark_project_pending(db, &project),
                "delete" => delete_project_item(db, &project),
                _ => return Err(format!("Subcommand {subcommand} not found")),
            };
        } else {
            // If no subcommand, just show project details
            show_project(cfg, &project);
        }
    }
    Ok(())
}

fn show_project(cfg: &ProjwarriorConfig, project: &Project) {
    let tasks = get_task_list(cfg).expect("Failed to get task list");
    let project_tasks: Vec<Task> = tasks
        .iter()
        .filter(|t| t.project() == Some(&project.name))
        .cloned()
        .collect();
    project_details_table(cfg, project, &project_tasks);
}

fn mark_project_done(db: &DB, project: &Project) {
    match db.update_project_status(&State::Complete, &project.uuid) {
        Ok(_) => println!(
            "Marked project {:?} - {:?} as done!",
            project.name, project.uuid
        ),
        Err(e) => println!("Unable to process project, error: {e:?}"),
    };
}

fn mark_project_incubate(db: &DB, project: &Project) {
    match db.update_project_status(&State::Incubate, &project.uuid) {
        Ok(_) => println!("Incubated project {:?} - {:?}!", project.name, project.uuid),
        Err(e) => println!("Unable to process project, error: {e:?}"),
    };
}

fn mark_project_pending(db: &DB, project: &Project) {
    match db.update_project_status(&State::Pending, &project.uuid) {
        Ok(_) => println!(
            "Marked project {:?} - {:?} as pending!",
            project.name, project.uuid
        ),
        Err(e) => println!("Unable to process project, error: {e:?}"),
    };
}

fn delete_project_item(db: &DB, project: &Project) {
    match db.delete_project(&project.uuid) {
        Ok(_) => println!(
            "Successfully removed project {:?} - {:?}",
            project.name, project.uuid
        ),
        Err(_) => println!("Failed to remove project {:?}", project.name),
    }
}
