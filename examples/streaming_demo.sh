#!/bin/bash

# Demo script to showcase streaming functionality

echo "==================================="
echo "  LLM CLI Streaming Demo"
echo "==================================="
echo ""

# Build the project if needed
if [ ! -f "./target/release/llm-cli" ]; then
    echo "Building the project..."
    cargo build --release
fi

echo "1. Non-streaming query (traditional mode):"
echo "Command: ./target/release/llm-cli query \"Tell me a haiku about coding\""
echo ""
./target/release/llm-cli query "Tell me a haiku about coding"

echo ""
echo "-----------------------------------"
echo ""

echo "2. Streaming query (real-time output):"
echo "Command: ./target/release/llm-cli query \"Tell me a haiku about coding\" --stream"
echo ""
./target/release/llm-cli query "Tell me a haiku about coding" --stream

echo ""
echo "-----------------------------------"
echo ""

echo "3. Interactive chat with streaming:"
echo "Command: ./target/release/llm-cli chat --stream"
echo ""
echo "Try typing 'What is Rust?' and watch the response stream in real-time!"
echo "Type 'exit' to quit the chat."
echo ""
echo "Press Enter to start chat mode with streaming..."
read -r

./target/release/llm-cli chat --stream