use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};

use crate::config::SETTINGS_FILE;
use crate::problem::sync_mappings::problem_exists;

/// Figure out the root directory of the AUCPL problemset project.
pub fn get_project_root() -> Result<PathBuf> {
    let mut path = std::env::current_dir()?;
    while !path.join(SETTINGS_FILE).exists() {
        if !path.pop() {
            bail!("Could not find the project root: {SETTINGS_FILE} not found");
        }
    }

    Ok(path)
}

/// Get the name of the problem from the current working directory if the
/// directory is a valid problem folder.
pub fn get_problem_from_cwd(problems_dir: &Path) -> Result<String> {
    let project_root = get_project_root()?;
    let path = std::env::current_dir()?;
    if project_root == path {
        bail!("You are in the project root directory. Please navigate to a problem directory");
    }
    let problem_name = path
        .file_name()
        .context("Failed to get problem name")?
        .to_str()
        .context("Failed to convert problem name to string")?
        .to_owned();

    // Validate that the problem exists
    let exists = problem_exists(problems_dir, &problem_name)?;
    if !exists {
        bail!("Problem '{problem_name}' does not exist. Make sure you are in a valid problem directory and that the problem mappings file is up to date");
    }

    Ok(problem_name)
}

/// Get a list of files in a directory that are not directories.
pub fn get_files_in_directory<P: AsRef<Path>>(path: P) -> Result<Vec<String>> {
    let entries = fs::read_dir(path)?;
    let file_names = entries
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            if path.is_file() {
                path.file_name()?.to_str().map(|s| s.to_owned())
            } else {
                None
            }
        })
        .collect();

    Ok(file_names)
}

pub fn get_input_files_in_directory<P: AsRef<Path>>(path: P) -> Result<Vec<String>> {
    let files = get_files_in_directory(path)?
        .into_iter()
        .filter(|name| name.ends_with(".in"))
        .collect();
    Ok(files)
}

pub fn is_file_empty<P: AsRef<Path>>(path: P) -> Result<bool> {
    let metadata = fs::metadata(path)?;
    Ok(metadata.len() == 0)
}

pub fn get_lang_from_extension<P: AsRef<Path>>(path: P) -> Result<String> {
    let lang = path
        .as_ref()
        .extension()
        .and_then(|s| s.to_str())
        .context("Failed to get file extension")?
        .to_string();
    Ok(lang)
}
