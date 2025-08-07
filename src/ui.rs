//! User interface components and interactions

use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Editor, Input};
use indicatif::{ProgressBar, ProgressStyle};
use std::io::{self, Write};

/// Display a welcome message
pub fn show_welcome() {
    println!("{}", "╔══════════════════════════════════════╗".cyan());
    println!(
        "{}",
        "║         LLM CLI - Chat Mode          ║".cyan().bold()
    );
    println!("{}", "╚══════════════════════════════════════╝".cyan());
    println!();
    println!("{}", "Type 'exit' or 'quit' to end the session".dimmed());
    println!("{}", "Type 'clear' to clear the screen".dimmed());
    println!("{}", "Type 'help' for more commands".dimmed());
    println!();
}

/// Get user input with a prompt
pub fn get_input(prompt: &str) -> io::Result<String> {
    Input::<String>::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .interact_text()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
}

/// Get multiline input
pub fn get_multiline_input() -> io::Result<String> {
    println!(
        "{}",
        "Enter your message (press Ctrl+D when done):".dimmed()
    );
    Editor::new()
        .edit("")
        .map(|s| s.unwrap_or_default())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
}

/// Display a response
pub fn display_response(response: &str, format: crate::cli::OutputFormat) {
    match format {
        crate::cli::OutputFormat::Text => {
            println!("\n{}", "Assistant:".green().bold());
            println!("{}", response);
        }
        crate::cli::OutputFormat::Json => {
            let json = serde_json::json!({
                "response": response,
                "timestamp": chrono::Utc::now().to_rfc3339(),
            });
            println!("{}", serde_json::to_string_pretty(&json).unwrap());
        }
        crate::cli::OutputFormat::Markdown => {
            println!("\n{}", "```markdown".dimmed());
            println!("{}", response);
            println!("{}", "```".dimmed());
        }
    }
    println!();
}

/// Display an error message
pub fn display_error(error: &str) {
    eprintln!("{} {}", "Error:".red().bold(), error);
}

/// Create a spinner for loading states
pub fn create_spinner(message: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    pb.set_message(message.to_string());
    pb.enable_steady_tick(std::time::Duration::from_millis(100));
    pb
}

/// Clear the terminal screen
pub fn clear_screen() {
    print!("\x1B[2J\x1B[1;1H");
    io::stdout().flush().unwrap();
}

/// Display help information
pub fn show_help() {
    println!("{}", "Available Commands:".yellow().bold());
    println!("  {}  - End the chat session", "exit/quit".cyan());
    println!("  {}      - Clear the screen", "clear".cyan());
    println!("  {}       - Show this help message", "help".cyan());
    println!("  {}    - Show current session history", "history".cyan());
    println!("  {}      - Save conversation to file", "save".cyan());
    println!("  {}   - Change the model", "model <name>".cyan());
    println!();
}
