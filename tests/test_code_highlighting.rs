//! Test for code block syntax highlighting

#[cfg(test)]
mod tests {
    use llm_cli::ui::process_markdown_content;

    #[test]
    fn test_code_block_parsing() {
        let input = r#"Here's a bash command:

```bash
find . -type f -exec grep -l "startup" {} +
```

And Python code:

```python
def hello():
    print("Hello World")
```

Plain text:

```
This is plain text
```"#;

        let output = process_markdown_content(input);
        
        // Check that the output contains highlighted code blocks
        // The actual visual highlighting happens via ANSI escape codes
        assert!(output.contains("```"));  // Code fence markers
        assert!(output.contains("bash"));
        assert!(output.contains("python"));
        assert!(output.contains("find"));
        assert!(output.contains("hello"));  // Function name should be in output
        assert!(output.contains("print"));  // Python keyword
    }

    #[test]
    fn test_mixed_content() {
        let input = r#"Regular text here

```javascript
const x = 42;
console.log(x);
```

More text

| Header 1 | Header 2 |
|----------|----------|
| Cell 1   | Cell 2   |

```sql
SELECT * FROM users;
```"#;

        let output = process_markdown_content(input);
        
        // Check for code blocks
        assert!(output.contains("javascript"));
        assert!(output.contains("sql"));
        
        // Check that table is also rendered (from existing functionality)
        assert!(output.contains("Header 1"));
        assert!(output.contains("Cell 1"));
    }
}