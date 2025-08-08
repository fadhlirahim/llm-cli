//! Test streaming markdown rendering

#[cfg(test)]
mod tests {
    use llm_cli::streaming_buffer::StreamingBuffer;
    
    #[test]
    fn test_streaming_lists() {
        let mut buffer = StreamingBuffer::new();
        
        // Test bullet points
        let (output1, _, _) = buffer.process_chunk("Here are some features:\n");
        assert!(!output1.is_empty());
        
        let (output2, _, _) = buffer.process_chunk("- First item\n");
        assert!(!output2.is_empty());
        assert!(output2.contains("First item"));
        
        let (output3, _, _) = buffer.process_chunk("- Second item\n");
        assert!(!output3.is_empty());
        assert!(output3.contains("Second item"));
        
        let (output4, _, _) = buffer.process_chunk("- Third item\n");
        assert!(!output4.is_empty());
        assert!(output4.contains("Third item"));
    }
    
    #[test]
    fn test_streaming_numbered_list() {
        let mut buffer = StreamingBuffer::new();
        
        let (output1, _, _) = buffer.process_chunk("Steps to follow:\n");
        assert!(!output1.is_empty());
        
        let (output2, _, _) = buffer.process_chunk("1. First step\n");
        assert!(!output2.is_empty());
        assert!(output2.contains("First step"));
        
        let (output3, _, _) = buffer.process_chunk("2. Second step\n");
        assert!(!output3.is_empty());
        assert!(output3.contains("Second step"));
    }
    
    #[test]
    fn test_streaming_mixed_markdown() {
        let mut buffer = StreamingBuffer::new();
        
        // Test mixed content with markdown
        let (output1, _, _) = buffer.process_chunk("## Header\n");
        assert!(!output1.is_empty());
        
        let (output2, _, _) = buffer.process_chunk("Some **bold** text\n");
        assert!(!output2.is_empty());
        
        let (output3, _, _) = buffer.process_chunk("- A list item\n");
        assert!(!output3.is_empty());
        
        // Start a code block
        let (output4, special, buffering) = buffer.process_chunk("```python\n");
        assert!(output4.is_empty()); // Should buffer
        assert!(buffering);
        
        let (output5, special, buffering) = buffer.process_chunk("print('hello')\n");
        assert!(output5.is_empty()); // Still buffering
        assert!(buffering);
        
        let (output6, special, buffering) = buffer.process_chunk("```\n");
        assert!(output6.is_empty());
        assert!(special.is_some()); // Code block complete
        assert!(!buffering);
    }
}