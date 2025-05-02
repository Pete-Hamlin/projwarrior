use crate::{
    config::GtdConfig,
    project::{Project, State},
};
use uuid::Uuid;

use rusqlite::{Connection, Result, params};

pub fn check_db(cfg: &GtdConfig) -> Result<()> {
    let conn = Connection::open(&cfg.storage_path)?;
    let result: Result<i32> = conn.query_row(
        "SELECT 1 FROM sqlite_master WHERE type='table' AND name='project'",
        [],
        |row| row.get(0),
    );
    match result {
        Ok(_) => (),
        Err(_) => init_db(&conn)?,
    };
    Ok(())
}

/// Sets up initial database tables
///
/// * `conn`: rusqlite database connection
fn init_db(conn: &Connection) -> Result<()> {
    println!("Initializing projects db...");
    conn.execute(
        "CREATE TABLE IF NOT EXISTS project (
            uuid TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            state TEXT NOT NULL
        )",
        [],
    )?;
    Ok(())
}

pub fn insert_projects(cfg: &GtdConfig, projects: &[Project]) -> Result<()> {
    let mut conn = Connection::open(&cfg.storage_path)?;
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare("INSERT INTO project (uuid, name, state) VALUES (?1, ?2, ?3)")?;
        for project in projects {
            stmt.execute(params![
                project.uuid.to_string(),
                project.name,
                project.state.to_string()
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}

pub fn update_project_status(cfg: &GtdConfig, state: &State, project_id: &Uuid) -> Result<()> {
    let conn = Connection::open(&cfg.storage_path)?;
    conn.execute(
        "UPDATE project SET state = ?1 WHERE uuid = ?2",
        params![state.to_string(), project_id.to_string()],
    )?;
    Ok(())
}

pub fn delete_project(cfg: &GtdConfig, project_id: &Uuid) -> Result<()> {
    let conn = Connection::open(&cfg.storage_path)?;
    conn.execute(
        "DELETE FROM project WHERE uuid = ?1",
        params![project_id.to_string()],
    )?;
    Ok(())
}

pub fn get_projects(
    cfg: &GtdConfig,
    state_filter: Option<&State>,
    uuid_filter: Option<&Uuid>,
) -> Result<Vec<Project>> {
    let conn = Connection::open(&cfg.storage_path)?;
    let mut query = "SELECT uuid, name, state FROM project WHERE 1=1".to_string();
    let mut params: Vec<&dyn rusqlite::ToSql> = Vec::new();

    if let Some(state) = state_filter {
        query.push_str(" AND state = ?");
        params.push(state);
    }
    if let Some(uuid) = uuid_filter {
        query.push_str(" AND uuid = ?");
        params.push(uuid);
    }

    let mut stmt = conn.prepare(&query)?;
    let mut id = 1;
    let project_iter = stmt.query_map(params.as_slice(), |row| {
        let uuid_str: String = row.get(0)?;
        let state: State = row.get(2)?;
        let task_id = match state {
            State::Pending => id,
            _ => 0,
        };
        if task_id != 0 {
            id += 1;
        }
        Ok(Project {
            id: task_id,
            uuid: Uuid::parse_str(&uuid_str).unwrap(),
            name: row.get(1)?,
            state: row.get(2)?,
        })
    })?;

    let mut projects = Vec::new();
    for project in project_iter {
        projects.push(project?);
    }
    Ok(projects)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    use tempfile::NamedTempFile;
    use uuid::Uuid;

    fn setup_temp_db() -> (GtdConfig, Connection) {
        let temp_file = NamedTempFile::new().unwrap();
        let storage_path = temp_file.path().to_str().unwrap().to_string();
        let cfg = GtdConfig {
            storage_path,
            ..Default::default()
        };
        let conn = Connection::open(&cfg.storage_path).unwrap();
        init_db(&conn).unwrap();
        (cfg, conn)
    }

    #[test]
    fn test_check_db() {
        let (cfg, _) = setup_temp_db();
        assert!(check_db(&cfg).is_ok());
    }

    #[test]
    fn test_insert_and_get_projects() {
        let (cfg, _) = setup_temp_db();
        let projects = vec![
            Project {
                id: 1,
                uuid: Uuid::new_v4(),
                name: "Project 1".to_string(),
                state: State::Pending,
            },
            Project {
                id: 2,
                uuid: Uuid::new_v4(),
                name: "Project 2".to_string(),
                state: State::Complete,
            },
        ];

        assert!(insert_projects(&cfg, &projects).is_ok());
        let retrieved_projects = get_projects(&cfg, None, None).unwrap();
        assert_eq!(retrieved_projects.len(), 2);
        assert_eq!(retrieved_projects[0].name, "Project 1");
        assert_eq!(retrieved_projects[1].state, State::Complete);
    }

    #[test]
    fn test_update_project_status() {
        let (cfg, _) = setup_temp_db();
        let project = Project {
            id: 1,
            uuid: Uuid::new_v4(),
            name: "Project 1".to_string(),
            state: State::Pending,
        };

        insert_projects(&cfg, &[project.clone()]).unwrap();
        assert!(update_project_status(&cfg, &State::Complete, &project.uuid).is_ok());
        let updated_project = get_projects(&cfg, None, Some(&project.uuid)).unwrap();
        assert_eq!(updated_project[0].state, State::Complete);
    }

    #[test]
    fn test_delete_project() {
        let (cfg, _) = setup_temp_db();
        let project = Project {
            id: 1,
            uuid: Uuid::new_v4(),
            name: "Project 1".to_string(),
            state: State::Pending,
        };

        insert_projects(&cfg, &[project.clone()]).unwrap();
        assert!(delete_project(&cfg, &project.uuid).is_ok());
        let remaining_projects = get_projects(&cfg, None, None).unwrap();
        assert!(remaining_projects.is_empty());
    }
}
