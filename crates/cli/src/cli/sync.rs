use std::fs;

use anyhow::Result;
use clap::{ArgMatches, Command};

use crate::config::get_settings;
use crate::problem::sync_mappings;
use crate::util::get_project_root;

pub fn cli() -> Command {
    Command::new("sync").about("Generate or update the problem mappings file")
}

pub fn exec(args: &ArgMatches) -> Result<()> {
    _ = args;
    let settings = get_settings()?;

    let problems_dir = get_project_root()?.join(&settings.problems_dir);
    if !fs::exists(&problems_dir).expect("Failed to check if path exists") {
        fs::create_dir(&problems_dir).expect("Failed to create directory");
    }

    sync_mappings::sync_mappings(&problems_dir)?;
    eprintln!("Updated problem mappings file");

    Ok(())
}
