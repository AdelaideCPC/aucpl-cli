use std::env;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{bail, Context, Result};

use crate::config::Settings;
use crate::problem::run::{get_python_executable, RunCommand, RunnableFile};
use crate::util::{get_input_files_in_directory, get_project_root};

use super::sync_mappings::get_problem;

const PYTHON_CHECKER_SCRIPT: &str = r#"
import importlib.util
import sys

checker_path = sys.argv[1]
process_output_path = sys.argv[2]
judge_output_path = sys.argv[3]
judge_input_path = sys.argv[4]

with open(process_output_path, "rb") as f:
    process_output = f.read()

with open(judge_output_path, "rb") as f:
    judge_output = f.read()

with open(judge_input_path, "rb") as f:
    judge_input = f.read()

spec = importlib.util.spec_from_file_location("aucpl_checker", checker_path)
if spec is None or spec.loader is None:
    print("Could not load checker.py", file=sys.stderr)
    sys.exit(2)

module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)

if not hasattr(module, "check"):
    print("checker.py must define a `check` function", file=sys.stderr)
    sys.exit(2)

result = module.check(
    process_output,
    judge_output,
    judge_input=judge_input
)

print("true" if bool(result) else "false")
"#;

struct CheckerTempFiles {
    process_output: PathBuf,
    judge_output: PathBuf,
}

impl CheckerTempFiles {
    fn new(process_output: &str, judge_output: &[u8]) -> Result<Self> {
        let temp_dir = env::temp_dir();
        let nonce = format!(
            "{}-{}",
            std::process::id(),
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
        );
        let process_output_path = temp_dir.join(format!("aucpl-process-output-{nonce}.txt"));
        let judge_output_path = temp_dir.join(format!("aucpl-judge-output-{nonce}.txt"));

        fs::write(&process_output_path, process_output)
            .context("Failed to write process output temp file")?;
        fs::write(&judge_output_path, judge_output)
            .context("Failed to write judge output temp file")?;

        Ok(Self {
            process_output: process_output_path,
            judge_output: judge_output_path,
        })
    }
}

impl Drop for CheckerTempFiles {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.process_output);
        let _ = fs::remove_file(&self.judge_output);
    }
}

fn run_custom_checker(
    settings: &Settings,
    checker_path: &Path,
    process_output: &str,
    judge_output: &[u8],
    input_file_path: &Path,
) -> Result<bool> {
    let python_cmd = get_python_executable(settings);
    let temp_files = CheckerTempFiles::new(process_output, judge_output)?;

    let checker_run = RunCommand::from_command(
        PathBuf::new(),
        checker_path.to_path_buf(),
        vec![
            python_cmd,
            "-c".to_string(),
            PYTHON_CHECKER_SCRIPT.to_string(),
            "@script_file".to_string(),
            temp_files.process_output.to_string_lossy().into_owned(),
            temp_files.judge_output.to_string_lossy().into_owned(),
            input_file_path.to_string_lossy().into_owned(),
        ],
    )
    .context("Failed to prepare checker command")?;
    let checker_result = checker_run
        .get_result(None)
        .context("Failed to run checker.py")?
        .output;

    let passed = match checker_result.trim().to_ascii_lowercase().as_str() {
        "true" => true,
        "false" => false,
        other => {
            bail!(
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
            run_custom_checker(
                settings,
                &checker_path,
                &out_str,
                expected,
                &input_file_path,
            )?
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
