use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;

use crate::config::GtdConfig;
use crate::parser::Task;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub name: String,
    // tags: Option<Vec<String>>,
}

impl Project {
    pub fn get_tasks(&self, tasks: &[Task]) -> i32 {
        let count = tasks
            .into_iter()
            .filter(|t| t.project.clone().unwrap_or_default() == self.name)
            .count();
        return count as i32;
    }
}

pub fn get_projects(cfg: &GtdConfig) -> Result<Vec<Project>, Box<dyn Error>> {
    let file = File::open(&cfg.storage_path)
        .expect("Project storage file not found - Check your config location");
    let projects: Vec<Project> = match serde_json::from_reader(file) {
        Ok(projects) => projects,
        Err(_) => vec![],
    };
    Ok(projects)
}

pub fn write_project_list(cfg: &GtdConfig, projects: &[Project]) -> Result<(), Box<dyn Error>> {
    serde_json::to_writer(&File::create(&cfg.storage_path)?, &projects)?;
    Ok(())
}
