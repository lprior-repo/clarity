# Zero-Unwrap Philosophy

## Core Principle

**No `unwrap()` or `expect()` calls in production code.**

This philosophy is enforced through:
1. Clippy lints (configured in `.clippy.toml` and `Cargo.toml`)
2. Compiler warnings/errors via workspace lint configuration
3. Code review practices
4. Testing strategies that demonstrate proper error handling

## Why Zero-Unwrap?

### 1. **Explicit Error Handling**
Every fallible operation must explicitly handle its errors. This makes error paths visible and testable.

### 2. **Railway-Oriented Programming**
Using `Result<T, E>` and `Option<T>` combinators creates clear success/failure tracks:
```rust
// GOOD: Functional composition
fn process(input: &str) -> Result<Output, Error> {
    parse(input)?
        .validate()?
        .transform()
}

// BAD: Hidden panic points
fn process(input: &str) -> Output {
    let parsed = parse(input).unwrap();  // Panics on parse error
    parsed.validate().unwrap()           // Panics on validation error
}
```

### 3. **Testability**
When errors are explicit, you can test error paths:
```rust
#[test]
fn test_parse_invalid_input() {
    let result = parse("invalid");
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Error::InvalidInput);
}
```

## Functional Error Handling Patterns

### Pattern 1: The `?` Operator (Railway Pattern)
```rust
fn read_config(path: &Path) -> Result<Config, IoError> {
    let content = fs::read_to_string(path)?;  // Early return on error
    let config = toml::from_str(&content)?;   // Early return on error
    Ok(config)
}
```

### Pattern 2: `map()` for Transformations
```rust
fn get_user_email(id: UserId) -> Result<Email, Error> {
    get_user(id)?
        .map(|user| user.email)
}

// Instead of:
fn get_user_email_bad(id: UserId) -> Result<Email, Error> {
    let user = get_user(id).unwrap();
    Ok(user.email)
}
```

### Pattern 3: `and_then()` for Chaining
```rust
fn get_user_posts(id: UserId) -> Result<Vec<Post>, Error> {
    get_user(id)?
        .and_then(|user| get_posts_by_user(&user.id))
}

// Instead of:
fn get_user_posts_bad(id: UserId) -> Result<Vec<Post>, Error> {
    let user = get_user(id).unwrap();
    get_posts_by_user(&user.id)
}
```

### Pattern 4: `ok_or()` for Option to Result
```rust
fn find_user(id: &str) -> Result<User, Error> {
    user_cache
        .get(id)
        .ok_or(Error::UserNotFound(id.to_string()))
}

// Instead of:
fn find_user_bad(id: &str) -> User {
    user_cache.get(id).expect("User not found")
}
```

### Pattern 5: Context with `map_err()`
```rust
fn load_config() -> Result<Config, Error> {
    fs::read_to_string("config.toml")
        .map_err(|e| Error::ConfigLoadFailed {
            path: "config.toml".into(),
            source: e,
        })
        .and_then(|content| toml::from_str(&content)
            .map_err(|e| Error::ConfigParseFailed { source: e }))
}
```

## Testing with Results

### Test Data: Use Builder Pattern
```rust
impl TestUser {
    fn new() -> Self {
        TestUser {
            id: "test-id".into(),
            email: "test@example.com".into(),
            // ... construct valid test data
        }
    }
}

#[test]
fn test_user_creation() {
    let user = TestUser::new();
    assert!(user.validate().is_ok());
}
```

### Test Error Paths Explicitly
```rust
#[test]
fn test_invalid_email_rejected() {
    let result = Email::new("not-an-email");
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), EmailError::InvalidFormat);
}
```

### Integration Tests: Use Result Propagation
```rust
#[tokio::test]
async fn test_database_operations() -> Result<(), Error> {
    let pool = create_test_pool()?;
    let user = create_test_user(&pool)?;

    // Test operations
    let fetched = get_user(&pool, &user.id)?;
    assert_eq!(fetched.id, user.id);

    // Cleanup
    delete_user(&pool, &user.id)?;
    Ok(())
}
```

## When (If Ever) to Use unwrap()

### Acceptable Uses:
1. **Test data construction** (but prefer builders)
2. **Examples in documentation** (clearly mark as examples)
3. **Prototyping code** (never commit to main)

### Even Then: Consider Alternatives
```rust
// Instead of unwrap in tests:
#[test]
fn test_something() {
    let result = operation();
    if let Ok(value) = result {
        assert_eq!(value.field, expected);
    } else {
        panic!("Expected Ok, got: {:?}", result);
    }
}
```

## Enforcement

### Automated Checks
1. **Clippy**: `cargo clippy --all-targets --all-features -- -D warnings`
2. **Workspace lints**: Configured in `Cargo.toml` `[workspace.lints.clippy]`
3. **CI**: Must pass `moon run :quick` before merging

### Code Review Checklist
- [ ] No `unwrap()` calls in production code
- [ ] No `expect()` calls in production code
- [ ] All fallible operations return `Result<T, E>`
- [ ] Error paths are tested
- [ ] Error types provide meaningful context

## Migration Strategy

For existing code with unwrap:

1. **Identify**: Use `cargo clippy` to find all unwrap calls
2. **Analyze**: Determine error type for each operation
3. **Replace**: Use `?` operator or combinators
4. **Test**: Add tests for error paths
5. **Verify**: Run `moon run :quick`

## Resources

- [Error Handling in Rust](https://doc.rust-lang.org/book/ch09-00-error-handling.html)
- [Railway-Oriented Programming](https://fsharpforfunandprofit.com/rop/)
- [Results and Combinators](https://stegosaurusdormant.com/understanding-rust-result-and-combinators/)
