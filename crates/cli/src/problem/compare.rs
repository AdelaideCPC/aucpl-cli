use std::path::Path;
use std::time::Duration;

use anyhow::{bail, Context, Result};

use super::run::{RunCommand, RunnableFile};
use super::sync_mappings::get_problem;
use crate::config::Settings;
use crate::problem::run::RunResult;
use crate::util::{get_input_files_in_directory, get_project_root};

/// Arguments for the compare command.
pub struct CompareArgs<'a> {
    pub problems_dir: &'a Path,
    pub problem_name: String,
    pub solution_files: Vec<RunnableFile>,
}

/// Compare two solutions.
pub fn compare(settings: &Settings, compare_args: &CompareArgs) -> Result<()> {
    let project_root = get_project_root()?;
    let CompareArgs {
        problems_dir,
        problem_name,
        solution_files,
    } = compare_args;
    let problem_path = project_root.join(get_problem(problems_dir, problem_name)?);

    let mut run_commands: Vec<RunCommand> = Vec::new();
    for (i, file) in solution_files.iter().enumerate() {
        run_commands.push(RunCommand::new(
            settings,
            &problem_path,
            file,
            problem_path.join(format!("solutions/solution_{i}.out")),
            problem_path.join(format!("{file}")),
        )?);
    }

    // If there aren't at least two solutions, we can't compare so return an error
    if run_commands.len() < 2 {
        bail!("At least two solutions are required to compare.");
    }

    let test_files = get_input_files_in_directory(problem_path.join("tests"))?;

    let mut tests_passed = 0;
    let mut total_tests = 0;
    let mut total_times: Vec<Duration> = vec![Duration::new(0, 0); run_commands.len()];

    eprintln!("Running the solution files for each test case...");

    for test_file in test_files {
        let input_file_path = problem_path.join(format!("tests/{}", test_file));

        let mut results: Vec<RunResult> = Vec::new();
        for (i, run_cmd) in run_commands.iter().enumerate() {
            let result = run_cmd
                .get_result(Some(&input_file_path))
                .context(format!("Failed to get output from solution {i}"))?;
            results.push(result);
        }

        let mut passed = true;
        let mut avg_duration = Duration::new(0, 0);
        // We verify earlier that there is at least two solutions so indexing 0 is no issue
        let result_1 = &results[0];
        for (i, result) in results.iter().enumerate().skip(1) {
            // TODO: compare 1st, 2nd and nth result for a "best of three" (if applicable)?
            if result_1.output.as_bytes() != result.output.as_bytes() {
                eprintln!(
                        "  ! Test case failed: {test_file}, solution 0 took {:.5}s, solution {i} took {:.5}s",
                        result_1.elapsed_time.as_secs_f64(),
                        result.elapsed_time.as_secs_f64()
                    );
                passed = false;
                break;
            }
            avg_duration += result.elapsed_time;
        }

        if passed {
            eprintln!(
                "  + Test case passed: {test_file}, average time taken: {:.5}s",
                avg_duration.as_secs_f64() / (results.len() as f64)
            );
            tests_passed += 1;
        }

        total_tests += 1;
        for (i, result) in results.iter().enumerate() {
            total_times[i] += result.elapsed_time;
        }
    }

    eprintln!("{tests_passed} out of {total_tests} test cases passed");
    for (i, time) in total_times.iter().enumerate() {
        eprintln!(" Time taken for solution {i}: {:.5}s", time.as_secs_f64());
    }

    for run_command in run_commands {
        run_command.cleanup()?;
    }

    Ok(())
}
