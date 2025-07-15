use std::fs;

use anyhow::{anyhow, bail, Context, Result};
use clap::{value_parser, Arg, ArgAction, ArgMatches, Command};

use crate::config::get_settings;
use crate::problem::fuzz;
use crate::problem::run::{RunnableCategory, RunnableFile};
use crate::problem::{archive, check, compare, create, generate, solve, test};
use crate::util::{get_problem_from_cwd, get_project_root};

pub fn cli() -> Command {
    Command::new("problem")
        .about("Commands related to a problem")
        .subcommand(
            Command::new("archive")
                .about("Archive a problem")
                .arg(
                    Arg::new("problem")
                        .short('p')
                        .long("problem")
                        .help("Problem name (this is not the problem title)")
                        .action(ArgAction::Set)
                ),
        )
        .subcommand(
            Command::new("check")
                .about("Check that the problem folder and test files are valid")
                .arg(
                    Arg::new("problem")
                        .short('p')
                        .long("problem")
                        .help("Problem name (this is not the problem title)")
                        .action(ArgAction::Set)
                ),
        )
        .subcommand(
            Command::new("compare")
                .about("Compare two or more solutions")
                .args([
                    Arg::new("file")
                        .long("file")
                        .help("Name of the solution file")
                        .action(ArgAction::Append),
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
            Command::new("fuzz")
                .about("Find potential edge cases in two or more solutions")
                .args([
                    Arg::new("file")
                        .long("file")
                        .help("Name of the solution file")
                        .action(ArgAction::Append),
                    Arg::new("generator-file")
                        .long("generator-file")
                        .help("Name of the generator file")
                        .action(ArgAction::Set),
                    Arg::new("problem")
                        .short('p')
                        .long("problem")
                        .help("Problem name (this is not the problem title)")
                        .action(ArgAction::Set),
                ]),
        )
        .subcommand(
            Command::new("generate")
                .about("Generate a test case input with a generator file")
                .args([
                    Arg::new("file")
                        .long("file")
                        .help("Name of the generator file")
                        .action(ArgAction::Set),
                    Arg::new("problem")
                        .short('p')
                        .long("problem")
                        .help("Problem name (this is not the problem title)")
                        .action(ArgAction::Set),
                    Arg::new("test-name")
                        .long("test-name")
                        .help("Name of the test case (default: \"generated\", which generates \"tests/generated.in\")")
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

pub fn exec(args: &ArgMatches) -> Result<()> {
    let settings = get_settings()?;
    let problems_dir = get_project_root()?.join(&settings.problems_dir);
    if !fs::exists(&problems_dir).expect("Failed to check if path exists") {
        fs::create_dir(&problems_dir).expect("Failed to create directory");
    }

    match args.subcommand() {
        Some(("archive", cmd)) => {
            let problem_name = match cmd.try_get_one::<String>("problem")? {
                Some(name) => name,
                None => &get_problem_from_cwd(&problems_dir)?,
            };

            archive::archive(&problems_dir, problem_name)?;
            eprintln!("Archived problem '{problem_name}'");
        }
        Some(("check", cmd)) => {
            let problem_name = match cmd.try_get_one::<String>("problem")? {
                Some(name) => name,
                None => &get_problem_from_cwd(&problems_dir)?,
            };

            check::check(problems_dir, problem_name)
                .map_err(|err| anyhow!("Failed check: {err}"))?;
        }
        Some(("compare", cmd)) => {
            let problem_name = match cmd.try_get_one::<String>("problem")? {
                Some(name) => name,
                None => &get_problem_from_cwd(&problems_dir)?,
            };

            let files: Vec<&String> = cmd
                .try_get_many::<String>("file")?
                .context("At least two solution files are required for comparison")?
                .collect();

            if files.len() < 2 {
                bail!("At least two solution files are required for comparison");
            }

            let mut solution_files: Vec<RunnableFile> = Vec::new();
            for f in files {
                let solution_file =
                    RunnableFile::new(&settings, RunnableCategory::Solution, Some(f), None);
                solution_files.push(solution_file?);
            }

            let compare_args = compare::CompareArgs {
                problems_dir: &problems_dir,
                problem_name: problem_name.to_string(),
                solution_files,
            };

            compare::compare(&settings, &compare_args)?;
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
        Some(("fuzz", cmd)) => {
            let problem_name = match cmd.try_get_one::<String>("problem")? {
                Some(name) => name,
                None => &get_problem_from_cwd(&problems_dir)?,
            };

            let files: Vec<&String> = cmd
                .try_get_many::<String>("file")?
                .context("At least two solution files are required for fuzzing")?
                .collect();

            if files.len() < 2 {
                bail!("At least two solution files are required for fuzzing");
            }

            let mut solution_files: Vec<RunnableFile> = Vec::new();
            for f in files {
                let solution_file =
                    RunnableFile::new(&settings, RunnableCategory::Solution, Some(f), None);
                solution_files.push(solution_file?);
            }

            let generator = RunnableFile::new(
                &settings,
                RunnableCategory::Generator,
                cmd.try_get_one::<String>("generator-file")?,
                None,
            )?;

            let fuzz_args = fuzz::FuzzArgs {
                problems_dir: &problems_dir,
                problem_name: problem_name.to_string(),
                solution_files,
                generator,
            };

            fuzz::fuzz(&settings, &fuzz_args)?;
        }
        Some(("generate", cmd)) => {
            let problem_name = match cmd.try_get_one::<String>("problem")? {
                Some(name) => name,
                None => &get_problem_from_cwd(&problems_dir)?,
            };

            let generator = RunnableFile::new(
                &settings,
                RunnableCategory::Generator,
                cmd.try_get_one::<String>("file")?,
                None,
            )?;

            let test_name = cmd
                .try_get_one::<String>("test-name")?
                .map(|f| f.as_str())
                .unwrap_or("generated");

            generate::generate(
                &settings,
                &problems_dir,
                problem_name,
                &generator,
                test_name,
            )?;
        }
        Some(("solve", cmd)) => {
            let problem_name = match cmd.try_get_one::<String>("problem")? {
                Some(name) => name,
                None => &get_problem_from_cwd(&problems_dir)?,
            };

            let solution_file = cmd.try_get_one::<String>("file")?;
            let solution_lang = cmd.try_get_one::<String>("lang")?;

            let solution_file = RunnableFile::new(
                &settings,
                RunnableCategory::Solution,
                solution_file,
                solution_lang,
            )?;

            solve::solve(&settings, &problems_dir, problem_name, &solution_file)?;
        }
        Some(("test", cmd)) => {
            let problem_name = match cmd.try_get_one::<String>("problem")? {
                Some(name) => name,
                None => &get_problem_from_cwd(&problems_dir)?,
            };

            let solution_file = cmd.try_get_one::<String>("file")?;
            let solution_lang = cmd.try_get_one::<String>("lang")?;

            let solution_file = RunnableFile::new(
                &settings,
                RunnableCategory::Solution,
                solution_file,
                solution_lang,
            )?;

            test::test(&settings, &problems_dir, problem_name, &solution_file)?;
        }
        _ => {}
    }

    Ok(())
}
