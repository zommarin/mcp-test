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
- Capability declarations for tools, resources, and prompts (empty for now)