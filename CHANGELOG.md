# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2025-10-24

### Added
- **Initial release** of the Grok Rust SDK
- **Complete chat API** support for all Grok models (grok-4-fast-reasoning, grok-4, grok-3, grok-2, grok-1)
- **Tool calling system** with automatic function execution and result handling
- **Stateful sessions** for persistent conversations with context preservation
- **Collections API** for organizing and managing conversation groups
- **Streaming responses** for real-time interactions
- **Comprehensive error handling** with custom `GrokError` types
- **Async/await support** throughout using Tokio
- **Type-safe APIs** with strong typing for all operations
- **Extensive documentation** and working examples
- **Integration tests** for core functionality

### Features
- Tool registry with async trait-based execution
- Session management with automatic metadata tracking
- Collection organization with search and tagging
- Helper macros for easy parameter schema creation
- Production-ready error handling and logging

### Examples
- `basic_chat.rs` - Simple chat completion example
- `tool_calling.rs` - Function calling with custom tools
- `sessions.rs` - Stateful conversation management
- `collections.rs` - Collection organization and search

### Technical Details
- Built with Rust 2021 edition
- Uses reqwest for HTTP client, serde for serialization
- Comprehensive async architecture with proper error propagation
- Modular design allowing easy extension and customization