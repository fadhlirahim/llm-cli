//! User interface components and interactions

use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Editor, Input};
use indicatif::{ProgressBar, ProgressStyle};
use std::io::{self, Write};
use tabled::{
    builder::Builder,
    settings::{Style, Width, object::Rows, Modify, Alignment},
};
use textwrap::{wrap, Options};

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

/// Get terminal width for proper text wrapping with margins
fn get_terminal_width() -> usize {
    let full_width = terminal_size::terminal_size()
        .map(|(width, _)| width.0 as usize)
        .unwrap_or(80); // Default to 80 if we can't detect terminal size
    
    // Add left and right margins (4 chars each side = 8 total)
    let margin = 8;
    if full_width > margin {
        full_width - margin
    } else {
        full_width.saturating_sub(4) // Minimum margin if terminal is very narrow
    }
}

/// Wrap text to fit terminal width with margins
pub fn wrap_text(text: &str) -> String {
    let width = get_terminal_width();
    let options = Options::new(width)
        .break_words(false) // Don't break words
        .wrap_algorithm(textwrap::WrapAlgorithm::FirstFit);
    
    let left_margin = "  "; // 2 spaces left margin
    
    let lines: Vec<String> = text
        .lines()
        .flat_map(|line| {
            if line.trim().is_empty() {
                vec![String::new()]
            } else {
                wrap(line, &options)
                    .into_iter()
                    .map(|cow| format!("{}{}", left_margin, cow))
                    .collect::<Vec<_>>()
            }
        })
        .collect();
    
    lines.join("\n")
}

/// Check if a line looks like a markdown table row
fn is_table_row(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with('|') && trimmed.ends_with('|') && trimmed.contains('|')
}

/// Check if a line is a markdown table separator
fn is_table_separator(line: &str) -> bool {
    let trimmed = line.trim();
    if !is_table_row(trimmed) {
        return false;
    }
    
    // Remove leading and trailing pipes and split
    let content = trimmed.trim_start_matches('|').trim_end_matches('|');
    content.split('|').all(|cell| {
        let cell = cell.trim();
        cell.chars().all(|c| c == '-' || c == ':' || c == ' ')
            && cell.contains('-')
    })
}

/// Parse a markdown table from text lines
fn parse_markdown_table(lines: &[&str]) -> Option<Vec<Vec<String>>> {
    if lines.len() < 2 {
        return None;
    }
    
    let mut table_data = Vec::new();
    
    for line in lines.iter() {
        if !is_table_row(line) {
            continue;
        }
        
        // Skip separator rows
        if is_table_separator(line) {
            continue;
        }
        
        // Parse cells from the row
        let cells: Vec<String> = line
            .trim()
            .trim_start_matches('|')
            .trim_end_matches('|')
            .split('|')
            .map(|cell| cell.trim().to_string())
            .collect();
        
        table_data.push(cells);
    }
    
    if table_data.is_empty() {
        None
    } else {
        Some(table_data)
    }
}

/// Render a parsed table using tabled
fn render_table(table_data: Vec<Vec<String>>) -> String {
    if table_data.is_empty() {
        return String::new();
    }
    
    let mut builder = Builder::default();
    
    // Add all rows to the builder
    for row in table_data {
        builder.push_record(row);
    }
    
    let terminal_width = get_terminal_width();
    
    // Build and style the table
    let mut table = builder.build();
    table
        .with(Style::modern())
        .with(Width::wrap(terminal_width))
        .with(Width::increase(terminal_width))
        .with(Modify::new(Rows::first()).with(Alignment::center()));
    
    format!("  {}", table.to_string().replace('\n', "\n  "))
}

/// Process text and render any markdown tables found
pub fn process_markdown_content(text: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let mut result = Vec::new();
    let mut i = 0;
    
    while i < lines.len() {
        // Check if this line starts a table
        if is_table_row(lines[i]) {
            // Collect all consecutive table lines
            let mut table_lines = vec![lines[i]];
            let mut j = i + 1;
            
            while j < lines.len() && is_table_row(lines[j]) {
                table_lines.push(lines[j]);
                j += 1;
            }
            
            // Try to parse and render the table
            if let Some(table_data) = parse_markdown_table(&table_lines) {
                result.push(render_table(table_data));
                i = j;
                continue;
            }
        }
        
        // Not a table, wrap normally
        if lines[i].trim().is_empty() {
            result.push(String::new());
        } else {
            result.push(wrap_text(lines[i]));
        }
        i += 1;
    }
    
    result.join("\n")
}

/// Display a response
pub fn display_response(response: &str, format: crate::cli::OutputFormat) {
    println!(); // Add vertical space before response
    
    match format {
        crate::cli::OutputFormat::Text => {
            println!("{}", "Assistant:".green().bold());
            println!(); // Space between label and content
            // Process markdown content to render tables properly
            let processed = process_markdown_content(response);
            println!("{}", processed);
        }
        crate::cli::OutputFormat::Json => {
            let json = serde_json::json!({
                "response": response,
                "timestamp": chrono::Utc::now().to_rfc3339(),
            });
            println!("{}", serde_json::to_string_pretty(&json).unwrap());
        }
        crate::cli::OutputFormat::Markdown => {
            println!("{}", "```markdown".dimmed());
            println!(); // Space after markdown marker
            // Also process tables in markdown format for better display
            let processed = process_markdown_content(response);
            println!("{}", processed);
            println!(); // Space before closing marker
            println!("{}", "```".dimmed());
        }
    }
    
    println!(); // Add vertical space after response
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

/// Display wrapped error message
pub fn display_error_wrapped(error: &str) {
    let wrapped = wrap_text(error);
    eprintln!("{} {}", "Error:".red().bold(), wrapped);
}

/// Display streaming response header
pub fn display_streaming_header() {
    println!(); // Add vertical space before response
    println!("{}", "Assistant:".green().bold());
    println!(); // Space between label and content
    print!("  "); // Initial indent for content
    io::stdout().flush().unwrap();
}

/// Display a streaming chunk
pub fn display_streaming_chunk(chunk: &str) {
    // Handle newlines in chunks properly with indentation
    for (i, line) in chunk.split('\n').enumerate() {
        if i > 0 {
            println!(); // New line
            print!("  "); // Indent for new line
        }
        print!("{}", line);
    }
    io::stdout().flush().unwrap();
}

/// Finish streaming display
pub fn finish_streaming_display() {
    println!(); // Final newline
    println!(); // Add vertical space after response
}
