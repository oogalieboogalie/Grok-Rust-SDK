//! Tool calling functionality

use crate::error::{GrokError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Trait for executable tools
#[async_trait::async_trait]
pub trait ToolExecutor: Send + Sync {
    /// Execute the tool with the given arguments
    async fn execute(&self, args: serde_json::Value) -> Result<serde_json::Value>;

    /// Get the tool specification
    fn spec(&self) -> ToolSpec;
}

/// Tool specification for function calling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSpec {
    /// Name of the tool
    pub name: String,
    /// Description of what the tool does
    pub description: String,
    /// Parameters schema (JSON Schema)
    pub parameters: serde_json::Value,
}

/// Tool definition for API requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    /// Type of tool (currently only "function" is supported)
    #[serde(rename = "type")]
    pub tool_type: String,
    /// Function specification
    pub function: ToolSpec,
}

impl Tool {
    /// Create a new tool definition
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        parameters: serde_json::Value,
    ) -> Self {
        Self {
            tool_type: "function".to_string(),
            function: ToolSpec {
                name: name.into(),
                description: description.into(),
                parameters,
            },
        }
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

/// Result of a tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    /// ID of the tool call this result corresponds to
    pub tool_call_id: String,
    /// The result content
    pub content: String,
}

/// Tool registry for managing available tools
#[derive(Debug)]
pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn ToolExecutor>>,
}

impl ToolRegistry {
    /// Create a new empty tool registry
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    /// Register a tool executor
    pub fn register<T: ToolExecutor + 'static>(&mut self, executor: T) {
        let spec = executor.spec();
        self.tools.insert(spec.name.clone(), Box::new(executor));
    }

    /// Get a tool by name
    pub fn get(&self, name: &str) -> Option<&dyn ToolExecutor> {
        self.tools.get(name).map(|t| t.as_ref())
    }

    /// Get all registered tools as API tool definitions
    pub fn api_tools(&self) -> Vec<Tool> {
        self.tools
            .values()
            .map(|executor| {
                let spec = executor.spec();
                Tool::new(
                    spec.name.clone(),
                    spec.description.clone(),
                    spec.parameters.clone(),
                )
            })
            .collect()
    }

    /// Execute a tool call
    pub async fn execute_tool_call(&self, tool_call: &ToolCall) -> Result<ToolResult> {
        let executor = self.get(&tool_call.function.name).ok_or_else(|| {
            GrokError::ToolExecution(format!("Tool '{}' not found", tool_call.function.name))
        })?;

        let args: serde_json::Value = serde_json::from_str(&tool_call.function.arguments)
            .map_err(|e| GrokError::ToolExecution(format!("Invalid tool arguments: {}", e)))?;

        // Validate arguments against the tool's parameter schema
        let spec = executor.spec();
        let schema = jsonschema::JSONSchema::compile(&spec.parameters)
            .map_err(|e| GrokError::ToolExecution(format!("Invalid parameter schema: {}", e)))?;

        if let Err(errors) = schema.validate(&args) {
            let error_messages: Vec<String> = errors.map(|e| e.to_string()).collect();
            return Err(GrokError::ToolExecution(format!(
                "Tool arguments validation failed: {}",
                error_messages.join(", ")
            )));
        }

        let result = executor
            .execute(args)
            .await
            .map_err(|e| GrokError::ToolExecution(format!("Tool execution failed: {}", e)))?;

        let content = serde_json::to_string(&result)
            .map_err(|e| GrokError::ToolExecution(format!("Failed to serialize result: {}", e)))?;

        Ok(ToolResult {
            tool_call_id: tool_call.id.clone(),
            content,
        })
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper macro to create tool parameter schemas
#[macro_export]
macro_rules! tool_params {
    ($($key:literal: $value:expr),* $(,)?) => {
        serde_json::json!({
            "type": "object",
            "properties": {
                $($key: $value,)*
            },
            "required": [$($key,)*]
        })
    };
}

/// Helper macro to create a simple tool parameter
#[macro_export]
macro_rules! param {
    (string, $desc:literal) => {
        serde_json::json!({
            "type": "string",
            "description": $desc
        })
    };
    (number, $desc:literal) => {
        serde_json::json!({
            "type": "number",
            "description": $desc
        })
    };
    (boolean, $desc:literal) => {
        serde_json::json!({
            "type": "boolean",
            "description": $desc
        })
    };
    (array, $items:expr, $desc:literal) => {
        serde_json::json!({
            "type": "array",
            "items": $items,
            "description": $desc
        })
    };
    (object, $properties:expr, $desc:literal) => {
        serde_json::json!({
            "type": "object",
            "properties": $properties,
            "description": $desc
        })
    };
}
