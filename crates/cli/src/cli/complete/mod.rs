//! Shell completion entrypoints and final result rendering.

use std::collections::BTreeSet;

use anyhow::Result;
use clap::{value_parser, Arg, ArgAction, ArgMatches, Command};

use crate::cli::complete::resolve::{resolve_request, CompletionRequest};
use crate::cli::complete::values::complete_arg_values;

mod resolve;
mod values;

/// Final completion payload ready to be printed to stdout.
struct CompletionResult {
    /// Candidate values to emit.
    values: Vec<String>,
    /// Prefix to prepend to each printed completion.
    replacement_prefix: String,
}

/// Define the hidden internal completion subcommand.
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

/// Execute the hidden completion command by printing matching candidates.
pub fn exec(args: &ArgMatches) -> Result<()> {
    let cword = args.get_one::<usize>("cword").copied().unwrap_or(0);
    let words: Vec<String> = args
        .try_get_many::<String>("words")?
        .map(|vals| vals.cloned().collect())
        .unwrap_or_default();

    let root = super::root();
    let result = complete_request(resolve_request(&root, &words, cword));

    print_completions(&result.values, &result.replacement_prefix);

    Ok(())
}

fn filter_prefix_matches<I>(values: I, current: &str) -> Vec<String>
where
    I: IntoIterator<Item = String>,
{
    values
        .into_iter()
        .filter(|value| current.is_empty() || value.starts_with(current))
        .collect()
}

/// Return visible subcommand names for a command in sorted order.
fn visible_subcommands(cmd: &Command, current: &str) -> Vec<String> {
    let mut names: Vec<String> = cmd
        .get_subcommands()
        .filter(|sub| !sub.is_hide_set())
        .map(|sub| sub.get_name().to_owned())
        .collect();
    names.sort();
    filter_prefix_matches(names, current)
}

/// Return visible option names and aliases for a command in sorted order.
fn visible_options(cmd: &Command, current: &str) -> Vec<String> {
    let mut names = BTreeSet::new();

    for arg in cmd.get_arguments() {
        if arg.is_positional() || arg.is_hide_set() {
            continue;
        }

        if let Some(shorts) = arg.get_short_and_visible_aliases() {
            for short in shorts {
                names.insert(format!("-{short}"));
            }
        }

        if let Some(longs) = arg.get_long_and_visible_aliases() {
            for long in longs {
                names.insert(format!("--{long}"));
            }
        }
    }

    filter_prefix_matches(names, current)
}

/// Print completion candidates, optionally re-attaching an inline option prefix.
fn print_completions(values: &[String], replacement_prefix: &str) {
    for value in values {
        println!("{replacement_prefix}{value}");
    }
}

/// Convert a completion request into a final printable completion result.
fn complete_request(request: CompletionRequest<'_>) -> CompletionResult {
    match request {
        CompletionRequest::CommandName { cmd, current } => CompletionResult {
            values: visible_subcommands(cmd, &current),
            replacement_prefix: String::new(),
        },
        CompletionRequest::OptionName { cmd, current } => CompletionResult {
            values: visible_options(cmd, &current),
            replacement_prefix: String::new(),
        },
        CompletionRequest::ArgValue(target) => CompletionResult {
            values: complete_arg_values(target.arg, &target.current_value),
            replacement_prefix: target.replacement_prefix,
        },
        CompletionRequest::None => CompletionResult {
            values: vec![],
            replacement_prefix: String::new(),
        },
    }
}
