//! Archive a problem (e.g. problems that have been used in a competition).

use std::fs;
use std::path::Path;

use anyhow::{bail, Result};

use crate::util::get_project_root;

use super::sync_mappings::{get_problem, sync_mappings};

/// Archive problems by moving them from the `new` to the `archive`
/// problems folder.
pub fn archive(problems_dir: &Path, problem_name: &str) -> Result<()> {
    sync_mappings(problems_dir)?;

    let problem_path = match get_problem(problems_dir, problem_name) {
        Ok(val) => val,
        Err(_) => {
            bail!("Failed to archive the problem. Does the problem exist?");
        }
    };

    let updated_problem_path = problem_path.replace("new", "archive");
    let project_root = get_project_root()?;

    // Create the directory and its parent folders in case it doesn't exist
    fs::create_dir_all(&updated_problem_path)?;
    fs::rename(
        project_root.join(problem_path),
        project_root.join(updated_problem_path),
    )?;
    sync_mappings(problems_dir)?;

    Ok(())
}
