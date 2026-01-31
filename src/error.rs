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
    Replica(String),
    TaskList,
    NoProj,
    SubCommand(String),
    InvalidUndo,
    Other,
}

impl fmt::Display for ProjChampionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ProjChampionError::Replica(e) => write!(f, "Unable to setup replica: {e}"),
            ProjChampionError::TaskList => write!(f, "Error querying taskwarrior"),
            ProjChampionError::NoProj => write!(f, "No project specified"),
            ProjChampionError::SubCommand(e) => write!(f, "Subcommand {e} not valid."),
            ProjChampionError::InvalidUndo => write!(f, "Unable to undo last operation."),
            ProjChampionError::Other => write!(f, "An error has occured"),
        }
    }
}

impl Error for ProjChampionError {}
impl From<taskchampion::Error> for ProjChampionError {
    fn from(err: taskchampion::Error) -> Self {
        Self::Replica(err.to_string())
    }
}
