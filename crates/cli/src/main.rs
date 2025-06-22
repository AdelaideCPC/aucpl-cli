use anyhow::Result;
use clap::Command;

mod cli;
mod comp;
mod config;
mod problem;
mod publish;
mod sync;
mod util;

pub const NAME: &str = "AUCPL CLI";
pub const BIN_NAME: &str = env!("CARGO_BIN_NAME");
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const ABOUT: &str = env!("CARGO_PKG_DESCRIPTION");

fn main() -> Result<()> {
    let about_text = format!("{} {}\n{}", NAME, VERSION, ABOUT);
    let after_help_text = format!(
        "See '{} help <command>' for more information on a command",
        BIN_NAME
    );

    let cli = Command::new(NAME)
        .bin_name(BIN_NAME)
        .name(NAME)
        .version(VERSION)
        .about(about_text)
        .after_help(after_help_text)
        .arg_required_else_help(true)
        .subcommands(cli::builtin())
        .subcommand_required(true);

    let matches = cli.get_matches();

    match matches.subcommand() {
        Some(("comp", cmd)) => cli::comp::exec(cmd)?,
        Some(("init", cmd)) => cli::init::exec(cmd)?,
        Some(("problem", cmd)) => cli::problem::exec(cmd)?,
        Some(("publish", cmd)) => cli::publish::exec(cmd)?,
        Some(("sync", cmd)) => cli::sync::exec(cmd)?,
        _ => unreachable!(),
    }

    Ok(())
}
