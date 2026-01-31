use serde::Deserialize;
use serde_json::from_str;
use std::error::Error;
use std::path::PathBuf;
use std::process::Command;
use std::str;
use task_hookrs::status::TaskStatus;

use crate::ProjwarriorConfig;
use task_hookrs::task::Task;

pub fn get_task_list(cfg: &ProjwarriorConfig) -> Result<Vec<Task>, Box<dyn Error>> {
    // Try reading from cache if enabled
    if cfg.use_cache
        && let Some(ref task_cache) = cfg.task_cache
    {
        match task_cache.read() {
            Ok(tasks) => return Ok(tasks),
            Err(err) => eprintln!("Failed to read cache: {err} — falling back to live export."),
        }
    }

    // Fallback to live task export
    let tasks = parse_json_from_command::<Vec<Task>>(&cfg.task_path, &["export"])?;

    let filtered_tasks: Vec<Task> = tasks
        .into_iter()
        .filter(|t| t.project().is_some())
        .filter(|t| matches!(t.status(), TaskStatus::Pending | TaskStatus::Waiting))
        .collect();

    if cfg.use_cache
        && let Some(ref task_cache) = cfg.task_cache
        && let Err(err) = task_cache.write(&filtered_tasks)
    {
        eprintln!("Failed to write to cache: {err}");
    }

    Ok(filtered_tasks)
}

pub fn parse_json_from_command<T>(command: &PathBuf, args: &[&str]) -> Result<T, Box<dyn Error>>
where
    T: for<'de> Deserialize<'de>,
{
    let output = Command::new(command).args(args).output()?;

    //NOTE: This sequence can contain invalid chars (e.g. certain emojis, so we need to use lossy here)
    let value = String::from_utf8_lossy(&output.stdout);
    let tasks: T = from_str(&value)?;
    Ok(tasks)
}
