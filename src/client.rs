//! Main client for interacting with the Grok API

use crate::chat::{ChatCompletion, ChatRequest, ChatResponse, Message, Model, Tool};
use crate::error::{GrokError, Result};
use crate::session::SessionManager;
use crate::collections::CollectionManager;
use reqwest::{Client as HttpClient, Response};
use serde::de::DeserializeOwned;
use std::sync::Arc;

/// Main client for the Grok API
#[derive(Debug)]
pub struct Client {
    http_client: HttpClient,
    api_key: String,
    base_url: String,
}

impl Client {
    /// Create a new client with an API key
    pub fn new(api_key: impl Into<String>) -> Result<Self> {
        Ok(Self {
            http_client: HttpClient::new(),
            api_key: api_key.into(),
            base_url: "https://api.x.ai/v1".to_string(),
        })
    }

    /// Create a new client with custom configuration
    pub fn with_config(api_key: impl Into<String>, base_url: impl Into<String>) -> Result<Self> {
        Ok(Self {
            http_client: HttpClient::new(),
            api_key: api_key.into(),
            base_url: base_url.into(),
        })
    }

    /// Create a session manager for this client
    pub fn session_manager(&self) -> Arc<SessionManager> {
        Arc::new(SessionManager::new(Arc::new(self.clone())))
    }

    /// Create a collection manager for this client
    pub fn collection_manager(&self, session_manager: Arc<SessionManager>) -> Arc<CollectionManager> {
        Arc::new(CollectionManager::new(session_manager))
    }

    /// Send a chat completion request
    pub async fn chat(
        &self,
        model: Model,
        messages: Vec<Message>,
        tools: Option<Vec<Tool>>,
    ) -> Result<ChatCompletion> {
        self.chat_with_options(model, messages, tools, None).await
    }

    /// Send a chat completion request with full options
    pub async fn chat_with_options(
        &self,
        model: Model,
        messages: Vec<Message>,
        tools: Option<Vec<Tool>>,
        options: Option<ChatOptions>,
    ) -> Result<ChatCompletion> {
        let request = ChatRequest {
            model: model.as_str().to_string(),
            messages,
            max_tokens: options.as_ref().and_then(|o| o.max_tokens),
            temperature: options.as_ref().and_then(|o| o.temperature),
            top_p: options.as_ref().and_then(|o| o.top_p),
            tools,
            tool_choice: options.as_ref().and_then(|o| o.tool_choice.clone()),
            response_format: options.as_ref().and_then(|o| o.response_format.clone()),
            stop: options.as_ref().and_then(|o| o.stop.clone()),
            stream: options.as_ref().and_then(|o| o.stream),
        };

        let response: ChatResponse = self.post("/chat/completions", &request).await?;

        let choice = response.choices.into_iter().next()
            .ok_or_else(|| GrokError::Api {
                status: 500,
                message: "No choices returned".to_string(),
            })?;

        Ok(ChatCompletion {
            id: response.id,
            model: response.model,
            usage: response.usage,
            message: choice.message,
            finish_reason: choice.finish_reason,
        })
    }

    /// Stream a chat completion
    pub async fn chat_stream(
        &self,
        model: Model,
        messages: Vec<Message>,
        tools: Option<Vec<Tool>>,
    ) -> Result<impl futures::Stream<Item = Result<crate::chat::ChatChunk>>> {
        use futures::StreamExt;

        let mut request = ChatRequest {
            model: model.as_str().to_string(),
            messages,
            max_tokens: None,
            temperature: None,
            top_p: None,
            tools,
            tool_choice: None,
            response_format: None,
            stop: None,
            stream: Some(true),
        };

        let response = self.http_client
            .post(&format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            return Err(GrokError::Api { status, message });
        }

        let stream = response.bytes_stream()
            .map(|result| match result {
                Ok(bytes) => {
                    // Parse SSE format
                    let text = String::from_utf8_lossy(&bytes);
                    let lines: Vec<&str> = text.lines().collect();

                    for line in lines {
                        if line.starts_with("data: ") {
                            let data = &line[6..];
                            if data == "[DONE]" {
                                return Ok(None);
                            }

                            match serde_json::from_str::<crate::chat::ChatChunk>(data) {
                                Ok(chunk) => return Ok(Some(chunk)),
                                Err(e) => return Err(GrokError::Json(e)),
                            }
                        }
                    }

                    Ok(None)
                }
                Err(e) => Err(GrokError::Http(e)),
            })
            .filter_map(|result| async move {
                match result {
                    Ok(Some(chunk)) => Some(Ok(chunk)),
                    Ok(None) => None,
                    Err(e) => Some(Err(e)),
                }
            });

        Ok(stream)
    }

    /// Make a POST request to the API
    async fn post<T: serde::Serialize, R: DeserializeOwned>(&self, endpoint: &str, body: &T) -> Result<R> {
        let url = format!("{}{}", self.base_url, endpoint);
        let response = self.http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Handle API response
    async fn handle_response<R: DeserializeOwned>(&self, response: Response) -> Result<R> {
        if response.status().is_success() {
            response.json().await.map_err(GrokError::from)
        } else {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            Err(GrokError::Api { status, message })
        }
    }
}

impl Clone for Client {
    fn clone(&self) -> Self {
        Self {
            http_client: HttpClient::new(),
            api_key: self.api_key.clone(),
            base_url: self.base_url.clone(),
        }
    }
}

/// Options for chat completion requests
#[derive(Debug, Clone, Default)]
pub struct ChatOptions {
    /// Maximum tokens to generate
    pub max_tokens: Option<u32>,
    /// Temperature for randomness (0.0 to 2.0)
    pub temperature: Option<f32>,
    /// Top-p sampling parameter
    pub top_p: Option<f32>,
    /// Tool choice strategy
    pub tool_choice: Option<serde_json::Value>,
    /// Response format specification
    pub response_format: Option<serde_json::Value>,
    /// Stop sequences
    pub stop: Option<Vec<String>>,
    /// Enable streaming responses
    pub stream: Option<bool>,
}