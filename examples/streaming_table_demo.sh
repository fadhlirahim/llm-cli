#!/bin/bash

# Demo script to showcase streaming with table formatting

echo "========================================="
echo "  LLM CLI - Streaming Tables Demo"
echo "========================================="
echo ""
echo "This demo shows how tables are properly formatted"
echo "even when responses are streamed in real-time."
echo ""

# Build if needed
if [ ! -f "./target/release/llm-cli" ]; then
    echo "Building the project..."
    cargo build --release
fi

echo "1. Query with table (streaming mode):"
echo "======================================"
echo ""
echo "Command: ./target/release/llm-cli query \"Create a markdown table comparing Python and Rust in terms of speed, memory usage, and learning curve\" --stream"
echo ""
echo "Watch as the table is buffered and formatted properly:"
echo ""

./target/release/llm-cli query "Create a markdown table comparing Python and Rust in terms of speed, memory usage, and learning curve" --stream

echo ""
echo "-----------------------------------"
echo ""

echo "2. Complex query with mixed content (streaming):"
echo "================================================"
echo ""
echo "Command: ./target/release/llm-cli query \"Explain the differences between TCP and UDP, then create a comparison table\" --stream"
echo ""

./target/release/llm-cli query "Explain the differences between TCP and UDP. Then create a comparison table with columns for Feature, TCP, and UDP. Include rows for reliability, speed, use cases, and overhead." --stream

echo ""
echo "-----------------------------------"
echo ""

echo "Demo complete!"
echo ""
echo "Notice how:"
echo "- Regular text streams immediately character by character"
echo "- Tables are buffered until complete, then formatted nicely"
echo "- The formatted table appears with proper alignment and borders"
echo "- Mixed content (text + tables) is handled correctly"