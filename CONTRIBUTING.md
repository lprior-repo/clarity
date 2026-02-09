# Contributing to Clarity

Thank you for your interest in contributing to Clarity! This guide will help you get started and ensure your contributions can be effectively reviewed and merged.

## Table of Contents

- [Quick Start for Contributors](#quick-start-for-contributors)
- [Development Environment Setup](#development-environment-setup)
- [Project Philosophy](#project-philosophy)
- [Development Workflow](#development-workflow)
- [Code Quality Standards](#code-quality-standards)
- [Testing Requirements](#testing-requirements)
- [Commit Guidelines](#commit-guidelines)
- [Pull Request Process](#pull-request-process)
- [Issue Tracking with Beads](#issue-tracking-with-beads)
- [Getting Help](#getting-help)

## Quick Start for Contributors

### For First-Time Contributors

1. **Fork and clone the repository**
   ```bash
   git clone https://github.com/yourusername/clarity.git
   cd clarity
   ```

2. **Install prerequisites**
   - Rust (latest stable)
   - MoonRepo build system
   - PostgreSQL
   - SQLx CLI

   See [Development Environment Setup](#development-environment-setup) for detailed instructions.

3. **Set up your development environment**
   ```bash
   # Create database
   createdb clarity

   # Set environment variable
   export DATABASE_URL="postgresql://localhost/clarity"

   # Run migrations
   moon run :db-migrate

   # Verify setup
   moon run :test
   ```

4. **Find something to work on**
   - Check [open beads](https://github.com/yourusername/clarity/issues) (our issue tracker)
   - Look for issues labeled `good first issue`
   - Join discussions in existing issues

5. **Make your changes**
   ```bash
   # Create a branch
   git checkout -b feature/your-feature-name

   # Make changes following our workflow (see below)
   # Run tests
   moon run :test

   # Format and lint
   moon run :fmt-fix
   moon run :quick
   ```

6. **Submit your contribution**
   ```bash
   git push origin feature/your-feature-name
   # Then create a Pull Request on GitHub
   ```

## Development Environment Setup

### Required Tools

#### 1. Rust Toolchain

Install the latest stable Rust:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

Verify installation:
```bash
rustc --version
cargo --version
```

#### 2. MoonRepo Build System

MoonRepo provides fast, cached builds and is required for all development:

```bash
curl -fsSL https://moonrepo.dev/install/setup.sh | bash
```

Verify installation:
```bash
moon --version
```

#### 3. PostgreSQL Database

**Linux (Arch/Manjaro):**
```bash
sudo pacman -S postgresql
sudo -u postgres initdb -D /var/lib/postgres/data
sudo systemctl start postgresql
sudo systemctl enable postgresql
```

**macOS:**
```bash
brew install postgresql@14
brew services start postgresql@14
```

**Ubuntu/Debian:**
```bash
sudo apt update
sudo apt install postgresql postgresql-contrib
sudo systemctl start postgresql
sudo systemctl enable postgresql
```

#### 4. SQLx CLI

```bash
cargo install sqlx-cli --no-default-features --features rustls,postgres
```

### Database Setup

1. **Create the database:**
   ```bash
   createdb clarity
   ```

2. **Configure environment:**
   ```bash
   # Add to your shell profile (~/.bashrc or ~/.zshrc)
   export DATABASE_URL="postgresql://localhost/clarity"
   ```

3. **Run migrations:**
   ```bash
   moon run :db-migrate
   ```

### Verify Your Setup

Run the full test suite to ensure everything works:

```bash
moon run :test
moon run :quick
```

## Project Philosophy

### Core Principles

Clarity is built on functional programming principles with strict quality standards:

1. **Zero-Panic Architecture**
   - No `unwrap()`, `expect()`, or `panic!()` in production code
   - All errors handled explicitly with `Result<T, E>`
   - Proper error propagation using `?` operator

2. **Test-Driven Development (TDD)**
   - Write tests FIRST, then implement
   - Follow RED-GREEN-REFACTOR cycle
   - Test failure modes, not just happy paths
   - Adversarial testing: actively try to break code

3. **Functional Programming**
   - Immutable data structures by default
   - Pure functions (no side effects)
   - Iterator combinators over loops
   - Function composition over mutation

4. **Type Safety**
   - Leverage Rust's type system
   - Catch errors at compile time
   - Use newtypes for semantic clarity

### Technology Stack

- **Frontend**: Dioxus 0.7 (React-like framework for Rust)
- **Backend**: Axum 0.8 (web framework with WebSocket support)
- **Database**: SQLx 0.8 with PostgreSQL (compile-time checked queries)
- **Build System**: MoonRepo (aggressive caching, parallel execution)
- **Runtime**: Tokio (async runtime)

### Three-Crate Architecture

```
clarity/
├── clarity-client/     # Dioxus frontend (UI components, routing)
├── clarity-core/       # Shared types, validation, database layer
└── clarity-server/     # Axum backend (REST API, WebSocket)
```

**Key separation of concerns:**
- `clarity-core` contains business logic, types, and database operations
- `clarity-server` handles HTTP/WebSocket routing and server-side logic
- `clarity-client` manages UI components and client-side state

## Development Workflow

### ALWAYS Use MoonRepo

**CRITICAL**: Never use raw `cargo` commands. Always use Moon:

```bash
# ✅ CORRECT
moon run :test
moon run :build
moon run :fmt-fix

# ❌ WRONG
cargo test
cargo build
cargo fmt
```

**Why?**
- Moon provides 6-7ms cached builds vs ~450ms uncached
- Parallel execution across crates
- Dependency-aware builds
- Consistent environment

### Daily Development Loop

#### Quick Iteration (6-7ms with cache)

```bash
# Edit your code...
moon run :quick  # Parallel format + lint check
```

#### Before Committing

```bash
moon run :fmt-fix  # Auto-fix formatting issues
moon run :ci       # Full pipeline (format, lint, test)
```

### Running Tests

```bash
# All tests
moon run :test

# Unit tests only
moon run :test-unit

# Documentation tests
moon run :test-doc

# Specific test
moon run :test -- user::tests::test_create_user
```

### Building

```bash
# Fast type check
moon run :check

# Full build (debug)
moon run :build

# Release build
moon run :release
```

### Running the Application

```bash
# Backend server (Axum)
moon run :server

# Frontend (Dioxus)
moon run :client
```

### Database Operations

```bash
# Run migrations
moon run :db-migrate

# Create new migration
moon run :db-migrate-add <name>
```

## Code Quality Standards

### Zero-Panic Policy

**Forbidden in production code:**
- `unwrap()` - Use `?` or `ok_or_else()` instead
- `expect()` - Use proper error handling
- `panic!()` - Return `Result<T, E>`
- `todo!()` - Write tests first
- `unimplemented!()` - Write tests first

**Examples:**

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

### Functional Patterns

**Use iterator combinators:**

```rust
// ❌ WRONG: Loop with mutation
let mut result = Vec::new();
for item in items {
    if item.is_valid() {
        result.push(item);
    }
}

// ✅ CORRECT: Iterator combinators
let result: Vec<Item> = items
    .into_iter()
    .filter(|item| item.is_valid())
    .collect();
```

**Use immutable data:**

```rust
// ❌ WRONG: Mutable
fn process_items(items: &mut Vec<Item>) {
    items.push(Item::new());
}

// ✅ CORRECT: Immutable
fn process_items(items: Vec<Item>) -> Vec<Item> {
    items.into_iter().chain(Some(Item::new())).collect()
}
```

### Linting

**CRITICAL RULE**: Never modify clippy configuration. If clippy reports warnings, fix the code.

```bash
# Check formatting
moon run :fmt

# Auto-fix formatting
moon run :fmt-fix

# Run clippy (strict mode)
moon run :clippy
```

The project uses strict lints enforced by the compiler. See `Cargo.toml` workspace.lints section.

## Testing Requirements

### Test-Driven Development (TDD)

**MANDATORY**: Write tests FIRST for all new code.

#### The TDD Cycle

1. **RED**: Write a failing test
2. **GREEN**: Write minimal code to pass
3. **REFACTOR**: Improve while keeping tests green

#### What to Test

**✅ Test these:**
- Edge cases: empty strings, zero values, max limits
- Error paths: network failures, invalid data, timeouts
- Concurrent access: race conditions, multiple connections
- Resource limits: out of memory, connection pool exhaustion
- Invalid inputs: negative numbers, malformed UUIDs, bad UTF-8

**❌ Don't just test:**
- Happy paths (they're boring)
- Obvious behavior (1 + 1 = 2)
- Trivial getters/setters

#### Example: TDD in Action

```rust
// Step 1: Write failing test (RED)
#[test]
fn test_user_rejects_empty_email() {
    let result = User::new("");
    assert!(matches!(result, Err(UserError::EmptyEmail)));
}

// Step 2: Implement minimal code (GREEN)
impl User {
    pub fn new(email: &str) -> Result<Self, UserError> {
        if email.is_empty() {
            return Err(UserError::EmptyEmail);
        }
        // ... rest of implementation
    }
}

// Step 3: Refactor for functional purity
impl User {
    pub fn new(email: &str) -> Result<Self, UserError> {
        email
            .validate_non_empty()?
            .validate_max_length(254)?
            .validate_email_format()
            .map(|_| User { /* ... */ })
    }
}
```

### Running Tests

```bash
# All tests
moon run :test

# Watch mode (re-run on changes)
moon run :test -- --watch

# With output
moon run :test -- --nocapture

# Release mode (faster)
moon run :test -- --release
```

### Code Coverage

We aim for high test coverage. Before submitting:

```bash
# Run tests with coverage (if configured)
moon run :test --all-features
```

## Commit Guidelines

### Conventional Commits

We use [Conventional Commits](https://www.conventionalcommits.org/) format:

```
<type>: <description>

[optional body]

[optional footer]
```

### Commit Types

- `feat:` - New feature
- `fix:` - Bug fix
- `refactor:` - Code refactoring
- `test:` - Adding or updating tests
- `docs:` - Documentation changes
- `chore:` - Maintenance tasks
- `perf:` - Performance improvements
- `style:` - Code style changes (formatting, etc.)

### Examples

```
feat: add user authentication

Implement JWT-based authentication with login/logout endpoints.
Add password hashing with bcrypt.
Add session management middleware.

Closes #123
```

```
fix: handle database connection timeout

Add retry logic for transient connection failures.
Return proper error to client on timeout.

Fixes #456
```

```
test: add edge case tests for email validation

Test empty strings, invalid formats, and length limits.
Ensure proper error codes for each failure mode.
```

### Commit Messages Should:

- Use imperative mood ("add" not "added" or "adds")
- Be concise but descriptive
- Reference relevant issues (if any)
- Explain **why**, not just **what**

## Pull Request Process

### Before Creating a PR

1. **Ensure all tests pass**
   ```bash
   moon run :ci
   ```

2. **Format your code**
   ```bash
   moon run :fmt-fix
   ```

3. **Write a good description**
   - What does this PR do?
   - Why is it needed?
   - How does it solve the problem?
   - Related issues/PRs

### PR Title Format

Use conventional commits format:

```
feat: add user authentication
fix: resolve memory leak in connection pool
docs: update contributing guide
```

### PR Description Template

```markdown
## Summary
Brief description of changes (2-3 sentences).

## Changes
- Bullet point 1
- Bullet point 2
- Bullet point 3

## Testing
- [ ] All tests pass (`moon run :ci`)
- [ ] Added tests for new functionality
- [ ] Tested edge cases and error paths

## Checklist
- [ ] Code follows project philosophy
- [ ] No `unwrap()`, `expect()`, or `panic!()`
- [ ] Functional patterns used
- [ ] Documentation updated (if needed)
- [ ] Tests added/updated

## Related Issues
Closes #123
Related to #456
```

### Review Process

1. **Automated checks**: CI must pass
2. **Code review**: Maintainer review
3. **Testing verification**: May request additional tests
4. **Approval**: At least one maintainer approval required

### After Review

1. **Address feedback**: Make requested changes
2. **Push updates**: Add commits to your branch
3. **Request re-review**: Comment that changes are ready

### Merging

- Maintainers will squash and merge your PR
- Commit message will be based on PR title and description
- Your branch will be automatically deleted after merge

## Issue Tracking with Beads

We use "beads" for lightweight issue tracking (`.beads/issues.jsonl`).

### For Contributors

1. **Find issues**: Check GitHub Issues or ask maintainers
2. **Claim work**: Comment on the issue to indicate you're working on it
3. **Ask questions**: Use GitHub Issues for discussions

### For Maintainers

```bash
# List beads
br list

# Show bead details
br show <id>

# Update status
br update <id> --status in_progress

# Mark complete
br close <id>
```

### Bead Lifecycle

```
open → in_progress → closed
                     ↓
                  blocked
```

## Getting Help

### Documentation

- [README.md](README.md) - Project overview and setup
- [AGENTS.md](AGENTS.md) - AI agent development guidelines
- [Rust Documentation](https://doc.rust-lang.org/)
- [Axum Guide](https://docs.rs/axum/)
- [Dioxus Guide](https://dioxuslabs.com/learn/0.7/)

### Community

- **GitHub Issues**: Bug reports, feature requests
- **GitHub Discussions**: Questions, ideas, general discussion
- **Pull Requests**: Code contributions

### Asking Questions

When asking for help:

1. **Search existing issues** first
2. **Provide context**:
   - What are you trying to do?
   - What did you try?
   - What error did you get?
   - Environment details (OS, Rust version, etc.)
3. **Use code blocks** for code/error messages
4. **Be patient**: Maintainers volunteer their time

### Reporting Bugs

Use the bug report template:

```markdown
**Describe the bug**
Clear description of what's wrong

**To Reproduce**
Steps to reproduce the behavior:
1. Go to '...'
2. Click on '....'
3. Scroll down to '....'
4. See error

**Expected behavior**
What should happen

**Screenshots**
If applicable

**Environment**
- OS: [e.g. Arch Linux]
- Rust version: [e.g. 1.80.0]
- Moon version: [e.g. 1.0.0]
- PostgreSQL version: [e.g. 14]

**Additional context**
Logs, error messages, etc.
```

## Coding Standards Summary

### DO's

✅ Use MoonRepo for all builds
✅ Write tests first (TDD)
✅ Follow functional programming principles
✅ Handle all errors with `Result<T, E>`
✅ Use iterator combinators
✅ Write clear commit messages
✅ Test edge cases and error paths
✅ Document public APIs

### DON'Ts

❌ Use raw `cargo` commands
❌ Use `unwrap()`, `expect()`, `panic!()`
❌ Write code without tests
❌ Modify clippy configuration
❌ Use mutable data when immutable works
❌ Skip code review
❌ Commit without testing

## Recognition

Contributors are recognized in:
- Release notes (for significant contributions)
- CONTRIBUTORS file (for substantial contributions)
- Git history (all contributions)

Thank you for contributing to Clarity! 🎉

---

**Need help?** Open an issue or discussion and we'll be happy to assist!
