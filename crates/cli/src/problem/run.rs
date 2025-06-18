use anyhow::{bail, Context, Result};
use normpath::PathExt;
use std::ffi::OsStr;
use std::fmt;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use subprocess::{Exec, Redirection};

use crate::config::Settings;

/// Represents the category of a runnable file, either a solution or a generator.
#[derive(Eq, PartialEq)]
pub enum RunnableCategory {
    Solution,
    Generator,
}

impl fmt::Display for RunnableCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RunnableCategory::Solution => write!(f, "solution"),
            RunnableCategory::Generator => write!(f, "generator"),
        }
    }
}

/// Represents a runnable file, which can be either a solution or a generator.
pub struct RunnableFile {
    pub category: RunnableCategory,
    pub name: String,
    pub lang: String,
}

impl RunnableFile {
    /// Sets the name and language if given, otherwise defaults to the category name
    /// and the default language from the settings.
    pub fn new(
        settings: &Settings,
        category: RunnableCategory,
        name: Option<&String>,
        lang: Option<&String>,
    ) -> Self {
        Self {
            name: name.cloned().unwrap_or(format!("{category}")),
            lang: lang
                .cloned()
                .unwrap_or(if category == RunnableCategory::Solution {
                    settings.problem.default_lang.clone()
                } else {
                    settings.problem.default_generator_lang.clone()
                }),
            category,
        }
    }
}

impl fmt::Display for RunnableFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}s/{}.{}", self.category, self.name, self.lang)
    }
}

/// Represents a command to run a solution or generator file.
// TODO: Technically it wouldn't really be correct to have a "script_file"
// if the file is only compiled, so we should probably make bin_file and
// script_file mutually exclusive
pub struct RunCommand {
    bin_file: PathBuf,
    script_file: PathBuf,
    run_command: Vec<String>,
}

pub struct RunResult {
    pub output: String,
    pub elapsed_time: Duration,
}

impl RunCommand {
    /// Creates a new `RunCommand` instance, compiling the file if necessary.
    pub fn new(
        settings: &Settings,
        problem: &Path,
        file: &RunnableFile,
        bin_file: PathBuf,
        script_file: PathBuf,
    ) -> Result<Self> {
        let mut file_path = problem.join(format!("{}", file));
        file_path = file_path
            .normalize()
            .context(format!(
                "Failed to normalize {} (does the file exist?)",
                file_path.display()
            ))?
            .into();
        if !fs::exists(&file_path).expect("Failed to check if path exists") {
            bail!("{} file does not exist: {:?}", file.category, file_path);
        }

        eprintln!("Using {} file at: {}", file.category, file_path.display());

        let lang_settings = settings
            .problem
            .solution
            .get(file.lang.as_str())
            .context(format!(
                "Could not get settings for language `{}`",
                file.lang
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
            eprint!("Compiling the {} file... ", file.category);
            // Run the compile command
            final_cmd.join()?;
            eprintln!("Done");
        }

        let run_command = lang_settings.run_command.clone().unwrap_or_default();
        if run_command.is_empty() {
            bail!("No run command specified in the settings. It must be specified!");
        }

        Ok(Self {
            bin_file,
            script_file,
            run_command,
        })
    }

    /// Returns the result of running the command, capturing its output and elapsed time.
    /// If `input_file_path` is provided, it will be used as the standard input for the command.
    pub fn get_result(&self, input_file_path: Option<&PathBuf>) -> Result<RunResult> {
        let cmd_iter = self.run_command.iter();
        let mut cmd_iter_clone = cmd_iter.clone();
        let cmd = cmd_iter_clone.next().context("Failed to get command")?;
        let mut final_cmd = Exec::cmd(match cmd.as_str() {
            "@bin_file" => self.bin_file.as_os_str(),
            "@script_file" => self.script_file.as_os_str(),
            _ => OsStr::new(cmd),
        });

        for c in cmd_iter_clone {
            // Replace strings where necessary
            final_cmd = match c.as_str() {
                "@bin_file" => final_cmd.arg(&self.bin_file),
                "@script_file" => final_cmd.arg(&self.script_file),
                _ => final_cmd.arg(c),
            }
        }

        let start_time = Instant::now();
        if input_file_path.is_some() {
            final_cmd = final_cmd
                .stdin(File::open(input_file_path.unwrap()).context("Failed to get input file")?)
                .stdout(Redirection::Pipe);
        } else {
            final_cmd = final_cmd.stdout(Redirection::Pipe);
        }
        let output = final_cmd.capture()?.stdout_str();
        let elapsed_time = start_time.elapsed();

        Ok(RunResult {
            output,
            elapsed_time,
        })
    }

    /// Cleans up the generated binary file if it exists.
    pub fn cleanup(&self) -> Result<()> {
        if self.bin_file.exists() {
            fs::remove_file(&self.bin_file).context("Failed to remove binary file")?;
        }
        Ok(())
    }
}
