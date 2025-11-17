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

#[cfg(test)]
mod tests {
    use task_hookrs::task::TW26;

    use super::*;
    use std::fs;
    use std::time::{Duration, SystemTime};
    use task_hookrs::task::TaskBuilder;
    use tempfile::tempdir;

    fn setup_test_cache(timeout: u64) -> Cache {
        let tmp = tempdir()
            .expect("Failed to create temp dir")
            .path()
            .to_path_buf();
        Cache::new(timeout, Some(tmp), true).expect("Failed to create Cache")
    }

    fn default_task() -> Task {
        TaskBuilder::<TW26>::default()
            .description("Test Task")
            .build()
            .unwrap()
    }

    #[test]
    fn test_cache_creation() {
        let cache = setup_test_cache(60);
        assert!(
            !cache.path.exists(),
            "Cache file should not exist initially"
        );
    }

    #[test]
    fn test_cache_file_creation_on_write() {
        let cache = setup_test_cache(60);
        let tasks = vec![]; // Empty task list
        cache.write(&tasks).expect("Failed to write to cache");
        assert!(cache.path.exists(), "Cache file should exist after writing");
    }

    #[test]
    fn test_cache_read_write() {
        let cache = setup_test_cache(60);
        let tasks = vec![default_task()];
        println!("{tasks:#?}");
        cache.write(&tasks).expect("Failed to write to cache");
        let read_tasks = cache.read().expect("Failed to read from cache");
        assert_eq!(
            tasks.len(),
            read_tasks.len(),
            "Number of tasks should match"
        );
    }

    #[test]
    fn test_cache_validation_within_timeout() {
        let cache = setup_test_cache(60);
        let tasks = vec![default_task()];
        cache.write(&tasks).expect("Failed to write to cache");

        // Simulate time within timeout
        let metadata = fs::metadata(&cache.path).unwrap();
        let modified_time = metadata.modified().unwrap();
        let now = SystemTime::now();
        assert!(now.duration_since(modified_time).unwrap() < Duration::from_secs(60));

        assert!(
            cache.validate().is_ok(),
            "Cache should be valid within the timeout"
        );
    }

    #[test]
    fn test_cache_validation_outside_timeout() {
        let cache = setup_test_cache(1); // Set a very short timeout
        let tasks = vec![default_task()];
        cache.write(&tasks).expect("Failed to write to cache");

        // Simulate time outside timeout
        std::thread::sleep(Duration::from_secs(2));
        assert!(
            cache.validate().is_err(),
            "Cache should be invalid after timeout"
        );
    }

    #[test]
    fn test_cache_refresh_behavior() {
        let cache = setup_test_cache(60); // Enable refresh
        let tasks = vec![default_task()];
        cache.write(&tasks).expect("Failed to write to cache");

        let refreshed_cache = setup_test_cache(60);
        assert!(
            !refreshed_cache.path.exists(),
            "Cache file should be deleted on refresh"
        );
    }

    #[test]
    fn test_cache_read_invalid_file() {
        let cache = setup_test_cache(60);
        fs::write(&cache.path, "Invalid JSON").expect("Failed to write invalid data");

        assert!(
            cache.read().is_err(),
            "Reading invalid cache file should fail"
        );
    }
}
