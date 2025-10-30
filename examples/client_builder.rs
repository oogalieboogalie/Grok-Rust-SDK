//! Example demonstrating the ClientBuilder pattern for advanced client configuration

use grok_rust_sdk::Client;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Basic client creation (still works)
    let basic_client = Client::new("your-api-key-here")?;

    // Advanced client creation with builder pattern
    let advanced_client = Client::builder()
        .api_key("your-api-key-here")
        .base_url("https://api.x.ai/v1") // Optional, defaults to this
        .timeout(Duration::from_secs(30)) // Custom timeout
        .user_agent("MyApp/1.0") // Custom user agent
        .request_id("req-12345") // Custom request ID for tracing
        .build()?;

    println!("Client configured with:");
    println!("- Base URL: {}", advanced_client.base_url);
    println!("- Timeout: {:?}", advanced_client.timeout);
    println!("- User Agent: {:?}", advanced_client.user_agent);
    println!("- Request ID: {:?}", advanced_client.request_id);

    // You can now use the client for chat requests
    // let response = advanced_client.chat(Model::Grok4Fast, messages, None).await?;

    Ok(())
}
