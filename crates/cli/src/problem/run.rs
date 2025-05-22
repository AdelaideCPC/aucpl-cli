use anyhow::{bail, Context, Result};
use normpath::PathExt;
use std::ffi::OsStr;
use std::fs::{self, File};
use std::path::PathBuf;
use std::time::{Duration, Instant};
use subprocess::{Exec, Redirection};

use crate::config::Settings;

pub fn get_cmd(
    settings: &Settings,
    problem: &PathBuf,
    file_type: &str,
    file_name: Option<&str>,
    lang: &String,
    bin_file: &PathBuf,
) -> Result<Vec<String>> {
    let mut file_path = problem.join(format!("{}s/{}.{}", file_type, file_type, lang));
    if file_name.is_some() {
        file_path = problem.join(format!(
            "{file_type}s/{}",
            file_name.context(format!("Failed to get {file_type} file name"))?
        ));
    }
    file_path = file_path.normalize().context(format!("Failed to normalize {} (does the file exist?)", file_path.display()))?.into();

    if !fs::exists(&file_path).expect("Failed to check if path exists") {
        bail!("{file_type} file does not exist: {:?}", file_path);
    }

    eprintln!("Using {file_type} file at: {}", file_path.display());

    let lang_settings = settings
        .problem
        .solution
        .get(lang)
        .context(format!(
            "Could not get settings for language `{lang}`"
        ))?;

    let compile_command = lang_settings.compile_command.clone();

    // Check if the file is a script (if it needs compilation or not)
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
                "@in_file" => final_cmd.arg(&file_path),
                "@bin_file" => final_cmd.arg(&bin_file),
                _ => final_cmd.arg(c),
            }
        }
        eprint!("Compiling the {file_type} file... ");
        // Run the compile command
        final_cmd.join()?;
        eprintln!("Done");
    }

    let run_command = lang_settings.run_command.clone().unwrap_or_default();
    if run_command.is_empty() {
        bail!("No run command specified in the settings. It must be specified!");
    }

    Ok(run_command)
}

pub fn get_output(
    bin_file: &PathBuf,
    script_file: &PathBuf,
    run_command: &Vec<String>,
    input_file_path: Option<&PathBuf>,
) -> Result<(String, Duration)> {
    let cmd_iter = run_command.iter();
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
    if input_file_path.is_some() {
        let input_file = File::open(input_file_path.context("Failed to get input file")?)?;
        final_cmd = final_cmd.stdin(input_file).stdout(Redirection::Pipe);
    } else {
        final_cmd = final_cmd.stdout(Redirection::Pipe);
    }
    let out_str = final_cmd.capture()?.stdout_str();
    let elapsed_time = start_time.elapsed();

    Ok((out_str, elapsed_time))
}
