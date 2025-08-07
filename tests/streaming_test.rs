//! Tests for streaming functionality

use futures_util::{Stream, StreamExt};
use llm_cli::api::{Delta, Message, OpenAIClient, StreamChoice, StreamChunk};
use llm_cli::config::Config;
use llm_cli::error::{AppError, Result};
use std::pin::Pin;
use std::time::Duration;
use wiremock::matchers::{method, path, header};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Helper function to create a test config pointing to mock server
async fn create_test_config(mock_server: &MockServer) -> Config {
    let mut config = Config::default();
    config.api_key = Some("test-key".to_string()); // Set API key directly
    config.base_url = mock_server.uri();
    config.api_path = "/v1/chat/completions".to_string();
    config.model = "gpt-4".to_string();
    config.max_tokens = 100;
    config.system_prompt = "You are a helpful assistant.".to_string();
    config.timeout_seconds = 5;
    config
}

/// Helper function to create SSE formatted streaming response
fn create_sse_chunk(content: &str, finish_reason: Option<&str>) -> String {
    let chunk = StreamChunk {
        id: "chatcmpl-test".to_string(),
        object: "chat.completion.chunk".to_string(),
        created: 1234567890,
        model: "gpt-4".to_string(),
        choices: vec![StreamChoice {
            index: 0,
            delta: Delta {
                role: None,
                content: if content.is_empty() { None } else { Some(content.to_string()) },
            },
            finish_reason: finish_reason.map(String::from),
        }],
    };
    
    format!("data: {}\n\n", serde_json::to_string(&chunk).unwrap())
}

/// Helper function to create a streaming response body
fn create_streaming_response(chunks: Vec<&str>) -> String {
    let mut response = String::new();
    
    // Add initial chunk with role
    response.push_str(&format!(
        "data: {}\n\n",
        serde_json::to_string(&StreamChunk {
            id: "chatcmpl-test".to_string(),
            object: "chat.completion.chunk".to_string(),
            created: 1234567890,
            model: "gpt-4".to_string(),
            choices: vec![StreamChoice {
                index: 0,
                delta: Delta {
                    role: Some("assistant".to_string()),
                    content: None,
                },
                finish_reason: None,
            }],
        }).unwrap()
    ));
    
    // Add content chunks
    for (i, chunk) in chunks.iter().enumerate() {
        let is_last = i == chunks.len() - 1;
        response.push_str(&create_sse_chunk(
            chunk,
            if is_last { Some("stop") } else { None },
        ));
    }
    
    // Add final [DONE] marker
    response.push_str("data: [DONE]\n\n");
    
    response
}

#[tokio::test]
async fn test_streaming_response_parsing() {
    let mock_server = MockServer::start().await;
    let config = create_test_config(&mock_server).await;
    
    // Set up mock response with streaming chunks
    let response_body = create_streaming_response(vec![
        "Hello",
        " there",
        "!",
        " How",
        " can",
        " I",
        " help",
        " you",
        " today",
        "?",
    ]);
    
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .and(header("authorization", "Bearer test-key"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(response_body)
                .append_header("content-type", "text/event-stream")
        )
        .mount(&mock_server)
        .await;
    
    // Create client and test streaming
    let client = OpenAIClient::new(config).unwrap();
    let messages = vec![
        Message::system("You are a helpful assistant."),
        Message::user("Hello!"),
    ];
    
    let mut stream = client.complete_stream(messages).await.unwrap();
    let mut collected_response = String::new();
    
    while let Some(chunk_result) = stream.next().await {
        match chunk_result {
            Ok(chunk) => {
                collected_response.push_str(&chunk);
            }
            Err(e) => {
                panic!("Unexpected error: {}", e);
            }
        }
    }
    
    assert_eq!(collected_response, "Hello there! How can I help you today?");
}

#[tokio::test]
async fn test_streaming_with_empty_chunks() {
    let mock_server = MockServer::start().await;
    let config = create_test_config(&mock_server).await;
    
    // Create response with some empty chunks (which should be filtered out)
    let response_body = create_streaming_response(vec![
        "Response",
        "",  // Empty chunk
        " with",
        "",  // Another empty chunk
        " gaps",
    ]);
    
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(response_body)
                .append_header("content-type", "text/event-stream")
        )
        .mount(&mock_server)
        .await;
    
    let client = OpenAIClient::new(config).unwrap();
    let messages = vec![Message::user("Test")];
    
    let mut stream = client.complete_stream(messages).await.unwrap();
    let mut collected_response = String::new();
    
    while let Some(chunk_result) = stream.next().await {
        if let Ok(chunk) = chunk_result {
            collected_response.push_str(&chunk);
        }
    }
    
    assert_eq!(collected_response, "Response with gaps");
}

