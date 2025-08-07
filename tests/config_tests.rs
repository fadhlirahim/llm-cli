//! Configuration tests

use llm_cli::config::Config;

#[test]
fn test_config_with_api_key() {
    // Test config with API key set
    let config = Config::test_config_with(
        Some("test-key".to_string()),
        "https://api.openai.com".to_string(),
        "gpt-4-turbo".to_string(),
        2048,
    );

    assert_eq!(config.api_key.unwrap(), "test-key");
    assert_eq!(config.model, "gpt-4-turbo");
    assert_eq!(config.max_tokens, 2048);
    assert_eq!(config.base_url, "https://api.openai.com");
}

#[test]
fn test_config_missing_api_key_for_cloud_service() {
    // Test that cloud services require an API key
    let config = Config::test_config_with(
        None, // No API key
        "https://api.openai.com".to_string(),
        "gpt-4o".to_string(),
        4096,
    );

    // Should fail validation because no API key for non-local service
    assert!(config.validate().is_err());
}

#[test]
fn test_config_local_service_no_api_key() {
    // Test that local services don't require an API key
    let mut config = Config::test_config_with(
        None, // No API key initially
        "http://localhost:1234".to_string(),
        "local-model".to_string(),
        4096,
    );

    // Validation should pass for localhost even without API key
    assert!(config.validate().is_ok());

    // The actual load() function would set a dummy key, let's simulate that
    if config.api_key.is_none() {
        config.api_key = Some("local-service".to_string());
    }
    assert_eq!(config.api_key.unwrap(), "local-service");
}

#[test]
fn test_config_api_url_generation() {
    let config = Config::test_config_with(
        Some("key".to_string()),
        "https://api.example.com".to_string(),
        "model".to_string(),
        1000,
    );

    assert_eq!(
        config.api_url(),
        "https://api.example.com/v1/chat/completions"
    );
}

#[test]
fn test_config_api_url_with_trailing_slash() {
    let config = Config::test_config_with(
        Some("key".to_string()),
        "https://api.example.com/".to_string(), // trailing slash
        "model".to_string(),
        1000,
    );

    // Should handle trailing slash correctly
    assert_eq!(
        config.api_url(),
        "https://api.example.com/v1/chat/completions"
    );
}

#[test]
fn test_default_config_values() {
    let config = Config::test_config();

    assert_eq!(config.model, "gpt-4o");
    assert_eq!(config.max_tokens, 4096);
    assert_eq!(config.base_url, "https://api.openai.com");
    assert_eq!(config.api_path, "/v1/chat/completions");
    assert_eq!(config.timeout_seconds, 30);
    assert!(!config.debug);
}

