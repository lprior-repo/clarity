# Clarity

> **Source of truth:** The target product PRD, architecture spec, domain contract, and bead-decomposition contract live in [`MASTER_DOC.md`](./MASTER_DOC.md). This README may describe current repository/development context; do not use it as the end-state specification.

A modern desktop application built with Rust and Dioxus following functional programming principles and test-driven development.

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

Clarity is a desktop Rust application that demonstrates modern application development practices with a focus on:

- **Functional Programming**: Immutable data structures, pure functions, and explicit error handling
- **Test-Driven Development**: ATDD (Acceptance Test-Driven Development) with the RED-GREEN-REFACTOR cycle
- **Type Safety**: Leverage Rust's type system to prevent runtime errors at compile time
- **Zero-Panic Architecture**: No `unwrap()`, `expect()`, or `panic!()` - proper error handling with `Result<T, E>`

The application uses a two-crate architecture with clear separation of concerns:
- **Frontend**: Dioxus desktop framework
- **Core**: Shared types, validation, and embedded database layer

## Architecture

### Three-Crate Structure

```
clarity/
├── clarity-client/     # Dioxus desktop frontend (UI, components)
├── clarity-core/       # Shared types, validation, embedded database
└── migrations/         # SQLx database migrations
```

### Crate Responsibilities

#### clarity-core
- Shared data models and domain types
- Input validation and business logic
- Database operations with SQLx
- Reusable utilities and error types
- No framework-specific code


#### clarity-client
- Dioxus desktop application
- Native UI components
- Client-side state management
- Direct database integration through clarity-core
- User interaction handling

### Technology Stack

- **Rust**: Latest stable toolchain (2024 edition)
- **Dioxus 0.7**: Desktop UI framework for Rust
- **SQLx 0.8**: Compile-time checked database queries
- **SQLite**: Embedded database with UUID primary keys
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

#### Installing SQLx CLI

```bash
cargo install sqlx-cli --no-default-features --features rustls,sqlite
```

### Database Setup

1. **Run migrations** (SQLite database will be created automatically):
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

```bash
moon run :client
```

The desktop application will start as a native window.

### 5. Verify It's Working

- The application window should open
- Try interacting with the application
- Check the terminal for any errors

## Development Setup

### Initial Setup

1. **Fork and clone the repository:**
   ```bash
   git clone https://github.com/yourusername/clarity.git
   cd clarity
   ```

2. **Set up the database:**
   ```bash
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

- **`moon run :client`** - Run the Dioxus desktop client

### Full Pipeline

- **`moon run :ci`** - Run complete CI pipeline (format, lint, test)

### Database Operations

- **`moon run :db-migrate`** - Run database migrations
- **`moon run :db-migrate-add <name>`** - Create a new migration

### Individual Crate Builds

- **`moon run :build-core`** - Build clarity-core only
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
├── clarity-client/             # Desktop application
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
├── migrations/                 # Symlink to clarity-core/migrations
├── Cargo.toml                  # Workspace configuration
├── Cargo.lock
├── AGENTS.md                   # Development guidelines for AI agents
└── README.md                   # This file
```

### Key Directories

- **`clarity-client/`**: Dioxus desktop UI with components and routes
- **`clarity-core/`**: Shared types, validation, and embedded database
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

For comprehensive troubleshooting information, see the [Troubleshooting Guide](docs/troubleshooting.md).

### Quick Links

- [Database Issues](docs/troubleshooting.md#database-issues) - Connection problems, migrations, permissions
- [Build & Compilation Issues](docs/troubleshooting.md#build--compilation-issues) - Moon, Cargo, Clippy errors
- [Testing Issues](docs/troubleshooting.md#testing-issues) - Test failures, panics, flaky tests
- [Runtime Issues](docs/troubleshooting.md#runtime-issues) - Port conflicts, WebSocket, performance
- [Development Environment Issues](docs/troubleshooting.md#development-environment-issues) - Tool setup, editor integration
- [Performance Issues](docs/troubleshooting.md#performance-issues) - Compile times, test execution

### Quick Diagnostics

If you're experiencing issues, run these commands first:

```bash
# Check your environment
rustc --version && cargo --version && moon --version

# Check database
ls -la clarity.db

# Check build status
moon run :quick
```

### Common Issues

**Database connection issues:**
```bash
# Re-run migrations to recreate database
moon run :db-migrate
```

**Moon command not found:**
```bash
curl -fsSL https://moonrepo.dev/install/setup.sh | bash
```

**Clippy warnings:**
```bash
# NEVER modify clippy config - fix the code instead
moon run :clippy
# See docs/troubleshooting.md for zero-unwrap patterns
```

**Tests failing:**
```bash
# Ensure database is set up
moon run :db-migrate
moon run :test
```

For detailed solutions to these and other issues, see the full [Troubleshooting Guide](docs/troubleshooting.md).

## License

MIT License - see LICENSE file for details

## Acknowledgments

Built with:
- [Rust](https://www.rust-lang.org/)
- [Dioxus](https://dioxuslabs.com/)
- [SQLx](https://github.com/launchbadge/sqlx)
- [MoonRepo](https://moonrepo.dev/)

## Resources

- [Rust Documentation](https://doc.rust-lang.org/)
- [Dioxus Guide](https://dioxuslabs.com/learn/0.7/)
- [SQLx Guide](https://docs.rs/sqlx/)
- [MoonRepo Documentation](https://moonrepo.dev/docs)

---

**Note**: This project follows strict functional programming and testing principles. For detailed development guidelines, see [AGENTS.md](AGENTS.md).
