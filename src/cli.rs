//! CLI interface and command handling

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// LLM CLI - A universal command-line interface for Large Language Models
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    /// Subcommand to execute
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Enable debug logging
    #[arg(short, long, env = "OPENAI_DEBUG")]
    pub debug: bool,

    /// Configuration file path
    #[arg(short, long, env = "OPENAI_CONFIG")]
    pub config: Option<PathBuf>,

    /// Override the model to use
    #[arg(short, long, env = "OPENAI_MODEL")]
    pub model: Option<String>,

    /// Override maximum tokens
    #[arg(short = 't', long, env = "OPENAI_MAX_TOKENS")]
    pub max_tokens: Option<u32>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Start an interactive chat session
    Chat {
        /// Initial message to send
        message: Option<String>,

        /// Enable multiline input mode
        #[arg(short, long)]
        multiline: bool,

        /// Enable Vim-like input mode
        #[arg(long)]
        vim: bool,
        
        /// Enable streaming responses
        #[arg(short, long)]
        stream: bool,
    },

    /// Send a single query and get a response
    Query {
        /// The query to send
        message: String,

        /// Output format (text, json, markdown)
        #[arg(short, long, default_value = "text")]
        format: OutputFormat,
        
        /// Enable streaming responses
        #[arg(short, long)]
        stream: bool,
    },

    /// Configure the CLI
    Config {
        /// Show current configuration
        #[arg(short, long)]
        show: bool,

        /// Set API key
        #[arg(long)]
        api_key: Option<String>,

        /// Set default model
        #[arg(long)]
        model: Option<String>,

        /// Set system prompt
        #[arg(long)]
        system_prompt: Option<String>,
        
        /// Set base URL for API (e.g., https://api.openai.com or custom endpoint)
        #[arg(long)]
        base_url: Option<String>,
        
        /// Set API path (e.g., /v1/chat/completions)
        #[arg(long)]
        api_path: Option<String>,
    },

    /// List available models
    Models,

    /// Show token usage statistics
    Stats,
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum OutputFormat {
    Text,
    Json,
    Markdown,
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Text => write!(f, "text"),
            Self::Json => write!(f, "json"),
            Self::Markdown => write!(f, "markdown"),
        }
    }
}
