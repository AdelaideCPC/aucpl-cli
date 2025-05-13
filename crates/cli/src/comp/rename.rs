use std::fs::{self, File};
use std::path::Path;

use anyhow::{bail, Context, Result};
use serde_json::{from_reader, to_writer_pretty};

use super::{Competitions, COMPETITIONS_FILE};

pub fn rename(problems_dir: &Path, old_comp_name: &str, new_comp_name: &str) -> Result<()> {
    let comp_file_path = problems_dir.join(COMPETITIONS_FILE);
    if !fs::exists(&comp_file_path)? {
        bail!("Competitions file does not exist");
    }

    let comp_file = File::open(&comp_file_path)?;
    let mut data: Competitions = from_reader(&comp_file)?;

    let comp_data = data
        .get(old_comp_name)
        .context(format!("Competition '{old_comp_name}' not found"))?
        .to_owned();
    data.insert(new_comp_name.to_string(), comp_data);
    data.remove(old_comp_name);

    let comp_file = File::options()
        .write(true)
        .truncate(true)
        .open(comp_file_path)?;
    to_writer_pretty(&comp_file, &data)?;
    eprintln!("Renamed competition from '{old_comp_name}' to '{new_comp_name}'");

    Ok(())
}
