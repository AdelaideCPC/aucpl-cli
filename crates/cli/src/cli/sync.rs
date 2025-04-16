use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use clap::{ArgMatches, Command};

use crate::config::Settings;
use crate::problem::sync_mappings;

pub fn cli() -> Command {
    Command::new("sync").about("Generate or update the problem mappings file")
}

pub fn exec(args: &ArgMatches, settings: &Settings) -> Result<()> {
    _ = args;

    let problems_dir = PathBuf::new().join(&settings.problems_dir);
    if !fs::exists(&problems_dir).expect("Failed to check if path exists") {
        fs::create_dir(&problems_dir).expect("Failed to create directory");
    }

    sync_mappings::sync_mappings(&problems_dir)?;
    eprintln!("Updated problem mappings file");

    Ok(())
}
