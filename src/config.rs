use crate::cache::Cache;
use crate::error::ConfigError;
use clap::Parser;
use dirs::data_dir;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum CommandType {
    Init,
    Reset,
    List,
    Count,
    Add,
    Query(FilterType),
}

impl FromStr for CommandType {
    type Err = String;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input.to_lowercase().as_str() {
            "init" => Ok(CommandType::Init),
            "reset" => Ok(CommandType::Reset),
            "list" => Ok(CommandType::List),
            "count" => Ok(CommandType::Count),
            "add" => Ok(CommandType::Add),
            _ => {
                let query = FilterType::from_str(input)?;
                Ok(CommandType::Query(query))
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum FilterType {
    ID(u32),
    Uuid(Uuid),
    Filter(String),
}

impl FromStr for FilterType {
    type Err = String;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        if let Ok(id) = input.parse::<u32>() {
            Ok(Self::ID(id))
        } else if let Ok(uuid) = Uuid::parse_str(input) {
            Ok(Self::Uuid(uuid))
        } else {
            // Default is to treat arg as a string filter
            Ok(Self::Filter(input.to_string()))
        }
    }
}

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    /// Command to run (init, list, add, reset or a project ID)
    // #[arg(value_parser = clap::builder::ValueParser::from_str::<CommandType>(), required = false)]
    #[arg(value_parser = clap::value_parser!(CommandType), required = false)]
    pub command: Option<CommandType>,
    /// Optional subcommand to work on
    pub subcommand: Option<String>,

    /// Display all projects (Override config)
    #[clap(short, long)]
    pub long: bool,

    /// Force a refresh of the cache file
    #[clap(short, long)]
    pub refresh: bool,

    /// Disable cache entirely for this run
    #[clap(short, long)]
    pub no_cache: bool,

    /// Display only projects without tasks
    #[clap(short, long)]
    pub short: bool,

    /// Display output with color
    #[clap(short, long)]
    pub color: bool,
}

// Config setup
#[derive(Serialize, Deserialize)]
pub struct ProjwarriorConfig {
    pub storage_path: PathBuf,
    pub task_path: PathBuf,
    pub short: bool,
    pub color: bool,
    pub use_cache: bool,
    pub cache_length: u64,
    pub cache: Option<Cache>,
}

impl ::std::default::Default for ProjwarriorConfig {
    fn default() -> Self {
        let task_path = get_task_bin().unwrap();
        let storage_path = PathBuf::from_iter([
            data_dir().unwrap(),
            PathBuf::from_str("projwarrior/projects.sqlite").unwrap(),
        ]);
        Self {
            task_path,
            storage_path,
            short: false,
            color: true,
            use_cache: true,
            cache_length: 60,
            cache: None,
        }
    }
}

fn get_task_bin() -> Result<PathBuf, ConfigError> {
    // Check if `task` in current path
    let task_bin = match Command::new("which").arg("task").output() {
        Ok(task) => task,
        Err(_) => {
            return Err(ConfigError::NoTask);
        }
    };
    Ok(PathBuf::from(
        String::from_utf8(task_bin.stdout)
            .unwrap()
            .trim()
            .to_owned(),
    ))
}

pub fn get_config(args: &Cli) -> ProjwarriorConfig {
    // Load config
    let cfg: ProjwarriorConfig = confy::load("projwarrior", None).expect("Failed to load config");
    let cache = match cfg.use_cache {
        true => Cache::new(cfg.cache_length, dirs::cache_dir(), args.refresh),
        false => None,
    };
    // Overwrite config file with CLI options
    ProjwarriorConfig {
        short: if args.short || args.long {
            !args.long
        } else {
            cfg.short
        },
        cache: if args.no_cache { None } else { cache },
        color: if args.color { true } else { cfg.color },
        ..cfg
    }
}
