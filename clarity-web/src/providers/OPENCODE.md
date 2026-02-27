# OpenCode Extraction Provider

## Overview

The OpenCode provider is a production-ready implementation of the `ExtractionProvider` trait that communicates with OpenCode session APIs to extract structured fields from unstructured text.

## Architecture

### Core Components

1. **OpenCodeProvider** - Main client struct
   - `endpoint`: Base URL of the OpenCode API
   - `session_id`: Unique session identifier for tracking
   - `model`: Optional model override (for example `zai-coding-plan/glm-5`)
   - `routing_provider`: Optional backend routing provider
   - `client`: HTTP client with 30-second timeout

2. **HTTP Client Configuration**
    - 30-second timeout on all requests
    - Session ID header (`X-Session-ID`) included in health checks
    - Automatic error mapping from HTTP errors to domain errors
    - Extraction path uses OpenCode session APIs (`POST /session`, `POST /session/:id/message`)

3. **Error Handling**
   - Network errors → `ExtractionError::NetworkError`
   - Timeouts → `ExtractionError::Timeout`
   - 401/403 → `ExtractionError::AuthenticationError`
   - 429 → `ExtractionError::RateLimited`
   - 400/422 → `ExtractionError::InvalidInput`
   - 402 → `ExtractionError::QuotaExceeded`
   - 500+ → `ExtractionError::ApiError`

## API Endpoints

### POST /session

Create an OpenCode session for extraction.

**Request:**
```json
{
  "title": "clarity-extraction"
}
```

**Response:**
```json
{
  "id": "ses_xxx",
  "title": "clarity-extraction"
}
```

### Extraction result format

Provider expects assistant text to contain JSON like:

```json
{
  "fields": [
    {
      "name": "name",
      "field_type": "text",
      "value": "John Doe",
      "confidence": 0.98,
      "justification": "First line of text"
    },
    {
      "name": "email",
      "field_type": "email",
      "value": "john@example.com",
      "confidence": 0.95,
      "justification": "Found after 'Email:' label"
    }
  ],
  "confidence": 0.965,
  "model": "opencode-v1",
  "extra": {
    "tokens_used": 100
  }
}
```

### POST /session/:id/message

Send extraction prompt as session message (`parts` payload).

**Request:**
```json
{
  "model": {
    "providerID": "zai-coding-plan",
    "modelID": "glm-5"
  },
  "providerID": "zai-coding-plan",
  "modelID": "glm-5",
  "agent": "build",
  "parts": [
    {
      "type": "text",
      "text": "<extraction prompt with input/context/schema and strict JSON contract>"
    }
  ]
}
```

**Response:** Assistant message with text content containing extraction JSON.

### GET /health

Check if the API is operational.

**Headers:**
```
X-Session-ID: <session_id>
```

**Response:** 200 OK if healthy

### OpenCode Server compatibility

If you are running `opencode serve` (default port 4096), health is exposed at
`GET /global/health` rather than `GET /health`. The provider health check tries
both routes so local server mode and hosted extraction endpoints both work.

Extraction calls use OpenCode session APIs directly and send a `parts` payload
with model routing metadata.

## Usage Examples

### Basic Usage

```rust
use clarity_web::providers::{OpenCodeProvider, ExtractionContext, ExtractionProvider};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create provider
    let provider = OpenCodeProvider::new(
        "https://api.opencode.ai/v1".to_string(),
        "my-session-123".to_string(),
    )?;

    // Prepare context
    let context = ExtractionContext {
        document_type: Some("email".to_string()),
        locale: Some("en_US".to_string()),
        schema: None,
        extra: json!({}),
    };

    // Extract fields
    let text = "From: alice@example.com\nTo: bob@example.com";
    let result = provider.extract_fields(text, &context).await?;

    // Process results
    for field in result.fields {
        println!("{}: {}", field.name, field.value);
    }

    Ok(())
}
```

### Schema-Based Extraction

```rust
use clarity_web::providers::{SchemaField, FieldType};

let schema = vec![
    SchemaField {
        name: "email".to_string(),
        field_type: FieldType::Email,
        required: true,
        description: Some("Email address".to_string()),
        options: None,
    },
    SchemaField {
        name: "phone".to_string(),
        field_type: FieldType::Phone,
        required: false,
        description: Some("Phone number".to_string()),
        options: None,
    },
];

let result = provider
    .extract_fields_with_schema(text, &schema, &context)
    .await?;
```

