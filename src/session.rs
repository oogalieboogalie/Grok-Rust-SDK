//! Stateful conversation sessions

use crate::chat::{Message, Model, Tool};
use crate::error::{GrokError, Result};
use crate::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// A stateful conversation session
#[derive(Debug)]
pub struct Session {
    /// Unique session ID
    pub id: String,
    /// The client used for API calls
    client: Arc<Client>,
    /// Model to use for this session
    model: Model,
    /// Conversation history
    messages: RwLock<Vec<Message>>,
    /// Available tools
    tools: Vec<Tool>,
    /// Session metadata
    metadata: SessionMetadata,
}

/// Session metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    /// Human-readable title
    pub title: Option<String>,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last activity timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,
    /// Total tokens used in this session
    pub total_tokens: u64,
    /// Number of messages in the session
    pub message_count: usize,
}

impl Session {
    /// Create a new session
    pub fn new(client: Arc<Client>, model: Model, title: Option<String>) -> Self {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now();

        Self {
            id,
            client,
            model,
            messages: RwLock::new(Vec::new()),
            tools: Vec::new(),
            metadata: SessionMetadata {
                title,
                created_at: now,
                updated_at: now,
                total_tokens: 0,
                message_count: 0,
            },
        }
    }

    /// Add a tool to the session
    pub fn add_tool(&mut self, tool: Tool) {
        self.tools.push(tool);
    }

    /// Add multiple tools to the session
    pub fn add_tools(&mut self, tools: Vec<Tool>) {
        self.tools.extend(tools);
    }

    /// Append a message to the conversation
    pub async fn append(&self, message: Message) -> Result<()> {
        let mut messages = self.messages.write().await;
        messages.push(message);
        drop(messages);

        let mut metadata = &mut self.metadata;
        metadata.message_count += 1;
        metadata.updated_at = chrono::Utc::now();

        Ok(())
    }

    /// Send a user message and get assistant response
    pub async fn chat(&self, content: impl Into<String>) -> Result<crate::chat::ChatCompletion> {
        let user_message = Message {
            role: crate::chat::Role::User,
            content: content.into(),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        };

        self.append(user_message).await?;

        let messages = self.messages.read().await.clone();
        let tools = if self.tools.is_empty() {
            None
        } else {
            Some(self.tools.clone())
        };

        let response = self.client.chat(self.model, messages, tools).await?;

        // Add assistant response to history
        self.append(response.message.clone()).await?;

        Ok(response)
    }

    /// Execute tool calls and continue the conversation
    pub async fn execute_tools(
        &self,
        tool_calls: &[crate::chat::ToolCall],
        tool_registry: &crate::tools::ToolRegistry,
    ) -> Result<()> {
        for tool_call in tool_calls {
            let result = tool_registry.execute_tool_call(tool_call).await?;

            let tool_message = Message {
                role: crate::chat::Role::Tool,
                content: result.content,
                tool_calls: None,
                tool_call_id: Some(result.tool_call_id),
                name: Some(tool_call.function.name.clone()),
            };

            self.append(tool_message).await?;
        }

        Ok(())
    }

    /// Get the conversation history
    pub async fn messages(&self) -> Vec<Message> {
        self.messages.read().await.clone()
    }

    /// Get session metadata
    pub fn metadata(&self) -> &SessionMetadata {
        &self.metadata
    }

    /// Clear the conversation history (keep system messages)
    pub async fn clear_history(&self) -> Result<()> {
        let mut messages = self.messages.write().await;
        let system_messages: Vec<Message> = messages
            .drain(..)
            .filter(|msg| matches!(msg.role, crate::chat::Role::System))
            .collect();
        *messages = system_messages;
        drop(messages);

        let mut metadata = &mut self.metadata;
        metadata.message_count = messages.len();
        metadata.updated_at = chrono::Utc::now();

        Ok(())
    }

    /// Get the number of messages in the session
    pub async fn message_count(&self) -> usize {
        self.messages.read().await.len()
    }
}

/// Session manager for handling multiple conversations
#[derive(Debug)]
pub struct SessionManager {
    client: Arc<Client>,
    sessions: RwLock<HashMap<String, Arc<Session>>>,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new(client: Arc<Client>) -> Self {
        Self {
            client,
            sessions: RwLock::new(HashMap::new()),
        }
    }

    /// Create a new session
    pub async fn create_session(&self, model: Model, title: Option<String>) -> Arc<Session> {
        let session = Arc::new(Session::new(self.client.clone(), model, title));
        let session_id = session.id.clone();

        let mut sessions = self.sessions.write().await;
        sessions.insert(session_id, session.clone());

        session
    }

    /// Get a session by ID
    pub async fn get_session(&self, session_id: &str) -> Option<Arc<Session>> {
        let sessions = self.sessions.read().await;
        sessions.get(session_id).cloned()
    }

    /// List all sessions
    pub async fn list_sessions(&self) -> Vec<Arc<Session>> {
        let sessions = self.sessions.read().await;
        sessions.values().cloned().collect()
    }

    /// Delete a session
    pub async fn delete_session(&self, session_id: &str) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        sessions
            .remove(session_id)
            .ok_or_else(|| GrokError::Session(format!("Session '{}' not found", session_id)))?;
        Ok(())
    }

    /// Get session statistics
    pub async fn stats(&self) -> SessionStats {
        let sessions = self.sessions.read().await;
        let total_sessions = sessions.len();
        let total_messages = sessions.values().map(|s| s.metadata.message_count).sum();
        let total_tokens = sessions.values().map(|s| s.metadata.total_tokens).sum();

        SessionStats {
            total_sessions,
            total_messages,
            total_tokens,
        }
    }
}

/// Session statistics
#[derive(Debug, Clone)]
pub struct SessionStats {
    /// Total number of sessions
    pub total_sessions: usize,
    /// Total number of messages across all sessions
    pub total_messages: usize,
    /// Total tokens used across all sessions
    pub total_tokens: u64,
}
