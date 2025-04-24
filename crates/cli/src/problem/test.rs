use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::Read;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};
use subprocess::{Exec, Redirection};

use crate::config::Settings;
use crate::util::{get_files_in_directory, get_project_root};

use super::sync_mappings::get_problem;

/// Automatically run tests on the problem.
pub fn test(
    settings: &Settings,
    problems_dir: &PathBuf,
    problem_name: &str,
    solution_file_name: Option<&str>,
    solution_lang: Option<&String>,
) -> Result<()> {
    let project_root = get_project_root()?;
    let problem = project_root.join(get_problem(problems_dir, problem_name)?);

    let solution_lang = solution_lang.unwrap_or(&settings.problem.default_lang);
    let mut solution_file = problem.join(format!("solutions/solution.{}", solution_lang));
    if solution_file_name.is_some() {
        solution_file = problem.join(format!(
            "solutions/{}",
            solution_file_name.context("Failed to get solution file name")?
        ));
    }
    solution_file = fs::canonicalize(solution_file)?;

    if !fs::exists(&solution_file).expect("Failed to check if path exists") {
        bail!("Solution file does not exist: {:?}", solution_file);
    }

    eprintln!("Using solution file at: {}", solution_file.display());

    let bin_file = problem.join("solutions/solution.out");
    let script_file = problem.join(format!("solutions/solution.{}", solution_lang));

    let lang_settings = settings
        .problem
        .solution
        .get(solution_lang)
        .context(format!(
            "Could not get settings for language `{solution_lang}`"
        ))?;

    let compile_command = lang_settings.compile_command.clone();

    // Check if the solution file is a script (if it needs compilation or not)
    let needs_compilation = compile_command.is_some();
    let compile_command = compile_command.unwrap_or_default();
    if needs_compilation && compile_command.is_empty() {
        bail!("compile_command specified in the settings, but array is empty");
    }

    if needs_compilation {
        let mut cmd_iter = compile_command.iter();
        let mut final_cmd = Exec::cmd(cmd_iter.next().context("Failed to get command")?);
        for c in cmd_iter {
            // Replace strings where necessary
            final_cmd = match c.as_str() {
                "@in_file" => final_cmd.arg(&solution_file),
                "@bin_file" => final_cmd.arg(&bin_file),
                _ => final_cmd.arg(c),
            }
        }
        eprint!("Compiling the solution file... ");
        // Run the compile command
        final_cmd.join()?;
        eprintln!("Done");
    }

    let run_command = lang_settings.run_command.clone().unwrap_or_default();
    if run_command.is_empty() {
        bail!("No run command specified in the settings. It must be specified!");
    }
    let cmd_iter = run_command.iter();
    let test_files = get_files_in_directory(problem.join("tests"))?;

    eprintln!("Running the solution file for each test case...");

    let mut tests_passed = 0;
    let mut total_tests = 0;
    let mut total_time: Duration = Duration::new(0, 0);

    for test_file in test_files {
        // Check if the file is a .in file
        if !test_file.ends_with(".in") {
            continue;
        }

        let input_file_path = problem.join(format!("tests/{}", test_file));
        let output_file_path = problem.join(format!(
            "tests/{}.out",
            test_file
                .strip_suffix(".in")
                .context("Failed to strip suffix of test file")?
        ));

        let input_contents = fs::read_to_string(input_file_path)?;
        let mut output_file = File::open(output_file_path)?;

        let mut cmd_iter_clone = cmd_iter.clone();
        let cmd = cmd_iter_clone.next().context("Failed to get command")?;
        let mut final_cmd = Exec::cmd(match cmd.as_str() {
            "@bin_file" => bin_file.as_os_str(),
            "@script_file" => script_file.as_os_str(),
            _ => OsStr::new(cmd),
        });

        for c in cmd_iter_clone {
            // Replace strings where necessary
            final_cmd = match c.as_str() {
                "@bin_file" => final_cmd.arg(&bin_file),
                "@script_file" => final_cmd.arg(&script_file),
                _ => final_cmd.arg(c),
            }
        }

        let start_time = Instant::now();
        final_cmd = final_cmd.stdin(&*input_contents).stdout(Redirection::Pipe);
        let out_str = final_cmd.capture()?.stdout_str();
        let elapsed_time = start_time.elapsed();

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

    // Delete the compiled run file, if it exists
    if bin_file.exists() {
        fs::remove_file(bin_file)?;
    }

    Ok(())
}
