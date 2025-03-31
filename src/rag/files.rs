// src/rag/files.rs
use std::{
    fs::{read_dir, DirEntry},
    path::PathBuf,
};

use log::{info, warn};

use crate::config::app_config::AppConfiguration;
use crate::llama::LlamaService;
use crate::prelude::*;
use crate::rag::vectors::{NoteVector, VectorDB};

/// Retrieves all markdown files from the configured directory.
pub fn get_markdown_files(config: &AppConfiguration) -> Result<Vec<PathBuf>> {
    let mut markdown_files = Vec::new();
    info!("Scanning directory: {}", config.scan_path);

    let directory = read_dir(&config.scan_path)?;

    for entry in directory.flatten() {
        markdown_files.extend(extract_markdown_files(&entry)?);
    }

    Ok(markdown_files)
}

/// Recursively extracts markdown file paths from a directory entry.
fn extract_markdown_files(dir_entry: &DirEntry) -> Result<Vec<PathBuf>> {
    let mut markdown_files = Vec::new();

    if dir_entry.file_type()?.is_dir() {
        for entry in read_dir(dir_entry.path())?.flatten() {
            markdown_files.extend(extract_markdown_files(&entry)?);
        }
    } else if let Some(ext) = dir_entry.path().extension() {
        if ext == "md" {
            info!("Found markdown file: {:?}", dir_entry.path());
            markdown_files.push(dir_entry.path());
        }
    }

    Ok(markdown_files)
}

/// Processes markdown files, extracts text, generates embeddings, and stores in VectorDB
pub async fn process_markdown_files(
    files: &[PathBuf],
    llama_service: &LlamaService,
    vector_db: &VectorDB,
) -> Result<()> {
    info!("Processing {} markdown files", files.len());

    for (i, file_path) in files.iter().enumerate() {
        info!("Processing file {}/{}: {:?}", i + 1, files.len(), file_path);

        // Extract text from markdown
        let (title, content) = llama_service.extract_text_from_markdown(file_path).await?;

        // Generate embedding
        info!("Generating embedding for: {}", title);
        let embeddings = match llama_service.get_embedding(&content).await {
            Ok(emb) => emb,
            Err(e) => {
                warn!("Error generating embedding for {:?}: {}", file_path, e);
                continue;
            }
        };

        // Create note vector
        let note_vector = NoteVector::new(
            title.clone(),
            content.clone(),
            file_path.clone(),
            embeddings,
        );

        // Store in VectorDB
        if let Err(e) = vector_db.save_note_vector(note_vector).await {
            warn!("Error saving vector for {:?}: {}", file_path, e);
        }
    }

    info!("Finished processing all markdown files");
    Ok(())
}
