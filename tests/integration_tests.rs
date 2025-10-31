//! Integration tests for the Grok Rust SDK

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use grok_rust_sdk::chat::{Message, Model, Role};
    use grok_rust_sdk::error::GrokError;
    use grok_rust_sdk::tools::{ToolExecutor, ToolRegistry, ToolSpec};
    use serde_json;

    #[derive(Debug)]
    struct MockTool;

    #[async_trait]
    impl ToolExecutor for MockTool {
        async fn execute(&self, args: serde_json::Value) -> Result<serde_json::Value, GrokError> {
            Ok(serde_json::json!({"result": "mock_response", "input": args}))
        }

        fn spec(&self) -> ToolSpec {
            ToolSpec {
                name: "mock_tool".to_string(),
                description: "A mock tool for testing".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "input": {"type": "string"}
                    },
                    "required": ["input"]
                }),
            }
        }
    }

    #[tokio::test]
    async fn test_tool_registry() {
        let mut registry = ToolRegistry::new();
        registry.register(MockTool);

        let tools = registry.api_tools();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].function.name, "mock_tool");
    }

    #[tokio::test]
    async fn test_tool_execution() {
        let mut registry = ToolRegistry::new();
        registry.register(MockTool);

        let tool_call = grok_rust_sdk::tools::ToolCall {
            id: "test-call-123".to_string(),
            function: grok_rust_sdk::tools::ToolFunction {
                name: "mock_tool".to_string(),
                arguments: r#"{"input": "test_value"}"#.to_string(),
            },
        };

        let result = registry.execute_tool_call(&tool_call).await.unwrap();
        assert_eq!(result.tool_call_id, "test-call-123");

        let parsed: serde_json::Value = serde_json::from_str(&result.content).unwrap();
        assert_eq!(parsed["result"], "mock_response");
        assert_eq!(parsed["input"]["input"], "test_value");
    }

    #[test]
    fn test_model_strings() {
        assert_eq!(Model::Grok4FastReasoning.as_str(), "grok-4-fast-reasoning");
        assert_eq!(Model::Grok4.as_str(), "grok-4");
        assert_eq!(Model::Grok3.as_str(), "grok-3");
        assert_eq!(Model::Grok2.as_str(), "grok-2");
        assert_eq!(Model::Grok1.as_str(), "grok-1");
    }

    #[test]
    fn test_message_creation() {
        let message = Message {
            role: Role::User,
            content: "Hello, world!".to_string(),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        };

        assert_eq!(message.role, Role::User);
        assert_eq!(message.content, "Hello, world!");
    }

    // Note: Integration tests with actual API calls would require XAI_API_KEY
    // and are not included here to avoid requiring API keys for basic testing

    // ============================================================================
    // Error Path Tests
    // ============================================================================

    #[test]
    fn test_api_key_validation_empty() {
        use grok_rust_sdk::Client;
        let result = Client::new("");
        assert!(matches!(result, Err(GrokError::InvalidApiKey(_))));
    }

    #[test]
    fn test_api_key_validation_too_short() {
        use grok_rust_sdk::Client;
        let result = Client::new("short");
        assert!(matches!(result, Err(GrokError::InvalidApiKey(_))));
    }

    #[test]
    fn test_api_key_validation_placeholder() {
        use grok_rust_sdk::Client;
        let result = Client::new("your-api-key");
        assert!(matches!(result, Err(GrokError::InvalidApiKey(_))));

        let result = Client::new("your-xai-api-key");
        assert!(matches!(result, Err(GrokError::InvalidApiKey(_))));

        let result = Client::new("replace-me");
        assert!(matches!(result, Err(GrokError::InvalidApiKey(_))));
    }

    #[test]
    fn test_api_key_validation_invalid_chars() {
        use grok_rust_sdk::Client;
        let result = Client::new("invalid@key#with$special%chars");
        assert!(matches!(result, Err(GrokError::InvalidApiKey(_))));
    }

    #[test]
    fn test_api_key_validation_whitespace_trim() {
        use grok_rust_sdk::Client;
        let result = Client::new("  valid-api-key-12345  ");
        assert!(result.is_ok());
    }

    #[test]
    fn test_model_from_str() {
        use std::str::FromStr;

        assert_eq!(
            Model::from_str("grok-4-fast-reasoning").unwrap(),
            Model::Grok4FastReasoning
        );
        assert_eq!(Model::from_str("grok-4").unwrap(), Model::Grok4);
        assert_eq!(Model::from_str("grok-3").unwrap(), Model::Grok3);
        assert_eq!(Model::from_str("grok-2").unwrap(), Model::Grok2);
        assert_eq!(Model::from_str("grok-1").unwrap(), Model::Grok1);

        // Test case insensitivity
        assert_eq!(Model::from_str("GROK-4").unwrap(), Model::Grok4);

        // Test invalid model
        let result = Model::from_str("invalid-model");
        assert!(matches!(result, Err(GrokError::InvalidConfig(_))));
    }

    #[test]
    fn test_message_builder() {
        let msg = Message::user("Hello, world!");
        assert_eq!(msg.role, Role::User);
        assert_eq!(msg.content, "Hello, world!");
        assert!(msg.tool_calls.is_none());

        let sys_msg = Message::system("You are a helpful assistant");
        assert_eq!(sys_msg.role, Role::System);

        let asst_msg = Message::assistant("I'm here to help!");
        assert_eq!(asst_msg.role, Role::Assistant);

        let tool_msg = Message::tool("result", "call-123", "my_tool");
        assert_eq!(tool_msg.role, Role::Tool);
        assert_eq!(tool_msg.tool_call_id, Some("call-123".to_string()));
        assert_eq!(tool_msg.name, Some("my_tool".to_string()));
    }

    #[test]
    fn test_message_builder_pattern() {
        use grok_rust_sdk::chat::MessageBuilder;

        let msg = MessageBuilder::new()
            .role(Role::User)
            .content("Test message")
            .build()
            .unwrap();

        assert_eq!(msg.role, Role::User);
        assert_eq!(msg.content, "Test message");

        // Test missing role
        let result = MessageBuilder::new().content("Test message").build();
        assert!(matches!(result, Err(GrokError::InvalidConfig(_))));

        // Test missing content
        let result = MessageBuilder::new().role(Role::User).build();
        assert!(matches!(result, Err(GrokError::InvalidConfig(_))));
    }

    #[test]
    fn test_client_builder() {
        use grok_rust_sdk::Client;
        use std::time::Duration;

        let client = Client::builder()
            .api_key("valid-api-key-12345")
            .timeout(Duration::from_secs(30))
            .max_retries(5)
            .build();

        assert!(client.is_ok());

        // Test missing API key
        let result = Client::builder().timeout(Duration::from_secs(30)).build();
        assert!(matches!(result, Err(GrokError::InvalidConfig(_))));
    }

    #[test]
    fn test_error_display() {
        let err = GrokError::InvalidApiKey("test error".to_string());
        let display = format!("{}", err);
        assert!(display.contains("Invalid API key"));

        let err = GrokError::InvalidConfig("config error".to_string());
        let display = format!("{}", err);
        assert!(display.contains("Invalid configuration"));

        let err = GrokError::RateLimit {
            retry_after: Some(60),
        };
        let display = format!("{}", err);
        assert!(display.contains("Rate limit exceeded"));
        assert!(display.contains("60 seconds"));
    }

    // ============================================================================
    // Validation Tests
    // ============================================================================

    #[tokio::test]
    async fn test_chat_options_validation_max_tokens() {
        use grok_rust_sdk::{client::ChatOptions, Client};

        let client = Client::new("valid-api-key-12345").unwrap();
        let messages = vec![Message::user("Test")];

        // Test max_tokens = 0
        let mut options = ChatOptions::default();
        options.max_tokens = Some(0);

        let result = client
            .chat_with_options(
                Model::Grok4FastReasoning,
                messages.clone(),
                None,
                Some(options),
            )
            .await;

        assert!(matches!(result, Err(GrokError::InvalidConfig(_))));

        // Test max_tokens too large
        let mut options = ChatOptions::default();
        options.max_tokens = Some(200_000);

        let result = client
            .chat_with_options(Model::Grok4FastReasoning, messages, None, Some(options))
            .await;

        assert!(matches!(result, Err(GrokError::InvalidConfig(_))));
    }

    #[tokio::test]
    async fn test_chat_options_validation_temperature() {
        use grok_rust_sdk::{client::ChatOptions, Client};

        let client = Client::new("valid-api-key-12345").unwrap();
        let messages = vec![Message::user("Test")];

        // Test temperature out of range
        let mut options = ChatOptions::default();
        options.temperature = Some(3.0);

        let result = client
            .chat_with_options(
                Model::Grok4FastReasoning,
                messages.clone(),
                None,
                Some(options),
            )
            .await;

        assert!(matches!(result, Err(GrokError::InvalidConfig(_))));

        // Test negative temperature
        let mut options = ChatOptions::default();
        options.temperature = Some(-1.0);

        let result = client
            .chat_with_options(Model::Grok4FastReasoning, messages, None, Some(options))
            .await;

        assert!(matches!(result, Err(GrokError::InvalidConfig(_))));
    }

    #[tokio::test]
    async fn test_chat_options_validation_top_p() {
        use grok_rust_sdk::{client::ChatOptions, Client};

        let client = Client::new("valid-api-key-12345").unwrap();
        let messages = vec![Message::user("Test")];

        // Test top_p > 1.0
        let mut options = ChatOptions::default();
        options.top_p = Some(1.5);

        let result = client
            .chat_with_options(Model::Grok4FastReasoning, messages, None, Some(options))
            .await;

        assert!(matches!(result, Err(GrokError::InvalidConfig(_))));
    }

    #[tokio::test]
    async fn test_chat_options_validation_stop_sequences() {
        use grok_rust_sdk::{client::ChatOptions, Client};

        let client = Client::new("valid-api-key-12345").unwrap();
        let messages = vec![Message::user("Test")];

        // Test too many stop sequences
        let mut options = ChatOptions::default();
        options.stop = Some(vec![
            "stop1".to_string(),
            "stop2".to_string(),
            "stop3".to_string(),
            "stop4".to_string(),
            "stop5".to_string(),
        ]);

        let result = client
            .chat_with_options(Model::Grok4FastReasoning, messages, None, Some(options))
            .await;

        assert!(matches!(result, Err(GrokError::InvalidConfig(_))));
    }

    #[tokio::test]
    async fn test_empty_messages_validation() {
        use grok_rust_sdk::Client;

        let client = Client::new("valid-api-key-12345").unwrap();
        let messages = vec![];

        let result = client.chat(Model::Grok4FastReasoning, messages, None).await;

        assert!(matches!(result, Err(GrokError::InvalidConfig(_))));
    }
}
