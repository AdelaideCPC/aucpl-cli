use std::collections::{BTreeMap, HashMap};
use std::fs::{self, File};
use std::path::Path;

use anyhow::{Context, Result};
use normpath::PathExt;
use serde_json::{from_reader, json, to_writer, to_writer_pretty};
use walkdir::WalkDir;

use crate::errors::CliError;
use crate::paths::{convert_legacy_path, normalize_for_storage, resolve_stored_path};
use crate::problem::PROBLEM_MAPPINGS_FILE;
use crate::suggest::suggest_corrections;
use crate::util::get_project_root;

/// Sync problem mappings
///
/// The problem mappings file, `problem-mappings.json` maps the problem name
/// to the relative file location of the problem.
pub fn sync_mappings(problems_dir: &Path) -> Result<()> {
    let project_root = get_project_root()?;
    let problems_dir = problems_dir.normalize()?;
    let path = &problems_dir.join(PROBLEM_MAPPINGS_FILE);
    if !fs::exists(path)? {
        let file = File::create(path)?;
        to_writer(&file, &json!({}))?;
    }

    let mut mappings_file = File::open(path)?;
    let mut mappings: HashMap<String, String> = from_reader(&mappings_file)?;

    // Remove non-existent problems
    // Convert stored Unix paths to platform-native paths for existence check
    mappings.retain(|_, path| {
        let platform_path = resolve_stored_path(&project_root, path);
        fs::exists(&platform_path).unwrap_or(false)
    });

    // Each problem would be in `./<status>/<difficulty>/<problem-name>`,
    // i.e. a depth of 3
    for entry in WalkDir::new(problems_dir)
        .min_depth(3)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_dir())
    {
        let relative_path = normalize_for_storage(&project_root, entry.path())?;

        let folder_name = entry
            .path()
            .file_name()
            .context("Could not get folder name")?
            .to_str()
            .context("Could not convert OS string to str")?
            .to_owned();
        mappings.insert(folder_name, relative_path);
    }

    let ordered_mappings: BTreeMap<_, _> = mappings.into_iter().collect();
    mappings_file = File::options().write(true).truncate(true).open(path)?;
    to_writer_pretty(mappings_file, &ordered_mappings)?;

    Ok(())
}

/// Get path to the problem from the problem mappings file.
/// If the problem doesn't exist, then the problem mappings file is synced.
pub fn get_problem(problems_dir: &Path, problem: &str) -> Result<String> {
    let mappings_file_path = &problems_dir.join(PROBLEM_MAPPINGS_FILE);
    if !fs::exists(mappings_file_path)? {
        let file = File::create(mappings_file_path)?;
        to_writer(&file, &json!({}))?;
    }

    let mut mappings_file = File::open(mappings_file_path)?;
    let mut mappings: HashMap<String, String> = from_reader(&mappings_file)?;

    // Return problem name if possible
    // Convert legacy Windows paths (with backslashes) to Unix-style paths
    match mappings.get(problem) {
        Some(val) => Ok(convert_legacy_path(val)),
        None => {
            // Sync mappings file and try again
            sync_mappings(problems_dir)?;
            mappings_file = File::open(mappings_file_path)?;
            mappings = from_reader(&mappings_file)?;

            match mappings.get(problem) {
                Some(val) => Ok(convert_legacy_path(val)),
                None => {
                    // Get suggestions for similar problem names
                    let candidates: Vec<&str> = mappings.keys().map(|s| s.as_str()).collect();
                    let suggestions = suggest_corrections(problem, &candidates, 3);

                    Err(CliError::NotFound {
                        resource_type: "problem".to_owned(),
                        name: problem.to_owned(),
                        verbose: Some(format!("Problem '{problem}' not found in mappings file")),
                        suggestions,
                    }
                    .into())
                }
            }
        }
    }
}

/// Check if a problem exists in the problem mappings file.
pub fn problem_exists(problems_dir: &Path, problem: &str) -> Result<bool> {
    let mappings_file_path = &problems_dir.join(PROBLEM_MAPPINGS_FILE);
    if !fs::exists(mappings_file_path)? {
        let file = File::create(mappings_file_path)?;
        to_writer(&file, &json!({}))?;
    }

    let mappings_file = File::open(mappings_file_path)?;
    let mappings: HashMap<String, String> = from_reader(&mappings_file)?;

    Ok(mappings.contains_key(problem))
}

/// Get all problem names from the mappings file.
pub fn get_all_problem_names(problems_dir: &Path) -> Result<Vec<String>> {
    let mappings_file_path = &problems_dir.join(PROBLEM_MAPPINGS_FILE);
    if !fs::exists(mappings_file_path)? {
        return Ok(vec![]);
    }

    let mappings_file = File::open(mappings_file_path)?;
    let mappings: HashMap<String, String> = from_reader(&mappings_file)?;

    Ok(mappings.keys().cloned().collect())
}
