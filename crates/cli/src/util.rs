use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Result};

use crate::config::SETTINGS_FILE;

/// Figure out the root directory of the AUCPL problemset project.
pub fn get_project_root() -> Result<PathBuf> {
    let mut path = std::env::current_dir()?;
    while !path.join(SETTINGS_FILE).exists() {
        if !path.pop() {
            bail!("Could not find the project root: {SETTINGS_FILE} not found");
        }
    }

    Ok(path)
}

/// Get a list of files in a directory that are not directories.
pub fn get_files_in_directory<P: AsRef<Path>>(path: P) -> Result<Vec<String>> {
    let entries = fs::read_dir(path)?;
    let file_names = entries
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            if path.is_file() {
                path.file_name()?.to_str().map(|s| s.to_owned())
            } else {
                None
            }
        })
        .collect();

    Ok(file_names)
}

pub fn is_file_empty<P: AsRef<Path>>(path: P) -> Result<bool> {
    let metadata = fs::metadata(path)?;
    Ok(metadata.len() == 0)
}
