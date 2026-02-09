# REST API Reference

## Overview

The Clarity server provides a REST API built on [Axum](https://github.com/tokio-rs/axum), a modular web framework for Rust. The server currently serves static web assets and HTML content.

**Base URL**: `http://127.0.0.1:4123`

**Server**: Axum (async Rust web framework)
**Default Port**: 4123
**Protocol**: HTTP

---

## Endpoints

### GET /

Returns the main HTML page for the Clarity application.

**Request**
- **Method**: `GET`
- **Path**: `/`
- **Headers**: None required

**Response**
- **Status Code**: `200 OK`
- **Content-Type**: `text/html`
- **Body**: HTML document with embedded CSS reference

**Example Response**
```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Clarity Application</title>
  <link rel="stylesheet" href="/assets/responsive.css">
</head>
<body>
  <div class="container">
    <h1>Clarity Application</h1>
    <p>Welcome to Clarity with responsive design!</p>
  </div>
</body>
</html>
```

**Implementation Details**
- The HTML is statically embedded in the Rust binary
- Uses Axum's `Html` type wrapper for proper content-type handling
- References responsive CSS for mobile-friendly design

---

### GET /assets/responsive.css

Serves the responsive CSS stylesheet for the Clarity application.

**Request**
- **Method**: `GET`
- **Path**: `/assets/responsive.css`
- **Headers**: None required

**Response**
- **Status Code**: `200 OK`
- **Content-Type**: `text/css; charset=utf-8`
- **Body**: CSS stylesheet content

**Implementation Details**
- CSS is embedded at compile time using Rust's `include_str!()` macro
- Served from `clarity-client/assets/responsive.css`
- Content type is explicitly set to `text/css; charset=utf-8`
- This approach avoids fragile runtime path dependencies

**Example Headers**
```http
HTTP/1.1 200 OK
content-type: text/css; charset=utf-8
content-length: <varies>
```

---

## Architecture

### Router Configuration

The server uses Axum's `Router` for route definition:

```rust
let app = Router::new()
  .route("/", get(root))
  .route("/assets/responsive.css", get(serve_css));
```

### Handler Functions

#### root() → Html<&'static str>
- **Purpose**: Serves the main HTML page
- **Return Type**: `Html<&'static str>` - HTML with static lifetime
- **Implementation**: Returns static HTML string

#### serve_css() → impl IntoResponse
- **Purpose**: Serves CSS with proper content-type headers
- **Return Type**: `impl IntoResponse` - Flexible response type
- **Implementation**: Sets content-type header and returns CSS content

### Performance Optimizations

1. **Global Allocator**: Uses `mimalloc` for 20-30% speedup over system allocator
2. **Compile-time Embedding**: CSS embedded at compile time to avoid runtime I/O
3. **Async Runtime**: Built on Tokio for efficient async I/O

---

## Error Handling

The server currently has minimal error handling for the documented endpoints:

- **Port Binding**: Fails if port 4123 is already in use
- **Invalid Routes**: Returns 404 Not Found for undefined routes
- **Server Errors**: Returns 500 Internal Server Error for unexpected failures

Future versions may include more comprehensive error handling and custom error types.

---

## Testing

Integration tests are located in:
- `/home/lewis/src/clarity/clarity-server/tests/websocket_tests.rs`
- `/home/lewis/src/clarity/clarity-server/tests/allocator_test.rs`
- `/home/lewis/src/clarity/clarity-server/tests/zero_unwrap_tests.rs`

Unit tests are located in:
- `/home/lewis/src/clarity/clarity-server/src/server_tests.rs`
- `/home/lewis/src/clarity/clarity-server/src/error_tests.rs`

---

## Running the Server

### Development
```bash
cd clarity-server
cargo run
```

### Production
```bash
cd clarity-server
cargo build --release
./target/release/clarity-server
```

The server will start listening on `http://127.0.0.1:4123` and log the startup message.

---

## Future Enhancements

Potential future REST endpoints may include:

- API endpoints for dynamic content
- WebSocket support for real-time updates (already tested)
- RESTful CRUD operations for data management
- Authentication and authorization endpoints
- Static file serving for additional assets

---

## See Also

- [Axum Documentation](https://docs.rs/axum/)
- [Tokio Documentation](https://tokio.rs/)
- [Server Implementation](../clarity-server/src/main.rs)
- [Client Code](../clarity-client/)

---

*Last Updated: 2026-02-09*
*Version: 1.0.0*