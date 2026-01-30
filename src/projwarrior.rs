use std::fs::remove_file;

use chrono::Utc;
use task_hookrs::task::Task;
use taskchampion::{Operations, Replica, SqliteStorage, storage::AccessMode};
use uuid::Uuid;

use crate::{
    config::{Cli, FilterType, ProjwarriorConfig, get_config},
    error::ProjChampionError,
    table::{Column, project_details_table, project_list_table},
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
        // TODO: Add confirmation to this action
        match remove_file(&self.conf.storage_path) {
            Ok(_) => Ok("Successfully removed project list".to_string()),
            Err(_) => Err(ProjChampionError::FileSystem),
        }
    }

    pub async fn list_projects(&mut self, _: &Option<String>) -> Result<String, ProjChampionError> {
        let tasks = self.get_tasks()?;
        let projects = self.replica.pending_tasks().await?;
        let working_set = self.replica.working_set().await?;

        let columns = vec![
            Column::Id,
            Column::Uuid,
            Column::Status,
            Column::Name,
            Column::Tasks,
            Column::Entry,
        ];
        project_list_table(&self.conf, &tasks, &projects, &working_set, &columns);
        Ok("Done.".to_string())
    }

    pub async fn count_projects(
        &mut self,
        _: &Option<String>,
    ) -> Result<String, ProjChampionError> {
        // let _ = self.get_tasks()?;
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

    pub async fn parse_query(
        &mut self,
        filter: &FilterType,
        subcommand: &Option<String>,
    ) -> Result<String, ProjChampionError> {
        let tasks = self.get_tasks()?;
        let projects = self.replica.pending_tasks().await?;
        let working_set = self.replica.working_set().await?;
        let filtered_projects = self
            .filter_projects(filter, &projects, &working_set)
            .await?;
        match filtered_projects.len() {
            0 => println!("No projects match given filter."),
            1 => {
                let mut proj = filtered_projects.into_iter().next().unwrap();
                if let Some(subcommand) = subcommand.as_deref() {
                    match subcommand {
                        "show" => self.show_project(&proj).await?,
                        "done" => self.mark_project_done(&mut proj).await?,
                        // "pending" => mark_project_pending(db, &project),
                        // "delete" => delete_project_item(db, &project),
                        _ => return Err(ProjChampionError::SubCommand(subcommand.to_string())),
                    };
                } else {
                    // If no subcommand, just show project details
                    self.show_project(&proj).await?;
                }
            }
            2.. => {
                let columns = vec![
                    Column::Id,
                    Column::Uuid,
                    Column::Status,
                    Column::Name,
                    Column::Tasks,
                    Column::Entry,
                ];
                project_list_table(
                    &self.conf,
                    &tasks,
                    &filtered_projects,
                    &working_set,
                    &columns,
                );
            }
        };
        Ok("Done.".to_string())
    }

    async fn filter_projects(
        &mut self,
        filter: &FilterType,
        projects: &[taskchampion::Task],
        working_set: &taskchampion::WorkingSet,
    ) -> Result<Vec<taskchampion::Task>, ProjChampionError> {
        let filtered_projects: Vec<taskchampion::Task> = match filter {
            FilterType::ID(id) => projects
                .iter()
                .filter(|p| working_set.by_uuid(p.get_uuid()) == Some(*id))
                .cloned()
                .collect(),
            FilterType::Uuid(uuid) => projects
                .iter()
                .filter(|p| p.get_uuid() == *uuid)
                .cloned()
                .collect(),
            FilterType::Filter(string) => projects
                .iter()
                .filter(|p| p.get_description().contains(string))
                .cloned()
                .collect(),
        };
        Ok(filtered_projects)
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

    async fn show_project(&mut self, proj: &taskchampion::Task) -> Result<(), ProjChampionError> {
        let tasks = self.get_tasks()?;
        let id = self.replica.working_set().await?.by_uuid(proj.get_uuid());
        let project_tasks: Vec<Task> = tasks
            .iter()
            .filter(|t| t.project() == Some(&proj.get_description().to_string()))
            .cloned()
            .collect();
        project_details_table(&self.conf, proj, &project_tasks, id);
        Ok(())
    }

    async fn mark_project_done(
        &mut self,
        proj: &mut taskchampion::Task,
    ) -> Result<(), ProjChampionError> {
        let mut ops = Operations::new();
        proj.done(&mut ops)?;
        self.replica.commit_operations(ops).await?;
        Ok(())
    }
}
