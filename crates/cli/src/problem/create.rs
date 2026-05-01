//! Create a new problem.

use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

use anyhow::{bail, Result};
use regex::Regex;

use crate::problem::category::validate_category;
use crate::problem::sync_mappings::{get_problem, sync_mappings};
use crate::problem::PROBLEM_NAME_REGEX_PATTERN;

/// Create a new problem.
pub fn create(problems_dir: &Path, problem_name: &str, category: &str) -> Result<()> {
    validate_category(category)?;

    let re = Regex::new(PROBLEM_NAME_REGEX_PATTERN)?;
    if !re.is_match(problem_name) {
        bail!("The problem name is invalid. It may only contain alphanumeric characters, dashes, and underscores.");
    }

    let path = problems_dir.join("new").join(category).join(problem_name);

    if fs::exists(&path)? || get_problem(problems_dir, problem_name).is_ok() {
        bail!("The problem '{problem_name}' already exists!");
    }

    fs::create_dir_all(&path)?;
    fs::create_dir(path.join("solutions"))?;
    fs::create_dir(path.join("tests"))?;

    let problem_statement_template = r#"# Problem Title

Problem description.

## Input

## Output

## Example

### Input

### Output
"#;

    let mut problem_file = File::create(path.join("problem.md"))?;
    problem_file.write_all(problem_statement_template.as_bytes())?;

    sync_mappings(problems_dir)?;

    eprintln!("Created problem '{problem_name}' in category '{category}'");

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::create;
    use crate::problem::sync_mappings::get_problem;
    use crate::problem::test_support::with_test_project;

    #[test]
    fn creates_problem_in_category_directory() {
        with_test_project(|problems_dir| {
            create(problems_dir, "two-sum", "easy").expect("problem should be created");

            let problem_dir = problems_dir.join("new").join("easy").join("two-sum");
            assert!(problem_dir.is_dir());
            assert!(problem_dir.join("solutions").is_dir());
            assert!(problem_dir.join("tests").is_dir());
            assert!(problem_dir.join("problem.md").is_file());
            assert_eq!(
                get_problem(problems_dir, "two-sum").expect("mapping should exist"),
                "problems/new/easy/two-sum"
            );
        });
    }

    #[test]
    fn rejects_invalid_category() {
        with_test_project(|problems_dir| {
            let err = create(problems_dir, "two-sum", "Graphs")
                .expect_err("invalid category should be rejected");

            assert!(
                err.to_string().contains("Invalid category 'Graphs'"),
                "unexpected error: {err}"
            );
            assert!(!fs::exists(problems_dir.join("new").join("Graphs")).unwrap());
        });
    }
}
