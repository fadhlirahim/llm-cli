//! LLM CLI Library - A universal CLI for LLMs

pub mod api;
pub mod cli;
pub mod config;
pub mod error;
pub mod session;
pub mod ui;

pub use error::{AppError, Result};
