//! Streaming buffer for handling markdown content with tables

/// Buffer for streaming content that can detect and format tables
pub struct StreamingBuffer {
    /// Current incomplete line being built from chunks
    current_line: String,
    /// Whether we're currently inside a potential table
    in_table: bool,
    /// Buffer for table lines while building
    table_buffer: Vec<String>,
    /// Buffer for accumulating content until we have something meaningful to display
    display_buffer: String,
}

impl StreamingBuffer {
    /// Create a new streaming buffer
    pub fn new() -> Self {
        Self {
            current_line: String::new(),
            in_table: false,
            table_buffer: Vec::new(),
            display_buffer: String::new(),
        }
    }

    /// Process a chunk of streaming text
    /// Returns (text_to_display, formatted_table_if_complete, is_buffering_table)
    pub fn process_chunk(&mut self, chunk: &str) -> (String, Option<String>, bool) {
        // Debug: Log what we receive
        if std::env::var("DEBUG_STREAMING").is_ok() {
            eprintln!("[BUFFER] Received chunk: {:?} (len: {})", chunk, chunk.len());
        }
        
        let mut output = String::new();
        let mut table_output = None;

        // Add chunk to display buffer
        self.display_buffer.push_str(chunk);

        // Process any complete lines in the buffer
        while let Some(newline_pos) = self.display_buffer.find('\n') {
            // Extract the complete line (including the newline)
            let line = self.display_buffer.drain(..=newline_pos).collect::<String>();
            let line_content = line.trim_end_matches('\n');
            
            // Combine with any partial line we had
            let complete_line = if !self.current_line.is_empty() {
                let result = self.current_line.clone() + line_content;
                self.current_line.clear();
                result
            } else {
                line_content.to_string()
            };
            
            let (immediate_output, table) = self.process_complete_line(complete_line);
            
            if !immediate_output.is_empty() {
                if !output.is_empty() {
                    output.push('\n');
                }
                output.push_str(&immediate_output);
            }
            
            if let Some(t) = table {
                table_output = Some(t);
            }
        }

        // Handle remaining partial content
        if !self.display_buffer.is_empty() && !self.in_table {
            // Check if this might be the start of a table
            let combined = self.current_line.clone() + &self.display_buffer;
            if !self.looks_like_table_start(&combined) {
                // Not a table, output what we have
                let mut to_output = String::new();
                
                if !self.current_line.is_empty() {
                    to_output.push_str(&self.current_line);
                    self.current_line.clear();
                }
                
                // Simply append the display buffer as-is to preserve spacing
                to_output.push_str(&self.display_buffer);
                self.display_buffer.clear();
                
                if !output.is_empty() {
                    output.push_str(&to_output);
                } else {
                    output = to_output;
                }
            } else {
                // Might be a table start, keep accumulating
                self.current_line.push_str(&self.display_buffer);
                self.display_buffer.clear();
            }
        } else if !self.display_buffer.is_empty() && self.in_table {
            // Currently buffering a table, accumulate
            self.current_line.push_str(&self.display_buffer);
            self.display_buffer.clear();
        }
        
        // Debug: Log what we're outputting
        if std::env::var("DEBUG_STREAMING").is_ok() && !output.is_empty() {
            eprintln!("[BUFFER] Outputting: {:?}", output);
        }

        (output, table_output, self.in_table)
    }

    /// Process a complete line
    fn process_complete_line(&mut self, line: String) -> (String, Option<String>) {
        // Check if this line is a table row
        if self.is_table_row(&line) {
            if !self.in_table {
                // Start buffering table
                self.in_table = true;
                self.table_buffer.clear();
            }
            self.table_buffer.push(line);
            (String::new(), None)
        } else if self.in_table {
            // We were in a table but this line isn't a table row
            // Table is complete, format and return it
            self.in_table = false;
            let table = self.format_buffered_table();
            (line, Some(table))
        } else {
            // Regular line, not in a table
            (line, None)
        }
    }

    /// Check if a line looks like a markdown table row
    fn is_table_row(&self, line: &str) -> bool {
        let trimmed = line.trim();
        trimmed.starts_with('|') && trimmed.ends_with('|') && trimmed.contains('|')
    }

    /// Check if a partial line might be the start of a table
    fn looks_like_table_start(&self, partial: &str) -> bool {
        partial.trim().starts_with('|')
    }

