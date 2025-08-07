//! Configuration tests

use llm_cli::config::Config;
use std::env;

#[tokio::test]
async fn test_config_env_override() {
    env::set_var("OPENAI_API_KEY", "test-key-from-env");
    env::set_var("OPENAI_MODEL", "gpt-4-turbo");
    env::set_var("OPENAI_MAX_TOKENS", "2048");

    let config = Config::load().await.unwrap();

    assert_eq!(config.api_key.unwrap(), "test-key-from-env");
    assert_eq!(config.model, "gpt-4-turbo");
    assert_eq!(config.max_tokens, 2048);

    env::remove_var("OPENAI_API_KEY");
    env::remove_var("OPENAI_MODEL");
    env::remove_var("OPENAI_MAX_TOKENS");
}

#[tokio::test]
async fn test_config_missing_api_key() {
    env::remove_var("OPENAI_API_KEY");

    let result = Config::load().await;
    assert!(result.is_err());

    match result.unwrap_err() {
        llm_cli::AppError::ApiKeyNotFound => (),
        e => panic!("Expected ApiKeyNotFound, got {:?}", e),
    }
}
