//! Collections for organizing conversations

use crate::error::{GrokError, Result};
use crate::session::{Session, SessionManager};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// A collection of related sessions
#[derive(Debug)]
pub struct Collection {
    /// Unique collection ID
    pub id: String,
    /// Collection metadata
    pub metadata: CollectionMetadata,
    /// Sessions in this collection
    sessions: RwLock<HashMap<String, Arc<Session>>>,
}

/// Collection metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionMetadata {
    /// Human-readable name
    pub name: String,
    /// Description of the collection
    pub description: Option<String>,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last activity timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,
    /// Tags for organization
    pub tags: Vec<String>,
    /// Total sessions in collection
    pub session_count: usize,
    /// Total messages across all sessions
    pub total_messages: usize,
    /// Total tokens used across all sessions
    pub total_tokens: u64,
}

impl Collection {
    /// Create a new collection
    pub fn new(name: impl Into<String>, description: Option<String>, tags: Vec<String>) -> Self {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now();

        Self {
            id,
            metadata: CollectionMetadata {
                name: name.into(),
                description,
                created_at: now,
                updated_at: now,
                tags,
                session_count: 0,
                total_messages: 0,
                total_tokens: 0,
            },
            sessions: RwLock::new(HashMap::new()),
        }
    }

    /// Add a session to the collection
    pub async fn add_session(&self, session: Arc<Session>) -> Result<()> {
        let session_id = session.id.clone();
        let mut sessions = self.sessions.write().await;
        sessions.insert(session_id, session);
        drop(sessions);

        self.update_metadata().await;
        Ok(())
    }

    /// Remove a session from the collection
    pub async fn remove_session(&self, session_id: &str) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        sessions.remove(session_id)
            .ok_or_else(|| GrokError::Collection(format!("Session '{}' not in collection", session_id)))?;
        drop(sessions);

        self.update_metadata().await;
        Ok(())
    }

    /// Get a session by ID
    pub async fn get_session(&self, session_id: &str) -> Option<Arc<Session>> {
        let sessions = self.sessions.read().await;
        sessions.get(session_id).cloned()
    }

    /// List all sessions in the collection
    pub async fn list_sessions(&self) -> Vec<Arc<Session>> {
        let sessions = self.sessions.read().await;
        sessions.values().cloned().collect()
    }

    /// Update collection metadata
    async fn update_metadata(&self) {
        let sessions = self.sessions.read().await;
        let session_count = sessions.len();
        let total_messages = sessions.values()
            .map(|s| s.metadata.message_count)
            .sum();
        let total_tokens = sessions.values()
            .map(|s| s.metadata.total_tokens)
            .sum();

        let mut metadata = &mut self.metadata;
        metadata.session_count = session_count;
        metadata.total_messages = total_messages;
        metadata.total_tokens = total_tokens;
        metadata.updated_at = chrono::Utc::now();
    }

    /// Search sessions by title or content
    pub async fn search_sessions(&self, query: &str) -> Vec<Arc<Session>> {
        let sessions = self.sessions.read().await;
        sessions.values()
            .filter(|session| {
                // Search in title
                if let Some(title) = &session.metadata.title {
                    if title.to_lowercase().contains(&query.to_lowercase()) {
                        return true;
                    }
                }

                // Search in message content (basic implementation)
                // In a real implementation, you might want to index messages
                false
            })
            .cloned()
            .collect()
    }
}

/// Collection manager for handling multiple collections
#[derive(Debug)]
pub struct CollectionManager {
    session_manager: Arc<SessionManager>,
    collections: RwLock<HashMap<String, Arc<Collection>>>,
}

impl CollectionManager {
    /// Create a new collection manager
    pub fn new(session_manager: Arc<SessionManager>) -> Self {
        Self {
            session_manager,
            collections: RwLock::new(HashMap::new()),
        }
    }

    /// Create a new collection
    pub async fn create_collection(&self, name: impl Into<String>, description: Option<String>, tags: Vec<String>) -> Arc<Collection> {
        let collection = Arc::new(Collection::new(name, description, tags));
        let collection_id = collection.id.clone();

        let mut collections = self.collections.write().await;
        collections.insert(collection_id, collection.clone());

        collection
    }

    /// Get a collection by ID
    pub async fn get_collection(&self, collection_id: &str) -> Option<Arc<Collection>> {
        let collections = self.collections.read().await;
        collections.get(collection_id).cloned()
    }

    /// List all collections
    pub async fn list_collections(&self) -> Vec<Arc<Collection>> {
        let collections = self.collections.read().await;
        collections.values().cloned().collect()
    }

    /// Delete a collection
    pub async fn delete_collection(&self, collection_id: &str) -> Result<()> {
        let mut collections = self.collections.write().await;
        collections.remove(collection_id)
            .ok_or_else(|| GrokError::Collection(format!("Collection '{}' not found", collection_id)))?;
        Ok(())
    }

    /// Search collections by name, description, or tags
    pub async fn search_collections(&self, query: &str) -> Vec<Arc<Collection>> {
        let collections = self.collections.read().await;
        let query_lower = query.to_lowercase();

        collections.values()
            .filter(|collection| {
                // Search in name
                if collection.metadata.name.to_lowercase().contains(&query_lower) {
                    return true;
                }

                // Search in description
                if let Some(desc) = &collection.metadata.description {
                    if desc.to_lowercase().contains(&query_lower) {
                        return true;
                    }
                }

                // Search in tags
                collection.metadata.tags.iter()
                    .any(|tag| tag.to_lowercase().contains(&query_lower))
            })
            .cloned()
            .collect()
    }

    /// Get collections by tag
    pub async fn collections_by_tag(&self, tag: &str) -> Vec<Arc<Collection>> {
        let collections = self.collections.read().await;
        collections.values()
            .filter(|collection| collection.metadata.tags.contains(&tag.to_string()))
            .cloned()
            .collect()
    }

    /// Get collection statistics
    pub async fn stats(&self) -> CollectionStats {
        let collections = self.collections.read().await;
        let total_collections = collections.len();
        let total_sessions = collections.values()
            .map(|c| c.metadata.session_count)
            .sum();
        let total_messages = collections.values()
            .map(|c| c.metadata.total_messages)
            .sum();
        let total_tokens = collections.values()
            .map(|c| c.metadata.total_tokens)
            .sum();

        CollectionStats {
            total_collections,
            total_sessions,
            total_messages,
            total_tokens,
        }
    }
}

/// Collection statistics
#[derive(Debug, Clone)]
pub struct CollectionStats {
    /// Total number of collections
    pub total_collections: usize,
    /// Total number of sessions across all collections
    pub total_sessions: usize,
    /// Total number of messages across all collections
    pub total_messages: usize,
    /// Total tokens used across all collections
    pub total_tokens: u64,
}