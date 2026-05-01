//! CLI commands and helper functions related to problems.

use std::fs;
use std::path::Path;

use anyhow::{bail, Context, Result};
use normpath::PathExt;

pub mod archive;
pub mod category;
pub mod check;
pub mod compare;
pub mod create;
pub mod fuzz;
pub mod generate;
pub mod reclassify;
pub mod run;
pub mod solve;
pub mod sync_mappings;
pub mod test;

#[cfg(test)]
pub mod test_support;

pub const PROBLEM_MAPPINGS_FILE: &str = "problem-mappings.json";
pub const PROBLEM_NAME_REGEX_PATTERN: &str = r"^[A-Za-z0-9_-]+$";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProblemLocation {
    pub status: String,
    pub category: String,
    pub problem_name: String,
}

pub(crate) fn problem_location_from_path(
    problems_dir: &Path,
    problem_path: &Path,
) -> Result<ProblemLocation> {
    let normalized_problems_dir = problems_dir.normalize()?.into_path_buf();
    let normalized_problem_path = problem_path.normalize()?.into_path_buf();

    let relative_path = normalized_problem_path
        .strip_prefix(&normalized_problems_dir)
        .with_context(|| {
            format!(
                "Problem path '{}' is not inside problems directory '{}'",
                problem_path.display(),
                problems_dir.display()
            )
        })?;

    let components = relative_path
        .iter()
        .map(|component| {
            component.to_str().context(format!(
                "Problem path '{}' contains invalid UTF-8",
                problem_path.display()
            ))
        })
        .collect::<Result<Vec<_>>>()?;

    match components.as_slice() {
        [status, category, problem_name] if matches!(*status, "new" | "archive") => {
            Ok(ProblemLocation {
                status: (*status).to_owned(),
                category: (*category).to_owned(),
                problem_name: (*problem_name).to_owned(),
            })
        }
        [status, ..] => bail!(
            "Invalid problem path '{}'. Unknown status '{}'.",
            problem_path.display(),
            status
        ),
        _ => bail!(
            "Invalid problem path '{}'. Expected '<status>/<category>/<problem-name>'.",
            problem_path.display()
        ),
    }
}

pub(crate) fn remove_dir_if_empty(path: &Path) -> Result<()> {
    if !fs::exists(path)? {
        return Ok(());
    }

    let mut entries = fs::read_dir(path)?;
    match entries.next() {
        None => fs::remove_dir(path)?,
        Some(entry) => {
            entry?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use super::{problem_location_from_path, ProblemLocation};

    #[test]
    fn problem_location_from_path_handles_dot_components_in_problems_dir() {
        let tempdir = TempDir::new().expect("tempdir should be created");
        let problems_dir = tempdir.path().join("problems");
        let dotted_problems_dir = tempdir.path().join(".").join("problems");
        let problem_path = problems_dir.join("new").join("easy").join("two-sum");

        fs::create_dir_all(&problem_path).expect("problem path should exist");

        let location = problem_location_from_path(&dotted_problems_dir, &problem_path)
            .expect("problem path should resolve inside problems dir");

        assert_eq!(
            location,
            ProblemLocation {
                status: "new".to_owned(),
                category: "easy".to_owned(),
                problem_name: "two-sum".to_owned(),
            }
        );
    }
}
