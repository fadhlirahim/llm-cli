//! Test for hybrid markdown rendering

#[cfg(test)]
mod tests {
    use llm_cli::ui::process_markdown_content;

    #[test]
    fn test_full_markdown_features() {
        let input = r#"# Header 1
## Header 2

This is **bold text** and this is *italic text*.

Here's a list:
- Item 1
- Item 2
- Item 3

> This is a blockquote
> with multiple lines

And a numbered list:
1. First item
2. Second item
3. Third item

Here's some `inline code` in the text.

```rust
fn main() {
    println!("Hello from Rust!");
}
```

| Column 1 | Column 2 |
|----------|----------|
| Data 1   | Data 2   |
| Data 3   | Data 4   |

```python
def hello():
    print("Hello from Python!")
```

Regular paragraph after everything."#;

        let output = process_markdown_content(input);
        
        // Check that various markdown elements are processed
        assert!(output.contains("Header 1")); // Headers preserved
        assert!(output.contains("bold text")); // Bold text
        assert!(output.contains("italic text")); // Italic text
        assert!(output.contains("Item 1")); // Lists
        assert!(output.contains("blockquote")); // Blockquotes
        assert!(output.contains("inline code")); // Inline code
        
        // Check that code blocks are highlighted (with syntect)
        assert!(output.contains("rust")); // Language indicator
        assert!(output.contains("python")); // Language indicator
        assert!(output.contains("```")); // Code fences
        
        // Check that tables are rendered (with tabled)
        assert!(output.contains("Column 1")); // Table headers
        assert!(output.contains("Data 1")); // Table data
    }
    
    #[test]
    fn test_mixed_content_rendering() {
        let input = r#"## Introduction

This is a **markdown** document with:
- Code blocks
- Tables
- Formatting

```bash
echo "Hello World"
ls -la
```

| Feature | Status |
|---------|--------|
| Code    | ✓      |
| Tables  | ✓      |

*End of document*"#;

        let output = process_markdown_content(input);
        
        // Verify all elements are rendered
        assert!(output.contains("Introduction"));
        assert!(output.contains("markdown"));
        assert!(output.contains("bash"));
        assert!(output.contains("Feature"));
        assert!(output.contains("✓"));
        assert!(output.contains("End of document"));
    }
}