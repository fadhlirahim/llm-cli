//! OpenAI API client implementation

use crate::config::Config;
use crate::error::{AppError, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
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
