use core::fmt;
use rusqlite::types::{FromSql, FromSqlError, ToSql, ToSqlOutput, ValueRef};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
}
