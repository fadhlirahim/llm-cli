//! Modern LLM CLI with best practices for 2025 - Supports OpenAI, LM Studio, Ollama, and more

mod api;
mod cli;
mod config;
mod error;
mod session;
mod streaming_buffer;
mod ui;

use anyhow::Context;
use clap::Parser;
use cli::{Cli, Commands, OutputFormat};
use colored::Colorize;
use config::Config;
use std::io::{self, Write};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Parse CLI arguments
    let cli = Cli::parse();

    // Initialize logging
    init_logging(cli.debug)?;

    // Load configuration
    let mut config = Config::load()
        .await
        .context("Failed to load configuration")?;

    // Override config with CLI arguments
    if let Some(model) = cli.model {
        config.model = model;
    }
    if let Some(max_tokens) = cli.max_tokens {
        config.max_tokens = max_tokens;
    }

    // Execute command
    match cli.command {
        None | Some(Commands::Chat { .. }) => {
            run_chat_mode(config, cli.command).await?;
        }
        Some(Commands::Query { message, format, stream }) => {
            run_query_mode(config, message, format, stream).await?;
        }
        Some(Commands::Config {
            show,
            api_key,
            model,
            system_prompt,
            base_url,
            api_path,
        }) => {
            run_config_command(config, show, api_key, model, system_prompt, base_url, api_path).await?;
        }
        Some(Commands::Models) => {
            list_models(config).await?;
        }
        Some(Commands::Stats) => {
            show_stats().await?;
        }
    }

    Ok(())
}

/// Initialize logging based on debug flag
fn init_logging(debug: bool) -> anyhow::Result<()> {
    let filter = if debug {
        EnvFilter::new("debug")
    } else {
        EnvFilter::new("info")
    };

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_file(false)
        .with_line_number(false)
        .compact()
        .init();

    Ok(())
}

/// Run interactive chat mode
async fn run_chat_mode(config: Config, command: Option<Commands>) -> anyhow::Result<()> {
    let (multiline, vim, stream, initial_message) = if let Some(Commands::Chat {
        multiline,
        vim,
        stream,
        message,
    }) = command
    {
        (multiline, vim, stream, message)
    } else {
        (false, false, false, None)
    };

    ui::clear_screen();
    ui::show_welcome();

    let client = api::OpenAIClient::new(config.clone())?;
    let mut session_manager = session::SessionManager::new();
    let session = session_manager.new_session(config.model.clone());

    // Add system message
    session.add_message(api::Message::system(&config.system_prompt));

    // Process initial message if provided
    if let Some(message) = initial_message {
        process_chat_message(&client, session, &message, stream).await?;
    }

    // Main chat loop
    loop {
        let input = if multiline {
            ui::get_multiline_input()?
        } else if vim {
            ui::get_input_vim("You")?
        } else {
            ui::get_input("You")?
        };

        let input = input.trim();

        // Handle special commands
        match input.to_lowercase().as_str() {
            "exit" | "quit" => {
                println!("Goodbye!");
                break;
            }
            "clear" => {
                ui::clear_screen();
                ui::show_welcome();
                continue;
            }
            "help" => {
                ui::show_help();
                continue;
            }
            "history" => {
                display_history(session);
                continue;
            }
            "save" => {
                save_session(session).await?;
                continue;
            }
            _ if input.starts_with("model ") => {
                let model_name = input.strip_prefix("model ").unwrap();
                session.model = model_name.to_string();
                println!("Model changed to: {}", model_name);
                continue;
            }
            _ => {}
        }

        if input.is_empty() {
            continue;
        }

        process_chat_message(&client, session, input, stream).await?;
    }

    Ok(())
}

