use std::fs;
use std::path::Path;
use std::time::Duration;

use anyhow::{Context, Result};
use uuid::Uuid;

use super::generate;
use super::run::{RunCommand, RunnableFile};
use super::sync_mappings::get_problem;
use crate::util::get_project_root;
use crate::{config::Settings, util::get_input_files_in_directory};

/// Compare two solutions.
pub fn compare(
    settings: &Settings,
    problems_dir: &Path,
    problem_name: &str,
    generate: &bool,
    solution_1: &RunnableFile,
    solution_2: &RunnableFile,
    generator: &RunnableFile,
) -> Result<()> {
    let project_root = get_project_root()?;
    let problem_path = project_root.join(get_problem(problems_dir, problem_name)?);

    let run_command_1 = RunCommand::new(
        settings,
        &problem_path,
        solution_1,
        problem_path.join("solutions/solution_1.out"),
        problem_path.join(format!("{solution_1}")),
    )?;
    let run_command_2 = RunCommand::new(
        settings,
        &problem_path,
        solution_2,
        problem_path.join("solutions/solution_2.out"),
        problem_path.join(format!("{solution_2}")),
    )?;

    if *generate {
        let mut total_tests = 0;
        let mut total_time_1: f64 = 0f64;
        let mut total_time_2: f64 = 0f64;

        loop {
            total_tests += 1;

            let test_name = format!("generated_{}", Uuid::new_v4());

            generate::generate(settings, problems_dir, problem_name, generator, &test_name)
                .context("Failed to generate test case")?;

            let input_file_path = problem_path.join(format!("tests/{}.in", test_name));

            let result_1 = run_command_1
                .get_result(Some(&input_file_path))
                .context("Failed to get output from solution 1")?;
            let result_2 = run_command_2
                .get_result(Some(&input_file_path))
                .context("Failed to get output from solution 2")?;

            if result_1.output.as_bytes() != result_2.output.as_bytes() {
                eprintln!(
                    "  ! Test case {total_tests} (tests/generated.in) failed, time taken: {:.5}s and {:.5}s respectively",
                    result_1.elapsed_time.as_secs_f64(),
                    result_2.elapsed_time.as_secs_f64()
                );
                break;
            } else {
                eprintln!(
                    "  + Test case {total_tests} passed, time taken: {:.5}s and {:.5}s respectively",
                    result_1.elapsed_time.as_secs_f64(),
                    result_2.elapsed_time.as_secs_f64()
                );
                eprintln!(
                    "    Total percentage time difference: {:.5}%",
                    (total_time_2 - total_time_1) * 100f64 / total_time_1
                );
                fs::remove_file(input_file_path).context("Failed to delete generated test case")?;
            }

            total_time_1 += result_1.elapsed_time.as_secs_f64();
            total_time_2 += result_2.elapsed_time.as_secs_f64();
        }
    } else {
        let test_files = get_input_files_in_directory(problem_path.join("tests"))?;

        let mut tests_passed = 0;
        let mut total_tests = 0;
        let mut total_time_1: Duration = Duration::new(0, 0);
        let mut total_time_2: Duration = Duration::new(0, 0);

        eprintln!("Running the solution files for each test case...");

        for test_file in test_files {
            let input_file_path = problem_path.join(format!("tests/{}", test_file));

            let result_1 = run_command_1
                .get_result(Some(&input_file_path))
                .context("Failed to get output from solution 1")?;
            let result_2 = run_command_2
                .get_result(Some(&input_file_path))
                .context("Failed to get output from solution 2")?;

            if result_1.output.as_bytes() != result_2.output.as_bytes() {
                eprintln!(
                    "  ! Test case failed: {test_file}, time taken: {:.5}s and {:.5}s respectively",
                    result_1.elapsed_time.as_secs_f64(),
                    result_2.elapsed_time.as_secs_f64()
                );
            } else {
                eprintln!(
                    "  + Test case passed: {test_file}, time taken: {:.5}s and {:.5}s respectively",
                    result_1.elapsed_time.as_secs_f64(),
                    result_2.elapsed_time.as_secs_f64()
                );
                tests_passed += 1;
            }
            total_tests += 1;
            total_time_1 += result_1.elapsed_time;
            total_time_2 += result_2.elapsed_time;
        }

        eprintln!(
            "{tests_passed} out of {total_tests} test cases passed, time taken: {:.5}s and {:.5}s respectively.",
            total_time_1.as_secs_f64(),
            total_time_2.as_secs_f64()
        );
    }

    run_command_1.cleanup()?;
    run_command_2.cleanup()?;

    Ok(())
}
