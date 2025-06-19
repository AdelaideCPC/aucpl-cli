use std::fs;
use std::path::Path;
use std::time::Duration;

use anyhow::{bail, Context, Result};
use uuid::Uuid;

use super::generate;
use super::run::{RunCommand, RunResult, RunnableFile};
use super::sync_mappings::get_problem;
use crate::{config::Settings, util::get_project_root};

pub struct FuzzArgs<'a> {
    pub problems_dir: &'a Path,
    pub problem_name: String,
    pub solution_files: Vec<RunnableFile>,
    pub generator: RunnableFile,
}

/// Generate new test cases until the solutions produce different results.
pub fn fuzz(settings: &Settings, fuzz_args: &FuzzArgs) -> Result<()> {
    let project_root = get_project_root()?;
    let FuzzArgs {
        problems_dir,
        problem_name,
        solution_files,
        generator,
    } = fuzz_args;
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
        bail!("At least two solutions are required for fuzzing.");
    }

    let mut total_tests = 0;
    let mut total_times: Vec<Duration> = vec![Duration::new(0, 0); run_commands.len()];

    // TODO: Set an optional limit?
    loop {
        total_tests += 1;

        let test_name = format!("generated_{}", Uuid::new_v4());

        generate::generate(settings, problems_dir, problem_name, generator, &test_name)
            .context("Failed to generate test case")?;

        let input_file_path = problem_path.join(format!("tests/{}.in", test_name));

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
                        "  ! Test case {total_tests} (tests/generated.in) failed, solution 1 took {:.5}s, solution {i} took {:.5}s",
                        result_1.elapsed_time.as_secs_f64(),
                        result.elapsed_time.as_secs_f64()
                    );
                passed = false;
                break;
            }
            avg_duration += result.elapsed_time;
        }

        if passed {
            let max_total_time = total_times
                .iter()
                .max()
                .unwrap_or(&Duration::new(0, 0))
                .to_owned();
            let min_total_time = total_times
                .iter()
                .min()
                .unwrap_or(&Duration::new(0, 0))
                .to_owned();

            eprintln!(
                "  + Test case {total_tests} passed, average time taken: {:.5}s",
                avg_duration.as_secs_f64() / (results.len() as f64)
            );
            eprintln!(
                "    Total percentage time difference (min, max times): {:.5}%",
                (max_total_time.abs_diff(min_total_time)).as_secs_f64() * 100f64
                    / min_total_time.as_secs_f64()
            );

            fs::remove_file(input_file_path).context("Failed to delete generated test case")?;
        } else {
            break;
        }

        for (i, result) in results.iter().enumerate() {
            total_times[i] += result.elapsed_time;
        }
    }

    for run_command in run_commands {
        run_command.cleanup()?;
    }

    Ok(())
}
