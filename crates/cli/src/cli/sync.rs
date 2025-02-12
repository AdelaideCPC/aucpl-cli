use anyhow::Result;
use clap::{ArgMatches, Command};

use crate::config::Settings;

pub fn cli() -> Command {
    Command::new("sync").about("Sync all problems with the remote server")
}

pub fn exec(args: &ArgMatches, settings: &Settings) -> Result<()> {
    _ = args;
    _ = settings;

    Ok(())
}
