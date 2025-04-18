use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use subprocess::{Exec, Redirection};

use super::sync_mappings::get_problem;
use crate::{config::Settings, util::get_files_in_directory};

/// Automatically generate test outputs for a problem, given pre-existing input files.
pub fn solve(
    settings: &Settings,
    problems_dir: &PathBuf,
    problem_name: &str,
    solution_file_name: Option<&str>,
) -> Result<()> {
    let problem = PathBuf::new().join(get_problem(problems_dir, problem_name)?);

    let solution_file_ext = settings.problem.solution_file_ext.clone();
    let mut solution_file = problem.join(format!("solutions/solution{}", solution_file_ext));
    if solution_file_name.is_some() {
        solution_file = problem.join(format!(
            "solutions/{}",
            solution_file_name.context("Failed to get solution file name")?
        ));
    }

    if !fs::exists(&solution_file).expect("Failed to check if path exists") {
        bail!("Solution file does not exist: {:?}", solution_file);
    }

    eprintln!("Using solution file at: {}", solution_file.display());

    let bin_file = problem.join("solutions/solution.out");
    let script_file = problem.join(format!("solutions/solution{}", solution_file_ext));

    // Compile the file (if set)
    let compile_command = settings.problem.solution_compile_command.clone();
    if !compile_command.is_empty() {
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

    let run_command = settings.problem.solution_run_command.clone();
    if run_command.is_empty() {
        bail!("No run command specified in the settings. It must be specified!");
    }
    let cmd_iter = run_command.iter();
    let test_files = get_files_in_directory(problem.join("tests"))?;

    eprintln!("Running the solution file for each test case...");
    // Run the file for every test input and generate the corresponding output
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
        let mut output_file = File::create(&output_file_path)?;

        let mut cmd_iter_clone = cmd_iter.clone();
        let cmd = cmd_iter_clone.next().context("Failed to get command")?;
        let mut final_cmd = Exec::cmd(match cmd.as_str() {
            "@bin_file" => bin_file.clone(),
            "@script_file" => script_file.clone(),
            _ => PathBuf::new().join(cmd),
        });

        for c in cmd_iter_clone {
            // Replace strings where necessary
            final_cmd = match c.as_str() {
                "@bin_file" => final_cmd.arg(&bin_file),
                "@script_file" => final_cmd.arg(&script_file),
                _ => final_cmd.arg(c),
            }
        }

        final_cmd = final_cmd.stdin(&*input_contents).stdout(Redirection::Pipe);
        let out_str = final_cmd.capture()?.stdout_str();
        output_file.write_all(out_str.as_bytes())?;
        eprintln!("  - generated output for test file: {}", test_file);
    }
    eprintln!("Finished generating outputs for all test cases");

    // Delete the compiled run file, if it exists
    if bin_file.exists() {
        fs::remove_file(bin_file)?;
    }

    Ok(())
}
