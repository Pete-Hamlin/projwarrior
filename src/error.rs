use std::error::Error;
use std::fmt;

#[derive(PartialEq, Debug)]
pub enum ConfigError {
    NoTask,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let description = match *self {
            ConfigError::NoTask => {
                "Unable to find task binary - please ensure the `task` command is available in your $PATH"
            }
        };
        f.write_str(description)
    }
}

impl Error for ConfigError {}

#[derive(PartialEq, Debug)]
pub enum ProjChampionError {
    ReplicaError,
    TaskError,
}

impl fmt::Display for ProjChampionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let description = match *self {
            ProjChampionError::ReplicaError => "Failed to setup replica.",
            ProjChampionError::TaskError => "Error querying taskwarrior",
        };
        f.write_str(description)
    }
}

impl Error for ProjChampionError {}
impl From<taskchampion::Error> for ProjChampionError {
    fn from(_: taskchampion::Error) -> Self {
        Self::ReplicaError
    }
}
