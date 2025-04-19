use std::collections::{BTreeMap, HashMap};
use std::fs::{self, File};
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde_json::{json, to_writer, to_writer_pretty};
use walkdir::WalkDir;

use crate::problem::PROBLEM_MAPPINGS_FILE;
use crate::util::get_project_root;

/// Sync problem mappings
///
/// The problem mappings file, `problem-mappings.json` maps the problem name
/// to the relative file location of the problem.
pub fn sync_mappings(problems_dir: &PathBuf) -> Result<()> {
    let project_root = get_project_root()?;
    let problems_dir = fs::canonicalize(problems_dir)?;
    let path = &problems_dir.join(PROBLEM_MAPPINGS_FILE);
    if !fs::exists(path)? {
        let file = File::create(path)?;
        to_writer(&file, &json!({}))?;
    }

    let mut mappings_file = File::open(path)?;
    let mut mappings: HashMap<String, String> = serde_json::from_reader(&mappings_file)?;

    // Remove non-existent problems
    mappings.retain(|_, path| fs::exists(path).unwrap_or(false));

    // Each problem would be in `./<status>/<difficulty>/<problem-name>`,
    // i.e. a depth of 3
    for entry in WalkDir::new(problems_dir)
        .min_depth(3)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_dir())
    {
        let relative_path = entry
            .path()
            .strip_prefix(&project_root)?
            .to_str()
            .context("Could not convert Path to str")?
            .to_string();

        let folder_name = entry
            .path()
            .file_name()
            .context("Could not get folder name")?
            .to_str()
            .context("Could not convert OS string to str")?
            .to_string();
        mappings.insert(folder_name, relative_path);
    }

    let ordered_mappings: BTreeMap<_, _> = mappings.into_iter().collect();
    mappings_file = File::options().write(true).truncate(true).open(path)?;
    to_writer_pretty(mappings_file, &ordered_mappings)?;

    Ok(())
}

/// Get path to the problem from the problem mappings file.
/// If the problem doesn't exist, then the problem mappings file is synced.
pub fn get_problem(problems_dir: &PathBuf, problem: &str) -> Result<String> {
    let mappings_file_path = &problems_dir.join(PROBLEM_MAPPINGS_FILE);
    if !fs::exists(mappings_file_path)? {
        let file = File::create(mappings_file_path)?;
        to_writer(&file, &json!({}))?;
    }

    let mut mappings_file = File::open(mappings_file_path)?;
    let mut mappings: HashMap<String, String> = serde_json::from_reader(&mappings_file)?;

    // Return problem name if possible
    match mappings.get(problem) {
        Some(val) => Ok(val.clone()),
        None => {
            // Sync mappings file and try again
            sync_mappings(problems_dir)?;
            mappings_file = File::open(mappings_file_path)?;
            mappings = serde_json::from_reader(&mappings_file)?;

            let val = mappings
                .get(problem)
                .context("Failed to get the problem. Maybe it doesn't exist?")?;
            Ok(val.clone())
        }
    }
}
