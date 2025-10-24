# Examples

This directory contains runnable examples demonstrating the Grok Rust SDK features.

## Prerequisites

- Rust 1.70+
- xAI API key (set as `XAI_API_KEY` environment variable)

## Running Examples

Set your API key:
```bash
export XAI_API_KEY=your-api-key-here
```

Run an example:
```bash
cargo run --example basic_chat
cargo run --example tool_calling
cargo run --example sessions
cargo run --example collections
```

## Examples Overview

### `basic_chat.rs`
Basic chat completion with a single message.

### `tool_calling.rs`
Demonstrates function calling with calculator and web search tools.

### `sessions.rs`
Shows session management for multi-turn conversations.

### `collections.rs`
Organizing conversations into searchable collections.