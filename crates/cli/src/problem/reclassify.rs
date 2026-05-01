//! Reclassify a problem into a different category.

use std::fs;
use std::path::Path;

use anyhow::{bail, Result};

use crate::paths::resolve_stored_path;
use crate::problem::category::validate_category;
use crate::problem::sync_mappings::{get_problem, sync_mappings};
use crate::problem::{problem_location_from_path, remove_dir_if_empty};
use crate::util::get_project_root;

/// Reclassify a problem by moving it to a new category folder.
///
/// If the problem is not provided, it will attempt to get the problem name
/// from the current working directory.
pub fn reclassify(problems_dir: &Path, problem_name: &str, category: &str) -> Result<()> {
    validate_category(category)?;

    sync_mappings(problems_dir)?;

    let problem_path_str = match get_problem(problems_dir, problem_name) {
        Ok(val) => val,
        Err(_) => {
            bail!("Failed to reclassify the problem '{problem_name}'. Does the problem exist?");
        }
    };

    let project_root = get_project_root()?;
    let current_problem_path = resolve_stored_path(&project_root, &problem_path_str);
    let location = problem_location_from_path(problems_dir, &current_problem_path)?;

    if location.category == category {
        eprintln!("Problem '{problem_name}' is already in category '{category}'");
        return Ok(());
    }

    let new_problem_path = problems_dir
        .join(&location.status)
        .join(category)
        .join(&location.problem_name);

    if fs::exists(&new_problem_path)? {
        bail!(
            "Cannot reclassify: a problem named '{problem_name}' already exists in category '{category}'"
        );
    }

    if let Some(parent) = new_problem_path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::rename(&current_problem_path, &new_problem_path)?;
    remove_dir_if_empty(&problems_dir.join(&location.status).join(&location.category))?;
    sync_mappings(problems_dir)?;

    eprintln!("Reclassified problem '{problem_name}' to category '{category}'");

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::reclassify;
    use crate::problem::sync_mappings::{get_problem, sync_mappings};
    use crate::problem::test_support::{create_problem_dir, with_test_project};

    #[test]
    fn preserves_problem_status_when_reclassifying() {
        with_test_project(|problems_dir| {
            create_problem_dir(problems_dir, "archive", "0800", "two-sum");
            sync_mappings(problems_dir).expect("mappings should sync");

            reclassify(problems_dir, "two-sum", "easy").expect("reclassify should succeed");

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
    fn no_ops_when_problem_is_already_in_category() {
        with_test_project(|problems_dir| {
            create_problem_dir(problems_dir, "new", "easy", "two-sum");
            sync_mappings(problems_dir).expect("mappings should sync");

            reclassify(problems_dir, "two-sum", "easy").expect("same-category move should succeed");

            assert!(problems_dir
                .join("new")
                .join("easy")
                .join("two-sum")
                .is_dir());
        });
    }

    #[test]
    fn moves_problem_to_new_category_and_removes_empty_source_directory() {
        with_test_project(|problems_dir| {
            create_problem_dir(problems_dir, "new", "0800", "two-sum");
            sync_mappings(problems_dir).expect("mappings should sync");

            reclassify(problems_dir, "two-sum", "easy").expect("reclassify should succeed");

            assert!(problems_dir
                .join("new")
                .join("easy")
                .join("two-sum")
                .is_dir());
            assert!(!fs::exists(problems_dir.join("new").join("0800")).unwrap());
        });
    }
}
