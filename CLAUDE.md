# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview
This is an MCP (Model Context Protocol) server implementation test repository. The project is currently in its initial state with only basic files present.

## Language and Framework
- **Language**: Rust (based on .gitignore patterns)
- **Expected Build System**: Cargo (Rust's package manager)

## Development Commands
Since this is a new Rust project, you'll likely need to initialize it first:

```bash
cargo init .
```

Once initialized, standard Rust development commands will apply:
- `cargo build` - Build the project
- `cargo run` - Run the project  
- `cargo test` - Run tests
- `cargo check` - Check code without building
- `cargo fmt` - Format code
- `cargo clippy` - Run linter

## Project Structure
Currently minimal - only contains:
- README.md - Basic project description
- .gitignore - Rust-specific ignore patterns
- CLAUDE.md - This file

## Notes
- This appears to be a template or starting point for an MCP server
- The .gitignore is configured for Rust development with Cargo
- No actual implementation code exists yet