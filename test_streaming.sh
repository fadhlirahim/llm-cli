#!/bin/bash

# Test streaming functionality

echo "Testing streaming in query mode..."
./target/debug/llm-cli query "Tell me a short joke" --stream

echo ""
echo "Testing streaming in chat mode with initial message..."
echo "exit" | ./target/debug/llm-cli chat "What is 2+2?" --stream

echo ""
echo "Streaming test complete!"