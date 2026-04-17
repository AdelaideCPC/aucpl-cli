use std::process::ExitCode;
use std::sync::OnceLock;

use anyhow::Result;
use clap::{Arg, ArgAction, Command};
use owo_colors::OwoColorize;

mod cli;
mod comp;
mod config;
mod errors;
mod paths;
mod problem;
mod publish;
mod suggest;
mod sync;
mod util;

pub const NAME: &str = "AUCPL CLI";
pub const BIN_NAME: &str = env!("CARGO_BIN_NAME");
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const ABOUT: &str = env!("CARGO_PKG_DESCRIPTION");

/// Global verbosity flag - set when CLI is parsed
static VERBOSE: OnceLock<bool> = OnceLock::new();

/// Check if verbose mode is enabled
pub fn is_verbose() -> bool {
    *VERBOSE.get_or_init(|| false)
}

/// Set the global verbosity flag
fn set_verbose(verbose: bool) {
    let _ = VERBOSE.set(verbose);
}

/// Format and print an error message with rich context and suggestions
fn print_error(err: &anyhow::Error, verbose: bool) {
    // Print main error message
    eprintln!("\n{}", err.to_string().red().bold());

    // Print additional details and suggestions if available
    if let Some(cli_err) = err.downcast_ref::<errors::CliError>() {
        // In verbose mode, show detailed context
        if verbose {
            if let Some(details) = cli_err.get_verbose() {
                eprintln!("\n{}", "Details:".yellow().bold());
                for line in details.lines() {
                    eprintln!("  {line}");
                }
            }
        }

        // Print suggestions
        if cli_err.has_suggestions() {
            eprintln!("\n{}", "Suggestions:".green().bold());
            for suggestion in cli_err.get_suggestions() {
                eprintln!("  - {suggestion}");
            }
        }
    }

    eprintln!();
}

/// Main entry point with proper error handling
fn run() -> Result<()> {
    let about_text = format!("{NAME} {VERSION}\n{ABOUT}");
    let after_help_text =
        format!("See '{BIN_NAME} help <command>' for more information on a command");

    let cli = Command::new(NAME)
        .bin_name(BIN_NAME)
        .name(NAME)
        .version(VERSION)
        .about(about_text)
        .after_help(after_help_text)
        .arg_required_else_help(true)
        .subcommands(cli::builtin())
        .subcommand_required(true)
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Enable verbose output with detailed error information")
                .action(ArgAction::SetTrue)
                .global(true),
        );

    let matches = cli.get_matches();

    // Set global verbose flag
    set_verbose(matches.get_flag("verbose"));

    match matches.subcommand() {
        Some(("cd", cmd)) => cli::cd::exec(cmd)?,
        Some(("comp", cmd)) => cli::comp::exec(cmd)?,
        Some(("__complete", cmd)) => cli::complete::exec(cmd)?,
        Some(("init", cmd)) => cli::init::exec(cmd)?,
        Some(("problem", cmd)) => cli::problem::exec(cmd)?,
        Some(("publish", cmd)) => cli::publish::exec(cmd)?,
        Some(("shellinit", cmd)) => cli::shellinit::exec(cmd)?,
        Some(("sync", cmd)) => cli::sync::exec(cmd)?,
        _ => unreachable!(),
    }

    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            print_error(&e, is_verbose());
            ExitCode::from(1)
        }
    }
}
