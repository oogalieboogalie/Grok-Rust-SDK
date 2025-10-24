//! Tool calling example for the Grok Rust SDK

use async_trait::async_trait;
use grok_rust_sdk::{
    chat::{Message, Model, Role},
    tools::{ToolExecutor, ToolRegistry, ToolSpec},
    Client,
};
use serde_json;

#[derive(Debug)]
struct CalculatorTool;

#[async_trait]
impl ToolExecutor for CalculatorTool {
    async fn execute(
        &self,
        args: serde_json::Value,
    ) -> Result<serde_json::Value, grok_rust_sdk::GrokError> {
        let expression = args["expression"].as_str().ok_or_else(|| {
            grok_rust_sdk::GrokError::ToolExecution("Missing expression".to_string())
        })?;

                // Simple calculator (in production, use a proper math library)
        let result = match expression {
            "2+2" => 4.0,
            "15*7" => 105.0,
            "10/2" => 5.0,
            "3-1" => 2.0,
            _ => {
                // For other expressions, try basic parsing
                if let Some(pos) = expression.find('+') {
                    let a: f64 = expression[..pos].trim().parse().unwrap_or(0.0);
                    let b: f64 = expression[pos+1..].trim().parse().unwrap_or(0.0);
                    a + b
                } else if let Some(pos) = expression.find('*') {
                    let a: f64 = expression[..pos].trim().parse().unwrap_or(0.0);
                    let b: f64 = expression[pos+1..].trim().parse().unwrap_or(0.0);
                    a * b
                } else {
                    return Err(grok_rust_sdk::GrokError::ToolExecution("Unsupported expression".to_string()));
                }
            }
        };
    }

    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: "calculate".to_string(),
            description: "Calculate a mathematical expression".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "expression": {
                        "type": "string",
                        "description": "Mathematical expression to evaluate (e.g., '2 + 2', 'sin(3.14)')"
                    }
                },
                "required": ["expression"]
            }),
        }
    }
}

#[derive(Debug)]
struct WebSearchTool;

#[async_trait]
impl ToolExecutor for WebSearchTool {
    async fn execute(
        &self,
        args: serde_json::Value,
    ) -> Result<serde_json::Value, grok_rust_sdk::GrokError> {
        let query = args["query"]
            .as_str()
            .ok_or_else(|| grok_rust_sdk::GrokError::ToolExecution("Missing query".to_string()))?;

        // Mock web search (in production, integrate with a real search API)
        let results = vec![
            format!("Result 1 for '{}': This is a mock search result.", query),
            format!(
                "Result 2 for '{}': Another mock result with more details.",
                query
            ),
            format!("Result 3 for '{}': Final mock result.", query),
        ];

        Ok(serde_json::json!({
            "query": query,
            "results": results,
            "total_results": results.len()
        }))
    }

    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: "web_search".to_string(),
            description: "Search the web for information".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Search query"
                    }
                },
                "required": ["query"]
            }),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load API key from environment
    let api_key =
        std::env::var("XAI_API_KEY").expect("XAI_API_KEY environment variable must be set");

    // Create client and tool registry
    let client = Client::new(api_key)?;
    let mut registry = ToolRegistry::new();

    // Register tools
    registry.register(CalculatorTool);
    registry.register(WebSearchTool);

    // Get API tool definitions
    let tools = registry.api_tools();

    println!("Registered {} tools:", tools.len());
    for tool in &tools {
        println!("- {}", tool.function.name);
    }

    // Create conversation
    let messages = vec![
        Message {
            role: Role::System,
            content: "You are a helpful assistant with access to tools. Use them when appropriate."
                .to_string(),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        },
        Message {
            role: Role::User,
            content:
                "What is 15 * 7? Also, can you search for the latest news about Rust programming?"
                    .to_string(),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        },
    ];

    println!("\nSending request with tool calling...");

    // Send request with tools
    let response = client
        .chat(Model::Grok4FastReasoning, messages.clone(), Some(tools))
        .await?;

    println!("Assistant response: {}", response.message.content);

    // Handle tool calls
    if let Some(tool_calls) = &response.message.tool_calls {
        println!("\nExecuting {} tool calls:", tool_calls.len());

        for (i, tool_call) in tool_calls.iter().enumerate() {
            println!("\nTool call {}: {}", i + 1, tool_call.function.name);
            println!("Arguments: {}", tool_call.function.arguments);

            // Execute the tool
            let result = registry.execute_tool_call(tool_call).await?;
            println!("Result: {}", result.content);

            // In a real conversation, you would add this result back to messages
            // and continue the conversation
        }
    } else {
        println!("\nNo tool calls made by assistant.");
    }

    Ok(())
}
