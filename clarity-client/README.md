# Clarity Client - Dioxus Frontend

This is the web frontend for the Clarity application, built with [Dioxus](https://dioxuslabs.com/).

## Features

- **Modern Reactive UI**: Built with Dioxus for efficient, type-safe reactive programming
- **Hot Reload**: Automatic hot reload in development mode for fast iteration
- **Result-Based Error Handling**: All errors handled through `Result` types - zero unwraps, zero panics
- **Routing**: Built-in routing system with navigation support
- **Responsive Design**: Mobile-first approach with breakpoints for tablets and desktops
- **Dark Mode**: Automatic dark mode support based on system preferences
- **Accessibility**: WCAG AA compliant with keyboard navigation and screen reader support

## Development

### Prerequisites

- Rust toolchain (stable)
- Dioxus CLI: `cargo install dioxus-cli`

### Running in Development

```bash
# Using Dioxus CLI (recommended for development with hot reload)
dx

# Or using cargo
cargo run
```

The application will be available at `http://localhost:8080`.

### Building for Production

```bash
# Build for web
dx build --release

# The output will be in the `dist/` directory
```

## Architecture

### Component Structure

- **App**: Root component managing routing and global state
- **HomePage**: Landing page component
- **AboutPage**: About page component
- **NotFoundPage**: 404 error page component
- **Link**: Reusable navigation link component

### State Management

The application uses Dioxus's `use_signal` hook for reactive state management:

```rust
pub struct AppState {
    pub current_route: String,
    pub error: Option<AppError>,
}
```

### Error Handling

All operations return `Result` types following the zero-unwrap philosophy:

```rust
pub fn navigate_to(&mut self, path: String) -> Result<(), AppError> {
    if path.is_empty() {
        return Err(AppError::InvalidRoute("Route path cannot be empty".to_string()));
    }
    // ...
}
```

## Testing

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_app_state_navigation_flow
```

## Code Quality

This project enforces strict code quality standards:

- **Zero Unwraps**: No `.unwrap()` or `.expect()` calls
- **Zero Panics**: No panicking in production code
- **Result-Based Error Handling**: All errors propagated through `Result` types
- **Comprehensive Testing**: Unit tests for all components
- **Clippy Lints**: All clippy warnings enforced

## Hot Reload Configuration

Hot reload is automatically enabled in debug mode. The configuration is in `dx.toml`:

```toml
[web.watcher]
watch_path = ["src"]
reload_html = true
watch_style = true

[web.compiler]
hot_reload = true
```

## Integration with Clarity Core

The frontend integrates with `clarity-core` types:

```rust
use clarity_core::{Schema, SchemaRegistry, SpecName, Url, HttpMethod};
```

These types will be used in future beads to display and interact with backend data.

## File Structure

```
clarity-client/
├── src/
│   ├── main.rs          # Application entry point
│   ├── lib.rs           # Library exports
│   └── app.rs           # Main app component and routing
├── public/
│   └── style.css        # Application styles
├── tests/
│   └── integration_test.rs  # Integration tests
├── Cargo.toml           # Dependencies
├── dx.toml             # Dioxus configuration
└── README.md           # This file
```

## Future Enhancements

- [ ] API integration with backend services
- [ ] Real-time updates with WebSockets
- [ ] Form validation and submission
- [ ] Authentication UI
- [ ] Interview management interface
- [ ] Documentation viewer
- [ ] Rich text editor
-