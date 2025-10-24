# Grok Rust SDK 🦀⚡

[![Crates.io](https://img.shields.io/crates/v/grok-rust-sdk.svg)](https://crates.io/crates/grok-rust-sdk)
[![Documentation](https://docs.rs/grok-rust-sdk/badge.svg)](https://docs.rs/grok-rust-sdk)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**The most comprehensive Rust SDK for xAI's Grok API** - Built for developers who want to harness the power of Grok in their Rust applications.

> 🚀 **Early Access**: One of the first complete Rust implementations for Grok's advanced features including tool calling, stateful sessions, and collections.

## ✨ Features

- **🔥 Full Chat API** - Support for all Grok models (grok-4-fast-reasoning, grok-4, grok-3, grok-2, grok-1)
- **🛠️ Tool Calling** - Function calling with automatic execution and result handling
- **💬 Stateful Sessions** - Persistent conversations with context preservation
- **📁 Collections** - Organize and manage conversation groups
- **⚡ Streaming** - Real-time streaming responses
- **🔒 Type Safe** - Comprehensive error handling and async/await support
- **📚 Well Documented** - Extensive examples and documentation

## 🚀 Quick Start

```rust
use grok_rust_sdk::{Client, chat::{Message, Role, Model}};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize client
    let client = Client::new("your-xai-api-key")?;

    // Create a conversation
    let messages = vec![
        Message {
            role: Role::User,
            content: "Hello, Grok! What's the meaning of life?".to_string(),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        }
    ];

    // Get response
    let response = client.chat(Model::Grok4FastReasoning, messages, None).await?;
    println!("🤖 Grok: {}", response.message.content);

    Ok(())
}
```

## 🛠️ Tool Calling Example

```rust
use grok_rust_sdk::tools::{ToolRegistry, ToolExecutor, ToolSpec};
use async_trait::async_trait;

#[derive(Debug)]
struct WeatherTool;

#[async_trait]
impl ToolExecutor for WeatherTool {
    async fn execute(&self, args: serde_json::Value) -> Result<serde_json::Value, grok_rust_sdk::GrokError> {
        let city = args["city"].as_str().unwrap_or("unknown");
        Ok(serde_json::json!({
            "city": city,
            "temperature": 72,
            "condition": "sunny",
            "humidity": 45
        }))
    }

    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: "get_weather".to_string(),
            description: "Get current weather for a city".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "city": {"type": "string", "description": "City name"}
                },
                "required": ["city"]
            }),
        }
    }
}

// Register and use tools
let mut registry = ToolRegistry::new();
registry.register(WeatherTool);
let tools = registry.api_tools();

let response = client.chat(Model::Grok4FastReasoning, messages, Some(tools)).await?;
```

## 💬 Stateful Sessions

```rust
// Create a session manager
let session_mgr = client.session_manager();

// Start a conversation session
let session = session_mgr.create_session(Model::Grok4FastReasoning, Some("Weather Chat")).await?;

// Chat with persistent context
let response1 = session.chat("What's the weather in Tokyo?").await?;
let response2 = session.chat("What about the humidity there?").await?; // Context preserved!

println!("Session has {} messages", session.message_count().await);
```

## 📁 Collections

```rust
// Organize conversations into collections
let collection_mgr = client.collection_manager(session_mgr);
let tech_collection = collection_mgr.create_collection(
    "Tech Discussions",
    Some("Conversations about technology and programming"),
    vec!["tech", "programming"]
).await?;

// Add sessions to collections
collection.add_session(session).await?;

// Search and manage collections
let tech_collections = collection_mgr.search_collections("tech").await;
```

## 📦 Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
grok-rust-sdk = "0.1.0"
tokio = { version = "1.0", features = ["full"] }
serde_json = "1.0"
```

Or install from source:

```bash
cargo add grok-rust-sdk
```

## 🔧 Requirements

- Rust 1.70+
- Valid xAI API key (get one at [x.ai](https://x.ai))

## 📚 Examples

Run the included examples:

```bash
# Basic chat
cargo run --example basic_chat

# Tool calling
cargo run --example tool_calling

# Sessions
cargo run --example sessions

# Collections
cargo run --example collections
```

## 🏗️ Architecture

```
grok-rust-sdk/
├── src/
│   ├── lib.rs           # Main exports
│   ├── client.rs        # HTTP client & API calls
│   ├── chat.rs          # Chat completions
│   ├── tools.rs         # Tool calling system
│   ├── session.rs       # Stateful sessions
│   ├── collections.rs   # Collection management
│   └── error.rs         # Error handling
├── examples/            # Usage examples
└── tests/              # Integration tests
```

## 🤝 Contributing

Contributions welcome! This SDK is built for the community:

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Submit a pull request

## 📄 License

Licensed under MIT OR Apache-2.0 - see [LICENSE](LICENSE) for details.

## 🙏 Acknowledgments

- Built for the xAI developer community
- Inspired by the power of Grok and the elegance of Rust
- Thanks to xAI for their amazing API

---

**Made with ❤️ by [@oogalieboogalie](https://twitter.com/oogalieboogalie)**

*Early access to cutting-edge AI in Rust* 🚀

```rust
use grok_rust_sdk::tools::{ToolRegistry, ToolExecutor, Tool};
use grok_rust_sdk::tool_params;
use async_trait::async_trait;

#[derive(Debug)]
struct WeatherTool;

#[async_trait]
impl ToolExecutor for WeatherTool {
    async fn execute(&self, args: serde_json::Value) -> Result<serde_json::Value, grok_rust_sdk::GrokError> {
        let location = args["location"].as_str().unwrap_or("unknown");
        Ok(serde_json::json!({
            "temperature": 72,
            "condition": "sunny",
            "location": location
        }))
    }

    fn spec(&self) -> grok_rust_sdk::tools::ToolSpec {
        grok_rust_sdk::tools::ToolSpec {
            name: "get_weather".to_string(),
            description: "Get current weather for a location".to_string(),
            parameters: tool_params!({
                "location": grok_rust_sdk::param!(string, "The city or location to get weather for")
            }),
        }
    }
}

// Register and use tools
let mut registry = ToolRegistry::new();
registry.register(WeatherTool);

let tools = registry.api_tools();
let response = client.chat(Model::Grok4FastReasoning, messages, Some(tools)).await?;

// Execute any tool calls
if let Some(tool_calls) = &response.message.tool_calls {
    for tool_call in tool_calls {
        let result = registry.execute_tool_call(tool_call).await?;
        println!("Tool result: {}", result.content);
    }
}
```

## Stateful Sessions

```rust
use grok_rust_sdk::session::SessionManager;

// Create a session manager
let session_mgr = client.session_manager();

// Create a new session
let session = session_mgr.create_session(Model::Grok4FastReasoning, Some("Weather Chat".to_string())).await;

// Chat with context maintained
let response1 = session.chat("What's the weather in New York?").await?;
let response2 = session.chat("What about London?").await?; // Context preserved

println!("Session has {} messages", session.message_count().await);
```

## Collections

```rust
use grok_rust_sdk::collections::CollectionManager;

// Create a collection manager
let collection_mgr = client.collection_manager(session_mgr);

// Create a collection
let collection = collection_mgr.create_collection(
    "Weather Discussions",
    Some("Conversations about weather patterns"),
    vec!["weather".to_string(), "climate".to_string()]
).await;

// Add sessions to collection
collection.add_session(session).await?;

// Search collections
let weather_collections = collection_mgr.search_collections("weather").await;
```

## Streaming

```rust
use futures::StreamExt;

let mut stream = client.chat_stream(Model::Grok4FastReasoning, messages, None).await?;

while let Some(chunk) = stream.next().await {
    match chunk {
        Ok(chunk) => {
            for choice in &chunk.choices {
                if let Some(content) = &choice.delta.content {
                    print!("{}", content);
                }
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

## Models

- `Model::Grok4FastReasoning` - Fast reasoning model (recommended for most use cases)
- `Model::Grok4` - Standard Grok-4 model
- `Model::Grok3` - Legacy Grok-3 model
- `Model::Grok2` - Legacy Grok-2 model
- `Model::Grok1` - Legacy Grok-1 model

## Error Handling

The SDK uses a custom `GrokError` type with specific variants:

- `GrokError::Http` - HTTP request failures
- `GrokError::Json` - Serialization errors
- `GrokError::Api` - API error responses
- `GrokError::InvalidConfig` - Configuration issues
- `GrokError::Authentication` - Auth failures
- `GrokError::RateLimit` - Rate limiting
- `GrokError::ToolExecution` - Tool execution failures
- `GrokError::Session` - Session management errors
- `GrokError::Collection` - Collection management errors

## Advanced Usage

### Custom Tool Parameters

```rust
use grok_rust_sdk::tool_params;
use grok_rust_sdk::param;

// Complex parameter schema
let params = tool_params!({
    "query": param!(string, "Search query"),
    "limit": param!(number, "Maximum results"),
    "include_metadata": param!(boolean, "Include metadata in results"),
    "tags": param!(array, param!(string, "Tag name"), "List of tags to filter by")
});
```

### Session Management

```rust
// List all sessions
let sessions = session_mgr.list_sessions().await;

// Get session by ID
if let Some(session) = session_mgr.get_session("session-id").await {
    // Use session
}

// Delete old sessions
session_mgr.delete_session("old-session-id").await?;
```

### Collection Organization

```rust
// Get collections by tag
let ai_collections = collection_mgr.collections_by_tag("ai").await;

// Get collection statistics
let stats = collection_mgr.stats().await;
println!("Total collections: {}", stats.total_collections);
```

## Contributing

This SDK is one of the first comprehensive Rust implementations for the Grok API. Contributions are welcome!

### Building from Source

```bash
git clone https://github.com/your-repo/grok-rust-sdk.git
cd grok-rust-sdk
cargo build
cargo test
```

### Running Tests

```bash
# Set your API key
export XAI_API_KEY=your-key-here

# Run tests
cargo test
```

## License

MIT OR Apache-2.0