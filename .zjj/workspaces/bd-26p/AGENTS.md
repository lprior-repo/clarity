# AGENTS.md - Agent Instructions for AI Agents

## Critical Rules

### Build System: Moon Only
**NEVER use raw cargo commands.** Always use Moon for all build operations:

```bash
# Correct
moon run :quick       # Format + type check
moon run :ci          # Full pipeline
moon run :test        # Run tests
moon run :fmt-fix     # Auto-fix formatting
moon run :check       # Fast type check

# WRONG - Never do this
cargo fmt            # NO
cargo clippy         # NO
cargo test           # NO
cargo build          # NO
```

### Code Quality: Zero-Panic Architecture
- **Zero unwraps**: `unwrap()` and `expect()` are forbidden
- **Zero panics**: `panic!()`, `todo!()`, `unimplemented!()` are forbidden
- All errors must use `Result<T, Error>` with proper propagation
- Use functional patterns: `map()`, `and_then()`, `?` operator
- Railway-Oriented Programming with combinators

### Functional Rust Principles
1. **Immutability**: All data structures are immutable by design
2. **Pure Functions**: No side effects, same input → same output
3. **Error Handling**: `Result` types instead of exceptions
4. **Type Safety**: Leverage Rust's type system to prevent runtime errors
5. **Function Composition**: Build complex operations from simple functions

---

## Project Structure

```
clarity/
├── clarity-client/     # Dioxus frontend application
├── clarity-core/       # Shared business logic and models
├── clarity-server/     # Axum backend server
├── migrations/         # SQLx database migrations
└── .beads/            # Issue tracking (beads system)
```

### Key Modules

#### clarity-core/src/
- `session.rs` - Session types with state machine
- `validation.rs` - Security validators (email, alphanumeric, etc.)
- `error.rs` - Exit code system (POSIX-compliant)
- `db/` - Database layer
  - `models.rs` - Domain models (User, Bead, Interview, Spec)
  - `pool.rs` - Connection pool management
  - `migrate.rs` - Migration runner
  - `error.rs` - Database errors

#### clarity-server/src/
- `main.rs` - Axum server with WebSocket support

#### clarity-client/
- `assets/responsive.css` - Mobile-first responsive design (1,100+ lines)
- `tests/responsive_design_test.rs` - 21 responsive design tests

---

## Development Workflow

### Issue Tracking (Beads)
```bash
# Find available work
br ready              # Show beads ready to start

# Claim and work
br show <id>          # View bead details
br update <id> --status in_progress  # Claim work

# Complete work
br close <id>         # Mark as complete
br sync --flush-only  # Sync to JSONL (no git)
git add .beads/
git commit -m "close bead <id>"
```

### Development (Moon CI/CD)
```bash
moon run :quick       # Fast checks (format + lint)
moon run :ci          # Full pipeline (all quality gates)
moon run :test        # Run tests
moon run :fmt-fix     # Auto-fix formatting
moon run :check       # Fast type check
```

### Workspace Management (Optional)
If using zjj for workspace isolation:
```bash
zjj add <name>        # Create session + workspace
zjj focus <name>      # Switch to session
zjj remove <name>     # Close workspace
zjj list              # Show all sessions
```

---

## Quality Standards

### Lint Configuration
**ABSOLUTE RULE: DO NOT MODIFY clippy or linting configuration.**

If clippy reports warnings or errors, fix the **code**, not the lint rules.

Current strict settings:
```rust
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
```

### Testing Standards
- All code must have comprehensive unit tests
- Test happy paths AND error paths
- Use property-based testing where appropriate
- Follow TDD: RED → GREEN → REFACTOR

### Documentation Standards
- Document all public APIs with Rustdoc
- Include `# Errors` sections on fallible functions
- Add examples for complex functions
- Keep comments concise and focused on "why", not "what"

---

## Build System (Moon)

### Moon Tasks

```yaml
# Quality Checks
moon run :quick       # Format + lint (fast feedback)
moon run :fmt-fix     # Auto-fix formatting issues

# Testing
moon run :test        # Run all tests
moon run :test-doc    # Run documentation tests

# Building
moon run :build       # Build all crates (debug)
moon run :release     # Build release binaries

# Running
moon run :server      # Run clarity-server
moon run :client      # Run clarity-client
```

### Cache Strategy
Moon uses aggressive caching for fast feedback:
- **Format check**: ~6-7ms (cached)
- **Clippy**: ~2-3s (cached)
- **Tests**: ~5-10s (cached)

---

## CI/CD Pipeline

### GitHub Actions (.github/workflows/ci.yml)

**Three parallel jobs:**

