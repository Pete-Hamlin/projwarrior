use std::{
    fs,
    io::{Read, Write},
    path::PathBuf,
    time::SystemTime,
};

use anyhow::Result;
use anyhow::anyhow;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use task_hookrs::task::Task;

#[derive(Serialize, Deserialize, Debug)]
pub struct Cache {
    path: PathBuf,
    timeout: u64,
}

impl Cache {
    pub fn new(timeout: u64) -> Option<Cache> {
        dirs::cache_dir().map(|path| {
            let cache_dir = path.join("projwarrior");
            fs::create_dir_all(&cache_dir).expect("Error creating cache dir");
            Cache {
                path: cache_dir.join("tasks.json"),
                timeout,
            }
        })
    }
    fn validate(&self) -> Result<()> {
        // Add metadata check here
        let metadata = fs::metadata(&self.path)?;
        let now = SystemTime::now();
        let last_updated = metadata.modified()?;
        if now.duration_since(last_updated)?.as_secs() < self.timeout {
            return Ok(());
        }
        Err(anyhow!("Cache invalid - Refresh required"))
    }

    pub fn read(&self) -> Result<Vec<Task>> {
        self.validate()?;

        let mut file = fs::File::open(&self.path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let cached: Vec<Task> = serde_json::from_str(&contents)?;
        Ok(cached)
    }
    pub fn write(&self, data: &Vec<Task>) -> Result<()> {
        let json = serde_json::to_string_pretty(data)?;
        let mut file = fs::File::create(&self.path)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }
}
