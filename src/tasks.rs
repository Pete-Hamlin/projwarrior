use serde::{Deserialize, Serialize};
use serde_json::from_str;
use std::error::Error;
use std::process::Command;
use std::str;

use crate::GtdConfig;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Task {
    pub id: i32,
    pub brainpower: Option<String>,
    pub priority: Option<String>,
    pub project: Option<String>,
    pub description: String,
    pub end: Option<String>,
    pub entry: String,
    pub modified: String,
    pub status: String,
    pub uuid: String,
    pub tags: Option<Vec<String>>,
    pub urgency: f64,
}

pub fn get_task_list(cfg: &GtdConfig) -> Result<Vec<Task>, Box<dyn Error>> {
    let tasks = parse_json_from_command::<Vec<Task>>(&cfg.task_path, &["export"])?;
    let filtered_tasks = tasks
        .into_iter()
        .filter(|t| t.project != None)
        .filter(|t| t.status == "pending" || t.status == "waiting")
        .collect();
    Ok(filtered_tasks)
}

pub fn parse_json_from_command<T>(command: &str, args: &[&str]) -> Result<T, Box<dyn Error>>
where
    T: for<'de> Deserialize<'de>,
{
    let output = Command::new(command).args(args).output()?;

    // This sequence can contain invalid chars (e.g. certain emojis, so we need to use lossy here)
    let value = String::from_utf8_lossy(&output.stdout);
    let tasks: T = from_str(&value)?;

    Ok(tasks)
}
