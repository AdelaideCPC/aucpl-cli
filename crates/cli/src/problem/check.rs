//! Validate the folder structure and files for a problem.
//!
//! TODO: Add linting and formatting checks

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Result};
use regex::Regex;

use crate::problem::sync_mappings::get_problem;
use crate::problem::PROBLEM_NAME_REGEX_PATTERN;
use crate::util::{get_files_in_directory, is_file_empty};

static REQUIRED_FILES: &[&str] = &["problem.md", "solutions", "tests"];

struct TestData {
    input_exists: bool,
    output_exists: bool,
}

/// Verify that there are no missing folders or files. If the file/folder
/// doesn't exist, it will return the one that is missing.
///
/// NOTE: This only checks the top-level so far.
fn valid_folder_structure(problem_dir: &PathBuf) -> Result<(&str, bool)> {
    for file in REQUIRED_FILES {
        if !Path::new(&problem_dir).join(file).try_exists()? {
            return Ok((file, false));
        }
    }

    Ok(("", true))
}

/// Check that no files or tests are missing, and that the problem name is valid
pub fn check(problems_dir: PathBuf, problem_name: &str) -> Result<()> {
    println!("Begin check...");

    let problem_path = get_problem(&problems_dir, problem_name)?;
    let path = PathBuf::new().join(problem_path);
    if !fs::exists(&path)? {
        bail!("The given path {path:?} doesn't exist");
    }

    // Check that the problem folder name is valid
    let re = Regex::new(PROBLEM_NAME_REGEX_PATTERN)?;
    if !re.is_match(problem_name) {
        bail!("The problem name is invalid. It may only contain alphanumeric characters, dashes, and underscores");
    }

    let (file, exists) = valid_folder_structure(&path)?;
    if !exists {
        bail!("The folder structure is not valid! Missing file {file}");
    }
    println!("Folder structure for '{}' is valid", problem_name);

    // Check that test files are valid, i.e.:
    // - A .in file must have a corresponding .out file
    // - The files are non-empty

    let tests_path = path.join("tests");
    let test_files = get_files_in_directory(&tests_path)?;
    let mut tests_data: HashMap<String, TestData> = HashMap::new();

    if test_files.is_empty() {
        bail!("There are no test files present!");
    }

    for file in &test_files {
        let test_path = &tests_path.join(file);

        match is_file_empty(test_path) {
            Ok(false) => {}
            Ok(true) => println!("Warning: `{}` is an empty file", &file),
            Err(err) => bail!("Failed to check if `{}` was empty: {err}", &file),
        }

        let file_parts: Vec<_> = file.split(".").collect();
        if file_parts.iter().len() != 2 {
            eprintln!("The test file name `{file}` is invalid. It must be in the format `<test_name>.<in|out>`");
            continue;
        }
        let test_name = file_parts[0];
        let test_suffix = file_parts[1];
        let test_entry = tests_data.entry(test_name.to_string()).or_insert(TestData {
            input_exists: false,
            output_exists: false,
        });

        match test_suffix {
            "in" => test_entry.input_exists = true,
            "out" => test_entry.output_exists = true,
            _ => {
                eprintln!("The file extension `.{test_suffix}` is invalid. Only `.in` and `.out` are valid. This file will be skipped...");
                continue;
            }
        }
    }

    let mut invalid_tests = 0;

    for (name, data) in &tests_data {
        if data.input_exists && data.output_exists {
            println!("Test case '{name}' is valid");
        } else {
            let in_exists = if data.input_exists {
                format!("`{name}.in` exists")
            } else {
                format!("`{name}.in` doesn't exist")
            };
            let out_exists = if data.output_exists {
                format!("`{name}.out` exists")
            } else {
                format!("`{name}.out` doesn't exist")
            };
            eprintln!("Test case '{name}' is invalid! Reason: {in_exists}, {out_exists}");
            invalid_tests += 1;
        }
    }

    if tests_data.iter().len() < 2 {
        println!("Warning: You have fewer than two test cases! Maybe add a few more?");
    }

    println!("Check completed. Found {invalid_tests} invalid tests");

    Ok(())
}
