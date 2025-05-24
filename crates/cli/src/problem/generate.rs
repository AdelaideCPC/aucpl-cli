use std::fs::File;
use std::io::Write;
use std::path::Path;

use anyhow::{bail, Context, Result};

use super::run::{RunCommand, RunnableFile};
use super::sync_mappings::get_problem;
use crate::config::Settings;
use crate::util::get_project_root;

/// Generate a test case input with a generator file.
pub fn generate(
    settings: &Settings,
    problems_dir: &Path,
    problem_name: &str,
    generator: &RunnableFile,
    test_name: &str,
) -> Result<()> {
    let project_root = get_project_root().context("Failed to get project root")?;
    let problem_path = project_root
        .join(get_problem(problems_dir, problem_name).context("Failed to get problem path")?);

    let test_path = problem_path.join(format!("tests/{test_name}.in"));
    if test_path.exists() {
        bail!(
            "Test file already exists: {:?}, use `--test-name` to specify another name",
            test_path
        );
    }

    let run_command = RunCommand::new(
        settings,
        &problem_path,
        generator,
        problem_path.join("generators/generator.out"),
        problem_path.join(format!("{generator}")),
    )
    .context("Failed to get generator command")?;

    let result = run_command
        .get_result(None)
        .context("Failed to get generator output")?;
    let mut test_file = File::create(test_path).context("Failed to create test file")?;
    test_file.write_all(result.output.as_bytes())?;

    run_command.cleanup()?;

    Ok(())
}
