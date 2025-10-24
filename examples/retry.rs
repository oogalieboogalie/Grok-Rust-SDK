//! Example demonstrating retry logic with exponential backoff

use grok_rust_sdk::{Client, chat::Message};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create client with custom retry configuration
    let client = Client::builder()
        .api_key("your-api-key-here")
        .max_retries(5)  // Retry up to 5 times
        .retry_delay(Duration::from_millis(500))  // Start with 500ms delay
        .timeout(Duration::from_secs(10))
        .build()?;

    println!("🤖 Client configured with:");
    println!("- Max retries: {}", client.max_retries);
    println!("- Base retry delay: {:?}", client.retry_delay);
    println!("- Timeout: {:?}", client.timeout);

    // Create messages
    let messages = vec![Message::user("Hello, Grok! Tell me a short story.")];

    // This will automatically retry on rate limits or network errors
    match client.chat(grok_rust_sdk::Model::Grok4FastReasoning, messages, None).await {
        Ok(response) => {
            println!("\n✅ Success!");
            println!("Response: {}", response.message.content);
        }
        Err(e) => {
            println!("\n❌ Failed after retries:");
            println!("Error: {}", e);
        }
    }

    Ok(())
}