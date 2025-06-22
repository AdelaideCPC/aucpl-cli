use std::collections::HashMap;
use std::env;
use std::fs;

use anyhow::{bail, Context, Result};
use config::{Config, ConfigError, File, FileFormat};
use serde::Deserialize;

use crate::util::get_project_root;

pub const SETTINGS_FILE_NAME: &str = "settings.toml";
pub const SETTINGS_FILE_VERSION: &str = "0.2";
pub const SETTINGS_FILE_DEFAULT_CONTENTS: &str = include_str!("../../../settings.toml.example");

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

impl Default for Settings {
    fn default() -> Self {
        Settings {
            version: SETTINGS_FILE_VERSION.to_string(),
            problems_dir: "problems".to_string(),
            problem: Problem {
                default_lang: "cpp".to_string(),
                default_generator_lang: "py".to_string(),
                solution: HashMap::new(),
            },
        }
    }
}

impl Settings {
    /// Load settings
    //
    // TODO: Some functionality such as checking for the project root relies on
    // there being a file called `settings.toml` in the project root so it will
    // not work if alternate settings files are used.
    pub fn new(config_file: Option<&str>) -> Result<Self, ConfigError> {
        let project_root =
            get_project_root().map_err(|err| ConfigError::Message(format!("{err}")))?;

        let settings_path = match config_file {
            Some(name) => project_root.join(name),
            None => project_root.join(SETTINGS_FILE_NAME),
        };
        if !settings_path.exists() {
            eprintln!(
                "Settings file not found at '{}'. A new settings file will be generated",
                settings_path.display()
            );
            create_settings_file().map_err(|err| {
                ConfigError::Message(format!("Failed to create settings file: {err}"))
            })?;
        }

        let settings_path_str = settings_path.to_str().ok_or_else(|| {
            ConfigError::Message("Could not get path of settings file".to_string())
        })?;

        // Load defaults first, then have settings file override them
        let s = Config::builder()
            .add_source(File::from_str(
                SETTINGS_FILE_DEFAULT_CONTENTS,
                FileFormat::Toml,
            ))
            .add_source(File::with_name(settings_path_str))
            .build()?;

        s.try_deserialize()
    }
}

/// Get the settings from the settings file.
pub fn get_settings() -> Result<Settings> {
    let settings = match Settings::new(None) {
        Ok(s) => s,
        Err(error) => bail!(
            "Failed to parse settings file: {error:?}\nMake sure that the settings file is up to date with the latest version (v{SETTINGS_FILE_VERSION})"
        ),
    };

    if settings.version != SETTINGS_FILE_VERSION {
        bail!(
            "The settings file version does not match! Expected '{SETTINGS_FILE_VERSION}', got '{}'",
            settings.version
        );
    }

    Ok(settings)
}

/// Create a new settings file with default contents if it does not already exist.
pub fn create_settings_file() -> Result<()> {
    let project_root = env::current_dir().context("Failed to get current directory")?;
    let settings_path = project_root.join(SETTINGS_FILE_NAME);

    if settings_path.exists() {
        bail!(
            "Settings file already exists at '{}'",
            settings_path.display()
        );
    }
    fs::write(&settings_path, SETTINGS_FILE_DEFAULT_CONTENTS)
        .context("Could not create settings file")?;

    eprintln!("Created settings file at '{}'", settings_path.display());
    Ok(())
}