/// Process a chat message
async fn process_chat_message(
    client: &api::OpenAIClient,
    session: &mut session::Session,
    input: &str,
    stream: bool,
) -> anyhow::Result<()> {
    use futures_util::StreamExt;
    
    // Add user message to session
    session.add_message(api::Message::user(input));

    if stream {
        // Streaming mode with table support
        use crate::streaming_buffer::StreamingBuffer;
        
        match client.complete_stream(session.history().to_vec()).await {
            Ok(mut stream) => {
                ui::display_streaming_header();
                
                let mut full_response = String::new();
                let mut buffer = StreamingBuffer::new();
                let mut needs_indent = true;  // Start with indent for first line
                let mut table_spinner: Option<indicatif::ProgressBar> = None;
                
                while let Some(chunk_result) = stream.next().await {
                    match chunk_result {
                        Ok(chunk) => {
                            if !chunk.is_empty() {
                                full_response.push_str(&chunk);
                                
                                // Process chunk through buffer for table/code block detection
                                let (text_output, special_output, is_buffering) = buffer.process_chunk(&chunk);
                                
                                // Don't show spinner for code blocks, only for tables
                                if is_buffering && buffer.is_buffering_table() && table_spinner.is_none() {
                                    // Start spinner for table buffering only
                                    if needs_indent {
                                        println!(); // New line before spinner
                                        needs_indent = false;
                                    }
                                    let spinner = ui::create_spinner("  Buffering table...");
                                    table_spinner = Some(spinner);
                                } else if !buffer.is_buffering_table() && table_spinner.is_some() {
                                    // Stop spinner when table buffering is done
                                    if let Some(spinner) = table_spinner.take() {
                                        spinner.finish_and_clear();
                                    }
                                }
                                
                                // Display any immediate text
                                if !text_output.is_empty() {
                                    ui::display_streaming_chunk_smart(&text_output, needs_indent);
                                    // Only reset needs_indent if we're at the start of a new line
                                    needs_indent = false;  // We've printed something, no more indent until newline
                                }
                                
                                // Display any completed special content (table or code block)
                                if let Some(special) = special_output {
                                    if needs_indent {
                                        println!(); // New line before special content
                                        needs_indent = false;
                                    }
                                    // Just print the formatted content directly
                                    print!("{}", special);
                                    std::io::Write::flush(&mut std::io::stdout()).unwrap();
                                }
                            }
                        }
                        Err(e) => {
                            // Clean up spinner if active
                            if let Some(spinner) = table_spinner.take() {
                                spinner.finish_and_clear();
                            }
                            ui::finish_streaming_display();
                            ui::display_error(&e.to_string());
                            session.messages.pop();
                            return Ok(());
                        }
                    }
                }
                
                // Clean up any remaining spinner
                if let Some(spinner) = table_spinner.take() {
                    spinner.finish_and_clear();
                }
                
                // Flush any remaining content (could be formatted code block or table)
                if let Some(remaining) = buffer.flush() {
                    // The flush might return formatted content (code blocks/tables)
                    // so we print it directly instead of passing through chunk display
                    print!("{}", remaining);
                    io::stdout().flush().unwrap();
                }
                
                ui::finish_streaming_display();
                
                // Add assistant message to session
                session.add_message(api::Message::assistant(&full_response));
            }
            Err(e) => {
                ui::display_error(&e.to_string());
                // Remove the user message if the request failed
                session.messages.pop();
            }
        }
    } else {
        // Non-streaming mode
        // Show spinner
        let spinner = ui::create_spinner("Thinking...");

        // Get response
        match client.complete(session.history().to_vec()).await {
            Ok(response) => {
                spinner.finish_and_clear();

                // Add assistant message to session
                session.add_message(api::Message::assistant(&response));

                // Display response
                ui::display_response(&response, OutputFormat::Text);
            }
            Err(e) => {
                spinner.finish_and_clear();
                ui::display_error(&e.to_string());

                // Remove the user message if the request failed
                session.messages.pop();
            }
        }
    }

    Ok(())
}

/// Run single query mode
async fn run_query_mode(
    config: Config,
    message: String,
    format: OutputFormat,
    stream: bool,
) -> anyhow::Result<()> {
    use futures_util::StreamExt;
    
    let client = api::OpenAIClient::new(config.clone())?;

    if stream {
        // Streaming mode with table support
        use crate::streaming_buffer::StreamingBuffer;
        
        let messages = vec![
            api::Message::system(&config.system_prompt),
            api::Message::user(&message),
        ];
        
        match client.complete_stream(messages).await {
            Ok(mut stream) => {
                if matches!(format, OutputFormat::Text) {
                    ui::display_streaming_header();
                    
                    let mut buffer = StreamingBuffer::new();
                    let mut needs_indent = true;  // Start with indent for first line
                    let mut table_spinner: Option<indicatif::ProgressBar> = None;
                    
                    while let Some(chunk_result) = stream.next().await {
                        match chunk_result {
                            Ok(chunk) => {
                                if !chunk.is_empty() {
                                    // Process chunk through buffer for table detection
                                    let (text_output, table_output, is_buffering_table) = buffer.process_chunk(&chunk);
                                    
                                    // Handle table buffering spinner
                                    if is_buffering_table && table_spinner.is_none() {
                                        // Start spinner for table buffering
                                        if needs_indent {
                                            println!(); // New line before spinner
                                            needs_indent = false;
                                        }
                                        let spinner = ui::create_spinner("  Buffering table...");
                                        table_spinner = Some(spinner);
                                    } else if !is_buffering_table && table_spinner.is_some() {
                                        // Stop spinner when table buffering is done
                                        if let Some(spinner) = table_spinner.take() {
                                            spinner.finish_and_clear();
                                        }
                                    }
                                    
                                    // Display any immediate text with proper wrapping
                                    if !text_output.is_empty() {
                                        ui::display_streaming_chunk_smart(&text_output, needs_indent);
                                        // Only reset needs_indent if we're at the start of a new line
                                        needs_indent = false;  // We've printed something, no more indent until newline
                                    }
                                    
                                    // Display any completed table
                                    if let Some(table) = table_output {
                                        if needs_indent {
                                            println!(); // New line before table
                                            needs_indent = false;
                                        }
                                        ui::display_streaming_table(&table);
                                    }
                                }
                            }
                            Err(e) => {
                                // Clean up spinner if active
                                if let Some(spinner) = table_spinner.take() {
                                    spinner.finish_and_clear();
                                }
                                ui::finish_streaming_display();
                                ui::display_error(&e.to_string());
                                return Ok(());
                            }
                        }
                    }
                    
                    // Clean up any remaining spinner
                    if let Some(spinner) = table_spinner.take() {
                        spinner.finish_and_clear();
                    }
                    
                    // Flush any remaining content
                    if let Some(remaining) = buffer.flush() {
                        ui::display_streaming_chunk_smart(&remaining, needs_indent);
                    }
                    
                    ui::finish_streaming_display();
                } else {
                    // For non-text formats, collect the full response first
                    let mut full_response = String::new();
                    
                    while let Some(chunk_result) = stream.next().await {
                        match chunk_result {
                            Ok(chunk) => {
                                full_response.push_str(&chunk);
                            }
                            Err(e) => {
                                ui::display_error(&e.to_string());
                                return Ok(());
                            }
                        }
                    }
                    
                    ui::display_response(&full_response, format);
                }
            }
            Err(e) => {
                ui::display_error(&e.to_string());
            }
        }
    } else {
        // Non-streaming mode
        let spinner = ui::create_spinner("Processing query...");

        match client.chat(&message).await {
            Ok(response) => {
                spinner.finish_and_clear();
                ui::display_response(&response, format);
            }
            Err(e) => {
                spinner.finish_and_clear();
                ui::display_error(&e.to_string());
            }
        }
    }

    Ok(())
}

