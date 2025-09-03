use serde::Deserialize;
use serde_json::from_str;
use std::error::Error;
use std::process::Command;
use std::str;
use task_hookrs::status::TaskStatus;

use crate::GtdConfig;
use task_hookrs::task::Task;

pub fn get_task_list(cfg: &GtdConfig) -> Result<Vec<Task>, Box<dyn Error>> {
    let tasks = parse_json_from_command::<Vec<Task>>(&cfg.task_path, &["export"])?;
    let filtered_tasks = tasks
        .into_iter()
        .filter(|t| t.project().is_some())
        .filter(|t| t.status() == &TaskStatus::Pending || t.status() == &TaskStatus::Waiting)
        .collect();
    Ok(filtered_tasks)
}

pub fn parse_json_from_command<T>(command: &str, args: &[&str]) -> Result<T, Box<dyn Error>>
where
    T: for<'de> Deserialize<'de>,
{
    let output = Command::new(command).args(args).output()?;

    //NOTE: This sequence can contain invalid chars (e.g. certain emojis, so we need to use lossy here)
    let value = String::from_utf8_lossy(&output.stdout);
    let tasks: T = from_str(&value)?;
    Ok(tasks)
}
