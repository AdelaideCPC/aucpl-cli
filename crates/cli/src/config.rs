use std::collections::HashMap;
use std::env;
use std::fs;

use anyhow::{bail, Context, Result};
use config::{Config, File, FileFormat};
use serde::Deserialize;

use crate::errors::CliError;
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
            version: SETTINGS_FILE_VERSION.to_owned(),
            problems_dir: "problems".to_owned(),
            problem: Problem {
                default_lang: "cpp".to_owned(),
                default_generator_lang: "py".to_owned(),
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
    pub fn new(config_file: Option<&str>) -> Result<Self> {
        let project_root = get_project_root()?;

        let settings_path = match config_file {
            Some(name) => project_root.join(name),
            None => project_root.join(SETTINGS_FILE_NAME),
        };
        if !settings_path.exists() {
            eprintln!(
                "Settings file not found at '{}'. A new settings file will be generated",
                settings_path.display()
            );
            create_settings_file()?;
        }

        let settings_path_str = settings_path
            .to_str()
            .ok_or_else(|| CliError::InvalidInput {
                message: "Settings file path contains invalid characters".to_owned(),
                verbose: Some(format!(
                    "Path: {}\nCould not convert to string (invalid UTF-8)",
                    settings_path.display()
                )),
                suggestions: vec![
                    "Move the project to a path with only ASCII characters".to_owned()
                ],
            })?;

        // Load defaults first, then have settings file override them
        let s = Config::builder()
            .add_source(File::from_str(
                SETTINGS_FILE_DEFAULT_CONTENTS,
                FileFormat::Toml,
            ))
            .add_source(File::with_name(settings_path_str))
            .build()
            .map_err(|e| CliError::ConfigurationError {
                message: format!("Failed to load settings file: {}", settings_path.display()),
                verbose: Some(format!(
                    "Error: {}\nMake sure the settings file is valid TOML",
                    e
                )),
                suggestions: vec![
                    format!("Check that {} is valid TOML syntax", SETTINGS_FILE_NAME),
                    format!(
                        "Ensure the settings file matches version {}",
                        SETTINGS_FILE_VERSION
                    ),
                ],
            })?;

        s.try_deserialize().map_err(|e| {
            CliError::ConfigurationError {
                message: format!("Failed to parse settings file: {}", settings_path.display()),
                verbose: Some(format!(
                    "Error: {}\nThe settings file structure may be incorrect",
                    e
                )),
                suggestions: vec![
                    format!(
                        "Make sure that {SETTINGS_FILE_NAME} is up to date with the latest version (v{SETTINGS_FILE_VERSION})"
                    ),
                    "Compare your settings file with the example in the documentation".to_owned(),
                ],
            }
            .into()
        })
    }
}

/// Get the settings from the settings file.
pub fn get_settings() -> Result<Settings> {
    let settings = Settings::new(None)?;

    if settings.version != SETTINGS_FILE_VERSION {
        return Err(CliError::ConfigurationError {
            message: format!(
                "Settings file version mismatch: expected '{}', got '{}'",
                SETTINGS_FILE_VERSION, settings.version
            ),
            verbose: Some(format!(
                "Settings file: {}",
                get_project_root()?.join(SETTINGS_FILE_NAME).display()
            )),
            suggestions: vec![format!(
                "Update the version field in {} to '{}'",
                SETTINGS_FILE_NAME, SETTINGS_FILE_VERSION
            )],
        }
        .into());
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
