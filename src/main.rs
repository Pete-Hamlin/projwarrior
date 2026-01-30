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
        Some(CommandType::Query(arg)) => pc.parse_query(&arg, &args.subcommand).await,
        // Default behaviour - just display list and exit
        None => pc.list_projects(&args.subcommand).await,
    }
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
