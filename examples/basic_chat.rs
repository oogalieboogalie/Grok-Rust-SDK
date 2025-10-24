//! Basic chat example for the Grok Rust SDK

use grok_rust_sdk::{Client, chat::{Message, Role, Model}};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load API key from environment
    let api_key = std::env::var("XAI_API_KEY")
        .expect("XAI_API_KEY environment variable must be set");

    // Create a client
    let client = Client::new(api_key)?;

    // Create messages
    let messages = vec![
        Message {
            role: Role::System,
            content: "You are a helpful AI assistant.".to_string(),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        },
        Message {
            role: Role::User,
            content: "Hello! Can you tell me about Rust programming?".to_string(),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        }
    ];

    println!("Sending chat request...");

    // Send chat request
    let response = client.chat(Model::Grok4FastReasoning, messages, None).await?;

    println!("Response ID: {}", response.id);
    println!("Model: {}", response.model);
    println!("Content: {}", response.message.content);

    if let Some(usage) = response.usage {
        println!("Tokens used - Prompt: {}, Completion: {}, Total: {}",
                 usage.prompt_tokens, usage.completion_tokens, usage.total_tokens);
    }

    if let Some(reason) = response.finish_reason {
        println!("Finish reason: {}", reason);
    }

    Ok(())
}