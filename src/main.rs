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
use config::{Cli, ProjwarriorConfig};
use db::DB;
use futures::executor::block_on;
use project::{Project, State};
use table::project_details_table;
use task_hookrs::task::Task;
use tasks::get_task_list;

use crate::{config::CommandType, error::ProjChampionError, projwarrior::Projchampion};

fn main() -> Result<(), String> {
    let args = Cli::parse();

    let mut pc = match block_on(Projchampion::new(&args)) {
        Ok(pc) => pc,
        Err(e) => return Err(format!("Error: Unable to initialize projchampion - {e:?}")),
    };

    match block_on(match_arg(args, &mut pc)) {
        Ok(result) => println!("{result}"),
        Err(e) => println!("Error: {e:?}"),
    };
    Ok(())
}

async fn match_arg(args: Cli, pc: &mut Projchampion) -> Result<String, ProjChampionError> {
    match args.command {
        Some(CommandType::Init) => pc.init_projects().await,
        Some(CommandType::Reset) => pc.reset_projects(),
        Some(CommandType::List) => pc.list_projects(&args.subcommand).await,
        Some(CommandType::Count) => pc.count_projects(&args.subcommand).await,
        Some(CommandType::Add) => pc.add_project(&args.subcommand).await,
        Some(CommandType::Query(arg)) => pc.parse_filter(&arg, &args.subcommand).await,
        // Default behaviour - just display list and exit
        None => pc.list_projects(&args.subcommand).await,
    }
}

// fn parse_filter(
//     cfg: &ProjwarriorConfig,
//     db: &DB,
//     filter_enum: &FilterType,
//     subcommand: &Option<String>,
// ) -> Result<(), String> {
//     let filters = match filter_enum {
//         FilterType::ID(id) => ProjectFilter::builder().id(id),
//         FilterType::Uuid(uuid) => ProjectFilter::builder().uuid(uuid),
//         FilterType::Filter(string) => ProjectFilter::builder().name(string),
//     };
//     let query = filters.build();
//     let projects = db.get_projects(&query).unwrap();
//     if projects.len() > 1 {
//         let tasks = get_task_list(cfg).expect("Failed to get task list");
//         let projects = match db.get_projects(&query) {
//             Ok(projects) => projects,
//             Err(e) => return Err(format!("Unable to query project list - {e:?}")),
//         };
//         project_list_table(
//             cfg,
//             &tasks,
//             &projects,
//             &[Column::Id, Column::Name, Column::Tasks, Column::Entry],
//         );
//     } else {
//         let project = match db.get_projects(&query) {
//             Ok(p) => p.into_iter().next().unwrap(),
//             Err(e) => return Err(format!("Unable to parse project entity - {e:?}")),
//         };
//         if let Some(subcommand) = subcommand.as_deref() {
//             match subcommand {
//                 "show" => show_project(cfg, &project),
//                 "done" => mark_project_done(db, &project),
//                 "incubate" => mark_project_incubate(db, &project),
//                 "pending" => mark_project_pending(db, &project),
//                 "delete" => delete_project_item(db, &project),
//                 _ => return Err(format!("Subcommand {subcommand} not found")),
//             };
//         } else {
//             // If no subcommand, just show project details
//             show_project(cfg, &project);
//         }
//     }
//     Ok(())
// }

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
