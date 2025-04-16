use std::fs;
use std::path::Path;

use anyhow::Result;

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
