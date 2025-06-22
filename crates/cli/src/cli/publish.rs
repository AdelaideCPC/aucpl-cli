use anyhow::Result;
use clap::{ArgMatches, Command};

pub fn cli() -> Command {
    Command::new("publish").about("Package test files and publish them to the remote server")
}

pub fn exec(args: &ArgMatches) -> Result<()> {
    _ = args;

    Ok(())
}
