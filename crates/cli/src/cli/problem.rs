use std::{fs, path::PathBuf};

use anyhow::{anyhow, Context, Result};
use clap::{value_parser, Arg, ArgAction, ArgMatches, Command};

use crate::config::Settings;
use crate::problem::{archive, check, create, solve};

pub fn cli() -> Command {
    Command::new("problem")
        .about("Commands related to a problem")
        .subcommand(
            Command::new("archive")
                .about("Archive a problem")
                .arg_required_else_help(true)
                .arg(
                    Arg::new("name")
                        .help("Problem name (this is not the problem title)")
                        .action(ArgAction::Set)
                        .required(true),
                ),
        )
        .subcommand(
            Command::new("check")
                .about("Check that the problem folder and test files are valid")
                .arg_required_else_help(true)
                .arg(
                    Arg::new("name")
                        .help("Problem name (this is not the problem title)")
                        .action(ArgAction::Set)
                        .required(true),
                ),
        )
        .subcommand(
            Command::new("create")
                .about("Create a new problem and generate necessary files")
                .arg_required_else_help(true)
                .arg(
                    Arg::new("difficulty")
                        .action(ArgAction::Set)
                        .help("Difficulty of the problem. For an unrated problem, put 0")
                        .required(true)
                        .value_parser(value_parser!(u16)),
                )
                .arg(
                    Arg::new("name")
                        .help("Problem name (this is not the problem title)")
                        .action(ArgAction::Set)
                        .required(true),
                ),
        )
        .subcommand(
            Command::new("solve")
                .about("Automatically generate test outputs for a problem, given pre-existing input files")
                .arg_required_else_help(true)
                .arg(
                    Arg::new("name")
                        .help("Problem name (this is not the problem title)")
                        .action(ArgAction::Set)
                        .required(true),
                ),
        )
        .subcommand(
            Command::new("test")
                .about("Run tests on the given problem")
                .arg_required_else_help(true)
                .arg(
                    Arg::new("name")
                        .help("Problem name (this is not the problem title)")
                        .action(ArgAction::Set)
                        .required(true),
                ),
        )
        .subcommand_required(true)
}

pub fn exec(args: &ArgMatches, settings: &Settings) -> Result<()> {
    let problems_dir = PathBuf::new().join(&settings.problems_dir);
    if !fs::exists(&problems_dir).expect("Failed to check if path exists") {
        fs::create_dir(&problems_dir).expect("Failed to create directory");
    }

    match args.subcommand() {
        Some(("archive", cmd)) => {
            let problem_name = cmd
                .try_get_one::<String>("name")?
                .context("Problem name is required")?;

            archive::archive(&problems_dir, problem_name)?;
            eprintln!("Archived problem '{problem_name}'");
        }
        Some(("check", cmd)) => {
            let problem_name = cmd
                .try_get_one::<String>("name")?
                .context("Problem name is required")?;

            check::check(problems_dir, problem_name)
                .map_err(|err| anyhow!("Failed check: {err}"))?;
        }
        Some(("create", cmd)) => {
            let problem_name = cmd
                .try_get_one::<String>("name")?
                .context("Problem name is required")?;

            let difficulty = cmd
                .try_get_one::<u16>("difficulty")?
                .context("Problem difficulty is required")?;

            create::create(&problems_dir, problem_name, *difficulty)?;
        }
        Some(("solve", cmd)) => {
            let problem_name = cmd
                .try_get_one::<String>("name")?
                .context("Problem name is required")?;

            solve::solve(settings, &problems_dir, problem_name)?;
        }
        Some(("test", cmd)) => {
            let problem_name = cmd
                .try_get_one::<String>("name")?
                .context("Problem name is required")?;

            // TODO
            _ = problem_name;
        }
        _ => {}
    }

    Ok(())
}
