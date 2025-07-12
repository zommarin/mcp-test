# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview
This is an MCP (Model Context Protocol) server implementation test repository. The project is currently in its initial state with only basic files present.

## Language and Framework
- **Language**: Rust (based on .gitignore patterns)
- **Expected Build System**: Cargo (Rust's package manager)

## Development Commands
Standard Rust development commands:
- `cargo build` - Build the project
- `cargo run` - Run the MCP server
- `cargo test` - Run tests
- `cargo check` - Check code without building
- `cargo fmt` - Format code
- `cargo clippy` - Run linter

## Logging
The server uses `env_logger` for configurable logging. Control log levels with the `RUST_LOG` environment variable:

- `RUST_LOG=debug cargo run` - Show all debug, info, warn, and error messages
- `RUST_LOG=info cargo run` - Show info, warn, and error messages (default level)
- `RUST_LOG=warn cargo run` - Show only warn and error messages
- `RUST_LOG=error cargo run` - Show only error messages

You can also filter by module:
- `RUST_LOG=mcp_test=debug cargo run` - Debug level for this crate only
- `RUST_LOG=mcp_test::main=trace cargo run` - Trace level for main module only

## Project Structure
- `src/main.rs` - Main MCP server implementation
- `Cargo.toml` - Project dependencies and metadata
- `README.md` - Basic project description
- `.gitignore` - Rust-specific ignore patterns

## Architecture
The MCP server is implemented as a JSON-RPC server that:
- Reads JSON-RPC requests from stdin
- Processes MCP protocol messages (initialize, initialized, etc.)
- Writes JSON-RPC responses to stdout
- Uses async/await with Tokio for I/O operations

Key components:
- `McpServer` - Main server struct handling requests
- `JsonRpcRequest`/`JsonRpcResponse` - JSON-RPC message structures
- `InitializeParams` - MCP initialization parameters

## MCP Protocol
Currently implements basic MCP protocol methods:
- `initialize` - Server initialization with capabilities
- `initialized` - Notification that initialization is complete
- `tools/list` - List available tools
- `tools/call` - Execute tool calls

## ClickHouse Integration
The server provides MCP tools for interacting with ClickHouse databases:

### Available Tools
- `list_databases` - List all databases in the ClickHouse instance
- `list_tables` - List all tables in a specific database
- `get_table_schema` - Get detailed schema information for a table

### Configuration
Set these environment variables to configure ClickHouse connection:
- `CLICKHOUSE_URL` - Default: http://localhost:8123
- `CLICKHOUSE_DATABASE` - Default: default
- `CLICKHOUSE_USERNAME` - Default: default
- `CLICKHOUSE_PASSWORD` - Default: (empty)

### Usage Examples
```bash
# List all databases
echo '{"jsonrpc": "2.0", "method": "tools/call", "params": {"name": "list_databases"}, "id": 1}' | cargo run

# List tables in a database
echo '{"jsonrpc": "2.0", "method": "tools/call", "params": {"name": "list_tables", "arguments": {"database": "system"}}, "id": 1}' | cargo run

# Get table schema
echo '{"jsonrpc": "2.0", "method": "tools/call", "params": {"name": "get_table_schema", "arguments": {"database": "system", "table": "tables"}}, "id": 1}' | cargo run
```