#!/bin/bash

# LM Studio Quick Setup Script for LLM CLI
# This script configures the LLM CLI to work with LM Studio

echo "🤖 LM Studio Setup for LLM CLI"
echo "==============================="
echo ""

# Check if LM Studio is running
if curl -s http://localhost:1234/v1/models > /dev/null 2>&1; then
    echo "✅ LM Studio server detected on port 1234"
    
    # Try to get model list
    echo ""
    echo "Available models in LM Studio:"
    curl -s http://localhost:1234/v1/models | jq -r '.data[].id' 2>/dev/null || echo "  (Could not list models - make sure a model is loaded in LM Studio)"
else
    echo "⚠️  LM Studio server not detected on port 1234"
    echo "   Please start LM Studio and load a model first"
    exit 1
fi

echo ""
echo "Setting up configuration..."

# Set environment variables
export OPENAI_API_KEY="lm-studio"
export OPENAI_BASE_URL="http://localhost:1234"
export OPENAI_API_PATH="/v1/chat/completions"

# Also save to config using the CLI
./target/release/llm-cli config \
    --api-key "lm-studio" \
    --base-url "http://localhost:1234" \
    --api-path "/v1/chat/completions" \
    --model "local-model" 2>/dev/null

if [ $? -eq 0 ]; then
    echo "✅ Configuration saved successfully"
else
    echo "⚠️  Could not save configuration (make sure the CLI is built)"
fi

echo ""
echo "Configuration complete! You can now use:"
echo "  ./target/release/llm-cli chat        # For interactive chat"
echo "  ./target/release/llm-cli query '...' # For single queries"
echo "  ./target/release/llm-cli models      # To list available models"
echo ""
echo "Note: Make sure you have loaded a model in LM Studio before using the CLI"