//! Archive a problem (e.g. problems that have been used in a competition).

use std::fs;
use std::path::Path;

use anyhow::{bail, Result};

use crate::paths::resolve_stored_path;
use crate::problem::{problem_location_from_path, remove_dir_if_empty};
use crate::util::get_project_root;

use super::sync_mappings::{get_problem, sync_mappings};

/// Archive problems by moving them from the `new` to the `archive`
/// problems folder.
pub fn archive(problems_dir: &Path, problem_name: &str) -> Result<()> {
    sync_mappings(problems_dir)?;

    let problem_path = match get_problem(problems_dir, problem_name) {
        Ok(val) => val,
        Err(_) => {
            bail!("Failed to archive the problem '{problem_name}'. Does the problem exist?");
        }
    };

    let project_root = get_project_root()?;
    let current_problem_path = resolve_stored_path(&project_root, &problem_path);
    let location = problem_location_from_path(problems_dir, &current_problem_path)?;

    if location.status == "archive" {
        bail!("Problem '{problem_name}' is already archived");
    }

    let updated_problem_path = problems_dir
        .join("archive")
        .join(&location.category)
        .join(&location.problem_name);

    if fs::exists(&updated_problem_path)? {
        bail!(
            "Cannot archive: a problem named '{problem_name}' already exists in category '{}'",
            location.category
        );
    }

    if let Some(parent) = updated_problem_path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::rename(&current_problem_path, &updated_problem_path)?;
    remove_dir_if_empty(&problems_dir.join("new").join(&location.category))?;
    sync_mappings(problems_dir)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::archive;
    use crate::problem::sync_mappings::{get_problem, sync_mappings};
    use crate::problem::test_support::{create_problem_dir, with_test_project};

    #[test]
    fn preserves_category_when_archiving() {
        with_test_project(|problems_dir| {
            create_problem_dir(problems_dir, "new", "easy", "two-sum");
            sync_mappings(problems_dir).expect("mappings should sync");

            archive(problems_dir, "two-sum").expect("archive should succeed");

            assert!(problems_dir
                .join("archive")
                .join("easy")
                .join("two-sum")
                .is_dir());
            assert_eq!(
                get_problem(problems_dir, "two-sum").expect("mapping should be updated"),
                "problems/archive/easy/two-sum"
            );
        });
    }

    #[test]
    fn removes_empty_source_category_directory_when_archiving() {
        with_test_project(|problems_dir| {
            create_problem_dir(problems_dir, "new", "easy", "two-sum");
            sync_mappings(problems_dir).expect("mappings should sync");

            archive(problems_dir, "two-sum").expect("archive should succeed");

            assert!(problems_dir
                .join("archive")
                .join("easy")
                .join("two-sum")
                .is_dir());
            assert!(problems_dir.join("new").is_dir());
            assert!(!fs::exists(problems_dir.join("new").join("easy")).unwrap());
        });
    }
}
