use std::fs;

use anyhow::{Context, Result};
use clap::{Arg, ArgAction, ArgMatches, Command};

use crate::comp::{add, create, finish, list, remove, rename, solve, test};
use crate::config::Settings;
use crate::util::get_project_root;

pub fn cli() -> Command {
    Command::new("comp")
        .about("Commands related to competitions")
        .subcommand(
            Command::new("add")
                .about("Add a problem to a competition")
                .arg_required_else_help(true)
                .args([
                    Arg::new("comp")
                        .short('c')
                        .long("comp")
                        .help("Competition name")
                        .action(ArgAction::Set)
                        .required(true),
                    Arg::new("problem")
                        .short('p')
                        .long("problem")
                        .help("Problem name")
                        .action(ArgAction::Set)
                        .required(true),
                ]),
        )
        .subcommand(
            Command::new("create")
                .about("Create a new competition")
                .arg_required_else_help(true)
                .arg(
                    Arg::new("name")
                        .help("Competition name")
                        .action(ArgAction::Set)
                        .required(true),
                ),
        )
        .subcommand(
            Command::new("finish")
                .about("Finish a competition and archive problems from the competition")
                .args([Arg::new("comp")
                    .help("Competition name")
                    .action(ArgAction::Set)
                    .required(true)]),
        )
        .subcommand(
            Command::new("list")
                .about("List all competitions or list problems in a competition")
                .args([Arg::new("comp")
                    .short('c')
                    .long("comp")
                    .help("Competition name")
                    .action(ArgAction::Set)]),
        )
        .subcommand(
            Command::new("remove")
                .about("Remove a problem from a competition")
                .arg_required_else_help(true)
                .args([
                    Arg::new("comp")
                        .short('c')
                        .long("comp")
                        .help("Competition name")
                        .action(ArgAction::Set)
                        .required(true),
                    Arg::new("problem")
                        .short('p')
                        .long("problem")
                        .help("Problem name")
                        .action(ArgAction::Set)
                        .required(true),
                ]),
        )
        .subcommand(
            Command::new("rename")
                .about("Rename a competition")
                .arg_required_else_help(true)
                .args([
                    Arg::new("old_name")
                        .long("old-name")
                        .help("Old competition name")
                        .action(ArgAction::Set)
                        .required(true),
                    Arg::new("new_name")
                        .long("new-name")
                        .help("New competition name")
                        .action(ArgAction::Set)
                        .required(true),
                ]),
        )
        .subcommand(
            Command::new("solve")
                .about("Generate output test cases for all problems in a competition")
                .arg_required_else_help(true)
                .args([
                    Arg::new("comp")
                        .help("Competition name")
                        .action(ArgAction::Set)
                        .required(true),
                    Arg::new("lang")
                        .long("lang")
                        .help("Language of the solution file (e.g. cpp, py)")
                        .action(ArgAction::Set),
                ]),
        )
        .subcommand(
            Command::new("test")
                .about("Run tests on all problems in a competition")
                .arg_required_else_help(true)
                .args([
                    Arg::new("comp")
                        .help("Competition name")
                        .action(ArgAction::Set)
                        .required(true),
                    Arg::new("lang")
                        .long("lang")
                        .help("Language of the solution file (e.g. cpp, py)")
                        .action(ArgAction::Set),
                ]),
        )
        .subcommand_required(true)
}

pub fn exec(args: &ArgMatches, settings: &Settings) -> Result<()> {
    let problems_dir = get_project_root()?.join(&settings.problems_dir);
    if !fs::exists(&problems_dir).expect("Failed to check if path exists") {
        fs::create_dir(&problems_dir).expect("Failed to create directory");
    }

    match args.subcommand() {
        Some(("add", cmd)) => {
            let comp_name = cmd
                .try_get_one::<String>("comp")?
                .context("Competition name is required")?;
            let problem_name = cmd
                .try_get_one::<String>("problem")?
                .context("Problem name is required")?;

            add::add(&problems_dir, comp_name, problem_name)?;
        }
        Some(("create", cmd)) => {
            let comp_name = cmd
                .try_get_one::<String>("name")?
                .context("Competition name is required")?;

            create::create(&problems_dir, comp_name)?;
        }
        Some(("finish", cmd)) => {
            let comp_name = cmd
                .try_get_one::<String>("comp")?
                .context("Competition name is required")?;

            finish::finish(&problems_dir, comp_name)?;
        }
        Some(("list", cmd)) => {
            let comp_name = cmd.try_get_one::<String>("comp")?;

            list::list(&problems_dir, comp_name)?;
        }
        Some(("remove", cmd)) => {
            let comp_name = cmd
                .try_get_one::<String>("comp")?
                .context("Competition name is required")?;
            let problem_name = cmd
                .try_get_one::<String>("problem")?
                .context("Problem name is required")?;

            remove::remove(&problems_dir, comp_name, problem_name)?;
        }
        Some(("rename", cmd)) => {
            let old_comp_name = cmd
                .try_get_one::<String>("old_name")?
                .context("Old competition name is required")?;
            let new_comp_name = cmd
                .try_get_one::<String>("new_name")?
                .context("New competition name is required")?;

            rename::rename(&problems_dir, old_comp_name, new_comp_name)?;
        }
        Some(("solve", cmd)) => {
            let comp_name = cmd
                .try_get_one::<String>("comp")?
                .context("Competition name is required")?;
            let solution_lang = cmd.try_get_one::<String>("lang")?;

            solve::solve(settings, &problems_dir, comp_name, solution_lang)?;
        }
        Some(("test", cmd)) => {
            let comp_name = cmd
                .try_get_one::<String>("comp")?
                .context("Competition name is required")?;
            let solution_lang = cmd.try_get_one::<String>("lang")?;

            test::test(settings, &problems_dir, comp_name, solution_lang)?;
        }
        _ => {}
    }

    Ok(())
}
