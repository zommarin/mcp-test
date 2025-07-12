# mcp-test

A Rust-based Model Context Protocol (MCP) server implementation with ClickHouse database integration and configurable logging.

## Features

- **JSON-RPC Protocol**: Full JSON-RPC 2.0 support for MCP communication
- **Async I/O**: Built with Tokio for efficient async operations
- **Configurable Logging**: Debug, info, warn, and error levels via `RUST_LOG`
- **MCP Protocol Support**: Initialize/initialized methods with tool capabilities
- **ClickHouse Integration**: Database introspection tools for listing databases, tables, and schemas
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

### ClickHouse Configuration

Set environment variables to configure ClickHouse connection:

```bash
export CLICKHOUSE_URL="http://localhost:8123"
export CLICKHOUSE_DATABASE="default"
export CLICKHOUSE_USERNAME="default"
export CLICKHOUSE_PASSWORD=""
```

### Testing

#### Basic MCP Protocol
Send JSON-RPC requests via stdin:

```bash
echo '{"jsonrpc": "2.0", "method": "initialize", "params": {"protocolVersion": "2024-11-05", "capabilities": {}, "clientInfo": {"name": "test-client", "version": "1.0"}}, "id": 1}' | cargo run
```

#### ClickHouse Tools
List available tools:
```bash
echo '{"jsonrpc": "2.0", "method": "tools/list", "params": {}, "id": 1}' | cargo run
```

List all databases:
```bash
echo '{"jsonrpc": "2.0", "method": "tools/call", "params": {"name": "list_databases"}, "id": 1}' | cargo run
```

List tables in a database:
```bash
echo '{"jsonrpc": "2.0", "method": "tools/call", "params": {"name": "list_tables", "arguments": {"database": "system"}}, "id": 1}' | cargo run
```

Get table schema:
```bash
echo '{"jsonrpc": "2.0", "method": "tools/call", "params": {"name": "get_table_schema", "arguments": {"database": "system", "table": "tables"}}, "id": 1}' | cargo run
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

### MCP Tools

The server provides three ClickHouse database introspection tools:

1. **list_databases** - Lists all databases in the ClickHouse instance
2. **list_tables** - Lists all tables in a specific database
3. **get_table_schema** - Shows detailed column information including data types, constraints, and key memberships

### Testing

Run the test suite:
```bash
cargo test
```

The tests include:
- Unit tests for data structure serialization
- JSON-RPC protocol validation
- ClickHouse client functionality
- Integration test framework (requires running ClickHouse instance)

## Dependencies

- **tokio** - Async runtime
- **serde** - Serialization/deserialization
- **clickhouse** - ClickHouse client library
- **log** / **env_logger** - Configurable logging
- **anyhow** - Error handling 
