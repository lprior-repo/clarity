# Testing Standards and Guidelines

## Overview

This document defines the testing standards for the Clarity project, aligned with our zero-unwrap philosophy and functional programming principles.

## Core Principles

1. **Zero-Unwrap in Tests**: Tests should demonstrate proper error handling, not hide it with `unwrap()` or `expect()`
2. **Explicit is Better Than Implicit**: Make assertions and error handling visible
3. **Test Error Paths**: Don't just test success cases - verify error handling works
4. **Functional Patterns**: Use Result/Option combinators in test assertions
5. **Consistent Lint Policies**: All test files follow the same lint standards

## Test File Lint Policy

### Standard Test File Header

```rust
//! Module/Purpose description
//!
//! Brief overview of what these tests verify and why they matter.

// No lint attributes needed - workspace defaults apply
// Tests follow zero-unwrap philosophy by default
```

### Allowed Lint Attributes (Use Sparingly)

**DO NOT USE**:
- `#![allow(clippy::unwrap_used)]` - Violates zero-unwrap philosophy
- `#![allow(clippy::expect_used)]` - Violates zero-unwrap philosophy
- `#![deny(clippy::unwrap_used)]` then use `unwrap()` - Contradictory!

**ONLY USE WHEN NECESSARY**:
- `#![allow(clippy::disallowed_methods)]` - When testing deprecated methods specifically
- `#![allow(clippy::panic)]` - When explicitly testing panic behavior

### Workspace Defaults Apply

The workspace `Cargo.toml` and `.clippy.toml` define the baseline:
- `unwrap_used = "deny"` - No unwrap in production OR tests
- `expect_used = "deny"` - No expect in production OR tests
- `panic = "deny"` - No panics in production OR tests

## Test Writing Patterns

### Pattern 1: Test Data Construction

**GOOD** - Builder pattern:
```rust
#[test]
fn test_user_validation() {
    // Given: Valid test data via builder
    let user = TestUser::new()
        .with_email("test@example.com")
        .with_name("Test User");

    // When: Validating
    let result = user.validate();

    // Then: Should succeed
    assert!(result.is_ok(), "User should be valid");
}
```

**BAD** - Using unwrap:
```rust
#[test]
fn test_user_validation() {
    let user = TestUser::new()
        .unwrap()  // DON'T - hides test data construction errors
        .with_email("test@example.com");
}
```

### Pattern 2: Result Assertions

**GOOD** - Pattern matching:
```rust
#[test]
fn test_invalid_email_rejected() {
    let result = Email::new("not-an-email");

    // Then: Should return error
    assert!(result.is_err(), "Invalid email should be rejected");

    // And: Error should be specific
    if let Err(EmailError::InvalidFormat) = result {
        // Test passes - correct error type
    } else {
        panic!("Expected InvalidFormat error, got: {:?}", result);
    }
}
```

**GOOD** - Using unwrap_err (when error is guaranteed):
```rust
#[test]
fn test_invalid_email_rejected() {
    let result = Email::new("not-an-email");

    assert!(result.is_err());
    let err = result.unwrap_err();  // Safe - we just checked it's an error
    assert!(matches!(err, EmailError::InvalidFormat));
}
```

**BAD** - Blind unwrap:
```rust
#[test]
fn test_email_creation() {
    let email = Email::new("test@example.com").unwrap();  // DON'T
    assert_eq!(email.value(), "test@example.com");
}
```

### Pattern 3: Integration Tests with Result Propagation

**GOOD** - Propagate errors:
```rust
#[tokio::test]
async fn test_database_operations() -> Result<(), Box<dyn std::error::Error>> {
    // Given: Test database
    let pool = create_test_pool()?;

    // When: Creating a user
    let user = create_user(&pool, "test@example.com").await?;

    // Then: User should exist
    let fetched = get_user(&pool, &user.id).await?;
    assert_eq!(fetched.email, "test@example.com");

    // Cleanup
    delete_user(&pool, &user.id).await?;
    Ok(())
}
```

**BAD** - Expect in test setup:
```rust
#[tokio::test]
async fn test_database_operations() {
    let pool = create_test_pool().expect("Pool creation failed");  // DON'T
    let user = create_user(&pool, "test@example.com")
        .await
        .expect("User creation failed");  // DON'T
}
```

### Pattern 4: Testing Error Paths

**GOOD** - Explicit error testing:
```rust
#[test]
fn test_database_connection_failure() {
    // Given: Invalid connection string
    let invalid_url = "postgres://invalid:9999/no-db";

    // When: Attempting to connect
    let result = connect_to_database(invalid_url);

    // Then: Should return specific error
    assert!(result.is_err());
    if let Err(DbError::ConnectionFailed { .. }) = result {
        // Correct error type
    } else {
        panic!("Expected ConnectionFailed, got: {:?}", result);
    }
}
```

