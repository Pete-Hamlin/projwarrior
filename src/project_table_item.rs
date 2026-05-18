use chrono::{DateTime, Utc};
use task_hookrs::task::Task;
use taskchampion::{Status, WorkingSet};
use uuid::Uuid;

#[derive(Debug)]
pub struct ProjectTableItem {
    pub name: String,
    pub id: Option<usize>,
    pub uuid: Uuid,
    pub status: Status,
    pub tasks: i32,
    pub entry: Option<DateTime<Utc>>,
}

impl ProjectTableItem {
    pub fn from_project(
        project: &taskchampion::Task,
        tasks: &[Task],
        working_set: &WorkingSet,
    ) -> ProjectTableItem {
        let name = project.get_description().to_string();
        let uuid = project.get_uuid();
        ProjectTableItem {
            tasks: get_tasks(&name, tasks),
            id: working_set.by_uuid(uuid),
            status: project.get_status(),
            entry: project.get_entry(),
            name,
            uuid,
        }
    }
}

fn get_tasks(name: &str, tasks: &[Task]) -> i32 {
    let count = tasks
        .iter()
        .filter(|t| t.project() == Some(&name.to_string()))
        .count();
    count as i32
}
