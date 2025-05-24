use std::collections::HashMap;

use config::{Config, ConfigError, File};
use serde::Deserialize;

use crate::util::get_project_root;

pub const SETTINGS_FILE_VERSION: &str = "0.2";
pub const SETTINGS_FILE: &str = "settings.toml";

/// Configuration for the CLI, loaded via a settings file.
#[allow(unused)]
#[derive(Debug, Deserialize)]
pub struct Settings {
    pub version: String,
    pub problems_dir: String,
    pub problem: Problem,
}
/// Settings specific to problem configuration.
#[allow(unused)]
#[derive(Debug, Deserialize)]
pub struct Problem {
    pub default_lang: String,
    pub default_generator_lang: String,
    pub solution: HashMap<String, LangSolution>,
}

#[allow(unused)]
#[derive(Debug, Deserialize)]
pub struct LangSolution {
    pub compile_command: Option<Vec<String>>,
    pub run_command: Option<Vec<String>>,
}

impl Settings {
    pub fn new(config_file: Option<&str>) -> Result<Self, ConfigError> {
        let project_root =
            get_project_root().map_err(|err| ConfigError::Message(format!("{err}")))?;

        let settings_path = match config_file {
            Some(name) => project_root.join(name),
            None => project_root.join(SETTINGS_FILE),
        };

        let settings_path_str = settings_path.to_str().ok_or_else(|| {
            ConfigError::Message("Could not get path of settings file".to_string())
        })?;

        let s = Config::builder()
            .add_source(File::with_name(settings_path_str))
            .build()?;

        s.try_deserialize()
    }
}
