use std::fs::File;
use std::io::Write;
use std::path::Path;

use anyhow::{Context, Result};

use super::sync_mappings::get_problem;
use crate::problem::run::{RunCommand, RunnableFile};
use crate::util::get_project_root;
use crate::{config::Settings, util::get_input_files_in_directory};

/// Automatically generate test outputs for a problem, given pre-existing input files.
pub fn solve(
    settings: &Settings,
    problems_dir: &Path,
    problem_name: &str,
    solution_file: &RunnableFile,
) -> Result<()> {
    let project_root = get_project_root()?;
    let problem_path = project_root.join(get_problem(problems_dir, problem_name)?);

    let run_command = RunCommand::new(
        settings,
        &problem_path,
        solution_file,
        problem_path.join("solutions/solution.out"),
        problem_path.join(format!("{solution_file}")),
    )?;

    let test_files = get_input_files_in_directory(problem_path.join("tests"))?;

    eprintln!("Running the solution file for each test case...");
    // Run the file for every test input and generate the corresponding output
    for test_file in test_files {
        let input_file_path = problem_path.join(format!("tests/{test_file}"));
        let output_file_path = problem_path.join(format!(
            "tests/{}.out",
            test_file
                .strip_suffix(".in")
                .context("Failed to strip suffix of test file")?
        ));

        let result = run_command.get_result(Some(&input_file_path))?;
        let mut output_file = File::create(output_file_path)?;
        output_file.write_all(result.output.as_bytes())?;

        eprintln!("  - generated output for test file: {test_file}");
    }
    eprintln!("Finished generating outputs for all test cases");

    run_command.cleanup()?;

    Ok(())
}
