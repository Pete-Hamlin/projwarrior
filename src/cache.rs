use std::{
    fs,
    io::{Read, Write},
    path::PathBuf,
    time::SystemTime,
};

use anyhow::Result;
use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use task_hookrs::task::Task;

#[derive(Serialize, Deserialize, Debug)]
pub struct Cache {
    path: PathBuf,
    timeout: u64,
}

impl Cache {
    pub fn new(timeout: u64, cache_path: Option<PathBuf>, refresh: bool) -> Option<Cache> {
        cache_path.map(|path| {
            let cache_dir = path.join("projwarrior");
            fs::create_dir_all(&cache_dir).expect("Error creating cache dir");
            let cache_file = cache_dir.join("tasks.json");
            if refresh && cache_file.exists() {
                fs::remove_file(&cache_file).expect("Error clearing cache file");
            }
            Cache {
                path: cache_file,
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
