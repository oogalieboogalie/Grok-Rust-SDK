# Grok Rust SDK 🦀

[![Crates.io](https://img.shields.io/crates/v/grok-rust-sdk.svg)](https://crates.io/crates/grok-rust-sdk)
[![Documentation](https://docs.rs/grok-rust-sdk/badge.svg)](https://docs.rs/grok-rust-sdk)
[![CI](https://github.com/oogalieboogalie/Grok-Rust-SDK/actions/workflows/ci.yml/badge.svg)](https://github.com/oogalieboogalie/Grok-Rust-SDK/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A Rust SDK for xAI's Grok API with chat completions, tool calling, and session management.

**Made by Grok For Grok! "GROK ON!"** 🚀🤖

## ✨ Features

- **Chat API** - Support for all Grok models
- **Tool Calling** - Function calling with execution
- **Sessions** - Persistent conversations
- **Collections** - Organize conversation groups
- **Streaming** - Real-time response streaming
- **Persistence** - SQLite storage for sessions and collections
- **Validation** - JSON Schema validation for tool arguments
- **Retry Logic** - Exponential backoff for rate limits and errors
- **Async/Await** - Built with tokio
- **Type Safe** - Comprehensive error handling

## 🚀 Quick Start

Install:

```bash
cargo add grok-rust-sdk
```

Basic usage:

```rust
use grok_rust_sdk::{Client, chat::Message};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set your API key
    let client = Client::new("your-xai-api-key")?;

    // Create messages
    let messages = vec![Message::user("Hello, Grok!")];

    // Chat
    let response = client.chat(grok_rust_sdk::Model::Grok4FastReasoning, messages, None).await?;

    println!("🤖 {}", response.message.content);
    Ok(())
}
```

## ⚙️ Advanced Configuration

Use the builder pattern for custom timeouts, user agents, and request IDs:

```rust
use grok_rust_sdk::Client;
use std::time::Duration;

let client = Client::builder()
    .api_key("your-xai-api-key")
    .timeout(Duration::from_secs(30))
    .user_agent("MyApp/1.0")
    .request_id("req-12345")
    .build()?;
```

## 🛠️ Tool Calling

```rust
use grok_rust_sdk::tools::{ToolRegistry, ToolExecutor};
use async_trait::async_trait;
use serde_json::Value;

#[derive(Debug)]
struct Calculator;

#[async_trait]
impl ToolExecutor for Calculator {
    async fn execute(&self, args: Value) -> Result<Value, grok_rust_sdk::GrokError> {
        let expr = args["expression"].as_str().unwrap_or("0");
        // Simple eval for demo (use a proper math library in production)
        let result = meval::eval_str(expr).unwrap_or(0.0);
        Ok(serde_json::json!({ "result": result }))
    }
}

// Register tool
let mut registry = ToolRegistry::new();
registry.register(Calculator);

// Use in chat
let tools = registry.api_tools();
let response = client.chat(grok_rust_sdk::Model::Grok4FastReasoning, messages, Some(tools)).await?;
```

## 💬 Sessions

```rust
// Create session manager
let session_mgr = client.session_manager();

// Start session
let session = session_mgr.create_session(grok_rust_sdk::Model::Grok4FastReasoning).await?;

// Chat with context
session.chat("What's 2+2?").await?;
session.chat("Now multiply by 3").await?; // Context preserved
```

## 💾 Persistence

Store sessions and collections in SQLite for long-term persistence:

```rust
use grok_rust_sdk::persistence::SqliteStorage;

// Create persistent storage
let storage = SqliteStorage::new("chat.db")?;

// Save sessions and collections
storage.save_session(&session).await?;
storage.save_collection(&collection).await?;

// Load them back later
if let Some(session) = storage.load_session("session-id").await? {
    // Continue the conversation
}
```

## � Retry Logic

Automatic retry with exponential backoff for rate limits and network errors:

```rust
let client = Client::builder()
    .api_key("your-api-key")
    .max_retries(5)  // Retry up to 5 times on 429/5xx errors
    .retry_delay(Duration::from_millis(500))  // Start with 500ms delay
    .build()?;

// Automatically handles rate limits and retries
let response = client.chat(model, messages, tools).await?;
```

## 📦 Installation

```toml
[dependencies]
grok-rust-sdk = "0.1.0"
tokio = { version = "1.0", features = ["full"] }
```

## 🔧 Requirements

- Rust 1.70+
- xAI API key from [x.ai](https://x.ai)

## 📚 Examples

See the `examples/` directory for runnable demos.

## 🤝 Contributing

Contributions welcome! Please add tests for new features.

## 📄 License

MIT OR Apache-2.0