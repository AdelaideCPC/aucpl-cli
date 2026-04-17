use std::fs;

use anyhow::{Context, Result};
use clap::{Arg, ArgAction, ArgMatches, Command};

use crate::config::get_settings;
use crate::paths::resolve_stored_path;
use crate::problem::sync_mappings::get_problem;
use crate::util::get_project_root;

pub fn cli() -> Command {
    Command::new("cd")
        .about("Print the target directory for a problem or the workspace root")
        .arg(
            Arg::new("shell-hook")
                .long("shell-hook")
                .help("Print shell integration snippet so 'aucpl cd' changes your current shell directory")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("problem")
                .help("Problem name")
                .action(ArgAction::Set),
        )
}

pub fn exec(args: &ArgMatches) -> Result<()> {
    if args.get_flag("shell-hook") {
        println!(
            "aucpl() {{\n  if [ \"$1\" = \"cd\" ]; then\n    shift\n    local target\n    target=\"$(command aucpl cd \"$@\")\" || return $?\n    builtin cd -- \"$target\"\n  else\n    command aucpl \"$@\"\n  fi\n}}"
        );
        return Ok(());
    }

    let settings = get_settings()?;
    let project_root = get_project_root()?;

    let problems_dir = project_root.join(&settings.problems_dir);
    if !fs::exists(&problems_dir).expect("Failed to check if path exists") {
        fs::create_dir(&problems_dir).expect("Failed to create directory");
    }

    let target_path = match args.try_get_one::<String>("problem")? {
        Some(problem_name) => {
            let relative_path = get_problem(&problems_dir, problem_name)?;
            resolve_stored_path(&project_root, &relative_path)
        }
        None => problems_dir,
    };

    let target = target_path
        .to_str()
        .context("Target path contains invalid UTF-8")?;

    println!("{target}");

    Ok(())
}
