use std::collections::BTreeMap;
use std::fs::{self, File};
use std::path::Path;

use anyhow::Result;
use clap::{value_parser, Arg, ArgAction, ArgMatches, Command};
use serde_json::Value;

use crate::comp::COMPETITIONS_FILE;
use crate::config::get_settings;
use crate::problem::sync_mappings::get_all_problem_names;
use crate::util::get_project_root;

fn dynamic_top_level_commands() -> Vec<String> {
    let mut names: Vec<String> = super::builtin()
        .into_iter()
        .map(|cmd| cmd.get_name().to_owned())
        .filter(|name| !name.starts_with("__"))
        .collect();
    names.sort();
    names
}

fn dynamic_subcommands(parent: &str) -> Vec<String> {
    let mut names: Vec<String> = super::builtin()
        .into_iter()
        .find(|cmd| cmd.get_name() == parent)
        .map(|cmd| {
            cmd.get_subcommands()
                .map(|sub| sub.get_name().to_owned())
                .filter(|name| !name.starts_with("__"))
                .collect()
        })
        .unwrap_or_default();
    names.sort();
    names
}

fn print_prefixed(values: &[String], current: &str) {
    for value in values {
        if current.is_empty() || value.starts_with(current) {
            println!("{value}");
        }
    }
}

pub fn cli() -> Command {
    Command::new("__complete")
        .about("Internal: print completion candidates")
        .hide(true)
        .arg(
            Arg::new("cword")
                .long("cword")
                .help("Index of the current word")
                .action(ArgAction::Set)
                .value_parser(value_parser!(usize))
                .required(true),
        )
        .arg(
            Arg::new("words")
                .help("Command words as received from the shell")
                .action(ArgAction::Set)
                .num_args(0..)
                .trailing_var_arg(true)
                .allow_hyphen_values(true),
        )
}

fn get_competition_names(problems_dir: &Path) -> Vec<String> {
    let comp_file_path = problems_dir.join(COMPETITIONS_FILE);
    if !fs::exists(&comp_file_path).unwrap_or(false) {
        return vec![];
    }

    let comp_file = match File::open(&comp_file_path) {
        Ok(file) => file,
        Err(_) => return vec![],
    };

    let data: BTreeMap<String, Value> = match serde_json::from_reader(comp_file) {
        Ok(data) => data,
        Err(_) => return vec![],
    };

    data.keys().cloned().collect()
}

fn expects_problem_name(prev: Option<&str>) -> bool {
    matches!(prev, Some("-p") | Some("--problem"))
}

fn expects_competition_name(prev: Option<&str>) -> bool {
    matches!(prev, Some("-c") | Some("--comp"))
}

fn consumes_next_value(token: &str) -> bool {
    matches!(
        token,
        "-c"
            | "--comp"
            | "-p"
            | "--problem"
            | "-d"
            | "--difficulty"
            | "--file"
            | "--lang"
            | "--generator-file"
            | "--generator-lang"
            | "--test-name"
            | "--old-name"
            | "--new-name"
    )
}

fn count_positionals(words: &[String], start: usize, end_exclusive: usize) -> usize {
    let mut count = 0;
    let mut skip_next = false;

    for token in words.iter().take(end_exclusive).skip(start).map(String::as_str) {
        if skip_next {
            skip_next = false;
            continue;
        }

        if consumes_next_value(token) {
            skip_next = true;
            continue;
        }

        if token.starts_with('-') {
            continue;
        }

        count += 1;
    }

    count
}

pub fn exec(args: &ArgMatches) -> Result<()> {
    let cword = args.get_one::<usize>("cword").copied().unwrap_or(0);
    let words: Vec<String> = args
        .try_get_many::<String>("words")?
        .map(|vals| vals.cloned().collect())
        .unwrap_or_default();

    let current = words.get(cword).map(String::as_str).unwrap_or("");
    let prev = cword
        .checked_sub(1)
        .and_then(|idx| words.get(idx))
        .map(String::as_str);

    let top_level_commands = dynamic_top_level_commands();

    if cword == 1 {
        print_prefixed(&top_level_commands, current);
        return Ok(());
    }

    let mut wants_problem = expects_problem_name(prev);
    let mut wants_competition = expects_competition_name(prev);

    let top_subcommand = words.get(1).map(String::as_str);
    let nested_subcommand = words.get(2).map(String::as_str);

    if cword == 2 {
        match top_subcommand {
            Some("problem") => {
                let subcommands = dynamic_subcommands("problem");
                print_prefixed(&subcommands, current);
                return Ok(());
            }
            Some("comp") => {
                let subcommands = dynamic_subcommands("comp");
                print_prefixed(&subcommands, current);
                return Ok(());
            }
            Some("help") => {
                print_prefixed(&top_level_commands, current);
                return Ok(());
            }
            _ => {}
        }
    }

    if top_subcommand == Some("cd") && cword == 2 {
        wants_problem = true;
    }

    if top_subcommand == Some("comp")
        && matches!(nested_subcommand, Some("finish") | Some("solve") | Some("test"))
    {
        let positionals_before_current = count_positionals(&words, 3, cword);
        if positionals_before_current == 0 && !current.starts_with('-') {
            wants_competition = true;
        }
    }

    if !wants_problem && !wants_competition {
        return Ok(());
    }

    let project_root = match get_project_root() {
        Ok(path) => path,
        Err(_) => return Ok(()),
    };

    let settings = match get_settings() {
        Ok(settings) => settings,
        Err(_) => return Ok(()),
    };

    let problems_dir = project_root.join(&settings.problems_dir);

    let mut values = if wants_problem {
        get_all_problem_names(&problems_dir).unwrap_or_default()
    } else {
        get_competition_names(&problems_dir)
    };

    values.sort();

    for value in values {
        if current.is_empty() || value.starts_with(current) {
            println!("{value}");
        }
    }

    Ok(())
}