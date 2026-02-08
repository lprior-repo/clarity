# Clarity

A modern fullstack application built with Rust, Axum, and Dioxus following functional programming principles and test-driven development.

## Table of Contents

- [Overview](#overview)
- [Architecture](#architecture)
- [Prerequisites](#prerequisites)
- [Quick Start](#quick-start)
- [Development Setup](#development-setup)
- [Available Commands](#available-commands)
- [Testing Philosophy](#testing-philosophy)
- [CI/CD Pipeline](#cicd-pipeline)
- [Project Structure](#project-structure)
- [Contributing](#contributing)
- [Troubleshooting](#troubleshooting)

## Overview

Clarity is a fullstack Rust application that demonstrates modern web development practices with a focus on:

- **Functional Programming**: Immutable data structures, pure functions, and explicit error handling
- **Test-Driven Development**: ATDD (Acceptance Test-Driven Development) with the RED-GREEN-REFACTOR cycle
- **Type Safety**: Leverage Rust's type system to prevent runtime errors at compile time
- **Zero-Panic Architecture**: No `unwrap()`, `expect()`, or `panic!()` - proper error handling with `Result<T, E>`

The application uses a three-crate architecture with clear separation of concerns:
- **Frontend**: Dioxus (React-like framework for Rust)
- **Backend**: Axum (web framework with WebSocket support)
- **Shared**: Common types, validation, and database layer

## Architecture

### Three-Crate Structure

```
clarity/
├── clarity-client/     # Dioxus frontend (responsive UI, components)
├── clarity-core/       # Shared types, validation, database layer
├── clarity-server/     # Axum backend (WebSocket, REST API)
└── migrations/         # SQLx database migrations
```

### Crate Responsibilities

#### clarity-core
- Shared data models and domain types
- Input validation and business logic
- Database operations with SQLx
- Reusable utilities and error types
- No framework-specific code

#### clarity-server
- Axum web server with REST API
- WebSocket support for real-time features
- Request handling and routing
- Database integration through clarity-core
- Server-side business logic

#### clarity-client
- Dioxus frontend application
- Responsive UI components
- Client-side state management
- API communication with clarity-server
- User interaction handling

### Technology Stack

- **Rust**: Latest stable toolchain (2024 edition)
- **Axum 0.8**: High-performance web framework with WebSocket support
- **Dioxus 0.7**: React-like frontend framework for Rust
- **SQLx 0.8**: Compile-time checked database queries
- **PostgreSQL**: Primary database with UUID primary keys
- **Tokio**: Async runtime for Rust
- **MoonRepo**: Build system with aggressive caching

### Design Principles

1. **Immutability**: All data structures are immutable by default
2. **Pure Functions**: No side effects, same input always produces same output
3. **Explicit Error Handling**: `Result<T, E>` instead of exceptions
4. **Type Safety**: Catch errors at compile time, not runtime
5. **Function Composition**: Build complex operations from simple functions

## Prerequisites

Before you begin, ensure you have the following installed:

### Required Tools

- **Rust**: Latest stable release
  ```bash
  rustup --version
  rustc --version
  cargo --version
  ```

- **MoonRepo**: Build system (version 1.0.0 or higher)
  ```bash
  moon --version
  ```

- **PostgreSQL**: Database server (version 12 or higher)
  ```bash
  psql --version
  ```

- **SQLx CLI**: Database migration tool
  ```bash
  sqlx --version
  ```

### Installation

#### Installing Rust

If you don't have Rust installed:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

#### Installing MoonRepo

```bash
curl -fsSL https://moonrepo.dev/install/setup.sh | bash
```

#### Installing PostgreSQL

**On Linux (Arch/Manjaro):**
```bash
sudo pacman -S postgresql
sudo -u postgres initdb -D /var/lib/postgres/data
sudo systemctl start postgresql
```

**On macOS:**
```bash
brew install postgresql@14
brew services start postgresql@14
```

**On Ubuntu/Debian:**
```bash
sudo apt update
sudo apt install postgresql postgresql-contrib
sudo systemctl start postgresql
```

#### Installing SQLx CLI

```bash
cargo install sqlx-cli --no-default-features --features rustls,postgres
```

### Database Setup

1. **Create the database:**
   ```bash
   createdb clarity
   ```

2. **Create a user (optional, but recommended):**
   ```bash
   createuser --interactive clarity_user
   psql -c "ALTER USER clarity_user PASSWORD 'your_password';"
   psql -c "GRANT ALL PRIVILEGES ON DATABASE clarity TO clarity_user;"
   ```

3. **Set environment variable:**
   ```bash
   export DATABASE_URL="postgresql://clarity_user:your_password@localhost/clarity"
   ```

4. **Run migrations:**
   ```bash
   moon run :db-migrate
   ```

## Quick Start

### 1. Clone the Repository

```bash
git clone https://github.com/yourusername/clarity.git
cd clarity
```

### 2. Install Dependencies

Moon will automatically manage Rust dependencies:
```bash
moon run :check
```

### 3. Database Setup

Run database migrations:
```bash
moon run :db-migrate
```

### 4. Run the Application

**Start the backend server:**
```bash
moon run :server
```

The server will start on `http://localhost:3000`

**In a new terminal, start the frontend:**
```bash
moon run :client
```

The client will typically run on `http://localhost:8080`

### 5. Verify It's Working

- Visit the frontend URL in your browser
- Check the server logs for successful startup
- Try interacting with the application

## Development Setup

### Initial Setup

1. **Fork and clone the repository:**
   ```bash
   git clone https://github.com/yourusername/clarity.git
   cd clarity
   ```

2. **Set up the database:**
   ```bash
   createdb clarity
   export DATABASE_URL="postgresql://localhost/clarity"
   moon run :db-migrate
   ```

3. **Verify your setup:**
   ```bash
   moon run :quick
   moon run :test
   ```

### Development Workflow

The project uses **MoonRepo** for all build operations. NEVER use raw cargo commands.

#### Quick Iteration Loop (6-7ms with cache)

When making code changes:
```bash
# Edit your code...
moon run :quick  # Parallel format + lint check (cached)
```

#### Before Committing

Always run the full pipeline:
```bash
moon run :fmt-fix  # Auto-fix formatting issues
moon run :ci       # Full CI pipeline (format, lint, test)
```

#### Running Tests

```bash
# Run all tests
moon run :test

# Run only unit tests
moon run :test-unit

# Run documentation tests
moon run :test-doc
```

#### Type Checking

```bash
# Fast type check without building
moon run :check

# Full build
moon run :build
```

## Available Commands

### Code Quality

- **`moon run :quick`** - Fast format + lint check (cached, 6-7ms)
- **`moon run :fmt-fix`** - Auto-fix code formatting issues
- **`moon run :fmt`** - Check code formatting (fails if not formatted)
- **`moon run :clippy`** - Run Clippy linter (strict mode)

### Testing

- **`moon run :test`** - Run all tests (unit, integration, doc)
- **`moon run :test-unit`** - Run unit tests only
- **`moon run :test-doc`** - Run documentation tests

### Building

- **`moon run :check`** - Fast type check without building
- **`moon run :build`** - Build all crates (debug mode)
- **`moon run :release`** - Build release binaries (optimized)

### Running the Application

- **`moon run :server`** - Run the Axum backend server
- **`moon run :client`** - Run the Dioxus frontend client

### Full Pipeline

- **`moon run :ci`** - Run complete CI pipeline (format, lint, test)

### Database Operations

- **`moon run :db-migrate`** - Run database migrations
- **`moon run :db-migrate-add <name>`** - Create a new migration

### Individual Crate Builds

- **`moon run :build-core`** - Build clarity-core only
- **`moon run :build-server`** - Build clarity-server only
- **`moon run :build-client`** - Build clarity-client only

### Important: Always Use Moon

**NEVER use raw cargo commands:**

```bash
# ❌ WRONG
cargo test
cargo build
cargo fmt

# ✅ CORRECT
moon run :test
moon run :build
moon run :fmt-fix
```

Moon provides:
- Aggressive caching (6-7ms vs ~450ms for cargo)
- Parallel execution across crates
- Dependency-aware builds
- Consistent environment

## Testing Philosophy

This project follows **ATDD (Acceptance Test-Driven Development)** principles.

### Core Principles

1. **Write Tests First**: Always write tests before implementing features
2. **RED-GREEN-REFACTOR**: The TDD cycle is mandatory
3. **Test Failure Modes**: Don't test happy paths - test edge cases and errors
4. **Zero-Panic Code**: Tests should verify proper error handling
5. **Adversarial Testing**: Tests should actively try to break the code

### The ATDD Cycle

For every feature, follow this cycle:

```
1. READ  - Read the acceptance criteria
2. WRITE - Write a test that codifies those criteria
3. RED   - Run the test (it MUST fail)
4. GREEN - Implement minimal code to pass
5. REFACTOR - Improve code while keeping tests green
6. REPEAT - Until all criteria are covered
```

### What to Test

**✅ Test these:**
- Edge cases: empty strings, zero values, max limits
- Error paths: network failures, invalid data, timeouts
- Concurrent access: multiple connections, race conditions
- Resource limits: out of memory, connection pool exhaustion
- Invalid inputs: negative numbers, malformed UUIDs, bad UTF-8

**❌ Don't just test:**
- Happy paths (they're boring)
- Obvious behavior (1 + 1 = 2)
- Trivial getters/setters

### Zero-Panic Architecture

This project has strict rules:

- **No `unwrap()`** - Use `?` operator or proper error handling
- **No `expect()`** - Handle errors gracefully
- **No `panic!()`** - Return `Result<T, E>` instead
- **No `todo!()` or `unimplemented!()`** - Write tests first

**Example:**

```rust
// ❌ WRONG
fn get_user(id: &str) -> User {
    User::find(id).unwrap()
}

// ✅ CORRECT
fn get_user(id: &str) -> Result<User, DbError> {
    let user = User::find(id)?;
    Ok(user)
}
```

### Running Tests

```bash
# Run all tests
moon run :test

# Run specific test
moon run :test -- user::tests::test_create_user

# Run tests with output
moon run :test -- --nocapture

# Run tests in release mode (faster)
moon run :test --release
```

For more detailed testing guidelines, see [AGENTS.md](AGENTS.md).

## CI/CD Pipeline

This project uses a comprehensive CI/CD pipeline with GitHub Actions and MoonRepo.

### Pipeline Stages

The CI pipeline runs in three parallel jobs:

#### 1. Code Quality (Fast Feedback)
- Check code formatting with rustfmt
- Run Clippy linter in strict mode
- Fails if any warnings are found

#### 2. Test (Parallel)
- Run tests on stable and nightly Rust
- Execute unit tests and integration tests
- Run documentation tests

#### 3. Build (Release)
- Build release binaries for all crates
- Upload artifacts for deployment
- Verify production build works

### Performance

MoonRepo provides aggressive caching:
- **Cached tasks**: 6-7ms (vs ~450ms uncached)
- **Parallel execution**: All crates build simultaneously
- **Dependency awareness**: Only rebuild what changed

### Running CI Locally

Before pushing, always run:
```bash
moon run :ci
```

This runs the complete pipeline:
1. Code formatting check
2. Clippy linting (strict mode)
3. All tests (unit, integration, doc)

### CI Configuration

The CI pipeline is defined in `.github/workflows/ci.yml` and mirrors the local Moon tasks.

## Project Structure

```
clarity/
├── .github/
│   └── workflows/
│       └── ci.yml              # CI/CD pipeline configuration
├── .moon/
│   ├── tasks.yml               # Moon task definitions
│   └── workspace.yml           # Moon workspace configuration
├── .beads/
│   └── issues.jsonl            # Issue tracking (beads)
├── clarity-client/             # Frontend application
│   ├── src/
│   │   ├── main.rs
│   │   ├── components/
│   │   └── routes/
│   └── Cargo.toml
├── clarity-core/               # Shared business logic
│   ├── src/
│   │   ├── models/
│   │   ├── validation/
│   │   ├── db/
│   │   └── lib.rs
│   ├── migrations/             # Database migrations
│   │   └── 001_initial_schema.sql
│   └── Cargo.toml
├── clarity-server/             # Backend server
│   ├── src/
│   │   ├── main.rs
│   │   ├── handlers/
│   │   ├── routes/
│   │   └── websocket/
│   └── Cargo.toml
├── migrations/                 # Symlink to clarity-core/migrations
├── Cargo.toml                  # Workspace configuration
├── Cargo.lock
├── AGENTS.md                   # Development guidelines for AI agents
└── README.md                   # This file
```

### Key Directories

- **`clarity-client/`**: Dioxus frontend with components and routes
- **`clarity-core/`**: Shared types, validation, and database layer
- **`clarity-server/`**: Axum backend with REST API and WebSocket support
- **`migrations/`**: SQLx database migrations
- **`.moon/`**: MoonRepo build configuration
- **`.github/`**: CI/CD pipeline configuration

## Contributing

We welcome contributions! Please follow these guidelines.

### For AI Agents

AI agents should read [AGENTS.md](AGENTS.md) for detailed development practices. Key requirements:

1. **Load Required Skills**:
   - `/tdd15` - 15-phase TDD workflow
   - `/zjj` - Workspace isolation with Jujutsu
   - `/functional-rust-generator` - Zero-panic functional patterns

2. **Follow ATDD**:
   - Write tests FIRST
   - Follow RED-GREEN-REFACTOR cycle
   - Test edge cases and failure modes

3. **Use Moon Commands**:
   - Never use raw cargo commands
   - Always use `moon run :task-name`
   - Run `moon run :ci` before completing work

4. **Zero-Panic Code**:
   - No `unwrap()`, `expect()`, or `panic!()`
   - Use `Result<T, E>` with proper error propagation
   - Functional patterns: `map()`, `and_then()`, combinators

### For Human Contributors

#### Setting Up Development Environment

1. **Fork and clone** the repository
2. **Create a feature branch**:
   ```bash
   git checkout -b feature/your-feature-name
   ```
3. **Set up the database** (see [Prerequisites](#prerequisites))
4. **Run tests** to verify setup:
   ```bash
   moon run :test
   ```

#### Making Changes

1. **Write tests first** following ATDD principles
2. **Implement the feature** to make tests pass
3. **Run the full pipeline**:
   ```bash
   moon run :fmt-fix
   moon run :ci
   ```
4. **Ensure all tests pass**

#### Commit Guidelines

Follow conventional commits:

- `feat:` - New feature
- `fix:` - Bug fix
- `refactor:` - Code refactoring
- `test:` - Adding or updating tests
- `docs:` - Documentation changes
- `chore:` - Maintenance tasks

Examples:
```
feat: add user authentication
fix: handle database connection timeout
refactor: simplify error handling in user module
test: add edge case tests for email validation
```

#### Submitting Changes

1. **Push your branch**:
   ```bash
   git push origin feature/your-feature-name
   ```

2. **Create a Pull Request** with:
   - Clear description of changes
   - Reference to related issues (if any)
   - Screenshots for UI changes (if applicable)

3. **Address review feedback**:
   - Make requested changes
   - Ensure CI passes
   - Respond to all comments

#### Code Review Process

All submissions go through review:
- Automated checks (CI pipeline)
- Human review for code quality and design
- Testing verification

#### Code Style

- Follow Rust standard naming conventions
- Use functional patterns (iterators, combinators)
- Prefer immutable data structures
- Document public APIs with rustdoc
- Keep functions small and focused

### Issue Tracking

We use "beads" for issue tracking. See [AGENTS.md](AGENTS.md) for details on working with beads.

## Troubleshooting

### Database Issues

#### "Connection refused" error

**Problem**: Cannot connect to PostgreSQL

**Solutions**:
1. Check if PostgreSQL is running:
   ```bash
   # Linux
   sudo systemctl status postgresql

   # macOS
   brew services list
   ```

2. Start PostgreSQL if not running:
   ```bash
   # Linux
   sudo systemctl start postgresql

   # macOS
   brew services start postgresql@14
   ```

3. Verify DATABASE_URL is set:
   ```bash
   echo $DATABASE_URL
   ```

#### "Database does not exist" error

**Solution**:
```bash
createdb clarity
```

#### Migration failures

**Solution**:
```bash
# Check current migration status
sqlx migrate info

# Re-run migrations (will skip already applied)
moon run :db-migrate

# If completely stuck, you can reset (WARNING: deletes data)
dropdb clarity && createdb clarity
moon run :db-migrate
```

### Build Issues

#### "Moon command not found"

**Problem**: Moon is not installed or not in PATH

**Solution**:
```bash
# Check if moon is installed
which moon

# If not found, install it
curl -fsSL https://moonrepo.dev/install/setup.sh | bash

# Restart your terminal or source your shell profile
source ~/.bashrc  # or ~/.zshrc
```

#### Cargo compilation errors

**Problem**: Rust code fails to compile

**Solutions**:
1. Check Rust version:
   ```bash
   rustc --version  # Should be latest stable
   ```

2. Update Rust if needed:
   ```bash
   rustup update stable
   ```

3. Clean and rebuild:
   ```bash
   moon run :clean  # If available
   cargo clean      # Manual clean
   moon run :build
   ```

#### Clippy warnings

**Problem**: Clippy reports warnings

**Solution**:
1. **NEVER** modify clippy configuration
2. Fix the code instead
3. Run:
   ```bash
   moon run :clippy
   ```
4. Address each warning

### Testing Issues

#### Tests fail with "database error"

**Problem**: Tests cannot connect to database

**Solution**:
1. Ensure DATABASE_URL is set:
   ```bash
   export DATABASE_URL="postgresql://localhost/clarity"
   ```

2. Ensure database exists:
   ```bash
   psql -l | grep clarity
   ```

3. Run migrations:
   ```bash
   moon run :db-migrate
   ```

#### "Test panicked" error

**Problem**: Code contains `panic!`, `unwrap()`, or `expect()`

**Solution**:
This violates our zero-panic policy. Replace with proper error handling:

```rust
// ❌ WRONG
let value = option.unwrap();

// ✅ CORRECT
let value = option.ok_or_else(|| Error::NotFound)?;
```

### Performance Issues

#### Moon cache not working

**Problem**: Tasks always run from scratch

**Solutions**:
1. Check Moon version:
   ```bash
   moon --version
   ```

2. Clear Moon cache:
   ```bash
   moon cache clean
   ```

3. Verify cache is enabled:
   ```bash
   moon run :check --verbose
   ```

#### Slow compile times

**Solutions**:
1. Use Moon's cached tasks:
   ```bash
   moon run :quick  # Much faster than cargo
   ```

2. Ensure you're using `--all-features` consistently:
   ```bash
   moon run :check  # Faster
   moon run :build  # Slower but complete
   ```

3. Consider using `sccache` for distributed compilation

### Environment Issues

#### "Command not found: cargo"

**Problem**: Rust toolchain not in PATH

**Solution**:
```bash
# Add Rust to PATH
source $HOME/.cargo/env

# Add to shell profile for persistence
echo 'source $HOME/.cargo/env' >> ~/.bashrc  # or ~/.zshrc
```

#### SQLx CLI not found

**Problem**: `sqlx` command not available

**Solution**:
```bash
cargo install sqlx-cli --no-default-features --features rustls,postgres
```

### Getting Help

If you encounter issues not covered here:

1. Check [AGENTS.md](AGENTS.md) for detailed development practices
2. Review GitHub Issues for similar problems
3. Create a new Issue with:
   - Error messages
   - Steps to reproduce
   - Your environment (OS, Rust version, Moon version)
   - What you've already tried

## License

MIT License - see LICENSE file for details

## Acknowledgments

Built with:
- [Rust](https://www.rust-lang.org/)
- [Axum](https://github.com/tokio-rs/axum)
- [Dioxus](https://dioxuslabs.com/)
- [SQLx](https://github.com/launchbadge/sqlx)
- [MoonRepo](https://moonrepo.dev/)

## Resources

- [Rust Documentation](https://doc.rust-lang.org/)
- [Axum Guide](https://docs.rs/axum/)
- [Dioxus Guide](https://dioxuslabs.com/learn/0.7/)
- [SQLx Guide](https://docs.rs/sqlx/)
- [MoonRepo Documentation](https://moonrepo.dev/docs)

---

**Note**: This project follows strict functional programming and testing principles. For detailed development guidelines, see [AGENTS.md](AGENTS.md).
