# AGENTS.md - Agent Instructions for AI Agents

**This document is designed for AI agents** - you are the primary audience and user of these guidelines.

## Required Skills for All Development

1. **holzman-rust** - Canonical Rust implementation, repair, review, and performance doctrine
   - Invoke before any Rust implementation, repair, review, async work, storage work, low-level systems work, or performance claim
   - Enforces NASA/JPL Power-of-Ten style reliability, bounded resources, explicit error taxonomies, and evidence-backed performance
   - **Precedence rule: if Holzman Rust conflicts with another Rust skill, Holzman wins**

2. **functional-rust** / **functional-rust-generator** - Zero-panic functional Rust patterns
   - Railway-Oriented Programming
   - Zero unwraps, zero panics, zero expects
   - Result<T, E> with proper error propagation
   - Data → Calculations → Actions layering

3. **dioxus** - Use only for future Rust UI work

- Clarity is currently CLI-first; do not add UI work unless explicitly requested
- If UI work returns, follow Dioxus 0.7 idiomatic patterns
  **How to load skills:**

```
When assigned a bead, automatically invoke these skills before starting implementation.
The tdd15 skill will guide you through writing tests first.
The zjj skill will create an isolated workspace.
The holzman-rust skill is mandatory for Rust work and wins on conflicts.
The functional-rust skill will ensure zero-panic functional code.
```

## Rust Skill Precedence

- Always invoke **holzman-rust** and **functional-rust** before Rust implementation, repair, review, or performance work.
- Holzman Rust is the tie-breaker when skill guidance conflicts.
- Functional Rust still applies for Data → Calculations → Actions, typed errors, and zero-panic railway-oriented code.
- Do not introduce production `unsafe`, `unwrap`, `expect`, `panic`, `todo`, `unimplemented`, unchecked indexing, unchecked arithmetic, lossy `as` conversions, or ignored fallible results.

## Dolt Beads Database

- Clarity beads use DoltHub remote `priorlewis43/clarity-database`.
- Local beads metadata belongs in `.beads/metadata.json` with `dolt_database` matching the active bd server database. With the clone at `.beads/dolt/`, `bd bootstrap` currently resolves this to `dolt`.
- Local Dolt clone belongs in `.beads/dolt/` and must never be committed to Git.
- Always use the `priorlewis43/` DoltHub prefix for Dolt remotes.
- Use `bd dolt pull` / `bd dolt push` to sync issue state when the beads database is configured.

## Functional Programming Principles

This project follows functional programming principles with the following key concepts:

1. **Immutability**: All data structures are immutable by design
2. **Pure Functions**: No side effects, same input → same output
3. **Error Handling**: `Result` types instead of exceptions
4. **Type Safety**: Leverage Rust's type system to prevent runtime errors
5. **Function Composition**: Build complex operations from simple functions

## ATDD: Acceptance Test-Driven Development

**You MUST write tests FIRST.** This is non-negotiable.

1. **RED**: Write a failing test based on acceptance criteria
2. **GREEN**: Write minimal code to make the test pass
3. **REFACTOR**: Improve the code while keeping tests green

**Why tests first?**

- Tests document the expected behavior
- Tests prove the code works before it exists
- Tests prevent over-engineering
- Tests give you confidence to refactor

**Your workflow:**

```
1. Read the bead's acceptance criteria
2. Write a test that validates those criteria
3. Run the test (it WILL fail - RED)
4. Write the minimal implementation
5. Run the test again (it should pass - GREEN)
6. Refactor for clarity and functional purity
7. Run all tests (should still pass - REFACTOR)
```

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

<!-- BEGIN BEADS INTEGRATION v:1 profile:full hash:d4f96305 -->
## Issue Tracking with bd (beads)

**IMPORTANT**: This project uses **bd (beads)** for ALL issue tracking. Do NOT use markdown TODOs, task lists, or other tracking methods.

### Why bd?

- Dependency-aware: Track blockers and relationships between issues
- Git-friendly: Dolt-powered version control with native sync
- Agent-optimized: JSON output, ready work detection, discovered-from links
- Prevents duplicate tracking systems and confusion

### Quick Start

**Check for ready work:**

```bash
bd ready --json
```

**Create new issues:**

```bash
bd create "Issue title" --description="Detailed context" -t bug|feature|task -p 0-4 --json
bd create "Issue title" --description="What this issue is about" -p 1 --deps discovered-from:bd-123 --json
```

**Claim and update:**

```bash
bd update <id> --claim --json
bd update bd-42 --priority 1 --json
```

**Complete work:**

```bash
bd close bd-42 --reason "Completed" --json
```

### Issue Types

- `bug` - Something broken
- `feature` - New functionality
- `task` - Work item (tests, docs, refactoring)
- `epic` - Large feature with subtasks
- `chore` - Maintenance (dependencies, tooling)

### Priorities

- `0` - Critical (security, data loss, broken builds)
- `1` - High (major features, important bugs)
- `2` - Medium (default, nice-to-have)
- `3` - Low (polish, optimization)
- `4` - Backlog (future ideas)

### Workflow for AI Agents

1. **Check ready work**: `bd ready` shows unblocked issues
2. **Claim your task atomically**: `bd update <id> --claim`
3. **Work on it**: Implement, test, document
4. **Discover new work?** Create linked issue:
   - `bd create "Found bug" --description="Details about what was found" -p 1 --deps discovered-from:<parent-id>`
5. **Complete**: `bd close <id> --reason "Done"`

### Auto-Sync

bd automatically syncs via Dolt:

- Each write auto-commits to Dolt history
- Use `bd dolt push`/`bd dolt pull` for remote sync
- No manual export/import needed!

### Important Rules

- ✅ Use bd for ALL task tracking
- ✅ Always use `--json` flag for programmatic use
- ✅ Link discovered work with `discovered-from` dependencies
- ✅ Check `bd ready` before asking "what should I work on?"
- ❌ Do NOT create markdown TODO lists
- ❌ Do NOT use external issue trackers
- ❌ Do NOT duplicate tracking systems

For more details, see README.md and docs/QUICKSTART.md.

## Landing the Plane (Session Completion)

**When ending a work session**, you MUST complete ALL steps below. Work is NOT complete until `git push` succeeds.

**MANDATORY WORKFLOW:**

1. **File issues for remaining work** - Create issues for anything that needs follow-up
2. **Run quality gates** (if code changed) - Tests, linters, builds
3. **Update issue status** - Close finished work, update in-progress items
4. **PUSH TO REMOTE** - This is MANDATORY:
   ```bash
   git pull --rebase
   bd dolt push
   git push
   git status  # MUST show "up to date with origin"
   ```
5. **Clean up** - Clear stashes, prune remote branches
6. **Verify** - All changes committed AND pushed
7. **Hand off** - Provide context for next session

**CRITICAL RULES:**
- Work is NOT complete until `git push` succeeds
- NEVER stop before pushing - that leaves work stranded locally
- NEVER say "ready to push when you are" - YOU must push
- If push fails, resolve and retry until it succeeds

<!-- END BEADS INTEGRATION -->
