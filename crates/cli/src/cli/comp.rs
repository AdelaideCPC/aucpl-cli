use anyhow::Result;
use clap::{ArgMatches, Command};

use crate::config::Settings;

pub fn cli() -> Command {
    Command::new("comp").about("Commands related to competitions")
}

pub fn exec(args: &ArgMatches, settings: &Settings) -> Result<()> {
    _ = args;
    _ = settings;

    Ok(())
}
