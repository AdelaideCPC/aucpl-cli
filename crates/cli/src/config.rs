use config::{Config, ConfigError, File};
use serde::Deserialize;

/// Settings specific to problem configuration.
#[allow(unused)]
#[derive(Debug, Deserialize)]
pub struct Problem {
    pub solution_compile_command: Vec<String>,
    pub solution_run_command: Vec<String>,
    pub solution_file_ext: String,
}

/// Configuration for the CLI, loaded via a `settings.toml` file.
#[allow(unused)]
#[derive(Debug, Deserialize)]
pub struct Settings {
    pub problems_dir: String,
    pub problem: Problem,
}

impl Settings {
    pub fn new(config_file: Option<&str>) -> Result<Self, ConfigError> {
        let file: &str = match config_file {
            Some(name) => name,
            None => "settings.toml",
        };

        let s = Config::builder()
            .add_source(File::with_name(file))
            .build()?;

        s.try_deserialize()
    }
}
