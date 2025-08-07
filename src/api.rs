//! OpenAI API client implementation

use crate::config::Config;
use crate::error::{AppError, Result};
use futures_util::{Stream, StreamExt};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::time::Duration;
use tracing::{debug, instrument};

/// Role in a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
}

/// A message in the conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

impl Message {
    /// Create a new system message
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: Role::System,
            content: content.into(),
        }
    }

    /// Create a new user message
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: Role::User,
            content: content.into(),
        }
    }

    /// Create a new assistant message
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: Role::Assistant,
            content: content.into(),
        }
    }
}

/// OpenAI API request
#[derive(Debug, Serialize)]
struct CompletionRequest {
    model: String,
    messages: Vec<Message>,
    max_tokens: u32,
    temperature: f32,
    stream: bool,
}

/// OpenAI API response choice
#[derive(Debug, Deserialize)]
struct Choice {
    message: Message,
    finish_reason: Option<String>,
    #[allow(dead_code)]
    index: usize,
}

/// OpenAI API response
#[derive(Debug, Deserialize)]
struct CompletionResponse {
    #[allow(dead_code)]
    id: String,
    #[allow(dead_code)]
    object: String,
    #[allow(dead_code)]
    created: u64,
    #[allow(dead_code)]
    model: String,
    choices: Vec<Choice>,
    #[allow(dead_code)]
    usage: Option<Usage>,
}

/// Token usage information
#[derive(Debug, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Streaming response chunk from OpenAI API
#[derive(Debug, Serialize, Deserialize)]
pub struct StreamChunk {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<StreamChoice>,
}

/// Choice in a streaming response
#[derive(Debug, Serialize, Deserialize)]
pub struct StreamChoice {
    pub index: usize,
    pub delta: Delta,
    pub finish_reason: Option<String>,
}

/// Delta content in streaming response
#[derive(Debug, Serialize, Deserialize)]
pub struct Delta {
    pub role: Option<String>,
    pub content: Option<String>,
}

/// OpenAI API error response
#[derive(Debug, Deserialize)]
struct ErrorResponse {
    error: ErrorDetail,
}

#[derive(Debug, Deserialize)]
struct ErrorDetail {
    message: String,
    #[serde(rename = "type")]
    #[allow(dead_code)]
    error_type: Option<String>,
    code: Option<String>,
}

/// OpenAI API client
pub struct OpenAIClient {
    client: Client,
    config: Config,
}

impl OpenAIClient {
    /// Create a new OpenAI client
    pub fn new(config: Config) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()?;

        Ok(Self { client, config })
    }

    /// Send a completion request
    #[instrument(skip(self, messages))]
    pub async fn complete(&self, messages: Vec<Message>) -> Result<String> {
        let request = CompletionRequest {
            model: self.config.model.clone(),
            messages,
            max_tokens: self.config.max_tokens,
            temperature: 0.7,
            stream: false,
        };

        debug!("Sending completion request");

        let response = self
            .client
            .post(&self.config.api_url())
            .header(
                "Authorization",
                format!("Bearer {}", self.config.api_key()?),
            )
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        let status = response.status();

        if !status.is_success() {
            let error_text = response.text().await?;

            // Try to parse as error response
            if let Ok(error_response) = serde_json::from_str::<ErrorResponse>(&error_text) {
                return match error_response.error.code.as_deref() {
                    Some("rate_limit_exceeded") => Err(AppError::RateLimitExceeded),
                    _ => Err(AppError::ApiError {
                        message: error_response.error.message,
                    }),
                };
            }

            return Err(AppError::ApiError {
                message: format!("API request failed with status {}: {}", status, error_text),
            });
        }

        let response: CompletionResponse = response.json().await?;

        let choice = response
            .choices
            .into_iter()
            .next()
            .ok_or_else(|| AppError::ApiError {
                message: "No response choices available".to_string(),
            })?;

        if let Some(reason) = choice.finish_reason {
            if reason == "length" {
                return Err(AppError::TokenLimitExceeded);
            }
        }

        Ok(choice.message.content)
    }

    /// Create a conversation with a single user message
    pub async fn chat(&self, user_input: &str) -> Result<String> {
        let messages = vec![
            Message::system(&self.config.system_prompt),
            Message::user(user_input),
        ];

        self.complete(messages).await
    }
    
    /// Send a streaming completion request
    #[instrument(skip(self, messages))]
    pub async fn complete_stream(
        &self,
        messages: Vec<Message>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String>> + Send>>> {
        let request = CompletionRequest {
            model: self.config.model.clone(),
            messages,
            max_tokens: self.config.max_tokens,
            temperature: 0.7,
            stream: true,
        };

        debug!("Sending streaming completion request");

        let response = self
            .client
            .post(&self.config.api_url())
            .header(
                "Authorization",
                format!("Bearer {}", self.config.api_key()?),
            )
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        let status = response.status();

        if !status.is_success() {
            let error_text = response.text().await?;

            // Try to parse as error response
            if let Ok(error_response) = serde_json::from_str::<ErrorResponse>(&error_text) {
                return match error_response.error.code.as_deref() {
                    Some("rate_limit_exceeded") => Err(AppError::RateLimitExceeded),
                    _ => Err(AppError::ApiError {
                        message: error_response.error.message,
                    }),
                };
            }

            return Err(AppError::ApiError {
                message: format!("API request failed with status {}: {}", status, error_text),
            });
        }

        let stream = response.bytes_stream();
        
        // Convert the bytes stream to a stream of parsed chunks
        let chunk_stream = stream
            .map(move |chunk| {
                match chunk {
                    Ok(bytes) => {
                        let text = String::from_utf8_lossy(&bytes);
                        
                        // Parse SSE format
                        let mut content = String::new();
                        for line in text.lines() {
                            if line.starts_with("data: ") {
                                let data = line.strip_prefix("data: ").unwrap_or("");
                                
                                if data == "[DONE]" {
                                    continue;
                                }
                                
                                if let Ok(chunk) = serde_json::from_str::<StreamChunk>(data) {
                                    for choice in chunk.choices {
                                        if let Some(delta_content) = choice.delta.content {
                                            content.push_str(&delta_content);
                                        }
                                    }
                                }
                            }
                        }
                        
                        if content.is_empty() {
                            Ok(String::new())
                        } else {
                            Ok(content)
                        }
                    }
                    Err(e) => Err(AppError::Network(e.to_string())),
                }
            });

        Ok(Box::pin(chunk_stream))
    }
    
    /// List available models from the API
    pub async fn list_models(&self) -> Result<Vec<String>> {
        let url = format!("{}/v1/models", self.config.base_url.trim_end_matches('/'));
        
        debug!("Fetching models from {}", url);
        
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key()?))
            .send()
            .await?;
        
        let status = response.status();
        
        if !status.is_success() {
            let error_text = response.text().await?;
            return Err(AppError::ApiError {
                message: format!("Failed to fetch models: {}", error_text),
            });
        }
        
        #[derive(Deserialize)]
        struct ModelsResponse {
            data: Vec<ModelInfo>,
        }
        
        #[derive(Deserialize)]
        struct ModelInfo {
            id: String,
            #[allow(dead_code)]
            object: String,
        }
        
        let models_response: ModelsResponse = response.json().await?;
        let model_ids: Vec<String> = models_response.data.into_iter().map(|m| m.id).collect();
        
        Ok(model_ids)
    }
}
