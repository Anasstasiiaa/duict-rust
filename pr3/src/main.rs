//! Practical Work 3: CLI binary for the file indexer.
//!
//! Provides the command-line interface and configuration handling,
//! utilizing `anyhow` for rich error context.runtime configuration.

#![warn(missing_docs)]
#![warn(rustdoc::broken_intra_doc_links)]
#![warn(rustdoc::missing_crate_level_docs)]
#![warn(unreachable_pub)]
#![warn(clippy::missing_panics_doc)]
#![warn(clippy::missing_errors_doc)]
#![warn(clippy::result_large_err)]
#![warn(clippy::clone_on_ref_ptr)]
#![warn(clippy::similar_names)]

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use config::{Config, Environment};
use pz03_indexer_refactored::{JsonStorage, SqliteStorage, Storage};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct AppConfig {
    /// Path to the index in "type:path" format (e.g., "json:index.json").
    index_path: String,
}

impl AppConfig {
    /// Creates a new application configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration building or deserialization fails.
    fn new() -> Result<Self> {
        let builder = Config::builder()
            .set_default("index_path", "json:./resources/files_index.json")?
            .add_source(Environment::with_prefix("FILES"));

        let settings = builder.build().context("Failed to build configuration")?;
        settings
            .try_deserialize()
            .context("Failed to deserialize configuration")
    }
}

#[derive(Parser)]
#[command(author, version, about = "Utility for indexing files by tags")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a file to the index with tags.
    Add {
        /// Path to the file.
        #[arg(long)]
        path: String,

        /// Comma-separated list of tags (e.g., tag1,tag2).
        #[arg(long, value_delimiter = ',')]
        tags: Vec<String>,
    },
    /// Find files by tags.
    Get {
        /// Comma-separated list of tags to search for.
        #[arg(long, value_delimiter = ',')]
        tags: Vec<String>,
    },
    /// List all unique tags and the number of files associated with them.
    Tags,
}

/// Parses the configuration string to select the storage backend.
///
/// # Errors
///
/// Returns an error if the format is invalid, the storage type is unknown,
/// or the underlying storage initialization fails.
fn get_storage_provider(config_str: &str) -> Result<Box<dyn Storage>> {
    let (provider_type, path) = config_str
        .split_once(':')
        .context("Invalid index path format. Expected 'TYPE:PATH' (e.g., 'json:index.json')")?;

    match provider_type.to_uppercase().as_str() {
        "JSON" => Ok(Box::new(
            JsonStorage::new(path).context("Failed to initialize JSON storage backend")?,
        )),
        "SQLITE" => Ok(Box::new(
            SqliteStorage::new(path).context("Failed to initialize SQLite storage backend")?,
        )),
        _ => bail!(
            "Unknown storage type: '{}'. Supported: JSON, SQLITE",
            provider_type
        ),
    }
}

fn main() -> Result<()> {
    let config = AppConfig::new().context("Application configuration initialization failed")?;
    let cli = Cli::parse();

    let mut storage = get_storage_provider(&config.index_path)
        .context("Failed to setup the required storage provider")?;

    match cli.command {
        Commands::Add { path, tags } => {
            storage
                .add(&path, tags)
                .with_context(|| format!("Failed to index file at path '{}'", path))?;
            println!("File '{}' successfully indexed.", path);
        }
        Commands::Get { tags } => {
            let files = storage
                .get(tags.clone())
                .with_context(|| format!("Failed to retrieve files for tags: {:?}", tags))?;

            if files.is_empty() {
                println!("No files found for tags: {:?}", tags);
            } else {
                println!("Found {} files for tags: {:?}", files.len(), tags);
                for file in files {
                    println!("- {}", file);
                }
            }
        }
        Commands::Tags => {
            let all_tags = storage
                .tags()
                .context("Failed to retrieve tag statistics from storage")?;

            if all_tags.is_empty() {
                println!("No tags found in the index.");
            } else {
                println!("Tags in the index:");
                for (tag, count) in all_tags {
                    println!("- {}: {} files", tag, count);
                }
            }
        }
    }

    Ok(())
}
