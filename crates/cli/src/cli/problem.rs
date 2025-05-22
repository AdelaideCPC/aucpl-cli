use std::fs;

use anyhow::{anyhow, Context, Result};
use clap::{value_parser, Arg, ArgAction, ArgMatches, Command};

use crate::config::Settings;
use crate::problem::{archive, check, compare, create, generate, solve, test};
use crate::util::{get_problem_from_cwd, get_project_root};

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
            Command::new("compare")
                .about("Compare two solutions")
                .args([
                    Arg::new("file1")
                        .long("file1")
                        .help("Name of the first solution file")
                        .action(ArgAction::Set),
                    Arg::new("lang1")
                        .long("lang1")
                        .help("Language of the first solution file (e.g. cpp, py)")
                        .action(ArgAction::Set),
                    Arg::new("file2")
                        .long("file2")
                        .help("Name of the second solution file")
                        .action(ArgAction::Set),
                    Arg::new("lang2")
                        .long("lang2")
                        .help("Language of the second solution file (e.g. cpp, py)")
                        .action(ArgAction::Set),
                    Arg::new("problem")
                        .short('p')
                        .long("problem")
                        .help("Problem name (this is not the problem title)")
                        .action(ArgAction::Set),
                ]),
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
            Command::new("generate")
                .about("Generate a test case")
                .args([
                    Arg::new("file")
                        .long("file")
                        .help("Name of the first generator file")
                        .action(ArgAction::Set),
                    Arg::new("lang")
                        .long("lang")
                        .help("Language of the first generator file (e.g. cpp, py)")
                        .action(ArgAction::Set),
                    Arg::new("problem")
                        .short('p')
                        .long("problem")
                        .help("Problem name (this is not the problem title)")
                        .action(ArgAction::Set),
                ]),
        )
        .subcommand(
            Command::new("solve")
                .about("Automatically generate test outputs for a problem, given pre-existing input files")
                .args([
                    Arg::new("file")
                        .long("file")
                        .help("Name of the solution file")
                        .action(ArgAction::Set),
                    Arg::new("lang")
                        .long("lang")
                        .help("Language of the solution file (e.g. cpp, py)")
                        .action(ArgAction::Set),
                    Arg::new("problem")
                        .short('p')
                        .long("problem")
                        .help("Problem name (this is not the problem title)")
                        .action(ArgAction::Set),
                ]),
        )
        .subcommand(
            Command::new("test")
                .about("Run tests on a given problem")
                .args([
                    Arg::new("file")
                        .long("file")
                        .help("Name of the solution file")
                        .action(ArgAction::Set),
                    Arg::new("lang")
                        .long("lang")
                        .help("Language of the solution file (e.g. cpp, py)")
                        .action(ArgAction::Set),
                    Arg::new("problem")
                        .short('p')
                        .long("problem")
                        .help("Problem name (this is not the problem title)")
                        .action(ArgAction::Set)
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
        Some(("compare", cmd)) => {
            let problem_name = match cmd.try_get_one::<String>("problem")? {
                Some(name) => name,
                None => &get_problem_from_cwd(&problems_dir)?,
            };

            let solution_file_1 = cmd.try_get_one::<String>("file1")?.map(|f| f.as_str());
            let solution_lang_1 = cmd.try_get_one::<String>("lang1")?;
            let solution_file_2 = cmd.try_get_one::<String>("file2")?.map(|f| f.as_str());
            let solution_lang_2 = cmd.try_get_one::<String>("lang2")?;

            compare::compare(
                settings,
                &problems_dir,
                problem_name,
                solution_file_1,
                solution_lang_1,
                solution_file_2,
                solution_lang_2,
            )?;
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
        Some(("generate", cmd)) => {
            let problem_name = match cmd.try_get_one::<String>("problem")? {
                Some(name) => name,
                None => &get_problem_from_cwd(&problems_dir)?,
            };

            let generator_file = cmd.try_get_one::<String>("file")?.map(|f| f.as_str());
            let generator_lang = cmd.try_get_one::<String>("lang")?;

            generate::generate(
                settings,
                &problems_dir,
                problem_name,
                generator_file,
                generator_lang,
            )?;
        }
        Some(("solve", cmd)) => {
            let problem_name = match cmd.try_get_one::<String>("problem")? {
                Some(name) => name,
                None => &get_problem_from_cwd(&problems_dir)?,
            };

            let solution_file = cmd.try_get_one::<String>("file")?.map(|f| f.as_str());
            let solution_lang = cmd.try_get_one::<String>("lang")?;

            solve::solve(
                settings,
                &problems_dir,
                problem_name,
                solution_file,
                solution_lang,
            )?;
        }
        Some(("test", cmd)) => {
            let problem_name = match cmd.try_get_one::<String>("problem")? {
                Some(name) => name,
                None => &get_problem_from_cwd(&problems_dir)?,
            };

            let solution_file = cmd.try_get_one::<String>("file")?.map(|f| f.as_str());
            let solution_lang = cmd.try_get_one::<String>("lang")?;

            test::test(
                settings,
                &problems_dir,
                problem_name,
                solution_file,
                solution_lang,
            )?;
        }
        _ => {}
    }

    Ok(())
}
