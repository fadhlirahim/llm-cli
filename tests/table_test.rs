#[cfg(test)]
mod table_tests {
    use llm_cli::ui;

    #[test]
    fn test_markdown_table_rendering() {
        let markdown = r#"Here's a table:

| Column 1 | Column 2 | Column 3 |
|----------|----------|----------|
| Data 1   | Data 2   | Data 3   |
| Row 2    | Value 2  | Item 2   |"#;

        let result = ui::process_markdown_content(markdown);
        
        // Should contain table characters
        assert!(result.contains("│"));
        assert!(result.contains("┌"));
        assert!(result.contains("└"));
        
        // Should contain data
        assert!(result.contains("Column 1"));
        assert!(result.contains("Data 1"));
        assert!(result.contains("Row 2"));
    }

    #[test]
    fn test_mixed_content() {
        let markdown = r#"Some text before.

| Header 1 | Header 2 |
|----------|----------|
| Value 1  | Value 2  |

Some text after."#;

        let result = ui::process_markdown_content(markdown);
        
        // Should contain both table and regular text
        assert!(result.contains("Some text before"));
        assert!(result.contains("Some text after"));
        assert!(result.contains("│"));
        assert!(result.contains("Header 1"));
    }
}