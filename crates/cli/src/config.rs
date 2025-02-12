use config::{Config, ConfigError, File};
use serde::Deserialize;

#[allow(unused)]
#[derive(Debug, Deserialize)]
pub struct Settings {
    pub problems_dir: String,
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
