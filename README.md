# LLM CLI

A modern, universal command-line interface for interacting with Large Language Models (OpenAI, LM Studio, Ollama, and more), built with Rust and following 2025 best practices.

## Features

- 🚀 **Interactive Chat Mode**: Engage in conversations with AI models
- 💬 **Single Query Mode**: Get quick responses without entering chat mode
- 🎨 **Multiple Output Formats**: Plain text, JSON, or Markdown
- 📝 **Session Management**: Save and load conversation history
- ⚙️ **Configuration Management**: Persistent settings with environment variable overrides
- 🔐 **Secure API Key Handling**: Safe storage and management of credentials
- 📊 **Token Usage Tracking**: Monitor your API usage
- 🎯 **Multiple Model Support**: Switch between different OpenAI models
- 🔄 **Async/Await Architecture**: Efficient, non-blocking operations
- 🛡️ **Comprehensive Error Handling**: Graceful error recovery with detailed messages

## Installation

### Prerequisites

- Rust 1.75 or later
- Git (for cloning the repository)

### Install from Source

```bash
# 1. Clone the repository
git clone https://github.com/fadhlirahim/llm-cli.git
cd llm-cli

# 2. Build the project
cargo build --release

# 3. (Optional) Install globally
cargo install --path .
# OR copy the binary to your PATH
sudo cp target/release/llm-cli /usr/local/bin/
```

After installation, you can run `llm-cli` from anywhere if installed globally, or use `./target/release/llm-cli` from the project directory.

### Verify Installation

```bash
# If installed globally
llm-cli --version

# If running from project directory
./target/release/llm-cli --version
```

## Quick Start

### For OpenAI Users
```bash
# Set your API key and start chatting
./target/release/llm-cli config --api-key "sk-your-openai-key"
./target/release/llm-cli chat
```

### For LM Studio Users
```bash
# 1. Start LM Studio and load a model
# 2. Configure the CLI for local use
./target/release/llm-cli config --base-url "http://localhost:1234" \
                                --api-key "lm-studio"
# 3. Start chatting
./target/release/llm-cli chat
```

## Configuration

**Note:** You must complete the installation above before running any configuration commands.

### Using with LM Studio

LM Studio provides a local OpenAI-compatible API server. Here's how to configure the CLI to work with LM Studio:

#### 1. Install LLM CLI First
Make sure you've completed the installation steps above. The `llm-cli` command must be available.

#### 2. Start LM Studio Server
Start the LM Studio server on your local machine. By default, it runs on port 1234.
Look for the message: "Success! HTTP server listening on port 1234"

#### 3. Configure the CLI for LM Studio

**Option A: Using Environment Variables**
```bash
export OPENAI_API_KEY="lm-studio"  # LM Studio doesn't require an API key, but we need to set something
export OPENAI_BASE_URL="http://localhost:1234"
export OPENAI_API_PATH="/v1/chat/completions"
export OPENAI_MODEL="local-model"  # Replace with your loaded model name
```

**Option B: Using CLI Commands**
```bash
# If installed globally:
llm-cli config --api-key "lm-studio" \
               --base-url "http://localhost:1234" \
               --api-path "/v1/chat/completions" \
               --model "local-model"

# If running from project directory:
./target/release/llm-cli config --api-key "lm-studio" \
                                --base-url "http://localhost:1234" \
                                --api-path "/v1/chat/completions" \
                                --model "local-model"
```

**Option C: Using Configuration File**
Create or edit `~/.config/llm-cli/config.toml`:
```toml
api_key = "lm-studio"  # LM Studio doesn't require API key, but field is required
model = "local-model"  # Replace with your actual model name in LM Studio
max_tokens = 4096
base_url = "http://localhost:1234"
api_path = "/v1/chat/completions"
system_prompt = "You are a helpful assistant."
timeout_seconds = 60  # Local models might need more time
debug = false
```

#### 4. List Available Models in LM Studio
```bash
# This will show models currently loaded in LM Studio
curl http://localhost:1234/v1/models

# Or use the CLI (after configuration)
llm-cli models
```

#### 5. Start Using the CLI
```bash
# Interactive chat mode
llm-cli chat

# Single query
llm-cli query "What is Rust?"
```

#### Troubleshooting LM Studio

**Issue: "API key not found" error**
- LM Studio doesn't require an API key, but the CLI needs one set. Use any value like "lm-studio"

**Issue: Connection refused**
- Ensure LM Studio server is running (check for "Server listening on port 1234" message)
- Verify the port number matches your configuration (default is 1234)

**Issue: "Model not found" error**
- Make sure you have loaded a model in LM Studio before using the CLI
- Check the model name in LM Studio and update your config accordingly

**Issue: Slow responses**
- Local models can be slower than cloud APIs
- Increase timeout: `llm-cli config --timeout 120`
- Consider using a smaller model or GPU acceleration in LM Studio

### Using with OpenAI API

For standard OpenAI API usage:

### Environment Variables

```bash
export OPENAI_API_KEY="your-api-key-here"
export OPENAI_MODEL="gpt-4o"           # Optional: default model
export OPENAI_MAX_TOKENS="4096"        # Optional: max response tokens
export OPENAI_BASE_URL="https://api.openai.com"  # Optional: custom API endpoint
export OPENAI_API_PATH="/v1/chat/completions"    # Optional: API path
export OPENAI_DEBUG="true"             # Optional: enable debug logging
```