    /// Check if a line is a markdown table separator
    fn is_table_separator(&self, line: &str) -> bool {
        let trimmed = line.trim();
        if !self.is_table_row(trimmed) {
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

    /// Format the buffered table
    fn format_buffered_table(&mut self) -> String {
        if self.table_buffer.is_empty() {
            return String::new();
        }

        // Parse the table
        let table_data = self.parse_table_buffer();
        
        // Format using tabled
        if let Some(data) = table_data {
            self.render_table(data)
        } else {
            // If parsing fails, return original lines
            let result = self.table_buffer.join("\n");
            self.table_buffer.clear();
            result
        }
    }

    /// Parse buffered table lines into structured data
    fn parse_table_buffer(&mut self) -> Option<Vec<Vec<String>>> {
        if self.table_buffer.len() < 2 {
            return None;
        }
        
        let mut table_data = Vec::new();
        
        for line in &self.table_buffer {
            if !self.is_table_row(line) {
                continue;
            }
            
            // Skip separator rows
            if self.is_table_separator(line) {
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
        
        self.table_buffer.clear();
        
        if table_data.is_empty() {
            None
        } else {
            Some(table_data)
        }
    }

    /// Render a parsed table using tabled
    fn render_table(&self, table_data: Vec<Vec<String>>) -> String {
        use tabled::{
            builder::Builder,
            settings::{Style, Width, object::Rows, Modify, Alignment},
        };
        
        if table_data.is_empty() {
            return String::new();
        }
        
        let mut builder = Builder::default();
        
        // Add all rows to the builder
        for row in table_data {
            builder.push_record(row);
        }
        
        let terminal_width = terminal_size::terminal_size()
            .map(|(width, _)| width.0 as usize)
            .unwrap_or(80)
            .saturating_sub(8); // Account for margins
        
        // Build and style the table
        let mut table = builder.build();
        table
            .with(Style::modern())
            .with(Width::wrap(terminal_width))
            .with(Width::increase(terminal_width))
            .with(Modify::new(Rows::first()).with(Alignment::center()));
        
        table.to_string()
    }

    /// Check if currently buffering a table
    pub fn is_buffering_table(&self) -> bool {
        self.in_table
    }
    
    /// Flush any remaining content
    pub fn flush(&mut self) -> Option<String> {
        let mut output = String::new();
        
        // Flush current line if any
        if !self.current_line.is_empty() {
            output.push_str(&self.current_line);
            self.current_line.clear();
        }
        
        // Flush table buffer if any
        if self.in_table && !self.table_buffer.is_empty() {
            if !output.is_empty() {
                output.push('\n');
            }
            output.push_str(&self.format_buffered_table());
        }
        
        if output.is_empty() {
            None
        } else {
            Some(output)
        }
    }
}

impl Default for StreamingBuffer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_text_streaming() {
        let mut buffer = StreamingBuffer::new();
        
        // Test proper space handling
        let (output, table, buffering) = buffer.process_chunk("Hello");
        assert_eq!(output, "Hello");
        assert!(table.is_none());
        assert!(!buffering);
        
        let (output, table, buffering) = buffer.process_chunk(" world!");
        assert_eq!(output, " world!");
        assert!(table.is_none());
        assert!(!buffering);
        
        // Newline completes the line
        let (output, table, buffering) = buffer.process_chunk("\n");
        assert_eq!(output, "");
        assert!(table.is_none());
        assert!(!buffering);
    }

    #[test]
    fn test_table_detection() {
        let mut buffer = StreamingBuffer::new();
        
        // Start of table
        let (output, table, buffering) = buffer.process_chunk("| Header 1 | Header 2 |\n");
        assert_eq!(output, "");
        assert!(table.is_none());
        assert!(buffering);
        
        // Separator
        let (output, table, buffering) = buffer.process_chunk("|----------|----------|\n");
        assert_eq!(output, "");
        assert!(table.is_none());
        assert!(buffering);
        
        // Data row
        let (output, table, buffering) = buffer.process_chunk("| Data 1   | Data 2   |\n");
        assert_eq!(output, "");
        assert!(table.is_none());
        assert!(buffering);
        
        // Non-table line triggers table rendering
        let (output, table, buffering) = buffer.process_chunk("Regular text\n");
        assert_eq!(output, "Regular text");
        assert!(table.is_some());
        assert!(!buffering);
    }

    #[test]
    fn test_mixed_content() {
        let mut buffer = StreamingBuffer::new();
        
        let (output, _, _) = buffer.process_chunk("Some text before\n");
        assert!(output.contains("Some text before"));
        
        let (output, _, buffering) = buffer.process_chunk("| Col1 | Col2 |\n");
        assert_eq!(output, "");
        assert!(buffering);
        
        let (output, _, _) = buffer.process_chunk("|------|------|\n");
        assert_eq!(output, "");
        
        let (output, _, _) = buffer.process_chunk("| A    | B    |\n");
        assert_eq!(output, "");
        
        let (output, table, buffering) = buffer.process_chunk("Text after table\n");
        assert_eq!(output, "Text after table");
        assert!(table.is_some());
        assert!(!buffering);
    }
}