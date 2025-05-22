use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

use anyhow::{Result, Context};

use super::run::{get_cmd, get_output};
use super::sync_mappings::get_problem;
use crate::config::Settings;
use crate::util::get_project_root;

pub fn generate(
    settings: &Settings,
    problems_dir: &Path,
    problem_name: &str,
    generator_file_name: Option<&str>,
    generator_lang: Option<&String>,
) -> Result<()> {
    let project_root = get_project_root().context("Failed to get project root")?;
    let problem_path = project_root.join(get_problem(problems_dir, problem_name).context("Failed to get problem path")?);

    let generator_lang = generator_lang.unwrap_or(&settings.problem.default_lang);

    let bin_file = problem_path.join("generators/generator.out");
    let script_file = problem_path.join(format!("generators/generator.{}", generator_lang));

    let run_command = get_cmd(
        settings,
        &problem_path,
        "generator",
        generator_file_name,
        generator_lang,
        &bin_file,
    ).context("Failed to get generator command")?;

    let (output, _) = get_output(&bin_file, &script_file, &run_command, None).context("Failed to get generator output")?;
    let mut test_file = File::create(problem_path.join("tests/generated.in")).context("Failed to create test file")?;
    test_file.write_all(output.as_bytes())?;

    // Delete the compiled run files, if it exists
    if bin_file.exists() {
        fs::remove_file(bin_file).context("Failed to remove binary file")?;
    }

    Ok(())
}
