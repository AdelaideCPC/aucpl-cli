//! Reclassify (rerate) a problem's difficulty.

use std::fs;
use std::path::Path;

use anyhow::{bail, Result};

use crate::paths::resolve_stored_path;
use crate::problem::difficulty::calculate_difficulty_bucket;
use crate::problem::sync_mappings::{get_problem, sync_mappings};
use crate::util::get_project_root;

/// Reclassify a problem's difficulty by moving it to a new difficulty folder.
///
/// If the problem is not provided, it will attempt to get the problem name
/// from the current working directory.
pub fn reclassify(problems_dir: &Path, problem_name: &str, difficulty: u16) -> Result<()> {
    // Sync mappings first to ensure we have up-to-date problem locations
    sync_mappings(problems_dir)?;

    // Get the current problem path from mappings
    let problem_path_str = match get_problem(problems_dir, problem_name) {
        Ok(val) => val,
        Err(_) => {
            bail!("Failed to reclassify the problem '{problem_name}'. Does the problem exist?");
        }
    };

    let project_root = get_project_root()?;
    let current_problem_path = resolve_stored_path(&project_root, &problem_path_str);

    // Calculate the new difficulty bucket
    let (bucketed_difficulty, difficulty_str) = calculate_difficulty_bucket(difficulty)?;

    // Extract the status (new/archive) from the current path
    let components: Vec<_> = problem_path_str.split('/').collect();
    let status = components.get(1).copied().unwrap_or("new");

    // Build the new path
    let new_problem_path = problems_dir
        .join(status)
        .join(&difficulty_str)
        .join(problem_name);

    // Check if destination already exists
    if fs::exists(&new_problem_path)? {
        bail!(
            "Cannot reclassify: a problem named '{problem_name}' already exists in difficulty '{difficulty_str}'"
        );
    }

    // Create the parent directory if it doesn't exist
    if let Some(parent) = new_problem_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Move the problem directory
    fs::rename(&current_problem_path, &new_problem_path)?;

    // Sync mappings to update the path
    sync_mappings(problems_dir)?;

    if difficulty == 0 {
        eprintln!("Reclassified problem '{problem_name}' to unrated");
    } else {
        eprintln!("Reclassified problem '{problem_name}' to difficulty {bucketed_difficulty}");
    }

    Ok(())
}
