use std::{
    fs,
    io::{Read, Write},
    path::PathBuf,
};

use anyhow::Result;
// use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use task_hookrs::task::Task;

#[derive(Serialize, Deserialize, Debug)]
pub struct Cache {
    path: PathBuf,
    timeout: u32,
}

impl Cache {
    pub fn new(timeout: u32) -> Option<Cache> {
        dirs::cache_dir().map(|path| {
            let cache_dir = path.join("projwarrior");
            fs::create_dir_all(&cache_dir).expect("Error creating cache dir");
            Cache {
                path: cache_dir.join("tasks.json"),
                timeout,
            }
        })
    }
    // fn valid(&self) -> bool {
    //     true
    // }
    pub fn read(&self) -> Result<Vec<Task>> {
        // match self.valid() {
        //     true => None,
        //     _ => None,
        // }

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
