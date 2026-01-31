#![recursion_limit = "1024"]
mod cache;
mod config;
mod error;
mod projwarrior;
mod table;
mod tasks;

use clap::Parser;
use config::{Cli, ProjwarriorConfig};

use crate::{config::CommandType, error::ProjChampionError, projwarrior::Projchampion};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), String> {
    let args = Cli::parse();

    let mut pc = match Projchampion::new(&args).await {
        Ok(pc) => pc,
        Err(e) => return Err(format!("Error: Unable to initialize projchampion - {e:?}")),
    };

    match match_arg(args, &mut pc).await {
        Ok(result) => println!("{result}"),
        Err(e) => println!("Error: {e:?}"),
    };
    Ok(())
}

async fn match_arg(args: Cli, pc: &mut Projchampion) -> Result<String, ProjChampionError> {
    match args.command {
        Some(CommandType::Import) => pc.init_projects().await,
        Some(CommandType::Undo) => pc.undo_last().await,
        Some(CommandType::Sync) => pc.sync_projects().await,
        Some(CommandType::List) => pc.list_projects(&args.subcommand).await,
        Some(CommandType::All) => pc.all_projects(&args.subcommand).await,
        Some(CommandType::Count) => pc.count_projects(&args.subcommand).await,
        Some(CommandType::Add) => pc.add_project(&args.subcommand).await,
        Some(CommandType::Query(arg)) => pc.parse_query(&arg, &args.subcommand).await,
        // Default behaviour - just display list and exit
        None => pc.list_projects(&args.subcommand).await,
    }
}
