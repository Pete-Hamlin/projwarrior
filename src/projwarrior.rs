use chrono::Utc;
use futures::executor::block_on;
use task_hookrs::task::Task;
use taskchampion::{Operations, Replica, SqliteStorage, storage::AccessMode};
use uuid::Uuid;

use crate::{
    config::{Cli, ProjwarriorConfig, get_config},
    error::ProjChampionError,
    tasks::get_task_list,
};

pub struct Projchampion {
    conf: ProjwarriorConfig,
    replica: Replica<SqliteStorage>,
}

impl Projchampion {
    pub async fn new(args: &Cli) -> Result<Projchampion, ProjChampionError> {
        let conf = get_config(args);
        let storage = SqliteStorage::new(&conf.storage_path, AccessMode::ReadWrite, true).await?;
        let replica = Replica::new(storage);
        Ok(Projchampion { conf, replica })
    }

    pub async fn init_projects(&mut self) -> Result<(), ProjChampionError> {
        let tasks = self.get_tasks()?;
        let mut name_list: Vec<String> = vec![];
        tasks.into_iter().for_each(|task| {
            let project_name = task.project().unwrap();
            if !name_list.contains(project_name) {
                name_list.push(project_name.clone());
            }
        });
        let mut ops = Operations::new();
        for name in name_list {
            block_on(self.add_project(&name, &mut ops))?;
        }
        self.replica.commit_operations(ops).await?;
        Ok(())
    }
    pub async fn list_projects(&mut self, _: &Option<String>) -> Result<(), ProjChampionError> {
        let _ = self.get_tasks()?;

        // let state = match subcommand.as_deref() {
        //     Some("all") => None,
        //     Some("incubate") => Some(&State::Incubate),
        //     Some("done") => Some(&State::Complete),
        //     _ => Some(&State::Pending),
        // };
        Ok(())
    }
    async fn add_project(
        &mut self,
        name: &str,
        ops: &mut Operations,
    ) -> Result<(), ProjChampionError> {
        let uuid = Uuid::new_v4();
        let mut t = self.replica.create_task(uuid, ops).await?;
        t.set_description(name.into(), ops)?;
        t.set_status(taskchampion::Status::Pending, ops)?;
        t.set_entry(Some(Utc::now()), ops)?;
        Ok(())
    }

    fn get_tasks(&self) -> Result<Vec<Task>, ProjChampionError> {
        match get_task_list(&self.conf) {
            Ok(tasks) => Ok(tasks),
            Err(_) => Err(ProjChampionError::TaskError),
        }
    }
}
