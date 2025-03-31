use clap::{Parser, Subcommand};
use config::app_config::AppConfiguration;
use log::{error, info};

use crate::llama::LlamaService;
use crate::prelude::*;
use crate::rag::files::{get_markdown_files, process_markdown_files};
use crate::rag::vectors::VectorDB;
use crate::tui::run_app;

mod config;
mod error;
mod llama;
mod prelude;
mod rag;
mod tui;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Index all markdown files in the configured directory
    Index,

    /// Start the TUI application
    Tui,

    /// Initialize the configuration
    Init {
        /// Path to scan for markdown files
        #[arg(short, long)]
        scan_path: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    initialize_logging();
    info!("Raggy Notes starting...");

    let cli = Cli::parse();

    // Initialize LLM service
    let llama_service = LlamaService::new("http://localhost", 11434);

    // Check models connection
    match llama_service.test_connection().await {
        Ok(models) => {
            info!("LLM Models available: [{}]", models.join(", "));
        }
        Err(e) => {
            error!("Error connecting to Ollama: {}", e);
            return Err(AppError::OllamaError(format!(
                "Failed to connect to Ollama: {}",
                e
            )));
        }
    }

    // Initialize VectorDB
    let vector_db = match VectorDB::new() {
        Ok(db) => db,
        Err(e) => {
            error!("Error creating VectorDB client: {}", e);
            return Err(AppError::VectorDBError(format!(
                "Failed to create VectorDB client: {}",
                e
            )));
        }
    };

    // Test VectorDB connection
    if let Err(e) = vector_db.test_connection().await {
        error!("Error connecting to Qdrant: {}", e);
        return Err(AppError::VectorDBError(format!(
            "Failed to connect to Qdrant: {}",
            e
        )));
    }

    // Initialize VectorDB collections
    if let Err(e) = vector_db.initialize_collections().await {
        error!("Error initializing collections: {}", e);
        return Err(AppError::VectorDBError(format!(
            "Failed to initialize collections: {}",
            e
        )));
    }

    // Process the specified command
    match &cli.command {
        Some(Commands::Init { scan_path }) => {
            info!("Initializing configuration with scan path: {}", scan_path);
            let config = AppConfiguration::new(scan_path);
            match config.save() {
                Ok(path) => {
                    info!("Configuration saved to {:?}", path);
                }
                Err(e) => {
                    error!("Error saving configuration: {}", e);
                    return Err(AppError::ConfigError(format!(
                        "Failed to save configuration: {}",
                        e
                    )));
                }
            }
        }
        Some(Commands::Index) => {
            info!("Indexing markdown files...");

            // Load configuration
            let config = match AppConfiguration::load() {
                Ok(config) => config,
                Err(e) => {
                    error!("Error loading configuration: {}", e);
                    return Err(AppError::ConfigError(format!(
                        "Failed to load configuration: {}. Run 'init' command first.",
                        e
                    )));
                }
            };

            // Get markdown files
            let files = match get_markdown_files(&config) {
                Ok(files) => files,
                Err(e) => {
                    error!("Error getting markdown files: {}", e);
                    return Err(e);
                }
            };

            info!("Found {} markdown files to process", files.len());

            // Process markdown files
            if let Err(e) = process_markdown_files(&files, &llama_service, &vector_db).await {
                error!("Error processing markdown files: {}", e);
                return Err(e);
            }

            info!("Indexing completed successfully");
        }
        Some(Commands::Tui) => {
            info!("Starting TUI application...");
            if let Err(e) = run_app(&llama_service, &vector_db).await {
                error!("Error running TUI application: {}", e);
                return Err(e);
            }
        }
        None => {
            // If no command is specified, default to TUI
            info!("No command specified, starting TUI application...");
            if let Err(e) = run_app(&llama_service, &vector_db).await {
                error!("Error running TUI application: {}", e);
                return Err(e);
            }
        }
    }

    info!("Application finished successfully");
    Ok(())
}

fn initialize_logging() {
    env_logger::init();
}
