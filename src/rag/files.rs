use std::{
    fs::{read_dir, DirEntry},
    io,
    path::PathBuf,
};

use crate::config::app_config::AppConfiguration;

/// Retrieves all markdown files from the configured directory.
pub fn get_markdown_files(config: &AppConfiguration) -> Result<Vec<PathBuf>, io::Error> {
    let mut markdown_files = Vec::new();

    let directory = read_dir(&config.scan_path)?;

    for entry in directory.flatten() {
        markdown_files.extend(extract_markdown_files(&entry)?);
    }

    Ok(markdown_files)
}

/// Recursively extracts markdown file paths from a directory entry.
fn extract_markdown_files(dir_entry: &DirEntry) -> Result<Vec<PathBuf>, io::Error> {
    let mut markdown_files = Vec::new();

    if dir_entry.file_type()?.is_dir() {
        for entry in read_dir(dir_entry.path())?.flatten() {
            markdown_files.extend(extract_markdown_files(&entry)?);
        }
    } else if let Some(ext) = dir_entry.path().extension() {
        if ext == "md" {
            // println!("Found markdown file: {:?}", dir_entry.path());
            markdown_files.push(dir_entry.path());
        }
    }

    Ok(markdown_files)
}
