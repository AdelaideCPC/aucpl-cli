use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;

use anyhow::{Context, Result};

use crate::config::Settings;
use crate::problem::run::{RunCommand, RunnableFile};
use crate::util::{get_input_files_in_directory, get_project_root};

use super::sync_mappings::get_problem;

const PYTHON_CHECKER_SCRIPT: &str = r#"
import importlib.util
import sys

checker_path = sys.argv[1]
process_output = sys.argv[2]
judge_output = sys.argv[3]

spec = importlib.util.spec_from_file_location("aucpl_checker", checker_path)
if spec is None or spec.loader is None:
    print("Could not load checker.py", file=sys.stderr)
    sys.exit(2)

module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)

if not hasattr(module, "check"):
    print("checker.py must define a `check` function", file=sys.stderr)
    sys.exit(2)

result = module.check(process_output, judge_output)

print("true" if bool(result) else "false")
"#;

fn run_custom_checker(
    checker_path: &Path,
    process_output: &str,
    judge_output: &[u8],
) -> Result<bool> {
    let judge_output = String::from_utf8_lossy(judge_output).into_owned();

    let checker_process = Command::new("python3")
        .arg("-c")
        .arg(PYTHON_CHECKER_SCRIPT)
        .arg(checker_path)
        .arg(process_output)
        .arg(&judge_output)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("Failed to run checker.py with python3")?;

    let output = checker_process
        .wait_with_output()
        .context("Failed to wait for checker.py execution")?;

    if !output.status.success() {
        anyhow::bail!(
            "checker.py failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }

    let checker_result = String::from_utf8_lossy(&output.stdout);
    let passed = match checker_result.trim().to_ascii_lowercase().as_str() {
        "true" => true,
        "false" => false,
        other => {
            anyhow::bail!(
                "checker.py must return a bool-compatible result, got: {}",
                other
            )
        }
    };

    Ok(passed)
}

/// Automatically run tests on the problem.
pub fn test(
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
    let checker_path = problem_path.join("checker.py");
    let use_custom_checker = checker_path.exists();

    eprintln!("Running the solution file for each test case...");
    if use_custom_checker {
        eprintln!("Using custom checker at: {}", checker_path.display());
    }

    let mut tests_passed = 0;
    let mut total_tests = 0;
    let mut total_time = Duration::new(0, 0);

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

        let passed = if use_custom_checker {
            run_custom_checker(&checker_path, &out_str, expected)?
        } else {
            expected == out_str.as_bytes()
        };

        if !passed {
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
