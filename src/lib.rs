//! # Grok Rust SDK
//!
//! A comprehensive Rust SDK for xAI's Grok API, supporting:
//! - Chat completions with multiple models
//! - Tool calling and function execution
//! - Stateful conversation sessions
//! - Collections for organizing conversations
//! - Streaming responses
//!
//! ## Example
//!
//! ```rust,no_run
//! use grok_rust_sdk::{Client, chat::{Message, Role}};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = Client::new("your-api-key")?;
//!
//!     let messages = vec![
//!         Message {
//!             role: Role::User,
//!             content: "Hello, Grok!".to_string(),
//!         }
//!     ];
//!
//!     let response = client.chat("grok-4-fast-reasoning", messages, None).await?;
//!     println!("Response: {}", response.content);
//!
//!     Ok(())
//! }
//! ```

pub mod chat;
pub mod client;
pub mod collections;
pub mod error;
pub mod persistence;
pub mod session;
pub mod tools;

pub use client::Client;
pub use error::{GrokError, Result};