1. **Quality** (10 min timeout)
   - Check formatting (rustfmt)
   - Run Clippy (strict mode, `-D warnings`)

2. **Test** (15 min timeout)
   - Matrix: stable + nightly Rust
   - Run all tests
   - Run doc tests

3. **Build** (20 min timeout)
   - Build release binaries
   - Upload artifacts (server + client)

### All Quality Gates Must Pass
- ✅ Zero format violations
- ✅ Zero clippy warnings
- ✅ All tests passing
- ✅ Clean release build

---

## Common Patterns

### Error Handling (Functional Style)

```rust
// WRONG - unwrap()
fn get_user(id: &str) -> User {
    User::find(id).unwrap()  // ❌ Forbidden
}

// CORRECT - Result with ?
fn get_user(id: &str) -> Result<User, DbError> {
    let user = User::find(id)?;  // ✅ Correct
    Ok(user)
}

// CORRECT - Combinators
fn validate_email(email: &str) -> Result<Email, ValidationError> {
    email
        .validate_non_empty()?    // Use ? for propagation
        .validate_max_length(254)? // Chain validators
        .validate_email_format()   // Final check
}
```

### Immutability by Default

```rust
// WRONG - Mutable
fn process_items(items: &mut Vec<Item>) {
    items.push(Item::new());  // ❌ Mutation
}

// CORRECT - Immutable
fn process_items(items: Vec<Item>) -> Vec<Item> {
    items
        .into_iter()
        .chain(Some(Item::new()))  // ✅ New vector
        .collect()
}
```

### Iterator Combinators

```rust
// WRONG - Loop with mutation
let mut result = Vec::new();
for item in items {
    if item.is_valid() {
        result.push(item);
    }
}

// CORRECT - Iterator combinators
let result: Vec<Item> = items
    .into_iter()
    .filter(|item| item.is_valid())
    .collect();
```

---

## Dependencies

### Runtime Dependencies
- **Axum** (0.8) - Web framework
- **SQLx** (0.8) - Database with compile-time query checking
- **Dioxus** (0.7) - Frontend framework
- **Tokio** - Async runtime

### Dev Dependencies
- **Moon** - Build system
- **Cargo-nextest** - Fast test runner
- **Tarpaulin** - Code coverage

### Beads Integration
- `.beads/issues.jsonl` - Issue tracking
- `br` CLI - Bead management (non-invasive, no git auto-push)

---

## Database Architecture

### Migration Strategy
```bash
# Run migrations
sqlx migrate run

# Create new migration
sqlx migrate add <name>

# Revert last migration
sqlx migrate revert
```

### Connection Pool
```rust
// Use PgPool for connection pooling
// Max connections: 5 (configurable)
// Timeout: 30s
// All database operations return DbResult<T>
```

### Schema
- `users` - User accounts with UUID primary keys
- `beads` - Issue tracking integration
- `interviews` - User interview sessions
- `specs` - Specification documents
- `sessions` - Session management

---

## WebSocket Architecture

### Endpoint
- **Route**: `/ws`
- **Protocol**: WebSocket upgrade via Axum
- **Pattern**: Echo handler with broadcast channel

### State Management
```rust
// Use tokio::sync::broadcast for pub/sub
// Channel capacity: 128 messages
// Graceful shutdown on connection errors
```

---

## Quick Reference

### Commands to Run

```bash
# Development
moon run :server      # Start backend server
moon run :client      # Start frontend client

# Quality
moon run :quick       # Fast format + lint check
moon run :ci          # Full quality pipeline

# Testing
moon run :test        # Run all tests
cargo test --workspace --all-features

# Building
moon run :release     # Build release binaries
```

### File Locations

```
Database:    clarity-core/migrations/001_initial_schema.sql
Websocket:   clarity-server/src/main.rs (/ws route)
Responsive:  clarity-client/assets/responsive.css
Sessions:    clarity-core/src/session.rs
Validation:  clarity-core/src/validation.rs
Exit Codes:  clarity-core/src/error.rs
```

---

## Anti-Patterns to Avoid

| Anti-Pattern | Problem | Correct Way |
|--------------|---------|-------------|
| `unwrap()` | Crashes on error | Use `?` or `match` |
| `expect()` | Crashes on error | Use `Result<T, E>` |
| `panic!()` | Crashes process | Return `Err` |
| `todo!()` | Incomplete code | Implement or file bead |
| `mut` by default | Side effects | Use immutable by default |
| Raw cargo commands | Bypasses moon cache | Use `moon run :<task>` |
| Editing clippy.toml | Weakens lint rules | Fix the code instead |

---

## Version: 1.0.0

**Last Updated**: February 2026
**Status**: Production Ready
**Maintainer**: lprior-repo
