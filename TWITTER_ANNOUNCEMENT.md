# Humble Twitter Announcement

## 🚀 Main Tweet
```
Just published a Rust SDK for xAI's Grok API! 🦀

Features:
• Basic chat with all Grok models
• Tool calling support
• Session management
• Async Rust implementation

Nothing fancy, but it works! Hope it helps fellow Rustaceans integrate with Grok.

#Rust #Grok #xAI

GitHub: https://github.com/oogalieboogalie/Grok-Rust-SDK
```

## 🧵 Simple Thread

### Tweet 2
```
Quick example:

```rust
let client = Client::new("your-api-key")?;
let response = client.chat(
    Model::Grok4FastReasoning,
    vec![Message::user("Hello, Grok!")],
    None
).await?;
```

Simple async API for Grok integration.
```

### Tweet 3
```
Tool calling works too:

```rust
#[async_trait]
impl ToolExecutor for MyTool {
    async fn execute(&self, args: Value) -> Result<Value> {
        // Your tool logic
        Ok(json!({"result": "done"}))
    }
}
```

Grok can call your Rust functions!
```

### Tweet 4
```
Install with: cargo add grok-rust-sdk

Still early days, but functional. Contributions welcome!

#OpenSource #RustLang
```