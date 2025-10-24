# Contributing to Grok Rust SDK

Thank you for your interest in contributing to the Grok Rust SDK! This document provides guidelines and information for contributors.

## 🚀 Getting Started

### Prerequisites
- Rust 1.70 or later
- A valid xAI API key for testing
- Git

### Setup
1. Fork the repository
2. Clone your fork:
   ```bash
   git clone https://github.com/your-username/grok-rust-sdk.git
   cd grok-rust-sdk
   ```
3. Add the upstream remote:
   ```bash
   git remote add upstream https://github.com/oogalieboogalie/grok-rust-sdk.git
   ```
4. Create a feature branch:
   ```bash
   git checkout -b feature/your-feature-name
   ```

### Development Setup
```bash
# Install dependencies
cargo build

# Run tests
cargo test

# Run examples
cargo run --example basic_chat

# Check code formatting
cargo fmt --check

# Run clippy for linting
cargo clippy
```

## 📝 Development Guidelines

### Code Style
- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `cargo fmt` for consistent formatting
- Use `cargo clippy` for code quality checks
- Write comprehensive documentation for public APIs

### Testing
- Write unit tests for new functionality
- Add integration tests for API interactions
- Test error conditions and edge cases
- Ensure all tests pass before submitting PRs

### Error Handling
- Use the custom `GrokError` type for SDK-specific errors
- Provide meaningful error messages
- Handle all error cases gracefully
- Don't expose internal implementation details in errors

### Async Code
- Use `async fn` for all async operations
- Prefer `tokio::sync` primitives for shared state
- Use `Arc` and `RwLock` for thread-safe shared data
- Avoid blocking operations in async contexts

## 🛠️ Architecture Guidelines

### Module Structure
- Keep modules focused and single-purpose
- Use clear, descriptive names
- Export only necessary public APIs
- Document module responsibilities

### Type Design
- Use strong typing with enums for constrained values
- Implement appropriate traits (`Debug`, `Clone`, etc.)
- Use builder patterns for complex construction
- Provide sensible defaults where appropriate

### API Design
- Follow RESTful principles where applicable
- Use consistent naming conventions
- Provide both simple and advanced APIs
- Document all parameters and return values

## 📋 Pull Request Process

1. **Create a Feature Branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make Your Changes**
   - Write clean, well-documented code
   - Add tests for new functionality
   - Update documentation as needed
   - Ensure all tests pass

3. **Commit Your Changes**
   ```bash
   git add .
   git commit -m "feat: add your feature description"
   ```
   Use conventional commit format:
   - `feat:` for new features
   - `fix:` for bug fixes
   - `docs:` for documentation
   - `test:` for tests
   - `refactor:` for code refactoring

4. **Push and Create PR**
   ```bash
   git push origin feature/your-feature-name
   ```
   Then create a pull request on GitHub.

5. **PR Requirements**
   - Clear description of changes
   - Reference any related issues
   - Include screenshots/demos for UI changes
   - Ensure CI checks pass
   - Get approval from maintainers

## 🎯 Areas for Contribution

### High Priority
- **Additional tool types** (web scraping, file processing, etc.)
- **More comprehensive examples** and tutorials
- **Performance optimizations**
- **Additional model support** as xAI releases new models

### Medium Priority
- **WebAssembly support** for browser usage
- **Rate limiting and retry logic**
- **Caching layer** for responses
- **Metrics and observability**

### Future Enhancements
- **Plugin system** for custom tools
- **GUI applications** built with the SDK
- **Multi-language bindings** (Python, Node.js, etc.)
- **Integration with popular Rust frameworks**

## 🐛 Reporting Issues

When reporting bugs, please include:
- Rust version (`rustc --version`)
- SDK version
- Complete error messages
- Minimal code to reproduce the issue
- Expected vs actual behavior

## 📚 Documentation

- Update README.md for new features
- Add examples for complex use cases
- Document breaking changes clearly
- Keep API documentation up to date

## 🤝 Code of Conduct

- Be respectful and inclusive
- Focus on constructive feedback
- Help newcomers learn and contribute
- Maintain professional communication

## 📞 Getting Help

- Open an issue for bugs or feature requests
- Join the discussion in GitHub Discussions
- Check existing issues before creating new ones

Thank you for contributing to the Grok Rust SDK! 🎉