### Configuration File

The CLI stores configuration in `~/.config/llm-cli/config.toml`:

```toml
api_key = "your-api-key"
model = "gpt-4o"
max_tokens = 4096
base_url = "https://api.openai.com"
api_path = "/v1/chat/completions"
system_prompt = "You are a helpful assistant."
timeout_seconds = 30
debug = false
```

## Usage

**Important:** Make sure you have either:
1. Configured the CLI with your OpenAI API key, OR
2. Set up a local LLM server (LM Studio, Ollama, etc.)

### Interactive Chat Mode

Start an interactive chat session:

```bash
# If installed globally
llm-cli chat

# If running from project directory
./target/release/llm-cli chat
```

With initial message:

```bash
llm-cli chat "Hello, how are you?"
```

Enable multiline input:

```bash
llm-cli chat --multiline
```

### Single Query Mode

Get a quick response:

```bash
llm-cli query "What is the capital of France?"
```

With JSON output:

```bash
llm-cli query "List 3 programming languages" --format json
```

### Configuration Management

Show current configuration:

```bash
llm-cli config --show
```

Set API key:

```bash
llm-cli config --api-key "your-new-key"
```

Change default model:

```bash
llm-cli config --model "gpt-4-turbo"
```

Set custom API endpoint (for OpenAI-compatible services):

```bash
llm-cli config --base-url "https://your-api.example.com" --api-path "/v1/chat/completions"
```

### List Available Models

```bash
llm-cli models
```

## Chat Mode Commands

While in chat mode, you can use these special commands:

- `exit` or `quit` - End the chat session
- `clear` - Clear the screen
- `help` - Show available commands
- `history` - Display conversation history
- `save` - Save the current session
- `model <name>` - Switch to a different model

## Architecture

The project follows a modular architecture with clear separation of concerns:

```
src/
├── main.rs       # Application entry point and orchestration
├── api.rs        # OpenAI API client implementation
├── cli.rs        # Command-line interface definitions
├── config.rs     # Configuration management
├── error.rs      # Error types and handling
├── session.rs    # Session and conversation management
├── ui.rs         # User interface components
└── lib.rs        # Library exports
```

## Development

### Running Tests

```bash
# Run all tests
cargo test

# Run with coverage
cargo tarpaulin

# Run specific test
cargo test test_api_client
```

### Code Quality

```bash
# Format code
cargo fmt

# Run linter
cargo clippy -- -D warnings

# Check for security issues
cargo audit
```

### Building for Release

```bash
# Build optimized binary
cargo build --release

# The binary will be at target/release/llm-cli
```

## Features Highlights

### Error Handling
- Comprehensive error types using `thiserror`
- Graceful degradation with helpful error messages
- Rate limit handling with automatic retry suggestions

### Performance
- Async/await for non-blocking I/O
- Efficient HTTP client with connection pooling
- Optimized release builds with LTO and single codegen unit

### Security
- No unsafe code (`#![forbid(unsafe_code)]`)
- Secure API key storage
- Input validation and sanitization

### User Experience
- Colored output for better readability
- Progress spinners for long operations
- Interactive prompts with `dialoguer`
- Command history in chat mode

## Contributing

Contributions are welcome! Please ensure:

1. Code passes all tests: `cargo test`
2. Code is formatted: `cargo fmt`
3. No clippy warnings: `cargo clippy`
4. Documentation is updated


## Compatible Services

This CLI works with any OpenAI-compatible API:

### LM Studio
Local models running on your machine
```bash
./lmstudio-setup.sh  # Quick setup script
# Or manually:
llm-cli config --base-url "http://localhost:1234" --api-key "lm-studio"
```

### Ollama
Another local model server
```bash
llm-cli config --base-url "http://localhost:11434" --api-path "/api/generate" --api-key "ollama"
```

### Azure OpenAI
Microsoft's hosted OpenAI service
```bash
llm-cli config --base-url "https://YOUR_RESOURCE.openai.azure.com" \
                  --api-path "/openai/deployments/YOUR_DEPLOYMENT/chat/completions?api-version=2024-02-01" \
                  --api-key "YOUR_AZURE_KEY"
```

### OpenRouter
Access multiple models through one API
```bash
llm-cli config --base-url "https://openrouter.ai" \
                  --api-path "/api/v1/chat/completions" \
                  --api-key "YOUR_OPENROUTER_KEY"
```

### Together AI
Hosted open-source models
```bash
llm-cli config --base-url "https://api.together.xyz" \
                  --api-path "/v1/chat/completions" \
                  --api-key "YOUR_TOGETHER_KEY"
```

## Roadmap

- [ ] Streaming responses support
- [ ] Function calling capabilities
- [ ] Image generation integration
- [ ] Voice input/output support
- [ ] Plugin system for extensions
- [ ] Web UI companion
- [ ] Batch processing mode
- [ ] Cost tracking and limits

## Acknowledgments

Built with modern Rust ecosystem tools:
- `tokio` for async runtime
- `reqwest` for HTTP client
- `clap` for CLI parsing
- `serde` for serialization
- `tracing` for structured logging


## License

MIT License - see LICENSE file for details
