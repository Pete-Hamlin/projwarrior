use clap::Parser;
use serde::{Deserialize, Serialize};
use std::env;
use std::process::Command;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    /// Command to run (init, list, add, reset or a project ID)
    pub command: Option<String>,
    /// Optional subcommand to work on
    pub subcommand: Option<String>,

    /// Display all projects (Override config)
    #[clap(short, long)]
    pub long: bool,

    /// Display only projects without tasks
    #[clap(short, long)]
    pub short: bool,

    /// Display output with color
    #[clap(short, long)]
    pub color: bool,
}

// Config setup
#[derive(Serialize, Deserialize)]
pub struct GtdConfig {
    pub storage_path: String,
    pub task_path: String,
    pub short: bool,
    pub color: bool,
}

impl ::std::default::Default for GtdConfig {
    fn default() -> Self {
        Self {
            task_path: get_task_bin(),
            storage_path: env::var("HOME").unwrap() + "/.local/share/projwarrior/projects.sqlite",
            short: false,
            color: true,
        }
    }
}

fn get_task_bin() -> String {
    // Check if `task` in current path
    let task_bin = Command::new("which").arg("task").output().expect(
        "Failed to find task binary - please ensure the `task` command is available in your $PATH",
    );
    return String::from_utf8(task_bin.stdout)
        .unwrap()
        .trim()
        .to_owned();
}

pub fn get_config(args: &Cli) -> GtdConfig {
    // Load config
    let cfg: GtdConfig = confy::load("projwarrior", None).expect("Failed to load config");
    // Overwrite config file with CLI options
    return GtdConfig {
        short: if args.short || args.long {
            !args.long
        } else {
            cfg.short
        },
        color: if args.color { true } else { cfg.color },
        ..cfg
    };
}
