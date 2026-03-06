# AGENTS.md - Agent Instructions for AI Agents

**This document is designed for AI agents** - you are the primary audience and user of these guidelines.

## Required Skills for All Development

1. **/functional-rust-generator** - Zero-panic functional Rust patterns
   - Railway-Oriented Programming
   - Zero unwraps, zero panics, zero expects
   - Result<T, E> with proper error propagation

2. Dioxus Skill as well

- Follow common dioxus skills pleas
- Use .7 latest and iodmatic code please
  **How to load skills:**

```
When assigned a bead, automatically invoke these skills before starting implementation.
The tdd15 skill will guide you through writing tests first.
The zjj skill will create an isolated workspace.
The functional-rust-generator skill will ensure zero-panic code.
```

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
