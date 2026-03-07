use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use numeric_sort::sort_unstable;

use crate::config::SETTINGS_FILE_NAME;
use crate::errors::CliError;
use crate::problem::sync_mappings::problem_exists;
use crate::suggest::suggest_corrections;

/// Figure out the root directory of the AUCPL problemset project.
pub fn get_project_root() -> Result<PathBuf> {
    let start_path = std::env::current_dir()?;
    let mut path = start_path.clone();

    while !path.join(SETTINGS_FILE_NAME).exists() {
        if !path.pop() {
            return Err(CliError::ProjectNotFound {
                searched_from: start_path.clone(),
                verbose: format!(
                    "Searched from: {}\nLooking for: {} in current and parent directories",
                    start_path.display(),
                    SETTINGS_FILE_NAME
                ),
                suggestions: vec![
                    "Make sure you're in an AUCPL project directory".to_owned(),
                    "Run 'aucpl init' to initialize a new project".to_owned(),
                ],
            }
            .into());
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
        return Err(CliError::InvalidInput {
            message: "No problem specified - you are in the project root directory".to_owned(),
            verbose: Some(format!(
                "Current directory: {}\nThis is the project root, not a problem directory",
                path.display()
            )),
            suggestions: vec![
                "Navigate to a specific problem directory and run the command again".to_owned(),
                "Use the -p/--problem flag to specify a problem name from the project root"
                    .to_owned(),
            ],
        }
        .into());
    }

    let problem_name = path
        .file_name()
        .ok_or_else(|| CliError::InvalidInput {
            message: "Could not determine problem name from current directory".to_owned(),
            verbose: Some(format!(
                "Current directory: {}\nFailed to extract directory name",
                path.display()
            )),
            suggestions: vec![
                "Make sure you are in a valid problem subdirectory".to_owned(),
                "Use the -p/--problem flag to specify the problem name explicitly".to_owned(),
            ],
        })?
        .to_str()
        .ok_or_else(|| CliError::InvalidInput {
            message: "Problem name contains invalid characters (non-UTF8)".to_owned(),
            verbose: Some("The directory name could not be converted to a valid string".to_owned()),
            suggestions: vec![
                "Use ASCII characters only in directory names".to_owned(),
                "Avoid special characters and spaces in directory names".to_owned(),
                "Rename the directory to use only alphanumeric characters, dashes, and underscores"
                    .to_owned(),
            ],
        })?
        .to_owned();

    // Validate that the problem exists
    let exists = problem_exists(problems_dir, &problem_name)?;
    if !exists {
        // Get list of all problems to suggest similar names
        let all_problems =
            crate::problem::sync_mappings::get_all_problem_names(problems_dir).unwrap_or_default();
        let candidates: Vec<&str> = all_problems.iter().map(|s| s.as_str()).collect();
        let suggestions = suggest_corrections(&problem_name, &candidates, 3);

        return Err(CliError::NotFound {
            resource_type: "problem".to_owned(),
            name: problem_name.clone(),
            verbose: Some(format!(
                "Problem '{}' not found in the mappings file\nSearched in: {}",
                problem_name,
                problems_dir.display()
            )),
            suggestions,
        }
        .into());
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
    let mut files: Vec<_> = get_files_in_directory(path)?
        .into_iter()
        .filter(|name| name.ends_with(".in"))
        .collect();
    sort_unstable(&mut files);
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
        .to_owned();
    Ok(lang)
}
