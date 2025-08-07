//! Session management for maintaining conversation history

use crate::api::Message;
use crate::error::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// A conversation session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub messages: Vec<Message>,
    pub model: String,
    pub total_tokens: u32,
}

impl Session {
    /// Create a new session
    pub fn new(model: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            created_at: Utc::now(),
            messages: Vec::new(),
            model,
            total_tokens: 0,
        }
    }

    /// Add a message to the session
    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);
    }

    /// Get the conversation history
    pub fn history(&self) -> &[Message] {
        &self.messages
    }

    /// Save session to file
    pub async fn save(&self, path: Option<PathBuf>) -> Result<PathBuf> {
        let path = path.unwrap_or_else(|| {
            let mut path = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
            path.push("llm-cli");
            path.push("sessions");
            path.push(format!("{}.json", self.id));
            path
        });

        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let json = serde_json::to_string_pretty(self)?;
        tokio::fs::write(&path, json).await?;

        Ok(path)
    }

    /// Load session from file
    pub async fn load(path: PathBuf) -> Result<Self> {
        let json = tokio::fs::read_to_string(path).await?;
        let session = serde_json::from_str(&json)?;
        Ok(session)
    }

    /// Export session as markdown
    pub fn to_markdown(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!("# Chat Session: {}\n", self.id));
        output.push_str(&format!(
            "**Date:** {}\n",
            self.created_at.format("%Y-%m-%d %H:%M:%S UTC")
        ));
        output.push_str(&format!("**Model:** {}\n\n", self.model));

        for message in &self.messages {
            let role = match message.role {
                crate::api::Role::System => "System",
                crate::api::Role::User => "User",
                crate::api::Role::Assistant => "Assistant",
            };

            output.push_str(&format!("## {}\n\n{}\n\n", role, message.content));
        }

        output
    }
}

/// Session manager for handling multiple sessions
pub struct SessionManager {
    sessions: Vec<Session>,
    current_session: Option<usize>,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new() -> Self {
        Self {
            sessions: Vec::new(),
            current_session: None,
        }
    }

    /// Create and set a new session as current
    pub fn new_session(&mut self, model: String) -> &mut Session {
        let session = Session::new(model);
        self.sessions.push(session);
        self.current_session = Some(self.sessions.len() - 1);
        self.current_session_mut().unwrap()
    }

    /// Get the current session
    pub fn current_session(&self) -> Option<&Session> {
        self.current_session.and_then(|idx| self.sessions.get(idx))
    }

    /// Get the current session mutably
    pub fn current_session_mut(&mut self) -> Option<&mut Session> {
        self.current_session
            .and_then(move |idx| self.sessions.get_mut(idx))
    }

    /// List all sessions
    pub fn list_sessions(&self) -> &[Session] {
        &self.sessions
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}
