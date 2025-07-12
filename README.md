# mcp-test

A Rust-based Model Context Protocol (MCP) server implementation with configurable logging.

## Features

- **JSON-RPC Protocol**: Full JSON-RPC 2.0 support for MCP communication
- **Async I/O**: Built with Tokio for efficient async operations
- **Configurable Logging**: Debug, info, warn, and error levels via `RUST_LOG`
- **MCP Protocol Support**: Initialize/initialized methods with capability declarations
- **Error Handling**: Proper JSON-RPC error responses for invalid requests

## Usage

### Running the Server

```bash
cargo run
```

### With Logging

```bash
# Show all log levels
RUST_LOG=debug cargo run

# Show only info and above
RUST_LOG=info cargo run

# Show only warnings and errors
RUST_LOG=warn cargo run
```

### Testing

Send JSON-RPC requests via stdin:

```bash
echo '{"jsonrpc": "2.0", "method": "initialize", "params": {"protocolVersion": "2024-11-05", "capabilities": {}, "clientInfo": {"name": "test-client", "version": "1.0"}}, "id": 1}' | cargo run
```

## Development

```bash
# Build the project
cargo build

# Run tests
cargo test

# Check code without building
cargo check

# Format code
cargo fmt

# Run linter
cargo clippy
```

## Architecture

The server implements a JSON-RPC interface that:
- Reads requests from stdin
- Processes MCP protocol messages
- Writes responses to stdout
- Logs operations at configurable levels

Currently supports basic MCP initialization flow with empty capability declarations for tools, resources, and prompts. 
