use chrono::{DateTime, Utc};
use core::fmt;
use rusqlite::types::{FromSql, FromSqlError, ToSql, ToSqlOutput, ValueRef};
use serde::{Deserialize, Serialize};
use task_hookrs::task::Task;
use uuid::Uuid;

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq)]
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

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub name: String,
    pub state: State,
    pub id: Option<u32>,
    pub uuid: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Project {
    pub fn get_tasks(&self, tasks: &[Task]) -> i32 {
        let count = tasks
            .into_iter()
            .filter(|t| t.project().as_deref() == Some(&self.name))
            .count();
        return count as i32;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_state_display() {
        assert_eq!(State::Pending.to_string(), "Pending");
        assert_eq!(State::Complete.to_string(), "Complete");
        assert_eq!(State::Incubate.to_string(), "Incubate");
    }
    #[test]
    fn test_state_from_sql() {
        assert_eq!(
            State::column_result(ValueRef::Text(b"Pending")).unwrap(),
            State::Pending
        );
        assert_eq!(
            State::column_result(ValueRef::Text(b"Complete")).unwrap(),
            State::Complete
        );
        assert_eq!(
            State::column_result(ValueRef::Text(b"Incubate")).unwrap(),
            State::Incubate
        );
        assert!(State::column_result(ValueRef::Text(b"Unknown")).is_err());
    }

    #[test]
    fn test_project_get_tasks() {
        let project = Project {
            name: "Test Project".to_string(),
            state: State::Pending,
            id: Some(1),
            uuid: Uuid::new_v4(),
        };

        // TODO: Make this actually work  intended (parse JSON output)
        // let tasks = vec![
        //     Task::new(),
        //     Task::new().project(Some("Other Project".to_string())),
        //     Task::new().project(Some("Test Project".to_string())),
        // ];
        //
        // assert_eq!(project.get_tasks(&tasks), 2);
    }

    #[test]
    fn test_project_default() {
        let project = Project::default();
        assert_eq!(project.name, "");
        assert_eq!(project.state, State::Pending);
        assert_eq!(project.id, 0);
    }
}
