//! # Grok Rust SDK
//!
//! A comprehensive, production-ready Rust SDK for xAI's Grok API.
//!
//! ## Features
//!
//! - **Chat Completions**: Support for all Grok models (Grok-4, Grok-3, etc.)
//! - **Tool Calling**: Async function execution with JSON schema validation
//! - **Sessions**: Stateful conversation management with history
//! - **Collections**: Organize and search conversation groups
//! - **Streaming**: Real-time response streaming with proper memory management
//! - **Persistence**: SQLite storage for sessions and collections
//! - **Retry Logic**: Exponential backoff for rate limits and network errors
//! - **Validation**: Comprehensive input validation for security and correctness
//! - **Type Safety**: Strong typing throughout with builder patterns
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use grok_rust_sdk::{Client, Model, chat::Message};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a client with API key validation
//!     let client = Client::new("your-xai-api-key")?;
//!
//!     // Create messages using convenient builders
//!     let messages = vec![
//!         Message::system("You are a helpful assistant"),
//!         Message::user("What is the capital of France?"),
//!     ];
//!
//!     // Make a chat request
//!     let response = client.chat(
//!         Model::Grok4FastReasoning,
//!         messages,
//!         None  // No tools
//!     ).await?;
//!
//!     println!("🤖 {}", response.message.content);
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Advanced Usage
//!
//! ### Streaming Responses
//!
//! ```rust,no_run
//! use grok_rust_sdk::{Client, Model, chat::Message};
//! use futures::StreamExt;
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let client = Client::new("your-api-key")?;
//! let messages = vec![Message::user("Write a haiku")];
//!
//! let mut stream = client.chat_stream(
//!     Model::Grok4FastReasoning,
//!     messages,
//!     None
//! ).await?;
//!
//! while let Some(chunk) = stream.next().await {
//!     let chunk = chunk?;
//!     // Process chunk as it arrives
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ### Custom Configuration
//!
//! ```rust,no_run
//! use grok_rust_sdk::Client;
//! use std::time::Duration;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let client = Client::builder()
//!     .api_key("your-api-key")
//!     .timeout(Duration::from_secs(30))
//!     .max_retries(5)
//!     .retry_delay(Duration::from_millis(500))
//!     .user_agent("MyApp/1.0")
//!     .build()?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Security
//!
//! - API keys are validated and sanitized on client creation
//! - All input parameters are validated before API calls
//! - No sensitive data is logged or exposed
//! - Uses rustls for secure TLS connections
//!
//! ## Error Handling
//!
//! All errors implement `std::error::Error` and use the [`GrokError`] enum:
//!
//! ```rust
//! use grok_rust_sdk::{Client, GrokError};
//!
//! # async fn example() {
//! match Client::new("invalid") {
//!     Ok(client) => { /* ... */ }
//!     Err(GrokError::InvalidApiKey(msg)) => {
//!         eprintln!("Invalid API key: {}", msg);
//!     }
//!     Err(e) => {
//!         eprintln!("Other error: {}", e);
//!     }
//! }
//! # }
//! ```

pub mod chat;
pub mod client;
pub mod collections;
pub mod error;
pub mod persistence;
pub mod session;
pub mod tools;

// Re-export main types for convenience
pub use chat::Model;
pub use client::Client;
pub use error::{GrokError, Result};
