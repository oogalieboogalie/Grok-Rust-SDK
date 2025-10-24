//! Main client for interacting with the Grok API

use crate::chat::{ChatCompletion, ChatRequest, ChatResponse, Message, Model, Tool, ChatChunk};
use crate::collections::CollectionManager;
use crate::error::{GrokError, Result};
use crate::session::SessionManager;
use reqwest::{Client as HttpClient, Response};
use serde::de::DeserializeOwned;
use std::sync::Arc;
use std::time::Duration;

/// Main client for the Grok API
#[derive(Debug)]
pub struct Client {
    http_client: HttpClient,
    api_key: String,
    base_url: String,
    timeout: Option<Duration>,
    user_agent: Option<String>,
    request_id: Option<String>,
    max_retries: u32,
    retry_delay: Duration,
}

impl Client {
    /// Create a new client with an API key
    pub fn new(api_key: impl Into<String>) -> Result<Self> {
        Ok(Self {
            http_client: HttpClient::new(),
            api_key: api_key.into(),
            base_url: "https://api.x.ai/v1".to_string(),
            timeout: None,
            user_agent: None,
            request_id: None,
            max_retries: 3,
            retry_delay: Duration::from_millis(1000),
        })
    }

    /// Create a new client with custom configuration
    pub fn with_config(api_key: impl Into<String>, base_url: impl Into<String>) -> Result<Self> {
        Ok(Self {
            http_client: HttpClient::new(),
            api_key: api_key.into(),
            base_url: base_url.into(),
            timeout: None,
            user_agent: None,
            request_id: None,
            max_retries: 3,
            retry_delay: Duration::from_millis(1000),
        })
    }

    /// Create a builder for advanced configuration
    pub fn builder() -> ClientBuilder {
        ClientBuilder::new()
    }

    /// Create a session manager for this client
    pub fn session_manager(&self) -> Arc<SessionManager> {
        Arc::new(SessionManager::new(Arc::new(self.clone())))
    }

    /// Create a collection manager for this client
    pub fn collection_manager(
        &self,
        session_manager: Arc<SessionManager>,
    ) -> Arc<CollectionManager> {
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

        let choice = response
            .choices
            .into_iter()
            .next()
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
    ) -> Result<impl futures::Stream<Item = Result<ChatChunk>>> {
        use futures::StreamExt;

        let request = ChatRequest {
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

        let mut request_builder = self
            .http_client
            .post(&format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json");

        if let Some(ref request_id) = self.request_id {
            request_builder = request_builder.header("X-Request-ID", request_id);
        }

        let response = request_builder.json(&request).send().await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            return Err(GrokError::Api { status, message });
        }

        // Collect all response data
        let body_bytes = response.bytes().await.map_err(GrokError::Http)?;
        let body_text = String::from_utf8_lossy(&body_bytes);

        // Parse SSE format and collect chunks
        let mut chunks = Vec::new();
        for line in body_text.lines() {
            if line.starts_with("data: ") {
                let data = &line[6..];
                if data == "[DONE]" {
                    break;
                }
                if let Ok(chunk) = serde_json::from_str::<ChatChunk>(data) {
                    chunks.push(chunk);
                }
            }
        }

        // Convert to stream
        let stream = futures::stream::iter(chunks.into_iter().map(Ok));

        Ok(stream)
    }

    /// Make a POST request to the API
    async fn post<T: serde::Serialize, R: DeserializeOwned>(
        &self,
        endpoint: &str,
        body: &T,
    ) -> Result<R> {
        use backon::ExponentialBuilder;
        use backon::Retryable;

        let url = format!("{}{}", self.base_url, endpoint);

        let operation = || async {
            let mut request = self
                .http_client
                .post(&url)
                .header("Authorization", format!("Bearer {}", self.api_key))
                .header("Content-Type", "application/json");

            if let Some(ref request_id) = self.request_id {
                request = request.header("X-Request-ID", request_id);
            }

            let response = request.json(body).send().await?;
            self.handle_response(response).await
        };

        // Retry on 429 (rate limit) and 5xx errors
        let backoff = ExponentialBuilder::default()
            .with_min_delay(self.retry_delay)
            .with_max_delay(Duration::from_secs(60))
            .with_max_times(self.max_retries);

        operation
            .retry(backoff)
            .when(|e: &GrokError| match e {
                GrokError::Api { status, .. } => *status == 429 || *status >= 500,
                GrokError::Http(_) => true, // Retry on network errors
                _ => false,
            })
            .await
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
            timeout: self.timeout,
            user_agent: self.user_agent.clone(),
            request_id: self.request_id.clone(),
            max_retries: self.max_retries,
            retry_delay: self.retry_delay,
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

/// Builder for creating a Client with custom configuration
#[derive(Debug, Clone)]
pub struct ClientBuilder {
    api_key: Option<String>,
    base_url: Option<String>,
    timeout: Option<Duration>,
    user_agent: Option<String>,
    request_id: Option<String>,
    max_retries: Option<u32>,
    retry_delay: Option<Duration>,
}

impl ClientBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            api_key: None,
            base_url: None,
            timeout: None,
            user_agent: None,
            request_id: None,
            max_retries: None,
            retry_delay: None,
        }
    }

    /// Set the API key
    pub fn api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    /// Set the base URL
    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = Some(base_url.into());
        self
    }

    /// Set the request timeout
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Set the user agent
    pub fn user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = Some(user_agent.into());
        self
    }

    /// Set a custom request ID
    pub fn request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = Some(request_id.into());
        self
    }

    /// Set the maximum number of retries for failed requests
    pub fn max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = Some(max_retries);
        self
    }

    /// Set the base delay between retries
    pub fn retry_delay(mut self, retry_delay: Duration) -> Self {
        self.retry_delay = Some(retry_delay);
        self
    }

    /// Build the client
    pub fn build(self) -> Result<Client> {
        let api_key = self.api_key.ok_or_else(|| GrokError::InvalidConfig("API key is required".to_string()))?;
        let base_url = self.base_url.unwrap_or_else(|| "https://api.x.ai/v1".to_string());

        let mut http_client_builder = HttpClient::builder();

        if let Some(timeout) = self.timeout {
            http_client_builder = http_client_builder.timeout(timeout);
        }

        if let Some(user_agent) = self.user_agent {
            http_client_builder = http_client_builder.user_agent(user_agent);
        }

        let http_client = http_client_builder.build().map_err(GrokError::Http)?;

        Ok(Client {
            http_client,
            api_key,
            base_url,
            timeout: self.timeout,
            user_agent: self.user_agent,
            request_id: self.request_id,
            max_retries: self.max_retries.unwrap_or(3),
            retry_delay: self.retry_delay.unwrap_or(Duration::from_millis(1000)),
        })
    }
}
