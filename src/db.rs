use crate::project::{Project, State};
use chrono::Utc;
use uuid::Uuid;

use rusqlite::{Connection, Result, params};

pub struct DB {
    conn: Connection,
}

impl DB {
    pub fn new(storage_path: &str) -> Result<DB> {
        Ok(DB {
            conn: Connection::open(storage_path)?,
        })
    }
    pub fn check(&self) -> Result<()> {
        let result: Result<i32> = self.conn.query_row(
            "SELECT 1 FROM sqlite_master WHERE type='table' AND name='project'",
            [],
            |row| row.get(0),
        );
        match result {
            Ok(_) => (),
            Err(_) => self.init()?,
        };
        Ok(())
    }

    /// Sets up initial database tables
    fn init(&self) -> Result<()> {
        println!("Initializing projects db...");
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS project (
            uuid TEXT PRIMARY KEY,
            id INT,
            name TEXT NOT NULL,
            state TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )",
            [],
        )?;
        Ok(())
    }

    fn rebuild_working_set(&self) -> Result<()> {
        self.conn.execute(
            "WITH ranked AS (
            SELECT uuid, RANK() OVER (ORDER BY created_at ASC) new_id
            FROM project
            WHERE state = 'Pending'
        )
        UPDATE project
        SET id = ranked.new_id
        FROM ranked
        WHERE project.uuid = ranked.uuid;",
            [],
        )?;
        Ok(())
    }

    pub fn insert_projects(&mut self, projects: &[Project]) -> Result<()> {
        let tx = self.conn.transaction()?;
        {
            let mut stmt = tx.prepare("INSERT INTO project (uuid, name, state, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)")?;
            for project in projects {
                stmt.execute(params![
                    project.uuid.to_string(),
                    project.name,
                    project.state.to_string(),
                    project.created_at.to_string(),
                    project.created_at.to_string(),
                ])?;
            }
        }
        tx.commit()?;
        self.rebuild_working_set()?;
        Ok(())
    }

    pub fn update_project_status(&self, state: &State, project_id: &Uuid) -> Result<()> {
        self.conn.execute(
            "UPDATE project SET state = ?1, updated_at = ?2 WHERE uuid = ?3",
            params![
                state.to_string(),
                Utc::now().to_string(),
                project_id.to_string()
            ],
        )?;
        self.rebuild_working_set()?;
        Ok(())
    }

    pub fn delete_project(&self, project_id: &Uuid) -> Result<()> {
        self.conn.execute(
            "DELETE FROM project WHERE uuid = ?1",
            params![project_id.to_string()],
        )?;
        self.rebuild_working_set()?;
        Ok(())
    }

    pub fn get_projects(
        &self,
        state_filter: Option<&State>,
        uuid_filter: Option<&Uuid>,
        id_filter: Option<&u32>,
    ) -> Result<Vec<Project>> {
        let mut query = "SELECT uuid, id, name, state FROM project WHERE 1=1".to_string();
        let mut params: Vec<&dyn rusqlite::ToSql> = Vec::new();

        // Build filters
        if let Some(state) = state_filter {
            query.push_str(" AND state = ?");
            params.push(state);
        }
        if let Some(uuid) = uuid_filter {
            query.push_str(" AND uuid = ?");
            params.push(uuid);
        }
        if let Some(id) = id_filter {
            query.push_str(" AND id = ?");
            params.push(id);
        }

        let mut stmt = self.conn.prepare(&query)?;
        let project_iter = stmt.query_map(params.as_slice(), |row| {
            let uuid_str: String = row.get(0)?;
            Ok(Project {
                id: row.get(1)?,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                uuid: Uuid::parse_str(&uuid_str).unwrap(),
                name: row.get(2)?,
                state: row.get(3)?,
                ..Project::default()
            })
        })?;

        let mut projects = Vec::new();
        for project in project_iter {
            projects.push(project?);
        }
        Ok(projects)
    }
}

