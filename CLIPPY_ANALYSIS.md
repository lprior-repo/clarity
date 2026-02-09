# Clippy Analysis Report

**Generated**: 2026-02-08
**Agent**: swarm22-clippy-check

## Summary

Total clippy errors: **89**

### Breakdown by Type

- **unwrap() on Result**: 66 errors
- **expect() on Result**: 14 errors
- **unwrap_err() on Result**: 7 errors
- **Compilation errors**: 2 errors (blocking test compilation)

### Breakdown by File

| File | Error Count |
|------|-------------|
| `clarity-core/src/interview.rs` | 78 |
| `clarity-core/src/path_utils.rs` | 32 |
| `clarity-core/tests/question_types_test.rs` | 27 |
| `clarity-core/src/db/sqlite_pool.rs` | 19 |
| `clarity-core/src/types/question.rs` | 9 |
| `clarity-core/src/json_formatter.rs` | 8 |
| `clarity-core/src/formatter.rs` | 4 |
| `clarity-core/src/lib.rs` | 3 |
| `clarity-core/src/db/pool.rs` | 2 |
| `clarity-core/src/db/mod.rs` | 1 |
| `clarity-client/` | 2 |

## Issues by Category

### 1. Test Code Issues (42 errors)
**Pattern**: Test code using `unwrap()`, `expect()`, `unwrap_err()`

**Files**:
- `clarity-core/tests/question_types_test.rs` (27 errors)
- `clarity-core/src/path_utils.rs` (32 errors - all in tests)
- `clarity-core/src/db/sqlite_pool.rs` (19 errors - all in tests)
- `clarity-core/src/db/pool.rs` (2 errors - all in tests)

**Context**: These are in `#[cfg(test)]` modules or test files where unwrap is acceptable.

### 2. Production Code Issues (47 errors)
**Pattern**: Production code using `unwrap()`, `expect()`, `unwrap_err()`

**Files**:
- `clarity-core/src/interview.rs` (78 errors - mixed production/test)
- `clarity-core/src/types/question.rs` (9 errors)
- `clarity-core/src/json_formatter.rs` (8 errors)
- `clarity-core/src/formatter.rs` (4 errors)
- `clarity-core/src/lib.rs` (3 errors)
- `clarity-core/src/db/mod.rs` (1 error)

## Lint Configuration

The following crates have strict clippy lints enabled:
- `clarity-core/src/db/mod.rs`: `#![deny(clippy::unwrap_used)]`
- `clarity-core/src/db/sqlite_pool.rs`: `#![deny(clippy::expect_used)]`
- `clarity-core/src/interview.rs`: `#![warn(clippy::pedantic)]`, `#![warn(clippy::nursery)]`

## Recommended Strategy

### Phase 1: Test Code (Low Risk)
Create beads to allow unwrap in test code by:
1. Adding `#![allow(clippy::unwrap_used)]` in test modules
2. Or using `.expect()` with descriptive messages

### Phase 2: Production Code (Medium Risk)
Create beads to fix production code by:
1. Replacing `unwrap()` with proper error handling
2. Using `?` operator where appropriate
3. Using `.expect()` with clear messages for truly infallible operations
4. Using `unwrap_or_default()` for safe defaults

### Phase 3: Lint Configuration (Low Risk)
Consider adjusting lint levels:
- Move some `deny` to `warn` for production code
- Keep `deny` for database code (high reliability required)

## Atomic Beads Needed

Based on file boundaries and issue types:

1. **bead-clippy-test-question-types**: Fix test file (27 issues)
2. **bead-clippy-path-utils-tests**: Fix path_utils tests (32 issues)
3. **bead-clippy-db-tests**: Fix database tests (21 issues)
4. **bead-clippy-interview-prod**: Fix interview.rs production code (78 issues)
5. **bead-clippy-question-types**: Fix types/question.rs (9 issues)
6. **bead-clippy-json-formatter**: Fix json_formatter.rs (8 issues)
7. **bead-clippy-formatter**: Fix formatter.rs (4 issues)
8. **bead-clippy-lib**: Fix lib.rs (3 issues)
9. **bead-clippy-db-mod**: Fix db/mod.rs (1 issue)
10. **bead-clippy-client**: Fix clarity-client (2 issues)

## Priority Order

1. **High**: Database tests (blocks other work)
2. **Medium**: Production code in interview.rs, question.rs
3. **Low**: Test files (non-blocking)
4. **Low**: Client code (separate crate)
