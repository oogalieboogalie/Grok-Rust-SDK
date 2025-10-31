//! Example demonstrating streaming chat completions

use futures::StreamExt;
use grok_rust_sdk::{chat::Message, Client};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create client
    let client = Client::new("your-api-key-here")?;

    // Create messages
    let messages = vec![Message::user(
        "Tell me a short story about a robot learning to paint.",
    )];

    // Stream the response
    let mut stream = client
        .chat_stream(grok_rust_sdk::Model::Grok4FastReasoning, messages, None)
        .await?;

    println!("🤖 Streaming response:");
    println!("---");

    while let Some(chunk_result) = stream.next().await {
        match chunk_result {
            Ok(chunk) => {
                // Process each chunk
                for choice in &chunk.choices {
                    if let Some(content) = &choice.delta.content {
                        print!("{}", content);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                break;
            }
        }
    }

    println!("\n---");
    println!("Stream complete!");

    Ok(())
}