### Pattern 5: Async Tests

**GOOD** - Proper async error handling:
```rust
#[tokio::test]
async fn test_websocket_broadcast() -> Result<(), Box<dyn std::error::Error>> {
    // Given: WebSocket state
    let state = WebSocketState::new(100)?;

    // When: Broadcasting message
    let result = state.broadcast("test message".to_string()).await;

    // Then: Should succeed or fail gracefully
    assert!(result.is_ok() || result.is_err());  // Documented behavior
    Ok(())
}
```

## Test File Template

```rust
//! [Module/Feature Name] Tests
//!
//! These tests verify [what the tests verify].
//!
//! Test Coverage:
//! - [Test case 1]
//! - [Test case 2]
//! - [Test case 3]
//!
//! See [related documentation or bead ID]

use [crate_or_module_names];

/// Test 1: [Brief description of what this tests]
///
/// **GIVEN** [preconditions]
/// **WHEN** [action performed]
/// **THEN** [expected outcome]
#[test]
fn test_[feature_or_behavior]() {
    // Given: [setup code]
    let input = [test_data];

    // When: [action being tested]
    let result = [function_under_test](input);

    // Then: [assertions]
    assert!(result.is_ok(), "Should succeed with valid input");

    // Additional verification
    if let Ok(value) = result {
        assert_eq!(value.field, expected);
    }
}

/// Test 2: [Brief description of error case]
///
/// **GIVEN** [preconditions including invalid state]
/// **WHEN** [action that should fail]
/// **THEN** [specific error should be returned]
#[test]
fn test_[error_case]() {
    // Given: [invalid input]
    let invalid_input = [test_data];

    // When: [action being tested]
    let result = [function_under_test](invalid_input);

    // Then: Should fail with specific error
    assert!(result.is_err());
    if let Err(ExpectedError) = result {
        // Test passes
    } else {
        panic!("Expected ExpectedError, got: {:?}", result);
    }
}

// Additional tests follow the same pattern...
```

## Specific Test Types

### Unit Tests

- Test single functions in isolation
- Use builder patterns for test data
- Test both success and error paths
- Keep tests focused and independent

### Integration Tests

- Test multiple components working together
- Propagate errors with `Result` return type
- Use `?` operator for clean error handling
- Clean up resources in test teardown

### Async Tests

- Use `#[tokio::test]` for async tests
- Return `Result<(), Error>` to propagate errors
- Avoid blocking operations in async tests
- Test timeout behavior where applicable

## Common Mistakes to Avoid

### 1. Contradictory Lint Declarations

```rust
#![deny(clippy::unwrap_used)]  // DON'T DO THIS

#[test]
fn test_something() {
    let value = some_operation().unwrap();  // CONTRADICTION!
}
```

### 2. Allow Lints That Violate Philosophy

```rust
#![allow(clippy::unwrap_used)]  // DON'T - undermines zero-unwrap

#[test]
fn test_something() {
    let value = some_operation().unwrap();  // Now it's allowed, but shouldn't be
}
```

### 3. Using Expect Instead of Unwrap

```rust
#[test]
fn test_something() {
    let value = some_operation().expect("This panics");  // STILL A PANIC
}
```

### 4. Ignoring Test Failures

```rust
#[test]
fn test_something() {
    let result = some_operation();
    // No assertion - test always passes!
}
```

## Verification

All test changes must pass:

```bash
moon run :quick
```

This runs:
- All tests across all crates
- Clippy linting
- Formatting checks
- Compilation checks

## Resources

- [Zero-Unwrap Philosophy](./zero-unwrap-philosophy.md)
- [Rust Testing Guide](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [TDD15 Workflow](/.tdd15-cache/plan.md)
- [Clippy Lints](https://rust-lang.github.io/rust-clippy/master/)

## Enforcement

These standards are enforced through:

1. **CI/CD**: All PRs must pass `moon run :quick`
2. **Code Review**: Check test files for unwrap/expect usage
3. **Lint Configuration**: Workspace-level deny lints
4. **Documentation**: This file as the source of truth

When reviewing test code, verify:

- [ ] No `unwrap()` calls
- [ ] No `expect()` calls
- [ ] No contradictory lint declarations
- [ ] Error paths are tested
- [ ] Test data uses builder patterns
- [ ] Async tests use Result propagation
- [ ] Tests are independent and focused
