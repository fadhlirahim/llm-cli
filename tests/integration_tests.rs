//! Integration tests for the LLM CLI

use llm_cli::api::{Message, Role};
use llm_cli::config::Config;
use llm_cli::session::Session;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_message_creation() {
    let system_msg = Message::system("You are a test assistant");
    assert!(matches!(system_msg.role, Role::System));
    assert_eq!(system_msg.content, "You are a test assistant");

    let user_msg = Message::user("Hello");
    assert!(matches!(user_msg.role, Role::User));
    assert_eq!(user_msg.content, "Hello");

    let assistant_msg = Message::assistant("Hi there!");
    assert!(matches!(assistant_msg.role, Role::Assistant));
    assert_eq!(assistant_msg.content, "Hi there!");
}

#[tokio::test]
async fn test_session_management() {
    let mut session = Session::new("gpt-4o".to_string());

    assert_eq!(session.model, "gpt-4o");
    assert_eq!(session.messages.len(), 0);

    session.add_message(Message::user("Test message"));
    assert_eq!(session.messages.len(), 1);

    let history = session.history();
    assert_eq!(history.len(), 1);
}

#[tokio::test]
async fn test_session_markdown_export() {
    let mut session = Session::new("gpt-4o".to_string());
    session.add_message(Message::user("What is Rust?"));
    session.add_message(Message::assistant(
        "Rust is a systems programming language.",
    ));

    let markdown = session.to_markdown();
    assert!(markdown.contains("# Chat Session:"));
    assert!(markdown.contains("**Model:** gpt-4o"));
    assert!(markdown.contains("## User"));
    assert!(markdown.contains("What is Rust?"));
    assert!(markdown.contains("## Assistant"));
    assert!(markdown.contains("Rust is a systems programming language"));
}

#[tokio::test]
async fn test_config_defaults() {
    let config = Config::default();

    assert_eq!(config.model, "gpt-4o");
    assert_eq!(config.max_tokens, 4096);
    assert_eq!(config.base_url, "https://api.openai.com");
    assert_eq!(config.api_path, "/v1/chat/completions");
    assert_eq!(config.api_url(), "https://api.openai.com/v1/chat/completions");
    assert_eq!(config.timeout_seconds, 30);
    assert!(!config.debug);
}

#[tokio::test]
async fn test_api_client_mock() {
    let mock_server = MockServer::start().await;

    let mock_response = r#"{
        "id": "chatcmpl-123",
        "object": "chat.completion",
        "created": 1677652288,
        "model": "gpt-4o",
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": "Hello! How can I help you today?"
            },
            "finish_reason": "stop"
        }],
        "usage": {
            "prompt_tokens": 10,
            "completion_tokens": 8,
            "total_tokens": 18
        }
    }"#;

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .and(header("authorization", "Bearer test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_string(mock_response))
        .mount(&mock_server)
        .await;

    let mut config = Config::default();
    config.api_key = Some("test-key".to_string());
    config.base_url = mock_server.uri();
    config.api_path = "/v1/chat/completions".to_string();

    let client = llm_cli::api::OpenAIClient::new(config).unwrap();
    let response = client.chat("Hello").await.unwrap();

    assert_eq!(response, "Hello! How can I help you today?");
}

#[tokio::test]
async fn test_error_handling() {
    let mock_server = MockServer::start().await;

    let error_response = r#"{
        "error": {
            "message": "Invalid API key provided",
            "type": "invalid_request_error",
            "code": "invalid_api_key"
        }
    }"#;

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(401).set_body_string(error_response))
        .mount(&mock_server)
        .await;

    let mut config = Config::default();
    config.api_key = Some("invalid-key".to_string());
    config.base_url = mock_server.uri();
    config.api_path = "/v1/chat/completions".to_string();

    let client = llm_cli::api::OpenAIClient::new(config).unwrap();
    let result = client.chat("Hello").await;

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("Invalid API key"));
}

#[tokio::test]
async fn test_rate_limit_error() {
    let mock_server = MockServer::start().await;

    let error_response = r#"{
        "error": {
            "message": "Rate limit exceeded",
            "type": "rate_limit_error",
            "code": "rate_limit_exceeded"
        }
    }"#;

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(429).set_body_string(error_response))
        .mount(&mock_server)
        .await;

    let mut config = Config::default();
    config.api_key = Some("test-key".to_string());
    config.base_url = mock_server.uri();
    config.api_path = "/v1/chat/completions".to_string();

    let client = llm_cli::api::OpenAIClient::new(config).unwrap();
    let result = client.chat("Hello").await;

    assert!(result.is_err());
    match result.unwrap_err() {
        llm_cli::AppError::RateLimitExceeded => (),
        e => panic!("Expected RateLimitExceeded, got {:?}", e),
    }
}
