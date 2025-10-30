//! Persistence layer for storing sessions and collections in SQLite

use crate::collections::CollectionManager;
use crate::error::{GrokError, Result};
use crate::session::{Session, SessionManager};
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

/// SQLite-based storage for sessions and collections
#[derive(Debug)]
pub struct SqliteStorage {
    conn: Arc<RwLock<Connection>>,
}

impl SqliteStorage {
    /// Create a new SQLite storage instance
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path)
            .map_err(|e| GrokError::Session(format!("Failed to open database: {}", e)))?;

        // Create tables
        conn.execute(
            "CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                model TEXT NOT NULL,
                created_at TEXT NOT NULL,
                messages TEXT NOT NULL
            )",
            [],
        )
        .map_err(|e| GrokError::Session(format!("Failed to create sessions table: {}", e)))?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS collections (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                created_at TEXT NOT NULL
            )",
            [],
        )
        .map_err(|e| GrokError::Collection(format!("Failed to create collections table: {}", e)))?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS collection_sessions (
                collection_id TEXT NOT NULL,
                session_id TEXT NOT NULL,
                added_at TEXT NOT NULL,
                PRIMARY KEY (collection_id, session_id),
                FOREIGN KEY (collection_id) REFERENCES collections(id) ON DELETE CASCADE,
                FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE
            )",
            [],
        )
        .map_err(|e| {
            GrokError::Collection(format!("Failed to create collection_sessions table: {}", e))
        })?;

        Ok(Self {
            conn: Arc::new(RwLock::new(conn)),
        })
    }

    /// Create an in-memory SQLite storage (for testing)
    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory().map_err(|e| {
            GrokError::Session(format!("Failed to create in-memory database: {}", e))
        })?;

        // Create tables (same as above)
        conn.execute(
            "CREATE TABLE sessions (
                id TEXT PRIMARY KEY,
                model TEXT NOT NULL,
                created_at TEXT NOT NULL,
                messages TEXT NOT NULL
            )",
            [],
        )
        .map_err(|e| GrokError::Session(format!("Failed to create sessions table: {}", e)))?;

        conn.execute(
            "CREATE TABLE collections (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                created_at TEXT NOT NULL
            )",
            [],
        )
        .map_err(|e| GrokError::Collection(format!("Failed to create collections table: {}", e)))?;

        conn.execute(
            "CREATE TABLE collection_sessions (
                collection_id TEXT NOT NULL,
                session_id TEXT NOT NULL,
                added_at TEXT NOT NULL,
                PRIMARY KEY (collection_id, session_id),
                FOREIGN KEY (collection_id) REFERENCES collections(id) ON DELETE CASCADE,
                FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE
            )",
            [],
        )
        .map_err(|e| {
            GrokError::Collection(format!("Failed to create collection_sessions table: {}", e))
        })?;

        Ok(Self {
            conn: Arc::new(RwLock::new(conn)),
        })
    }

    /// Save a session to storage
    pub async fn save_session(&self, session: &Session) -> Result<()> {
        let conn = self.conn.read().await;
        let messages_json = serde_json::to_string(&session.messages())
            .map_err(|e| GrokError::Session(format!("Failed to serialize messages: {}", e)))?;

        conn.execute(
            "INSERT OR REPLACE INTO sessions (id, model, created_at, messages) VALUES (?1, ?2, ?3, ?4)",
            params![
                session.id(),
                session.model().as_str(),
                session.created_at().to_rfc3339(),
                messages_json
            ],
        ).map_err(|e| GrokError::Session(format!("Failed to save session: {}", e)))?;

        Ok(())
    }

    /// Load a session from storage
    pub async fn load_session(&self, session_id: &str) -> Result<Option<Session>> {
        let conn = self.conn.read().await;
        let result = conn
            .query_row(
                "SELECT id, model, created_at, messages FROM sessions WHERE id = ?1",
                params![session_id],
                |row| {
                    let id: String = row.get(0)?;
                    let model_str: String = row.get(1)?;
                    let created_at_str: String = row.get(2)?;
                    let messages_json: String = row.get(3)?;

                    let model = match model_str.as_str() {
                        "grok-4-fast-reasoning" => crate::Model::Grok4FastReasoning,
                        "grok-4" => crate::Model::Grok4,
                        "grok-3" => crate::Model::Grok3,
                        "grok-2" => crate::Model::Grok2,
                        "grok-1" => crate::Model::Grok1,
                        _ => {
                            return Err(rusqlite::Error::InvalidColumnType(
                                1,
                                "model".to_string(),
                                rusqlite::types::Type::Text,
                            ))
                        }
                    };

                    let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)
                        .map_err(|_| {
                            rusqlite::Error::InvalidColumnType(
                                2,
                                "created_at".to_string(),
                                rusqlite::types::Type::Text,
                            )
                        })?
                        .with_timezone(&chrono::Utc);

                    let messages: Vec<crate::chat::Message> = serde_json::from_str(&messages_json)
                        .map_err(|_| {
                            rusqlite::Error::InvalidColumnType(
                                3,
                                "messages".to_string(),
                                rusqlite::types::Type::Text,
                            )
                        })?;

                    Ok(Session::restore(id, model, created_at, messages))
                },
            )
            .optional()
            .map_err(|e| GrokError::Session(format!("Failed to load session: {}", e)))?;

        Ok(result)
    }

    /// Delete a session from storage
    pub async fn delete_session(&self, session_id: &str) -> Result<()> {
        let conn = self.conn.read().await;
        conn.execute("DELETE FROM sessions WHERE id = ?1", params![session_id])
            .map_err(|e| GrokError::Session(format!("Failed to delete session: {}", e)))?;

        Ok(())
    }

    /// List all session IDs
    pub async fn list_sessions(&self) -> Result<Vec<String>> {
        let conn = self.conn.read().await;
        let mut stmt = conn
            .prepare("SELECT id FROM sessions ORDER BY created_at DESC")
            .map_err(|e| GrokError::Session(format!("Failed to prepare statement: {}", e)))?;

        let ids = stmt
            .query_map([], |row| row.get(0))?
            .collect::<std::result::Result<Vec<String>, _>>()
            .map_err(|e| GrokError::Session(format!("Failed to list sessions: {}", e)))?;

        Ok(ids)
    }

    /// Save a collection to storage
    pub async fn save_collection(&self, collection: &crate::collections::Collection) -> Result<()> {
        let conn = self.conn.read().await;
        conn.execute(
            "INSERT OR REPLACE INTO collections (id, name, description, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![
                collection.id(),
                collection.name(),
                collection.description(),
                collection.created_at().to_rfc3339()
            ],
        ).map_err(|e| GrokError::Collection(format!("Failed to save collection: {}", e)))?;

        // Save session associations
        for session_id in collection.session_ids() {
            conn.execute(
                "INSERT OR IGNORE INTO collection_sessions (collection_id, session_id, added_at) VALUES (?1, ?2, ?3)",
                params![
                    collection.id(),
                    session_id,
                    chrono::Utc::now().to_rfc3339()
                ],
            ).map_err(|e| GrokError::Collection(format!("Failed to save collection session: {}", e)))?;
        }

        Ok(())
    }

    /// Load a collection from storage
    pub async fn load_collection(
        &self,
        collection_id: &str,
    ) -> Result<Option<crate::collections::Collection>> {
        let conn = self.conn.read().await;

        // Load collection metadata
        let collection_data = conn
            .query_row(
                "SELECT id, name, description, created_at FROM collections WHERE id = ?1",
                params![collection_id],
                |row| {
                    let id: String = row.get(0)?;
                    let name: String = row.get(1)?;
                    let description: Option<String> = row.get(2)?;
                    let created_at_str: String = row.get(3)?;

                    let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)
                        .map_err(|_| {
                            rusqlite::Error::InvalidColumnType(
                                3,
                                "created_at".to_string(),
                                rusqlite::types::Type::Text,
                            )
                        })?
                        .with_timezone(&chrono::Utc);

                    Ok((id, name, description, created_at))
                },
            )
            .optional()
            .map_err(|e| GrokError::Collection(format!("Failed to load collection: {}", e)))?;

        if let Some((id, name, description, created_at)) = collection_data {
            // Load associated session IDs
            let mut stmt = conn.prepare("SELECT session_id FROM collection_sessions WHERE collection_id = ?1 ORDER BY added_at")
                .map_err(|e| GrokError::Collection(format!("Failed to prepare statement: {}", e)))?;

            let session_ids = stmt
                .query_map(params![collection_id], |row| row.get(0))?
                .collect::<std::result::Result<Vec<String>, _>>()
                .map_err(|e| {
                    GrokError::Collection(format!("Failed to load collection sessions: {}", e))
                })?;

            let collection = crate::collections::Collection::restore(
                id,
                name,
                description,
                created_at,
                session_ids,
            );
            Ok(Some(collection))
        } else {
            Ok(None)
        }
    }

    /// Delete a collection from storage
    pub async fn delete_collection(&self, collection_id: &str) -> Result<()> {
        let conn = self.conn.read().await;
        conn.execute(
            "DELETE FROM collections WHERE id = ?1",
            params![collection_id],
        )
        .map_err(|e| GrokError::Collection(format!("Failed to delete collection: {}", e)))?;

        Ok(())
    }

    /// List all collection IDs
    pub async fn list_collections(&self) -> Result<Vec<String>> {
        let conn = self.conn.read().await;
        let mut stmt = conn
            .prepare("SELECT id FROM collections ORDER BY created_at DESC")
            .map_err(|e| GrokError::Collection(format!("Failed to prepare statement: {}", e)))?;

        let ids = stmt
            .query_map([], |row| row.get(0))?
            .collect::<std::result::Result<Vec<String>, _>>()
            .map_err(|e| GrokError::Collection(format!("Failed to list collections: {}", e)))?;

        Ok(ids)
    }
}

/// Persistent session manager that uses SQLite storage
pub type PersistentSessionManager = SessionManager<SqliteStorage>;

/// Persistent collection manager that uses SQLite storage
pub type PersistentCollectionManager = CollectionManager<SqliteStorage>;