### Health Check

```rust
match provider.health_check().await {
    Ok(()) => println!("Provider is healthy!"),
    Err(e) => eprintln!("Health check failed: {}", e),
}
```

## Error Handling

All methods return `Result<T, ExtractionError>`. Handle errors appropriately:

```rust
match provider.extract_fields(text, &context).await {
    Ok(result) => {
        // Process extracted fields
    }
    Err(ExtractionError::InvalidInput(msg)) => {
        eprintln!("Invalid input: {}", msg);
    }
    Err(ExtractionError::NetworkError(msg)) => {
        eprintln!("Network error: {}", msg);
        // Implement retry logic
    }
    Err(ExtractionError::RateLimited { retry_after_seconds }) => {
        eprintln!("Rate limited. Retry after {}s", retry_after_seconds);
        // Implement backoff
    }
    Err(e) => {
        eprintln!("Extraction failed: {}", e);
    }
}
```

## Implementation Details

### Thread Safety

The `OpenCodeProvider` is `Send + Sync` and can be safely shared across threads:

```rust
use std::sync::Arc;

let provider = Arc::new(OpenCodeProvider::new(...)?);

// Share across tasks
let task1 = {
    let provider = Arc::clone(&provider);
    tokio::spawn(async move {
        provider.extract_fields(text1, &context).await
    })
};

let task2 = {
    let provider = Arc::clone(&provider);
    tokio::spawn(async move {
        provider.extract_fields(text2, &context).await
    })
};

let (result1, result2) = tokio::join!(task1, task2);
```

### Request/Response Types

```rust
// Internal request types (not public)
struct ExtractRequest {
    text: String,
    context: ExtractionContext,
}

struct ExtractWithSchemaRequest {
    text: String,
    schema: Vec<SchemaField>,
    context: ExtractionContext,
}

struct ExtractResponse {
    fields: Vec<ExtractResponseField>,
    confidence: f64,
    model: Option<String>,
    extra: Option<serde_json::Value>,
}

struct ExtractResponseField {
    name: String,
    field_type: FieldType,
    value: serde_json::Value,
    confidence: f64,
    justification: Option<String>,
}
```

## Testing

The provider includes comprehensive unit tests:

- Provider creation and configuration
- URL building with trailing slash handling
- HTTP error mapping for various status codes
- Response parsing and conversion to domain types
- Request/response serialization

Run tests with:
```bash
moon run clarity-web:test
```

## Validation

A verification script is provided to check implementation completeness:

```bash
./clarity-web/tests/verify_opencode.sh
```

## Dependencies

- `reqwest` 0.12 - HTTP client with JSON support
- `async-trait` - Async trait support
- `serde` - Serialization/deserialization
- `chrono` - Timestamp handling
- `tokio` - Async runtime (usually already in dependencies)

## Configuration

### Environment Variables (Recommended)

```rust
let endpoint = std::env::var("OPENCODE_ENDPOINT")
    .unwrap_or_else(|_| "https://api.opencode.ai/v1".to_string());

let session_id = std::env::var("OPENCODE_SESSION_ID")
    .unwrap_or_else(|_| uuid::Uuid::new_v4().to_string());

let provider = OpenCodeProvider::new(endpoint, session_id)?;
```

### Hardcoded (Not Recommended)

```rust
let provider = OpenCodeProvider::new(
    "https://api.opencode.ai/v1".to_string(),
    "fixed-session-id".to_string(),
)?;
```

## Performance Considerations

1. **Timeout**: Default 30-second timeout prevents hanging requests
2. **Connection Pooling**: reqwest automatically pools HTTP connections
3. **Async**: All operations are async and non-blocking
4. **Session Tracking**: Session ID enables request correlation

## Security Considerations

1. **HTTPS**: Always use HTTPS endpoints in production
2. **Session IDs**: Use cryptographically random session IDs (e.g., UUID v4)
3. **Secrets**: Never hardcode API keys or session IDs in code
4. **Error Messages**: Be careful not to leak sensitive information in error messages

## Future Enhancements

Potential improvements for future versions:

- [ ] Retry logic with exponential backoff
- [ ] Request/response compression
- [ ] Metrics collection (latency, success rate)
- [ ] Circuit breaker for API failures
- [ ] Request caching for identical inputs
- [ ] Streaming responses for large extractions
- [ ] Batch extraction API support
- [ ] WebSocket support for real-time updates

## License

MIT License - See project root for details.
