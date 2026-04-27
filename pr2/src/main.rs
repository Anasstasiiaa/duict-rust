//! Practical Work 2: Polymorphism
//!
//! This crate implements a CLI file indexing system using Rust's dynamic polymorphism (Trait Objects).
//! It allows classifying files by tags and storing the index in either JSON or SQLite backends based on runtime configuration.

#![warn(missing_docs)]
#![warn(rustdoc::broken_intra_doc_links)]
#![warn(missing_crate_level_docs)]
#![warn(unreachable_pub)]
#![warn(clippy::missing_panics_doc)]
#![warn(clippy::clone_on_ref_ptr)]
#![warn(clippy::similar_names)]

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use config::{Config, Environment};
use rusqlite::{Connection, params};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
struct AppConfig {
    /// Path to the index in "type:path" format (e.g., "json:index.json").
    index_path: String,
}

impl AppConfig {
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

/// Defines common behavior for any index storage.
trait Storage {
    fn add(&mut self, path: &str, tags: Vec<String>) -> Result<()>;
    fn get(&self, tags: Vec<String>) -> Result<Vec<String>>;
    fn tags(&self) -> Result<Vec<(String, usize)>>;
}

struct JsonStorage {
    file_path: PathBuf,
    data: HashMap<String, Vec<String>>,
}

impl JsonStorage {
    fn new(path: &str) -> Result<Self> {
        let file_path = PathBuf::from(path);
        let data = if file_path.exists() {
            let content = fs::read_to_string(&file_path)
                .with_context(|| format!("Failed to read JSON file: {:?}", file_path))?;

            if content.trim().is_empty() {
                HashMap::new()
            } else {
                serde_json::from_str(&content).context("Failed to parse JSON")?
            }
        } else {
            HashMap::new()
        };

        Ok(JsonStorage { file_path, data })
    }

    fn save_to_disk(&self) -> Result<()> {
        let content = serde_json::to_string_pretty(&self.data)?;
        fs::write(&self.file_path, content)?;
        Ok(())
    }
}

impl Storage for JsonStorage {
    fn add(&mut self, path: &str, tags: Vec<String>) -> Result<()> {
        let cleaned_tags: Vec<String> = tags.into_iter().map(|t| t.trim().to_string()).collect();
        self.data.insert(path.to_string(), cleaned_tags);
        self.save_to_disk()?;
        Ok(())
    }

    fn get(&self, tags: Vec<String>) -> Result<Vec<String>> {
        let search_tags: Vec<String> = tags.into_iter().map(|t| t.trim().to_string()).collect();
        let mut results = Vec::new();

        for (path, file_tags) in &self.data {
            let has_all_tags = search_tags.iter().all(|tag| file_tags.contains(tag));
            if has_all_tags {
                results.push(path.clone());
            }
        }
        Ok(results)
    }

    fn tags(&self) -> Result<Vec<(String, usize)>> {
        let mut tag_counts: HashMap<String, usize> = HashMap::new();

        for file_tags in self.data.values() {
            for tag in file_tags {
                *tag_counts.entry(tag.clone()).or_insert(0) += 1;
            }
        }

        let mut result: Vec<(String, usize)> = tag_counts.into_iter().collect();
        result.sort_by(|a, b| a.0.cmp(&b.0));

        Ok(result)
    }
}

struct SqliteStorage {
    conn: Connection,
}

impl SqliteStorage {
    fn new(path: &str) -> Result<Self> {
        let conn = Connection::open(path)
            .with_context(|| format!("Failed to open SQLite database: '{}'", path))?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS file_tags (
                path TEXT NOT NULL,
                tag TEXT NOT NULL,
                PRIMARY KEY (path, tag)
            )",
            [],
        )
        .context("Failed to create SQLite table")?;

        Ok(SqliteStorage { conn })
    }
}

impl Storage for SqliteStorage {
    fn add(&mut self, path: &str, tags: Vec<String>) -> Result<()> {
        let tx = self.conn.transaction()?;

        tx.execute("DELETE FROM file_tags WHERE path = ?1", params![path])?;

        {
            let mut stmt = tx.prepare("INSERT INTO file_tags (path, tag) VALUES (?1, ?2)")?;
            for tag in tags {
                stmt.execute(params![path, tag.trim()])?;
            }
        }

        tx.commit()?;
        Ok(())
    }

    fn get(&self, tags: Vec<String>) -> Result<Vec<String>> {
        let search_tags: Vec<String> = tags.into_iter().map(|t| t.trim().to_string()).collect();
        if search_tags.is_empty() {
            return Ok(Vec::new());
        }

        let placeholders = search_tags
            .iter()
            .map(|_| "?")
            .collect::<Vec<_>>()
            .join(", ");
        let query = format!(
            "SELECT path FROM file_tags
             WHERE tag IN ({})
             GROUP BY path
             HAVING COUNT(DISTINCT tag) = ?",
            placeholders
        );

        let mut stmt = self.conn.prepare(&query)?;

        let mut sql_params: Vec<&dyn rusqlite::ToSql> = Vec::new();
        for tag in &search_tags {
            sql_params.push(tag);
        }
        let count = search_tags.len() as i32;
        sql_params.push(&count);

        let rows = stmt.query_map(sql_params.as_slice(), |row| row.get::<_, String>(0))?;

        let mut results = Vec::new();
        for path in rows {
            results.push(path?);
        }

        Ok(results)
    }

    fn tags(&self) -> Result<Vec<(String, usize)>> {
        let mut stmt = self
            .conn
            .prepare("SELECT tag, COUNT(path) FROM file_tags GROUP BY tag ORDER BY tag")?;

        let rows = stmt.query_map([], |row| {
            let count: i64 = row.get(1)?;
            Ok((row.get::<_, String>(0)?, count as usize))
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }

        Ok(results)
    }
}

/// Parses the configuration string to select the storage backend.
fn get_storage_provider(config_str: &str) -> Result<Box<dyn Storage>> {
    let (provider_type, path) = config_str
        .split_once(':')
        .context("Invalid index path format. Expected 'TYPE:PATH' (e.g., 'json:index.json')")?;

    match provider_type.to_uppercase().as_str() {
        "JSON" => Ok(Box::new(JsonStorage::new(path)?)),
        "SQLITE" => Ok(Box::new(SqliteStorage::new(path)?)),
        _ => bail!(
            "Unknown storage type: '{}'. Supported: JSON, SQLITE",
            provider_type
        ),
    }
}

fn main() -> Result<()> {
    let config = AppConfig::new()?;
    let cli = Cli::parse();

    let mut storage = get_storage_provider(&config.index_path)?;

    match cli.command {
        Commands::Add { path, tags } => {
            storage.add(&path, tags)?;
            println!("File '{}' successfully indexed.", path);
        }
        Commands::Get { tags } => {
            let files = storage.get(tags.clone())?;
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
            let all_tags = storage.tags()?;
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
