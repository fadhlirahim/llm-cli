//! Configuration management for the OpenAI CLI

use crate::error::{AppError, Result};
use dirs::config_dir;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// OpenAI API key
    pub api_key: Option<String>,

    /// Model to use for completions
    #[serde(default = "default_model")]
    pub model: String,

    /// Maximum tokens for response
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,

    /// Base URL for the API (e.g., "https://api.openai.com" or custom endpoint)
    #[serde(default = "default_base_url")]
    pub base_url: String,
    
    /// API endpoint path
    #[serde(default = "default_api_path")]
    pub api_path: String,

    /// System prompt
    #[serde(default = "default_system_prompt")]
    pub system_prompt: String,

    /// Request timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,

    /// Enable debug logging
    #[serde(default)]
    pub debug: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_key: None,
            model: default_model(),
            max_tokens: default_max_tokens(),
            base_url: default_base_url(),
            api_path: default_api_path(),
            system_prompt: default_system_prompt(),
            timeout_seconds: default_timeout(),
            debug: false,
        }
    }
}

impl Config {
    /// Load configuration from environment and config file
    pub async fn load() -> Result<Self> {
        Self::load_with_file_support(true).await
    }
    
    /// Create a config directly for testing
    #[doc(hidden)]
    pub fn test_config() -> Self {
        Self::default()
    }
    
    /// Create a config with specific values for testing
    #[doc(hidden)]
    pub fn test_config_with(
        api_key: Option<String>,
        base_url: String,
        model: String,
        max_tokens: u32,
    ) -> Self {
        Self {
            api_key,
            model,
            max_tokens,
            base_url,
            api_path: "/v1/chat/completions".to_string(),
            system_prompt: "Test prompt".to_string(),
            timeout_seconds: 30,
            debug: false,
        }
    }
    
    /// Validate config (for testing)
    #[doc(hidden)]
    pub fn validate(&self) -> Result<()> {
        // Check if using local service
        let is_local = self.base_url.starts_with("http://localhost") 
            || self.base_url.starts_with("http://127.0.0.1")
            || self.base_url.starts_with("http://0.0.0.0");
        
        if !is_local && self.api_key.is_none() {
            return Err(AppError::ApiKeyNotFound);
        }
        
        Ok(())
    }
    
    async fn load_with_file_support(use_file: bool) -> Result<Self> {
        let mut config = if use_file {
            Self::load_from_file().await.unwrap_or_default()
        } else {
            Self::default()
        };

        // Override with environment variables
        if let Ok(api_key) = std::env::var("OPENAI_API_KEY") {
            config.api_key = Some(api_key);
        }

        if let Ok(model) = std::env::var("OPENAI_MODEL") {
            config.model = model;
        }

        if let Ok(max_tokens) = std::env::var("OPENAI_MAX_TOKENS") {
            config.max_tokens = max_tokens
                .parse()
                .map_err(|_| AppError::ConfigError("Invalid max_tokens value".to_string()))?;
        }
        
        if let Ok(base_url) = std::env::var("OPENAI_BASE_URL") {
            config.base_url = base_url;
        }
        
        if let Ok(api_path) = std::env::var("OPENAI_API_PATH") {
            config.api_path = api_path;
        }

        // Only require API key for cloud services
        if config.api_key.is_none() {
            // Check if using local service (LM Studio, Ollama, etc.)
            let is_local = config.base_url.starts_with("http://localhost") 
                || config.base_url.starts_with("http://127.0.0.1")
                || config.base_url.starts_with("http://0.0.0.0");
            
            if !is_local {
                return Err(AppError::ApiKeyNotFound);
            }
            
            // Set a dummy key for local services
            config.api_key = Some("local-service".to_string());
        }

        Ok(config)
    }

    /// Load configuration from file
    async fn load_from_file() -> Result<Self> {
        let config_path = Self::config_path()?;

        if !config_path.exists() {
            return Ok(Self::default());
        }

        let content = tokio::fs::read_to_string(&config_path).await?;
        let config: Self =
            toml::from_str(&content).map_err(|e| AppError::ConfigError(e.to_string()))?;

        Ok(config)
    }

    /// Save configuration to file
    pub async fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;

        if let Some(parent) = config_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let content =
            toml::to_string_pretty(self).map_err(|e| AppError::ConfigError(e.to_string()))?;

        tokio::fs::write(&config_path, content).await?;

        Ok(())
    }

    /// Get the configuration file path
    fn config_path() -> Result<PathBuf> {
        let mut path = config_dir()
            .ok_or_else(|| AppError::ConfigError("Could not find config directory".to_string()))?;
        path.push("llm-cli");
        path.push("config.toml");
        Ok(path)
    }

    /// Get the API key
    pub fn api_key(&self) -> Result<&str> {
        self.api_key.as_deref().ok_or(AppError::ApiKeyNotFound)
    }
    
    /// Get the full API URL
    pub fn api_url(&self) -> String {
        format!("{}{}", self.base_url.trim_end_matches('/'), self.api_path)
    }
    
}

fn default_model() -> String {
    "gpt-4o".to_string()
}

fn default_max_tokens() -> u32 {
    4096
}

fn default_base_url() -> String {
    "https://api.openai.com".to_string()
}

fn default_api_path() -> String {
    "/v1/chat/completions".to_string()
}

fn default_system_prompt() -> String {
    "You are a helpful assistant. Answer in a clear and concise manner.".to_string()
}

fn default_timeout() -> u64 {
    30
}
