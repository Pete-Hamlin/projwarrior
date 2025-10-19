use crate::filters::ProjectFilter;
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
            uuid BLOB PRIMARY KEY,
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

    pub fn reset(&self) -> Result<()> {
        println!("Dropping projects db...");
        self.conn.execute("DROP TABLE IF EXISTS project", [])?;
        Ok(())
    }

    /// Rebuilds the 'working set' indexes for pending projects.
    ///
    /// Should only be run on an action that affects the state of the working set (e.g.
    /// completing/deleting a project).
    /// Each pending project will be 1-indexed for ease of reference/access, same as taskwarrior.
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
        self.clear_old_ids()?;
        Ok(())
    }

    /// Simple SQL statement that sets all working IDs to NULL for non-pending projects.
    ///
    /// Used to clear out working index IDs for projects that are no longer pending.
    /// Without this, completed projects will be left with the working ID they had when they were
    /// moved from pending.
    fn clear_old_ids(&self) -> Result<()> {
        self.conn
            .execute("UPDATE project SET id = NULL WHERE state != 'Pending'", [])?;
        Ok(())
    }

    pub fn insert_projects(&mut self, projects: &[Project]) -> Result<()> {
        let tx = self.conn.transaction()?;
        {
            let mut stmt = tx.prepare("INSERT INTO project (uuid, name, state, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)")?;
            for project in projects {
                stmt.execute(params![
                    project.uuid,
                    project.name,
                    project.state,
                    project.created_at.to_rfc3339(),
                    project.updated_at.to_rfc3339(),
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
            params![state, Utc::now().to_rfc3339(), project_id],
        )?;
        self.rebuild_working_set()?;
        Ok(())
    }

    pub fn delete_project(&self, project_id: &Uuid) -> Result<()> {
        self.conn
            .execute("DELETE FROM project WHERE uuid = ?1", params![project_id])?;
        self.rebuild_working_set()?;
        Ok(())
    }

    pub fn get_projects(&self, options: &ProjectFilter) -> Result<Vec<Project>> {
        let mut query =
            "SELECT uuid, id, name, state, created_at, updated_at FROM project WHERE 1=1"
                .to_string();
        let mut params: Vec<&dyn rusqlite::ToSql> = Vec::new();

        // Build filters
        if let Some(uuid) = options.uuid {
            query.push_str(" AND uuid = ?");
            params.push(uuid);
        }
        if let Some(id) = options.id {
            query.push_str(" AND id = ?");
            params.push(id);
        }

        let name_param: String;
        if let Some(name) = options.name {
            query.push_str(" AND name LIKE ?");
            name_param = format!("%{}%", name);
            params.push(&name_param);
        }

        if let Some(state) = options.state {
            query.push_str(" AND state = ?");
            params.push(state);
        }

        let mut stmt = self.conn.prepare(&query)?;
        let project_iter = stmt.query_map(params.as_slice(), |row| {
            let created_at =
                chrono::DateTime::parse_from_rfc3339(row.get::<_, String>(4)?.as_str())
                    .unwrap()
                    .with_timezone(&Utc);
            let updated_at =
                chrono::DateTime::parse_from_rfc3339(row.get::<_, String>(5)?.as_str())
                    .unwrap()
                    .with_timezone(&Utc);
            Ok(Project {
                id: row.get(1)?,
                uuid: row.get(0)?,
                name: row.get(2)?,
                state: row.get(3)?,
                created_at,
                updated_at,
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
        let filters = ProjectFilter::builder().build();
        let retrieved_projects = db.get_projects(&filters).unwrap();
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

        let filters = ProjectFilter::builder().state(&State::Pending).build();
        let pending_projects = db.get_projects(&filters).unwrap();
        assert_eq!(pending_projects.len(), 1);

        let filters = ProjectFilter::builder().state(&State::Complete).build();
        let completed_projects = db.get_projects(&filters).unwrap();
        assert_eq!(completed_projects.len(), 1);
    }

    #[test]
    fn test_uuid_filter() {
        let mut db = setup_temp_db();
        let projects = setup_default_projects(&mut db);
        let filters = ProjectFilter::builder().uuid(&projects[0].uuid).build();
        let uuid_filter = db.get_projects(&filters).unwrap();
        assert_eq!(uuid_filter.len(), 1);
        assert_eq!(uuid_filter[0].name, projects[0].name);
    }

    #[test]
    fn test_id_filter() {
        let mut db = setup_temp_db();
        let projects = setup_default_projects(&mut db);
        let filters = ProjectFilter::builder().id(&1).build();
        let uuid_filter = db.get_projects(&filters).unwrap();
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

        db.insert_projects(std::slice::from_ref(&project)).unwrap();
        assert!(
            db.update_project_status(&State::Complete, &project.uuid)
                .is_ok()
        );
        let filters = ProjectFilter::builder().state(&State::Pending).build();
        let pending_projects = db.get_projects(&filters).unwrap();
        assert_eq!(pending_projects.len(), 0);

        let filters = ProjectFilter::builder().state(&State::Complete).build();
        let updated_project = db.get_projects(&filters).unwrap();
        assert_eq!(updated_project.len(), 1);

        // Sanity checks
        assert_eq!(updated_project[0].uuid, project.uuid);
        assert_ne!(updated_project[0].id, project.id);
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

        db.insert_projects(std::slice::from_ref(&project)).unwrap();
        assert!(db.delete_project(&project.uuid).is_ok());
        let filters = ProjectFilter::builder().build();
        let remaining_projects = db.get_projects(&filters).unwrap();
        assert!(remaining_projects.is_empty());
    }
}
