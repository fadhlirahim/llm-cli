//! User interface components and interactions

use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Editor, Input};
use indicatif::{ProgressBar, ProgressStyle};
use std::io::{self, Write};
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use syntect::util::{as_24_bit_terminal_escaped, LinesWithEndings};
use tabled::{
    builder::Builder,
    settings::{Style, Width, object::Rows, Modify, Alignment},
};
use termimad::{MadSkin, FmtText, minimad::TextTemplate};
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

/// Parse and highlight a code block
pub fn highlight_code_block(code: &str, language: &str) -> String {
    // Load syntax definitions and themes
    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();
    
    // Try to find the syntax for the given language
    let syntax = ps.find_syntax_by_token(language)
        .or_else(|| ps.find_syntax_by_extension(language))
        .unwrap_or_else(|| ps.find_syntax_plain_text());
    
    // Use a dark theme that works well in terminals
    let theme = &ts.themes["base16-ocean.dark"];
    
    let mut highlighter = HighlightLines::new(syntax, theme);
    let mut highlighted = String::new();
    
    // Add simple language indicator
    highlighted.push_str(&format!("\n  {} {}\n", "```".dimmed(), language.cyan()));
    
    // Highlight each line without box borders
    for line in LinesWithEndings::from(code) {
        let ranges = highlighter.highlight_line(line, &ps).unwrap_or_default();
        let escaped = as_24_bit_terminal_escaped(&ranges[..], false);
        highlighted.push_str(&format!("  {}", escaped));
    }
    
    // Add closing fence on its own line
    highlighted.push_str(&format!("\n  {}\n", "```".dimmed()));
    
    highlighted
}

/// Create a termimad skin for markdown rendering
fn create_markdown_skin() -> MadSkin {
    let mut skin = MadSkin::default();
    
    // Customize the skin for better terminal display
    skin.set_headers_fg(termimad::crossterm::style::Color::Cyan);
    skin.bold.set_fg(termimad::crossterm::style::Color::Yellow);
    skin.italic.set_fg(termimad::crossterm::style::Color::Magenta);
    skin.strikeout.add_attr(termimad::crossterm::style::Attribute::CrossedOut);
    skin.inline_code.set_fg(termimad::crossterm::style::Color::Green);
    skin.quote_mark.set_fg(termimad::crossterm::style::Color::DarkGrey);
    
    skin
}

/// Process a single line of markdown for streaming output
pub fn process_markdown_line(line: &str) -> String {
    // Quick check for lines that don't need processing
    if line.trim().is_empty() {
        return "\n".to_string();
    }
    
    // Check if this is a list item (bullet or numbered)
    let trimmed = line.trim();
    let is_list_item = trimmed.starts_with("- ") || 
                       trimmed.starts_with("* ") ||
                       trimmed.starts_with("+ ") ||
                       trimmed.chars().next().map_or(false, |c| c.is_ascii_digit() && 
                           trimmed.chars().nth(1).map_or(false, |c2| c2 == '.'));
    
    // Use termimad to process the line
    let skin = create_markdown_skin();
    let terminal_width = get_terminal_width();
    let rendered = FmtText::from(&skin, line, Some(terminal_width));
    
    // Add indentation and return with newline
    let output = rendered.to_string();
    if output.is_empty() {
        "\n".to_string()
    } else {
        // Always add newline for proper line separation
        format!("  {}\n", output)
    }
}

/// Process text and render markdown with hybrid approach
/// Uses termimad for general markdown, syntect for code blocks, and tabled for tables
pub fn process_markdown_content(text: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let mut result = Vec::new();
    let mut i = 0;
    
    while i < lines.len() {
        // Check if this line starts a code block
        if lines[i].trim().starts_with("```") {
            let fence_line = lines[i].trim();
            let language = fence_line
                .strip_prefix("```")
                .unwrap_or("")
                .trim()
                .to_string();
            
            // Collect all lines until the closing fence
            let mut code_lines = Vec::new();
            let mut j = i + 1;
            
            while j < lines.len() && !lines[j].trim().starts_with("```") {
                code_lines.push(lines[j]);
                j += 1;
            }
            
            // Skip the closing fence if found
            if j < lines.len() && lines[j].trim().starts_with("```") {
                j += 1;
            }
            
            // Join the code lines and highlight them with syntect
            let code = code_lines.join("\n");
            if !code.trim().is_empty() {
                let lang = if language.is_empty() { "text" } else { &language };
                // Use our syntect-based highlighter for code blocks
                result.push(highlight_code_block(&code, lang));
            }
            
            i = j;
            continue;
        }
        
        // Check if this line starts a table
        if is_table_row(lines[i]) {
            // Collect all consecutive table lines
            let mut table_lines = vec![lines[i]];
            let mut j = i + 1;
            
            while j < lines.len() && is_table_row(lines[j]) {
                table_lines.push(lines[j]);
                j += 1;
            }
            
            // Try to parse and render the table with tabled
            if let Some(table_data) = parse_markdown_table(&table_lines) {
                // Use our tabled-based renderer for tables
                result.push(render_table(table_data));
                i = j;
                continue;
            }
        }
        
        // Collect consecutive non-table, non-code lines for termimad processing
        let mut markdown_lines = Vec::new();
        while i < lines.len() 
            && !lines[i].trim().starts_with("```") 
            && !is_table_row(lines[i]) {
            markdown_lines.push(lines[i]);
            i += 1;
        }
        
        // Process these lines with termimad for general markdown rendering
        if !markdown_lines.is_empty() {
            let markdown_text = markdown_lines.join("\n");
            let skin = create_markdown_skin();
            
            // Render with termimad and add proper indentation
            let terminal_width = get_terminal_width();
            let rendered = FmtText::from(&skin, &markdown_text, Some(terminal_width));
            
            // Add indentation to match our style
            let indented: String = rendered.to_string()
                .lines()
                .map(|line| {
                    if line.is_empty() {
                        String::new()
                    } else {
                        format!("  {}", line)
                    }
                })
                .collect::<Vec<_>>()
                .join("\n");
            
            result.push(indented);
        }
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

/// Display streaming response header
pub fn display_streaming_header() {
    println!(); // Add vertical space before response
    println!("{}", "Assistant:".green().bold());
    println!(); // Space between label and content
    print!("  "); // Initial indent for content
    io::stdout().flush().unwrap();
}

/// Display a streaming chunk with smart indentation
pub fn display_streaming_chunk_smart(chunk: &str, needs_indent: bool) {
    // For streaming, display text exactly as it arrives
    // No manipulation that could introduce spacing issues
    
    if chunk.is_empty() {
        return;
    }
    
    // Debug: Log what we're about to display
    if std::env::var("DEBUG_STREAMING").is_ok() {
        eprintln!("[DISPLAY] About to print: {:?} (needs_indent: {})", chunk, needs_indent);
    }
    
    // Handle initial indentation
    if needs_indent {
        print!("  ");
    }
    
    // Print the chunk exactly as received, handling newlines
    for ch in chunk.chars() {
        if ch == '\n' {
            println!();
            print!("  "); // Indent the next line
        } else {
            print!("{}", ch);
        }
    }
    
    io::stdout().flush().unwrap();
}

/// Display a formatted table during streaming
pub fn display_streaming_table(table: &str) {
    // Tables are already formatted, just add proper indentation
    for line in table.lines() {
        println!("  {}", line);
    }
    io::stdout().flush().unwrap();
}

/// Finish streaming display
pub fn finish_streaming_display() {
    println!(); // Final newline
    println!(); // Add vertical space after response
}
