use std::fs;

use anyhow::{Context, Result};
use clap::{Arg, ArgAction, ArgMatches, Command};
use normpath::PathExt;

use crate::config::get_settings;
use crate::paths::resolve_stored_path;
use crate::problem::sync_mappings::get_problem;
use crate::util::get_project_root;

pub fn cli() -> Command {
    Command::new("cd")
        .about(
            "Print the target directory for a problem or the workspace root.
Evaluate `aucpl shellinit` to instead cd to the directory.",
        )
        .arg(
            Arg::new("problem")
                .help("Problem name")
                .action(ArgAction::Set),
        )
}

pub fn exec(args: &ArgMatches) -> Result<()> {
    let settings = get_settings()?;
    let project_root = get_project_root()?;

    let problems_dir = project_root
        .join(&settings.problems_dir)
        .normalize()?
        .into_path_buf();
    fs::create_dir_all(&problems_dir).with_context(|| {
        format!(
            "Failed to create problems directory at {}",
            problems_dir.display()
        )
    })?;

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
