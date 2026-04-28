//! Core library for the file indexer application.
//!
//! Handles storage implementations using both JSON and SQLite backends,
//! enforcing strict error handling via `thiserror`.

#![warn(missing_docs)]
#![warn(rustdoc::broken_intra_doc_links)]
#![warn(rustdoc::missing_crate_level_docs)]
#![warn(unreachable_pub)]
#![warn(clippy::missing_panics_doc)]
#![warn(clippy::missing_errors_doc)]
#![warn(clippy::result_large_err)]
#![warn(clippy::clone_on_ref_ptr)]
#![warn(clippy::similar_names)]

use rusqlite::{params, Connection};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

/// Custom error type for indexer operations.
#[derive(Debug, Error)]
pub enum IndexerError {
    /// Represents a file system error.
    #[error("File system error: {0}")]
    Io(#[from] std::io::Error),

    /// Represents a JSON parsing or serialization error.
    #[error("JSON processing error: {0}")]
    Json(#[from] serde_json::Error),

    /// Represents an SQLite database error.
    #[error("Database error: {0}")]
    Sqlite(#[from] rusqlite::Error),
}

/// Defines common behavior for any index storage.
pub trait Storage {
    /// Adds a file with tags to the storage.
    ///
    /// # Errors
    ///
    /// Returns `IndexerError` if writing to the underlying storage fails.
    fn add(&mut self, path: &str, tags: Vec<String>) -> Result<(), IndexerError>;

    /// Retrieves files matching all specified tags.
    ///
    /// # Errors
    ///
    /// Returns `IndexerError` if reading from the underlying storage fails.
    fn get(&self, tags: Vec<String>) -> Result<Vec<String>, IndexerError>;

    /// Retrieves all tags and their file counts.
    ///
    /// # Errors
    ///
    /// Returns `IndexerError` if reading from the underlying storage fails.
    fn tags(&self) -> Result<Vec<(String, usize)>, IndexerError>;
}

/// Storage implementation using a JSON file.
pub struct JsonStorage {
    file_path: PathBuf,
    data: HashMap<String, Vec<String>>,
}

impl JsonStorage {
    /// Creates a new JSON storage instance.
    ///
    /// # Errors
    ///
    /// Returns `IndexerError` if the file exists but cannot be read or parsed.
    pub fn new(path: &str) -> Result<Self, IndexerError> {
        let file_path = PathBuf::from(path);
        let data = if file_path.exists() {
            let content = fs::read_to_string(&file_path)?;
            if content.trim().is_empty() {
                HashMap::new()
            } else {
                serde_json::from_str(&content)?
            }
        } else {
            HashMap::new()
        };

        Ok(Self { file_path, data })
    }

    fn save_to_disk(&self) -> Result<(), IndexerError> {
        let content = serde_json::to_string_pretty(&self.data)?;
        fs::write(&self.file_path, content)?;
        Ok(())
    }
}

impl Storage for JsonStorage {
    fn add(&mut self, path: &str, tags: Vec<String>) -> Result<(), IndexerError> {
        let cleaned_tags: Vec<String> = tags.into_iter().map(|t| t.trim().to_string()).collect();
        self.data.insert(path.to_string(), cleaned_tags);
        self.save_to_disk()?;
        Ok(())
    }

    fn get(&self, tags: Vec<String>) -> Result<Vec<String>, IndexerError> {
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

    fn tags(&self) -> Result<Vec<(String, usize)>, IndexerError> {
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

/// Storage implementation using an SQLite database.
pub struct SqliteStorage {
    conn: Connection,
}

impl SqliteStorage {
    /// Creates a new SQLite storage instance.
    ///
    /// # Errors
    ///
    /// Returns `IndexerError` if connecting to the DB or table creation fails.
    pub fn new(path: &str) -> Result<Self, IndexerError> {
        let conn = Connection::open(path)?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS file_tags (
                path TEXT NOT NULL,
                tag TEXT NOT NULL,
                PRIMARY KEY (path, tag)
            )",
            [],
        )?;
        Ok(Self { conn })
    }
}

impl Storage for SqliteStorage {
    fn add(&mut self, path: &str, tags: Vec<String>) -> Result<(), IndexerError> {
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

    fn get(&self, tags: Vec<String>) -> Result<Vec<String>, IndexerError> {
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

    fn tags(&self) -> Result<Vec<(String, usize)>, IndexerError> {
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
