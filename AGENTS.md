# AGENTS.md

## Functional Programming Principles

This project follows functional programming principles with the following key concepts:

### Core Concepts
1. **Immutability**: All data structures are immutable by design
2. **Pure Functions**: Functions have no side effects and always return the same output for the same input
3. **Error Handling**: Use `Result` types instead of exceptions
4. **Zero Unwraps/Panics**: Avoid `unwrap()`, `expect()`, and `panic!` in production code
5. **Explicit Error Propagation**: Use `?` operator for error handling
6. **Type Safety**: Leverage Rust's type system to prevent runtime errors
7. **Function Composition**: Build complex operations from simple functions

### Pattern Examples

#### Error Handling with Result
```rust
fn divide(a: f64, b: f64) -> Result<f64, &'static str> {
    if b == 0.0 {
        Err("Division by zero")
    } else {
        Ok(a / b)
    }
}
```

#### Pure Functions
```rust
fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

#### Error Propagation
```rust
fn process_data(data: &str) -> Result<String, Box<dyn std::error::Error>> {
    let parsed = data.parse::<i32>()?;
    let doubled = parsed * 2;
    Ok(doubled.to_string())
}
```

#### Functional Style
```rust
fn process_numbers(numbers: Vec<i32>) -> Vec<i32> {
    numbers
        .into_iter()
        .filter(|&x| x > 0)
        .map(|x| x * 2)
        .collect()
}
```

## Code Quality Standards

### Testing
- All code must be tested with comprehensive unit tests
- Use property-based testing where appropriate
- Test error cases and edge conditions
- Follow the Red-Green-Refactor cycle

### Documentation
- Document all public APIs with Rustdoc
- Include examples for complex functions
- Maintain clear and concise comments

### Performance
- Avoid unnecessary allocations
- Use `Cow` for efficient string handling
- Consider lazy evaluation where appropriate

## Build System Integration

### Moon Configuration
This project uses Moon for build management with the following configuration:

#### Tasks
- `build`: Build all crates
- `test`: Run all tests
- `fmt`: Format code with rustfmt
- `clippy`: Run clippy linter
- `run-server`: Run the server
- `run-client`: Run the client

### Bazel-Remote Integration
The project is structured to take advantage of bazel-remote caching for:
- Faster builds
- Reusable build artifacts
- Improved CI/CD performance

## Development Workflow

### Workspace Management
This project uses zjj for isolated workspace management:
1. Create new workspace with `zjj spawn`
2. Use `zjj focus` to switch between workspaces
3. Sync changes with `zjj sync`

### Workflows
All development work must follow these workflows:

1. **Development**: Use `tdd15` for all new feature development
2. **Code Review**: Use `red-queen` for adversarial review of all code
3. **Session Completion**: Use `land-the-plane` skill for final session cleanup

This ensures consistent, high-quality development practices across all team members.