use clap::{Arg, ArgAction, Command};

use crate::{ABOUT, BIN_NAME, NAME, VERSION};

pub(crate) mod arg_builders;
pub mod cd;
pub mod comp;
pub mod complete;
pub mod init;
pub mod problem;
pub mod publish;
pub mod shellinit;
mod shellinit_scripts;
pub mod sync;

pub fn builtin() -> Vec<Command> {
    vec![
        cd::cli(),
        comp::cli(),
        complete::cli(),
        init::cli(),
        problem::cli(),
        publish::cli(),
        shellinit::cli(),
        sync::cli(),
    ]
}

/// Build the top-level CLI command tree used for parsing and introspection.
pub fn root() -> Command {
    let about_text = format!("{NAME} {VERSION}\n{ABOUT}");
    let after_help_text =
        format!("See '{BIN_NAME}' help <command> for more information on a command");

    let mut root = Command::new(NAME)
        .bin_name(BIN_NAME)
        .name(NAME)
        .version(VERSION)
        .about(about_text)
        .after_help(after_help_text)
        .arg_required_else_help(true)
        .subcommands(builtin())
        .subcommand_required(true)
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Enable verbose output with detailed error information")
                .action(ArgAction::SetTrue)
                .global(true),
        );

    root.build();
    root
}