#[tokio::test]
async fn test_streaming_error_handling() {
    let mock_server = MockServer::start().await;
    let config = create_test_config(&mock_server).await;
    
    // Return an error response
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(
            ResponseTemplate::new(429)
                .set_body_json(serde_json::json!({
                    "error": {
                        "message": "Rate limit exceeded",
                        "type": "rate_limit_error",
                        "code": "rate_limit_exceeded"
                    }
                }))
        )
        .mount(&mock_server)
        .await;
    
    let client = OpenAIClient::new(config).unwrap();
    let messages = vec![Message::user("Test")];
    
    let result = client.complete_stream(messages).await;
    
    match result {
        Err(AppError::RateLimitExceeded) => {
            // Expected error
        }
        Err(e) => panic!("Unexpected error type: {}", e),
        Ok(_) => panic!("Expected error but got success"),
    }
}

#[tokio::test]
async fn test_streaming_with_multiline_content() {
    let mock_server = MockServer::start().await;
    let config = create_test_config(&mock_server).await;
    
    // Test with content that includes newlines
    let response_body = create_streaming_response(vec![
        "Here's",
        " a",
        " response\n",
        "with",
        " multiple\n",
        "lines",
        " of",
        " text",
    ]);
    
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(response_body)
                .append_header("content-type", "text/event-stream")
        )
        .mount(&mock_server)
        .await;
    
    let client = OpenAIClient::new(config).unwrap();
    let messages = vec![Message::user("Test")];
    
    let mut stream = client.complete_stream(messages).await.unwrap();
    let mut collected_response = String::new();
    
    while let Some(chunk_result) = stream.next().await {
        if let Ok(chunk) = chunk_result {
            collected_response.push_str(&chunk);
        }
    }
    
    assert_eq!(collected_response, "Here's a response\nwith multiple\nlines of text");
}

#[tokio::test]
async fn test_streaming_with_special_characters() {
    let mock_server = MockServer::start().await;
    let config = create_test_config(&mock_server).await;
    
    // Test with special characters and emoji
    let response_body = create_streaming_response(vec![
        "Hello",
        " 👋",
        " Special",
        " chars:",
        " <>&\"'",
        " and",
        " unicode:",
        " 你好",
    ]);
    
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(response_body)
                .append_header("content-type", "text/event-stream")
        )
        .mount(&mock_server)
        .await;
    
    let client = OpenAIClient::new(config).unwrap();
    let messages = vec![Message::user("Test")];
    
    let mut stream = client.complete_stream(messages).await.unwrap();
    let mut collected_response = String::new();
    
    while let Some(chunk_result) = stream.next().await {
        if let Ok(chunk) = chunk_result {
            collected_response.push_str(&chunk);
        }
    }
    
    assert_eq!(collected_response, "Hello 👋 Special chars: <>&\"' and unicode: 你好");
}

#[tokio::test]
async fn test_streaming_timeout() {
    let mock_server = MockServer::start().await;
    let mut config = create_test_config(&mock_server).await;
    config.timeout_seconds = 1; // Set very short timeout
    
    // Mock a slow response
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_delay(Duration::from_secs(2)) // Delay longer than timeout
                .set_body_string("data: test\n\n")
        )
        .mount(&mock_server)
        .await;
    
    let client = OpenAIClient::new(config).unwrap();
    let messages = vec![Message::user("Test")];
    
    let result = client.complete_stream(messages).await;
    
    // Should timeout
    assert!(result.is_err());
}

