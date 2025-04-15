use anyhow::Result;
use clap::{ArgMatches, Command};

use crate::config::Settings;

pub fn cli() -> Command {
    Command::new("publish").about("Package test files and publish them to the remote server")
}

pub fn exec(args: &ArgMatches, settings: &Settings) -> Result<()> {
    _ = args;
    _ = settings;

    Ok(())
}
