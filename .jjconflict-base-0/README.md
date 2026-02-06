# Clarity

A modern fullstack application built with Rust, Axum, and Dioxus.

## Project Structure

This project is organized into three main crates:

- `clarity-client` - Dioxus frontend application
- `clarity-core` - Shared business logic and models
- `clarity-server` - Axum backend server

## Getting Started

### Prerequisites

- Rust toolchain (latest stable)
- Cargo
- Tokio runtime

### Running the Application

1. **Start the server:**
   ```bash
   cargo run -p clarity-server
   ```

2. **Build the client:**
   ```bash
   cargo run -p clarity-client
   ```

### Development

- Run tests: `cargo test`
- Format code: `cargo fmt`
- Lint code: `cargo clippy`

## Architecture

This project follows functional programming principles with:
- Immutability
- Pure functions
- Explicit error handling
- Type safety

## Contributing

Please read `AGENTS.md` for details on our code quality standards and development workflow.