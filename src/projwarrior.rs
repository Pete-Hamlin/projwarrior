use std::fs::remove_file;

use chrono::Utc;
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

    pub async fn init_projects(&mut self) -> Result<String, ProjChampionError> {
        let tasks = self.get_tasks()?;
        let mut name_list: Vec<String> = vec![];
        tasks.into_iter().for_each(|task| {
            let project_name = task.project().unwrap();
            if !name_list.contains(project_name) {
                name_list.push(project_name.clone());
            }
        });
        let proj_count = name_list.len();
        let mut ops = Operations::new();
        for name in name_list {
            self.init_proj(&name, &mut ops).await?;
        }
        self.replica.commit_operations(ops).await?;
        Ok(format!("Successfully initialized {proj_count} projects"))
    }

    pub fn reset_projects(&self) -> Result<String, ProjChampionError> {
        match remove_file(&self.conf.storage_path) {
            Ok(_) => Ok("Successfully removed project list".to_string()),
            Err(_) => Err(ProjChampionError::FileSystem),
        }
    }

    pub async fn list_projects(&mut self, _: &Option<String>) -> Result<String, ProjChampionError> {
        let _ = self.get_tasks()?;
        let projects = self.replica.all_tasks().await?;
        let proj_len = projects.len();

        // let state = match subcommand.as_deref() {
        //     Some("all") => None,
        //     Some("incubate") => Some(&State::Incubate),
        //     Some("done") => Some(&State::Complete),
        //     _ => Some(&State::Pending),
        // };

        Ok(format!("Processed {proj_len} projects"))
    }

    pub async fn count_projects(
        &mut self,
        _: &Option<String>,
    ) -> Result<String, ProjChampionError> {
        let _ = self.get_tasks()?;
        let projects = self.replica.all_task_uuids().await?;
        let proj_len = projects.len();
        // TODO: Add filtering

        Ok(format!("{proj_len}"))
    }

    pub async fn add_project(
        &mut self,
        subcommand: &Option<String>,
    ) -> Result<String, ProjChampionError> {
        if let Some(name) = subcommand.as_deref() {
            let mut ops = Operations::new();
            self.init_proj(name, &mut ops).await?;
            self.replica.commit_operations(ops).await?;
            Ok(format!("Successfully added project {name}"))
        } else {
            Err(ProjChampionError::NoProj)
        }
    }

    async fn init_proj(
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
            Err(_) => Err(ProjChampionError::TaskList),
        }
    }
}
