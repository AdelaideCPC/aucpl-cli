use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::time::Duration;

use anyhow::{Context, Result};

use crate::config::Settings;
use crate::problem::run::{RunCommand, RunnableCategory, RunnableFile};
use crate::util::{get_input_files_in_directory, get_project_root};

use super::sync_mappings::get_problem;

/// Automatically run tests on the problem.
pub fn test(
    settings: &Settings,
    problems_dir: &Path,
    problem_name: &str,
    solution_file_name: Option<&str>,
    solution_lang: Option<&String>,
) -> Result<()> {
    let project_root = get_project_root()?;
    let problem_path = project_root.join(get_problem(problems_dir, problem_name)?);

    let solution_lang = solution_lang.unwrap_or(&settings.problem.default_lang);
    let mut solution_file = format!("solution.{solution_lang}");

    // Use custom solution file or script file if it exists
    if solution_file_name.is_some() {
        solution_file = solution_file_name
            .context("Failed to get solution file name")?
            .to_string();
    }

    let runnable_file =
        RunnableFile::new(settings, RunnableCategory::Solution, Some(&solution_file))?;

    let run_command = RunCommand::new(
        settings,
        &problem_path,
        &runnable_file,
        problem_path.join("solutions/solution.out"),
        problem_path.join(&solution_file),
    )?;

    let test_files = get_input_files_in_directory(problem_path.join("tests"))?;

    eprintln!("Running the solution file for each test case...");

    let mut tests_passed = 0;
    let mut total_tests = 0;
    let mut total_time: Duration = Duration::new(0, 0);

    for test_file in test_files {
        let input_file_path = problem_path.join(format!("tests/{test_file}"));
        let output_file_path = problem_path.join(format!(
            "tests/{}.out",
            test_file
                .strip_suffix(".in")
                .context("Failed to strip suffix of test file")?
        ));

        let result = run_command.get_result(Some(&input_file_path))?;

        let mut output_file = File::open(output_file_path)?;
        let out_str = result.output;
        let elapsed_time = result.elapsed_time;

        // Compare the output with the expected output
        let expected: &mut Vec<u8> = &mut Vec::new();
        output_file.read_to_end(expected)?;

        if expected != out_str.as_bytes() {
            eprintln!(
                "  ! Test case failed: {test_file}, time taken: {:.5}s",
                elapsed_time.as_secs_f64()
            );
        } else {
            eprintln!(
                "  + Test case passed: {test_file}, time taken: {:.5}s",
                elapsed_time.as_secs_f64()
            );
            tests_passed += 1;
        }

        total_tests += 1;
        total_time += elapsed_time;
    }

    eprintln!(
        "{tests_passed} out of {total_tests} test cases passed, time taken: {:.5}s",
        total_time.as_secs_f64()
    );

    run_command.cleanup()?;

    Ok(())
}
