use core::fmt;
use rusqlite::types::{FromSql, FromSqlError, ToSql, ToSqlOutput, ValueRef};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use uuid::Uuid;

use crate::config::GtdConfig;
use crate::parser::Task;

#[derive(Debug, Default, Serialize, Deserialize)]
pub enum State {
    #[default]
    Pending,
    Complete,
    Incubate,
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            State::Pending => write!(f, "Pending"),
            State::Complete => write!(f, "Complete"),
            State::Incubate => write!(f, "Incubate"),
        }
    }
}

impl ToSql for State {
    fn to_sql(&self) -> Result<ToSqlOutput, rusqlite::Error> {
        Ok(ToSqlOutput::from(self.to_string()))
    }
}

impl FromSql for State {
    fn column_result(value: ValueRef) -> Result<Self, FromSqlError> {
        match value.as_str()? {
            "Pending" => Ok(State::Pending),
            "Complete" => Ok(State::Complete),
            "Incubate" => Ok(State::Incubate),
            _ => Err(FromSqlError::InvalidType),
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub name: String,
    pub state: State,
    pub id: u32,
    pub uuid: Uuid,
}

impl Project {
    pub fn get_tasks(&self, tasks: &[Task]) -> i32 {
        let count = tasks
            .into_iter()
            .filter(|t| t.project.clone().unwrap_or_default() == self.name)
            .count();
        return count as i32;
    }

    pub fn mark_complete(&mut self) {
        self.state = State::Complete;
    }

    pub fn mark_pending(&mut self) {
        self.state = State::Pending;
    }

    pub fn mark_incubate(&mut self) {
        self.state = State::Incubate;
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
