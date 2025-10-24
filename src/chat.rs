//! Chat completion functionality

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