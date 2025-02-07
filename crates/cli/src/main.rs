use anyhow::{Context, Result};
use clap::Parser;
use fs_extra::dir::{copy, CopyOptions};
use std::collections::HashMap;
use std::fs::{remove_dir_all, File};
use std::path::Path;
use std::{fs, io};
use walkdir::WalkDir;
use zip::{write::FileOptions, CompressionMethod, ZipWriter};
/// Search for a pattern in a file and display the lines that contain it.
#[derive(Parser)]
struct Cli {
    /// The pattern to look for
    //pattern: String,
    /// The path to the file to read
    path: std::path::PathBuf,
}

// for troubleshooting and finding types
fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>());
}

fn get_files_in_directory(path: &str) -> io::Result<Vec<String>> {
    // Get a list of all entries in the folder
    let entries = fs::read_dir(path)?;
    // Extract the filenames from the directory entries and store them in a vector
    let file_names: Vec<String> = entries
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

// Checks if the test case contains nothing
fn is_file_empty(path: &str) -> io::Result<bool> {
    let metadata = fs::metadata(path)?;
    Ok(metadata.len() == 0) // Returns the size of the file, in bytes, this metadata is for
}

fn do_hard_work() {
    for i in 0..100 {
        println!("helooooo");
    }
}

// Test case should have an input and an output
#[derive(Debug)]
struct FilePair {
    // option because it could be missing
    input: Option<String>,
    output: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    /*
    let args = Cli::parse();

    let content = std::fs::read_to_string(&args.path)
        .with_context(|| format!("could not read file `{}`", args.path.display()))?;

    for line in content.lines() {
        if line.contains(&args.pattern) {
            println!("{}", line);
        }
    }
    */

    /*
    let pb = indicatif::ProgressBar::new(100);
    for i in 0..100 {
        do_hard_work();
        pb.println(format!("[+] finished #{}", i));
        pb.inc(1);
    }
    pb.finish_with_message("done");
    */

    //let args = Cli::parse();

    let directory_path = "/Users/maged/AUCPL-CLI/aucpl-cli/src/baked-goods/";

    let files = get_files_in_directory(directory_path)?;

    // Counts number of test cases to ensure its a valid number (each input requries and output)
    let testcase_count = &files.len();

    if testcase_count % 2 != 0 {
        eprintln!("An odd number of test cases were found, therefore a test case is missing (either input or output)");
        eprintln!("The number of testcases found {}", testcase_count);
        eprintln!("The test cases found were: ");
        for file in &files {
            println!("{}", file);
        }
    }

    // Build a hashmap to ensure each test case has an input and output file
    let mut pairs: HashMap<String, FilePair> = HashMap::new();

    for file in &files {
        // Gets full path to file each file
        let path_to_file = format!("{}{}", directory_path, file);

        match is_file_empty(&path_to_file) {
            Ok(true) => {
                println!("WARNING: {} is an empty file", file);
            }
            Ok(false) => {}
            Err(e) => {
                eprintln!("ERROR: Failed to check if {} was empty", file);
            }
        }

        if file.ends_with(".in") {
            if let Some(base) = file.strip_suffix(".in") {
                let entry = pairs.entry(base.to_string()).or_insert(FilePair {
                    input: None,
                    output: None,
                });
                entry.input = Some(file.to_string());
            }
        } else if file.ends_with(".out") {
            if let Some(base) = file.strip_suffix(".out") {
                let entry = pairs.entry(base.to_string()).or_insert(FilePair {
                    input: None,
                    output: None,
                });
                entry.output = Some(file.to_string());
            }
        } else {
            eprintln!("ERROR: {} does not end with .in or .out", file);
        }
    }

    for (test_case, pair) in &pairs {
        if pair.input == None || pair.output == None {
            println!();
            println!("ERROR TEST CASE MATCH MISSING | {}: {:?}", test_case, pair);
            println!();
        } else {
            println!("Test case {}: {:?}", test_case, pair);
        }
    }

    let newDirectory = format!("{}../baked-goods2", directory_path);

    // remove the new destination contents if there is any
    match fs::remove_dir_all(&newDirectory) {
        Ok(_) => println!("Successfully deleted all destination contents"),

        Err(e) => eprintln!("No contents in old folder/old folder does not exist"),
    }

    // create the directory
    fs::create_dir(&newDirectory)?;

    // Create default copy options
    let mut options = CopyOptions::new();
    // will be copied into the destination directory (instead of creating a subfolder)
    options.copy_inside = false;
    options.overwrite = true; // overwrite files if they already exist

    // Copy the directory recursively | the function returns the total number of bytes copied
    match copy(&directory_path, &newDirectory, &options) {
        Ok(bytes_copied) => println!("Copied {} bytes successfully.", bytes_copied),
        Err(e) => eprintln!("An error occurred during copying: {}", e),
    }

    /* TO DO:
       - Zip the new folder after it is made
       - create the init yml file
       - accept user input through the command line (only just the name of the problem you want to update/upload)
       - search the directory at the start of the folder name
    */

    //zip::write_dir(newDirectory);

    //fs::copy(&directory_path, &newDirectory);

    //zip_folder(&newDirectory);

    print_type_of(&newDirectory);

    Ok(())
}
