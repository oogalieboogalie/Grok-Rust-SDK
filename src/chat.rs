//! Chat completion types and message handling
//!
//! This module provides types and builders for working with chat completions,
//! including messages, models, and streaming responses.
//!
//! # Key Types
//!
//! - [`Model`]: Enum representing available Grok models
//! - [`Message`]: Chat messages with role and content
//! - [`MessageBuilder`]: Builder for constructing complex messages
//! - [`ChatCompletion`]: Complete chat response
//! - [`ChatChunk`]: Streaming response chunk
//!
//! # Examples
//!
//! Creating messages:
//!
//! ```
//! use grok_rust_sdk::chat::Message;
//!
//! let user_msg = Message::user("Hello!");
//! let system_msg = Message::system("You are a helpful assistant");
//! let assistant_msg = Message::assistant("Hello! How can I help?");
//! ```
//!
//! Using the message builder:
//!
//! ```
//! use grok_rust_sdk::chat::{Message, Role};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let msg = Message::builder()
//!     .role(Role::User)
//!     .content("Tell me a joke")
//!     .build()?;
//! # Ok(())
//! # }
//! ```
//!
//! Parsing model names:
//!
//! ```
//! use grok_rust_sdk::chat::Model;
//! use std::str::FromStr;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let model = Model::from_str("grok-4-fast-reasoning")?;
//! assert_eq!(model, Model::Grok4FastReasoning);
//! # Ok(())
//! # }
//! ```

use crate::error::{GrokError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Available Grok models
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Model {
    /// Grok-4 with fast reasoning
    Grok4FastReasoning,
    /// Grok-4 standard
    Grok4,
    /// Grok-3 (legacy)
    Grok3,
    /// Grok-2 (legacy)
    Grok2,
    /// Grok-1 (legacy)
    Grok1,
}

impl Model {
    /// Get the model string identifier
    pub fn as_str(&self) -> &'static str {
        match self {
            Model::Grok4FastReasoning => "grok-4-fast-reasoning",
            Model::Grok4 => "grok-4",
            Model::Grok3 => "grok-3",
            Model::Grok2 => "grok-2",
            Model::Grok1 => "grok-1",
        }
    }
}

impl std::fmt::Display for Model {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for Model {
    type Err = GrokError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "grok-4-fast-reasoning" => Ok(Model::Grok4FastReasoning),
            "grok-4" => Ok(Model::Grok4),
            "grok-3" => Ok(Model::Grok3),
            "grok-2" => Ok(Model::Grok2),
            "grok-1" => Ok(Model::Grok1),
            _ => Err(GrokError::InvalidConfig(format!(
                "Unknown model: {}. Valid models are: grok-4-fast-reasoning, grok-4, grok-3, grok-2, grok-1",
                s
            ))),
        }
    }
}

/// Message roles in a conversation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// System message (instructions)
    System,
    /// User message
    User,
    /// Assistant response
    Assistant,
    /// Tool execution result
    Tool,
}

/// A message in a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// The role of the message sender
    pub role: Role,
    /// The content of the message
    pub content: String,
    /// Optional tool calls made by the assistant
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    /// Optional tool call ID (for tool results)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    /// Optional name of the tool (for tool results)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl Message {
    /// Create a user message
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: Role::User,
            content: content.into(),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        }
    }

    /// Create a system message
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: Role::System,
            content: content.into(),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        }
    }

    /// Create an assistant message
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: Role::Assistant,
            content: content.into(),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        }
    }

    /// Create a tool result message
    pub fn tool(
        content: impl Into<String>,
        tool_call_id: impl Into<String>,
        name: impl Into<String>,
    ) -> Self {
        Self {
            role: Role::Tool,
            content: content.into(),
            tool_calls: None,
            tool_call_id: Some(tool_call_id.into()),
            name: Some(name.into()),
        }
    }

    /// Create a builder for constructing messages with custom options
    pub fn builder() -> MessageBuilder {
        MessageBuilder::new()
    }
}

/// Builder for creating messages with custom options
#[derive(Debug, Default)]
pub struct MessageBuilder {
    role: Option<Role>,
    content: Option<String>,
    tool_calls: Option<Vec<ToolCall>>,
    tool_call_id: Option<String>,
    name: Option<String>,
}

impl MessageBuilder {
    /// Create a new message builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the role
    pub fn role(mut self, role: Role) -> Self {
        self.role = Some(role);
        self
    }

    /// Set the content
    pub fn content(mut self, content: impl Into<String>) -> Self {
        self.content = Some(content.into());
        self
    }

    /// Set tool calls
    pub fn tool_calls(mut self, tool_calls: Vec<ToolCall>) -> Self {
        self.tool_calls = Some(tool_calls);
        self
    }

