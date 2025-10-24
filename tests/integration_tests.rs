//! Integration tests for the Grok Rust SDK

#[cfg(test)]
mod tests {
    use grok_rust_sdk::chat::{Message, Role, Model};
    use grok_rust_sdk::tools::{ToolRegistry, ToolExecutor, ToolSpec};
    use grok_rust_sdk::error::GrokError;
    use async_trait::async_trait;
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
}