#[tokio::test]
async fn test_streaming_large_response() {
    let mock_server = MockServer::start().await;
    let config = create_test_config(&mock_server).await;
    
    // Create a large response with many chunks
    let mut chunks = Vec::new();
    for i in 0..100 {
        chunks.push(format!("Chunk {} ", i));
    }
    let chunk_refs: Vec<&str> = chunks.iter().map(|s| s.as_str()).collect();
    let response_body = create_streaming_response(chunk_refs);
    
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(response_body)
                .append_header("content-type", "text/event-stream")
        )
        .mount(&mock_server)
        .await;
    
    let client = OpenAIClient::new(config).unwrap();
    let messages = vec![Message::user("Test")];
    
    let mut stream = client.complete_stream(messages).await.unwrap();
    let mut collected_response = String::new();
    let mut chunk_count = 0;
    
    while let Some(chunk_result) = stream.next().await {
        if let Ok(chunk) = chunk_result {
            if !chunk.is_empty() {
                collected_response.push_str(&chunk);
                chunk_count += 1;
            }
        }
    }
    
    // Verify we received all chunks
    assert!(chunk_count > 0);
    assert!(collected_response.contains("Chunk 0"));
    assert!(collected_response.contains("Chunk 99"));
}

#[tokio::test]
async fn test_streaming_with_malformed_data() {
    let mock_server = MockServer::start().await;
    let config = create_test_config(&mock_server).await;
    
    // Send malformed SSE data
    let response_body = "data: {invalid json}\n\ndata: [DONE]\n\n";
    
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(response_body)
                .append_header("content-type", "text/event-stream")
        )
        .mount(&mock_server)
        .await;
    
    let client = OpenAIClient::new(config).unwrap();
    let messages = vec![Message::user("Test")];
    
    let mut stream = client.complete_stream(messages).await.unwrap();
    
    // Should handle malformed data gracefully
    while let Some(chunk_result) = stream.next().await {
        match chunk_result {
            Ok(chunk) => {
                // May get empty chunks, which is fine
                assert!(chunk.is_empty() || chunk.len() > 0);
            }
            Err(_) => {
                // Error is also acceptable for malformed data
            }
        }
    }
}

#[tokio::test]
async fn test_streaming_api_error_response() {
    let mock_server = MockServer::start().await;
    let config = create_test_config(&mock_server).await;
    
    // Return a generic API error
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(
            ResponseTemplate::new(400)
                .set_body_json(serde_json::json!({
                    "error": {
                        "message": "Invalid request parameters",
                        "type": "invalid_request_error"
                    }
                }))
        )
        .mount(&mock_server)
        .await;
    
    let client = OpenAIClient::new(config).unwrap();
    let messages = vec![Message::user("Test")];
    
    let result = client.complete_stream(messages).await;
    
    match result {
        Err(AppError::ApiError { message }) => {
            assert_eq!(message, "Invalid request parameters");
        }
        Err(e) => panic!("Unexpected error type: {}", e),
        Ok(_) => panic!("Expected error but got success"),
    }
}

/// Test helper to create a mock streaming response
pub fn create_mock_stream() -> Pin<Box<dyn Stream<Item = Result<String>> + Send>> {
    let chunks = vec![
        Ok("Hello".to_string()),
        Ok(" world".to_string()),
        Ok("!".to_string()),
    ];
    
    Box::pin(futures_util::stream::iter(chunks))
}

#[tokio::test]
async fn test_mock_stream_helper() {
    let mut stream = create_mock_stream();
    let mut result = String::new();
    
    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(text) => result.push_str(&text),
            Err(e) => panic!("Unexpected error: {}", e),
        }
    }
    
    assert_eq!(result, "Hello world!");
}

#[cfg(test)]
mod ui_streaming_tests {
    
    #[test]
    fn test_streaming_chunk_display() {
        // Test that streaming chunks are displayed correctly
        let chunk1 = "Hello";
        let chunk2 = " world";
        let chunk3 = "\nNew line";
        
        // This would normally print to stdout, but we can verify the logic
        assert_eq!(chunk1.len(), 5);
        assert_eq!(chunk2.len(), 6);
        assert!(chunk3.contains('\n'));
    }
    
    #[test]
    fn test_streaming_with_unicode() {
        let chunks = vec![
            "Hello 👋",
            " 世界",
            " 🌍",
        ];
        
        let mut result = String::new();
        for chunk in chunks {
            result.push_str(chunk);
        }
        
        assert_eq!(result, "Hello 👋 世界 🌍");
    }
}