    /// Set tool call ID
    pub fn tool_call_id(mut self, tool_call_id: impl Into<String>) -> Self {
        self.tool_call_id = Some(tool_call_id.into());
        self
    }

    /// Set name
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Build the message
    pub fn build(self) -> Result<Message> {
        let role = self
            .role
            .ok_or_else(|| GrokError::InvalidConfig("Message role is required".to_string()))?;
        let content = self
            .content
            .ok_or_else(|| GrokError::InvalidConfig("Message content is required".to_string()))?;

        Ok(Message {
            role,
            content,
            tool_calls: self.tool_calls,
            tool_call_id: self.tool_call_id,
            name: self.name,
        })
    }
}

/// Tool call made by the assistant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// Unique ID for this tool call
    pub id: String,
    /// The tool/function to call
    pub function: ToolFunction,
}

/// Function specification for a tool call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolFunction {
    /// Name of the function to call
    pub name: String,
    /// Arguments to pass to the function (as JSON string)
    pub arguments: String,
}

/// Tool definition for function calling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    /// Type of tool (currently only "function" is supported)
    #[serde(rename = "type")]
    pub tool_type: String,
    /// Function specification
    pub function: ToolSpec,
}

/// Function specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSpec {
    /// Name of the function
    pub name: String,
    /// Description of what the function does
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Parameters schema (JSON Schema)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<serde_json::Value>,
}

/// Chat completion request
#[derive(Debug, Serialize)]
struct ChatRequest {
    /// Model to use
    model: String,
    /// Messages in the conversation
    messages: Vec<Message>,
    /// Maximum tokens to generate
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    /// Temperature for randomness (0.0 to 2.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    /// Top-p sampling parameter
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    /// Tools available for function calling
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<Tool>>,
    /// Tool choice strategy
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_choice: Option<serde_json::Value>,
    /// Response format specification
    #[serde(skip_serializing_if = "Option::is_none")]
    response_format: Option<serde_json::Value>,
    /// Stop sequences
    #[serde(skip_serializing_if = "Option::is_none")]
    stop: Option<Vec<String>>,
    /// Enable streaming responses
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

/// Chat completion response
#[derive(Debug, Deserialize)]
struct ChatResponse {
    /// Unique ID for the completion
    id: String,
    /// Object type (always "chat.completion")
    object: String,
    /// Timestamp of creation
    created: u64,
    /// Model used
    model: String,
    /// Usage statistics
    usage: Option<Usage>,
    /// Response choices
    choices: Vec<Choice>,
}

/// Usage statistics for the completion
#[derive(Debug, Deserialize)]
pub struct Usage {
    /// Number of prompt tokens
    pub prompt_tokens: u32,
    /// Number of completion tokens
    pub completion_tokens: u32,
    /// Total number of tokens
    pub total_tokens: u32,
}

/// A completion choice
#[derive(Debug, Deserialize)]
struct Choice {
    /// Index of the choice
    index: u32,
    /// The message content
    message: Message,
    /// Finish reason
    finish_reason: Option<String>,
}

/// Chat completion result
#[derive(Debug)]
pub struct ChatCompletion {
    /// Unique ID for the completion
    pub id: String,
    /// Model used
    pub model: String,
    /// Usage statistics
    pub usage: Option<Usage>,
    /// The response message
    pub message: Message,
    /// Finish reason
    pub finish_reason: Option<String>,
}

/// Streaming chat completion chunk
#[derive(Debug, Deserialize)]
pub struct ChatChunk {
    /// Unique ID for the completion
    pub id: String,
    /// Object type
    pub object: String,
    /// Timestamp of creation
    pub created: u64,
    /// Model used
    pub model: String,
    /// Response choices
    pub choices: Vec<ChunkChoice>,
}

/// A chunk choice in streaming response
#[derive(Debug, Deserialize)]
pub struct ChunkChoice {
    /// Index of the choice
    pub index: u32,
    /// Delta content
    pub delta: MessageDelta,
    /// Finish reason
    pub finish_reason: Option<String>,
}

/// Delta for streaming message updates
#[derive(Debug, Deserialize)]
pub struct MessageDelta {
    /// Role (only present in first chunk)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<Role>,
    /// Content delta
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    /// Tool calls delta
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCallDelta>>,
}

/// Delta for tool calls in streaming
#[derive(Debug, Deserialize)]
pub struct ToolCallDelta {
    /// Index of the tool call
    pub index: u32,
    /// ID delta
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Function delta
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function: Option<ToolFunctionDelta>,
}

/// Delta for tool function in streaming
#[derive(Debug, Deserialize)]
pub struct ToolFunctionDelta {
    /// Name delta
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Arguments delta
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<String>,
}