// pub fn get_single_project(cfg: &GtdConfig, project_id: u32) -> Result<Project> {
//     let conn = Connection::open(&cfg.storage_path)?;
//     let query = "SELECT uuid, id, name, state FROM project WHERE id = ?";
//     Ok(())
// }

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    use uuid::Uuid;

    // Setup and in memory db and initialize tables
    fn setup_temp_db() -> DB {
        let conn = Connection::open_in_memory().unwrap();
        let db = DB { conn };
        db.init().unwrap();
        db
    }

    fn setup_default_projects(db: &mut DB) -> Vec<Project> {
        let projects = vec![
            Project {
                id: Some(1),
                uuid: Uuid::new_v4(),
                name: "Project 1".to_string(),
                state: State::Pending,
                ..Project::default()
            },
            Project {
                id: Some(2),
                uuid: Uuid::new_v4(),
                name: "Project 2".to_string(),
                state: State::Complete,
                ..Project::default()
            },
        ];
        db.insert_projects(&projects).unwrap();
        projects
    }

    #[test]
    fn test_check_db() {
        let db = setup_temp_db();
        assert!(db.check().is_ok());
    }

    #[test]
    fn test_insert_and_get_projects() {
        let mut db = setup_temp_db();
        let projects = vec![
            Project {
                uuid: Uuid::new_v4(),
                name: "Project 1".to_string(),
                state: State::Pending,
                ..Project::default()
            },
            Project {
                uuid: Uuid::new_v4(),
                name: "Project 2".to_string(),
                state: State::Complete,
                ..Project::default()
            },
        ];

        assert!(db.insert_projects(&projects).is_ok());
        let retrieved_projects = db.get_projects(None, None, None).unwrap();
        assert_eq!(retrieved_projects.len(), 2);
        assert_eq!(retrieved_projects[0].id, Some(1));
        assert_eq!(retrieved_projects[0].name, "Project 1");
        assert_eq!(retrieved_projects[0].uuid, projects[0].uuid);
        assert_eq!(retrieved_projects[0].state, State::Pending);
        assert_eq!(retrieved_projects[1].id, None);
        assert_eq!(retrieved_projects[1].name, "Project 2");
        assert_eq!(retrieved_projects[1].uuid, projects[1].uuid);
        assert_eq!(retrieved_projects[1].state, State::Complete);
    }

    #[test]
    fn test_state_filter() {
        let mut db = setup_temp_db();
        setup_default_projects(&mut db);
        let pending_projects = db.get_projects(Some(&State::Pending), None, None).unwrap();
        assert_eq!(pending_projects.len(), 1);
        let completed_projects = db.get_projects(Some(&State::Complete), None, None).unwrap();
        assert_eq!(completed_projects.len(), 1);
    }

    #[test]
    fn test_uuid_filter() {
        let mut db = setup_temp_db();
        let projects = setup_default_projects(&mut db);
        println!("{:?}", projects[0].uuid);
        let uuid_filter = db
            .get_projects(None, Some(&projects[0].uuid), None)
            .unwrap();
        assert_eq!(uuid_filter.len(), 1);
        assert_eq!(uuid_filter[0].name, projects[0].name);
    }

    #[test]
    fn test_id_filter() {
        let mut db = setup_temp_db();
        let projects = setup_default_projects(&mut db);
        let uuid_filter = db.get_projects(None, None, Some(&1)).unwrap();
        assert_eq!(uuid_filter.len(), 1);
        assert_eq!(uuid_filter[0].name, projects[0].name);
    }

    #[test]
    fn test_update_project_status() {
        let mut db = setup_temp_db();
        let project = Project {
            id: Some(1),
            uuid: Uuid::new_v4(),
            name: "Project 1".to_string(),
            state: State::Pending,
            ..Project::default()
        };

        db.insert_projects(&[project.clone()]).unwrap();
        assert!(
            db.update_project_status(&State::Complete, &project.uuid)
                .is_ok()
        );
        let pending_projects = db.get_projects(Some(&State::Pending), None, None).unwrap();
        assert_eq!(pending_projects.len(), 0);
        let updated_project = db.get_projects(Some(&State::Complete), None, None).unwrap();
        assert_eq!(updated_project.len(), 1);

        // Sanity checks
        assert_eq!(updated_project[0].id, project.id);
        assert_ne!(updated_project[0].updated_at, project.updated_at);
    }

    #[test]
    fn test_delete_project() {
        let mut db = setup_temp_db();
        let project = Project {
            id: Some(1),
            uuid: Uuid::new_v4(),
            name: "Project 1".to_string(),
            state: State::Pending,
            ..Project::default()
        };

        db.insert_projects(&[project.clone()]).unwrap();
        assert!(db.delete_project(&project.uuid).is_ok());
        let remaining_projects = db.get_projects(None, None, None).unwrap();
        assert!(remaining_projects.is_empty());
    }
}
