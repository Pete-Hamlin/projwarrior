use std::str::FromStr;

use chrono::Utc;
use task_hookrs::task::Task;
use taskchampion::{Operations, Replica, ServerConfig, SqliteStorage, Status, storage::AccessMode};
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
    columns: Vec<Column>,
}

impl Projchampion {
    pub async fn new(args: &Cli) -> Result<Projchampion, ProjChampionError> {
        let conf = get_config(args);
        let storage = SqliteStorage::new(&conf.storage_path, AccessMode::ReadWrite, true).await?;
        let replica = Replica::new(storage);
        let columns = vec![
            Column::Id,
            Column::Uuid,
            Column::Status,
            Column::Name,
            Column::Tasks,
            Column::Entry,
        ];

        Ok(Projchampion {
            conf,
            replica,
            columns,
        })
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
        ops.push(taskchampion::Operation::UndoPoint);
        for name in name_list {
            self.init_proj(&name, &mut ops).await?;
        }
        self.replica.commit_operations(ops).await?;
        Ok(format!("Successfully initialized {proj_count} projects"))
    }

    pub async fn undo_last(&mut self) -> Result<String, ProjChampionError> {
        let ops = self.replica.get_undo_operations().await?;
        let num_ops = ops.len();
        match self.replica.commit_reversed_operations(ops).await? {
            true => Ok(format!("Reverted last {num_ops} operation")),
            false => Err(ProjChampionError::InvalidUndo),
        }
    }

    pub async fn sync_projects(&mut self) -> Result<String, ProjChampionError> {
        // Check our server config values are present
        let url = match &self.conf.sync_url {
            Some(url) => url.clone(),
            _ => {
                return Err(ProjChampionError::Sync(
                    "Invalid sync server URL.".to_string(),
                ));
            }
        };

        let client_id = match self.conf.sync_client_id {
            Some(client_id) => client_id,
            _ => {
                return Err(ProjChampionError::Sync(
                    "Invalid sync server client_id.".to_string(),
                ));
            }
        };

        let encryption_secret = match &self.conf.sync_secret {
            Some(secret) => secret.clone().as_bytes().to_vec(),
            _ => {
                return Err(ProjChampionError::Sync(
                    "Invalid sync server secret.".to_string(),
                ));
            }
        };
        let server_conf = ServerConfig::Remote {
            url,
            client_id,
            encryption_secret,
        };
        let mut server = server_conf.into_server().await?;
        let changes = self.replica.num_local_operations().await?;
        self.replica.sync(&mut server, true).await?;
        Ok(format!("Synced {changes} to server."))
    }

    pub async fn list_projects(
        &mut self,
        subcommand: &Option<String>,
    ) -> Result<String, ProjChampionError> {
        let tasks = self.get_tasks()?;
        let projects: Vec<taskchampion::Task> = match subcommand {
            Some(subcommand) => {
                let working_set = self.replica.working_set().await?;
                let filter = match FilterType::from_str(subcommand.as_str()) {
                    Ok(filter) => filter,
                    Err(_) => {
                        return Err(ProjChampionError::SubCommand(subcommand.to_string()));
                    }
                };
                let all_proj: Vec<taskchampion::Task> = self.replica.pending_tasks().await?;
                self.filter_projects(&filter, &all_proj, &working_set)
                    .await?
            }
            None => self.replica.pending_tasks().await?,
        };
        let working_set = self.replica.working_set().await?;

        project_list_table(&self.conf, &tasks, &projects, &working_set, &self.columns);
        let notices = self.get_unsynced_changes().await?;
        Ok(notices)
    }

    pub async fn all_projects(
        &mut self,
        subcommand: &Option<String>,
    ) -> Result<String, ProjChampionError> {
        let tasks = self.get_tasks()?;
        let projects: Vec<taskchampion::Task> = match subcommand {
            Some(subcommand) => {
                let working_set = self.replica.working_set().await?;
                let filter = match FilterType::from_str(subcommand.as_str()) {
                    Ok(filter) => filter,
                    Err(_) => {
                        return Err(ProjChampionError::SubCommand(subcommand.to_string()));
                    }
                };
                let all_proj: Vec<taskchampion::Task> =
                    self.replica.all_tasks().await?.values().cloned().collect();
                self.filter_projects(&filter, &all_proj, &working_set)
                    .await?
            }
            None => self.replica.all_tasks().await?.values().cloned().collect(),
        };

        let working_set = self.replica.working_set().await?;

        project_list_table(&self.conf, &tasks, &projects, &working_set, &self.columns);
        let notices = self.get_unsynced_changes().await?;
        Ok(notices)
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
            ops.push(taskchampion::Operation::UndoPoint);
            self.init_proj(name, &mut ops).await?;
            self.replica.commit_operations(ops).await?;
            println!("Successfully added project {name}");
            let notices = self.get_unsynced_changes().await?;
            Ok(notices)
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
                        "delete" => self.mark_project_deleted(&mut proj).await?,
                        _ => return Err(ProjChampionError::SubCommand(subcommand.to_string())),
                    };
                } else {
                    // If no subcommand, just show project details
                    self.show_project(&proj).await?;
                }
            }
            2.. => {
                project_list_table(
                    &self.conf,
                    &tasks,
                    &filtered_projects,
                    &working_set,
                    &self.columns,
                );
            }
        };
        let notices = self.get_unsynced_changes().await?;
        Ok(notices)
    }

    pub async fn get_unsynced_changes(&mut self) -> Result<String, ProjChampionError> {
        let changes = self.replica.num_local_operations().await?;
        let output = match changes {
            0 => "Up to date.".to_string(),
            _ => format!("You have {changes} unsynced changes locally."),
        };
        Ok(output)
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
        ops.push(taskchampion::Operation::UndoPoint);
        proj.done(&mut ops)?;
        self.replica.commit_operations(ops).await?;
        self.replica.rebuild_working_set(false).await?;
        Ok(())
    }

    async fn mark_project_deleted(
        &mut self,
        proj: &mut taskchampion::Task,
    ) -> Result<(), ProjChampionError> {
        let mut ops = Operations::new();
        ops.push(taskchampion::Operation::UndoPoint);
        proj.set_status(Status::Deleted, &mut ops)?;
        self.replica.commit_operations(ops).await?;
        self.replica.rebuild_working_set(false).await?;
        Ok(())
    }
}
