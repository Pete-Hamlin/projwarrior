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
            storage_path: env::var("HOME").unwrap() + "/.task/projects.data",
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
    return String::from_utf8(task_bin.stdout).unwrap();
}

pub fn get_config(args: &Cli) -> GtdConfig {
    // Load config
    let cfg: GtdConfig = confy::load("projwarrior", None).expect("Failed to load config");
    // Overwrite config file with CLI options
    return GtdConfig {
        short: if args.short { true } else { cfg.short },
        color: if args.color { true } else { cfg.color },
        ..cfg
    };
}

//#[cfg(test)]
//mod tests {
//    use super::*;
//    use std::env;
//    use std::fs;
//
//    #[test]
//    fn test_get_task_bin() {
//        // Create a temporary directory and add a fake `task` binary
//        let temp_dir = tempfile::tempdir().unwrap();
//        let task_path = temp_dir.path().join("task");
//        fs::write(&task_path, "#!/bin/sh\necho task").unwrap();
//        env::set_var("PATH", temp_dir.path());
//
//        // Call the function and check the result
//        let result = get_task_bin();
//        assert!(result.contains("task"));
//    }
//
//    #[test]
//    fn test_get_config_default() {
//        // Create a default CLI struct
//        let args = Cli {
//            command: None,
//            subcommand: None,
//            short: false,
//        };
//
//        // Call the function and check the result
//        let config = get_config(&args);
//        assert_eq!(config.short, false);
//        assert!(config.storage_path.contains("/.task/projects.data"));
//    }
//
//    #[test]
//    fn test_get_config_with_args() {
//        // Create a CLI struct with short option set to true
//        let args = Cli {
//            command: None,
//            subcommand: None,
//            short: true,
//        };
//
//        // Call the function and check the result
//        let config = get_config(&args);
//        assert_eq!(config.short, true);
//    }
//}
