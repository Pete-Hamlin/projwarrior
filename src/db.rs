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
    let mut id = 0;
    let project_iter = stmt.query_map(params.as_slice(), |row| {
        let uuid_str: String = row.get(0)?;
        id += 1;
        Ok(Project {
            id,
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
