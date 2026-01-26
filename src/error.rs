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
