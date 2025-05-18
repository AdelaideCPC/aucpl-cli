use std::fs::{self, File};
use std::path::Path;

use anyhow::Result;
use normpath::PathExt;
use serde_json::{from_reader, json, to_writer, to_writer_pretty};

use super::{CompetitionData, Competitions, COMPETITIONS_FILE};

/// Create a new competition.
pub fn create(problems_dir: &Path, comp_name: &str) -> Result<()> {
    create_competitions_file(problems_dir)?;
    let comp_file_path = problems_dir.join(COMPETITIONS_FILE);
    let comp_file = File::open(&comp_file_path)?;
    let mut data: Competitions = from_reader(&comp_file)?;

    // Check if the competition exists or not
    if data.contains_key(comp_name) {
        eprintln!("The competition '{comp_name}' already exists!");
        return Ok(());
    }

    data.insert(
        comp_name.to_string(),
        CompetitionData {
            problems: Vec::new(),
            finished: false,
        },
    );

    let comp_file = File::options()
        .write(true)
        .truncate(true)
        .open(comp_file_path)?;
    to_writer_pretty(&comp_file, &json!(data))?;
    eprintln!("Created a new competition '{comp_name}'");

    Ok(())
}

pub fn create_competitions_file(problems_dir: &Path) -> Result<bool> {
    let problems_dir = problems_dir.normalize()?;
    let path = &problems_dir.join(COMPETITIONS_FILE);
    if !fs::exists(path)? {
        let file = File::create(path)?;
        to_writer(&file, &json!({}))?;
        return Ok(true);
    }

    Ok(false)
}
