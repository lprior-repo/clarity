# AGENTS.md - Agent Instructions for AI Agents

## Critical Rules

### NEVER Touch Clippy/Lint Configuration
**ABSOLUTE RULE: DO NOT MODIFY clippy or linting configuration files. EVER.**

If clippy reports warnings or errors, fix the **code**, not the lint rules.

### Build System: Moon Only
**NEVER use raw cargo commands.** Always use Moon for all build operations:

```bash
# Correct
moon run :quick       # Format + lint check
moon run :test        # Run tests
moon run :build       # Build all crates
moon run :ci          # Full pipeline
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

### Extensive Testing Philosophy
Tests should **actively try to break the code**. We don't test happy paths - we test:
- Edge cases and boundary conditions
- Error paths and failure modes
- Invalid inputs and malformed data
- Concurrent access and race conditions
- Resource exhaustion and limits
- **If the code can fail, test that it fails gracefully**

### Project Structure
```
clarity/
├── clarity-client/     # Dioxus frontend (responsive CSS, components)
├── clarity-core/       # Shared types, validation, database layer
├── clarity-server/     # Axum backend (WebSocket, REST API)
└── migrations/         # SQLx database migrations
```

### Key Decisions
- **Sync strategy**: Rebase (`jj rebase -d main`)
- **Testing**: Break-it-first philosophy (red queen adversarial testing)
- **Beads**: Hard requirement, always integrate with `.beads/issues.jsonl`
- **Functional Rust**: Zero panic, immutable by default, Result<T, E> throughout

### Tech Stack
- **Axum** (0.8) - Web framework with WebSocket support
- **SQLx** (0.8) - Compile-time checked database queries
- **Dioxus** (0.7) - React-like frontend framework
- **Tokio** - Async runtime
- **PostgreSQL** - Database with UUID primary keys

---

## Quick Reference

### Issue Tracking (Beads)
```bash
br ready              # Find available work
br show <id>          # View issue details
br update <id> --status in_progress  # Claim work
br close <id>         # Complete work
br sync --flush-only  # Sync to JSONL (no git)
git add .beads/
git commit -m "close bead <id>"
```

### Development (Moon CI/CD)
```bash
moon run :quick       # Fast checks (6-7ms with cache!)
moon run :ci          # Full pipeline (parallel)
moon run :fmt-fix     # Auto-fix formatting
moon run :test        # Run tests
moon run :server      # Run Axum server
moon run :client      # Run Dioxus client
```

### Workspace Management (zjj)
```bash
zjj add <name>        # Create isolated workspace
zjj focus <name>      # Switch to workspace
zjj remove <name>     # Close workspace
zjj list              # Show all workspaces
```

### Database Operations
```bash
moon run :db-migrate         # Run migrations
moon run :db-migrate-add     # Create new migration
sqlx migrate run             # Direct SQLx command
```

---

## Hyper-Fast CI/CD Pipeline

This project uses **Moon** for fast cached builds:

### Performance Characteristics
- **6-7ms** for cached tasks (vs ~450ms uncached)
- **Parallel execution** across all crates
- **Aggressive caching** for fast feedback

### Development Workflow

**1. Quick Iteration Loop** (6-7ms with cache):
```bash
# Edit code...
moon run :quick  # Parallel fmt + clippy check
```

**2. Before Committing**:
```bash
moon run :fmt-fix  # Auto-fix formatting
moon run :ci       # Full pipeline
```

### Build System Rules

**ALWAYS use Moon, NEVER raw cargo:**
- ✅ `moon run :test` (cached, fast)
- ✅ `moon run :check` (quick type check)
- ✅ `moon run :build` (dependency-aware)
- ❌ `cargo test` (no caching, slow)
- ❌ `cargo build` (no parallelism)

---

## Testing Philosophy: Break the Code

We don't write tests to prove code works. We write tests to **prove code breaks correctly**.

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

### Example: Good vs Bad Testing

```rust
// ❌ BAD: Only tests happy path
#[test]
fn test_add_user() {
    let user = User::new("test@example.com");
    assert!(user.is_valid());
}

// ✅ GOOD: Tests what breaks
#[test]
fn test_user_rejects_empty_email() {
    let result = User::new("");
    assert!(matches!(result, Err(UserError::EmptyEmail)));
}

#[test]
fn test_user_rejects_invalid_email_format() {
    let result = User::new("not-an-email");
    assert!(matches!(result, Err(UserError::InvalidEmail)));
}

#[test]
fn test_user_rejects_email_too_long() {
    let long_email = "a@".repeat(300) + ".com";
    let result = User::new(&long_email);
    assert!(matches!(result, Err(UserError::EmailTooLong)));
}
```

---

## Functional Rust Patterns

### Error Handling

```rust
// ❌ WRONG: unwrap()
fn get_user(id: &str) -> User {
    User::find(id).unwrap()
}

// ✅ CORRECT: Result with ?
fn get_user(id: &str) -> Result<User, DbError> {
    let user = User::find(id)?;
    Ok(user)
}
```

### Immutability

```rust
// ❌ WRONG: Mutable
fn process_items(items: &mut Vec<Item>) {
    items.push(Item::new());
}

// ✅ CORRECT: Immutable
fn process_items(items: Vec<Item>) -> Vec<Item> {
    items
        .into_iter()
        .chain(Some(Item::new()))
        .collect()
}
```

### Iterator Combinators

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
