//! Create a new problem.

use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

use anyhow::{bail, Result};
use regex::Regex;

use crate::problem::difficulty::calculate_difficulty_bucket;
use crate::problem::sync_mappings::{get_problem, sync_mappings};
use crate::problem::PROBLEM_NAME_REGEX_PATTERN;

/// Create a new problem
pub fn create(problems_dir: &Path, problem_name: &str, difficulty: u16) -> Result<()> {
    let (bucketed_difficulty, difficulty_str) = calculate_difficulty_bucket(difficulty)?;
    let re = Regex::new(PROBLEM_NAME_REGEX_PATTERN)?;
    if !re.is_match(problem_name) {
        bail!("The problem name is invalid. It may only contain alphanumeric characters, dashes, and underscores.");
    }

    let path = &problems_dir
        .join("new")
        .join(difficulty_str)
        .join(problem_name);

    if fs::exists(path)? || get_problem(problems_dir, problem_name).is_ok() {
        bail!("The problem '{problem_name}' already exists!");
    }

    fs::create_dir_all(path)?;
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

    if difficulty == 0 {
        eprintln!("Created problem '{problem_name}' with unrated difficulty");
    } else {
        eprintln!("Created problem '{problem_name}' with difficulty {bucketed_difficulty}");
    }

    Ok(())
}
