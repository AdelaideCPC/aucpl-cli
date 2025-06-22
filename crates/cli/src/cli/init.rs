use std::env;
use std::fs;

use anyhow::{bail, Context, Result};
use clap::{Arg, ArgMatches, Command};

use crate::config::SETTINGS_FILE_DEFAULT_CONTENTS;
use crate::config::SETTINGS_FILE_NAME;

pub fn cli() -> Command {
    Command::new("init")
        .about("Initialise a new project and generate necessary files")
        .arg(
            Arg::new("name")
                .long("name")
                .help("Name of the project")
                .action(clap::ArgAction::Set)
                .required(true),
        )
}

pub fn exec(args: &ArgMatches) -> Result<()> {
    let project_name = args
        .get_one::<String>("name")
        .context("Project name is required")?;

    let project_root = env::current_dir().context("Failed to get current directory")?;
    let project_path = project_root.join(project_name);
    if project_path.exists() {
        bail!(
            "Project directory already exists: {}",
            project_path.display()
        );
    }

    fs::create_dir(&project_path).context(format!(
        "Failed to create project directory: {}",
        project_path.display()
    ))?;
    fs::create_dir(project_path.join("problems")).context(format!(
        "Failed to create problems directory: {}",
        project_path.display()
    ))?;

    fs::write(
        project_path.join(SETTINGS_FILE_NAME),
        SETTINGS_FILE_DEFAULT_CONTENTS,
    )
    .context("Could not create settings file")?;

    eprintln!("Created project '{project_name}'");

    Ok(())
}