/// Run configuration command
async fn run_config_command(
    mut config: Config,
    show: bool,
    api_key: Option<String>,
    model: Option<String>,
    system_prompt: Option<String>,
    base_url: Option<String>,
    api_path: Option<String>,
) -> anyhow::Result<()> {
    if show {
        println!("{:#?}", config);
        println!("Full API URL: {}", config.api_url());
        return Ok(());
    }

    let mut modified = false;

    if let Some(key) = api_key {
        config.api_key = Some(key);
        modified = true;
        println!("API key updated");
    }

    if let Some(model) = model {
        config.model = model;
        modified = true;
        println!("Default model updated");
    }

    if let Some(prompt) = system_prompt {
        config.system_prompt = prompt;
        modified = true;
        println!("System prompt updated");
    }
    
    if let Some(url) = base_url {
        config.base_url = url;
        modified = true;
        println!("Base URL updated");
    }
    
    if let Some(path) = api_path {
        config.api_path = path;
        modified = true;
        println!("API path updated");
    }

    if modified {
        config.save().await?;
        println!("Configuration saved");
    } else {
        println!("No changes made");
    }

    Ok(())
}

/// List available models
async fn list_models(config: Config) -> anyhow::Result<()> {
    println!("Fetching available models from {}...\n", config.base_url);
    
    let client = api::OpenAIClient::new(config)?;
    
    match client.list_models().await {
        Ok(models) => {
            if models.is_empty() {
                println!("No models available");
            } else {
                println!("Available models:");
                for model in models {
                    println!("  - {}", model);
                }
            }
        }
        Err(e) => {
            // Fallback to showing common models if API doesn't support listing
            println!("Could not fetch models from API: {}", e);
            println!("\nCommon OpenAI models:");
            println!("  - gpt-4o");
            println!("  - gpt-4o-mini");
            println!("  - gpt-4-turbo");
            println!("  - gpt-4");
            println!("  - gpt-3.5-turbo");
            println!("\nFor LM Studio, check the loaded model in the LM Studio interface.");
        }
    }
    
    Ok(())
}

/// Show usage statistics
async fn show_stats() -> anyhow::Result<()> {
    println!("Token usage statistics are tracked per session in chat mode.");
    println!("Use the 'save' command in chat mode to persist session data.");
    Ok(())
}

/// Display session history
fn display_history(session: &session::Session) {
    println!("\n{}", "Session History:".cyan().bold());
    println!("{}", "─".repeat(60));

    for message in session.history() {
        let role = match message.role {
            api::Role::System => continue, // Skip system messages in display
            api::Role::User => "You".green(),
            api::Role::Assistant => "Assistant".blue(),
        };

        println!("\n{}:", role.bold());
        println!(); // Add space between role and content
        // Use the new markdown processing for better table display
        let processed = ui::process_markdown_content(&message.content);
        println!("{}", processed);
    }

    println!("\n{}", "─".repeat(60));
}

/// Save the current session
async fn save_session(session: &session::Session) -> anyhow::Result<()> {
    match session.save(None).await {
        Ok(path) => {
            println!("Session saved to: {}", path.display());
        }
        Err(e) => {
            ui::display_error(&format!("Failed to save session: {}", e));
        }
    }
    Ok(())
}
