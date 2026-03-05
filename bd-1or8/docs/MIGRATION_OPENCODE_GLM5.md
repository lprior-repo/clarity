# OpenCode GLM-5 Migration Guide

## Overview

This document describes the migration to using the GLM-5 model via the OpenCode provider for AI-powered field extraction in Clarity. The GLM-5 model provides improved extraction quality and reliability for structured data extraction from unstructured text.

## Migration Timeline

- **2026-02-25**: Initial GLM-5 integration completed
- **2026-02-28**: GLM-5 becomes the default model for new installations

## What Changed

### Default Model

The default AI model has been changed from unspecified to `zai-coding-plan/glm-5`:

```toml
[provider]
provider = "opencode"
endpoint = "https://api.opencode.ai/v1"
session_id = ""
model = "zai-coding-plan/glm-5"
routing_provider = "zai-coding-plan"
```

### Provider Configuration

The `OpenCodeProvider` now supports explicit model routing:

```rust
use clarity_web::providers::{OpenCodeProvider, OpenCodeProviderOptions};

let provider = OpenCodeProvider::new_with_options(
    "https://api.opencode.ai/v1".to_string(),
    "my-session".to_string(),
    OpenCodeProviderOptions {
        model: Some("zai-coding-plan/glm-5".to_string()),
        routing_provider: Some("zai-coding-plan".to_string()),
    },
)?;
```

## Migration Steps

### For New Installations

No action required. The default configuration automatically uses GLM-5.

### For Existing Installations

1. **Update your configuration file** at `~/.config/clarity/ai.toml`:

   ```toml
   [provider]
   provider = "opencode"
   endpoint = "https://api.opencode.ai/v1"
   session_id = ""
   model = "zai-coding-plan/glm-5"
   routing_provider = "zai-coding-plan"

   [quality]
   min_score = 70
   ```

2. **Restart any running Clarity processes** to pick up the new configuration.

3. **Verify the migration** by running the health check:

   ```rust
   match provider.health_check().await {
       Ok(()) => println!("Provider is healthy!"),
       Err(e) => eprintln!("Health check failed: {}", e),
   }
   ```

### For Developers

If you are using the `OpenCodeProvider` directly in code:

1. Update to use `new_with_options` for explicit model configuration:

   ```rust
   // Old (still works, uses defaults)
   let provider = OpenCodeProvider::new(endpoint, session_id)?;

   // New (recommended for explicit control)
   let provider = OpenCodeProvider::new_with_options(
       endpoint,
       session_id,
       OpenCodeProviderOptions {
           model: Some("zai-coding-plan/glm-5".to_string()),
           routing_provider: Some("zai-coding-plan".to_string()),
       },
   )?;
   ```

2. The model can be specified in two formats:
   - Combined: `model = "zai-coding-plan/glm-5"` (provider/model)
   - Separate: `model = "glm-5"` with `routing_provider = "zai-coding-plan"`

## Configuration Reference

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `OPENCODE_ENDPOINT` | API endpoint URL | `https://api.opencode.ai/v1` |
| `OPENCODE_SESSION_ID` | Session identifier | UUID v4 (auto-generated) |

### Configuration File Fields

| Field | Type | Description | Default |
|-------|------|-------------|---------|
| `provider` | string | Provider type | `opencode` |
| `endpoint` | string | API endpoint URL | `https://api.opencode.ai/v1` |
| `session_id` | string | Session identifier | Empty (auto-generated) |
| `model` | string | Model identifier | `zai-coding-plan/glm-5` |
| `routing_provider` | string | Backend routing provider | None |

### Quality Configuration

| Field | Type | Description | Default |
|-------|------|-------------|---------|
| `min_score` | u8 | Minimum quality score (0-100) | 70 |

## Troubleshooting

### Common Issues and Solutions

#### 1. Connection Refused / Network Error

**Symptoms:**
```
ExtractionError::NetworkError("Failed to connect to https://api.opencode.ai/v1")
```

**Solutions:**
- Verify network connectivity: `curl https://api.opencode.ai/v1/health`
- Check firewall rules allow HTTPS outbound
- Verify proxy settings if applicable
- Try alternative endpoint if using custom deployment

#### 2. Authentication Error (401/403)

**Symptoms:**
```
ExtractionError::AuthenticationError("Invalid token")
```

**Solutions:**
- Verify API credentials are correct
- Check if session ID is required for your deployment
- Ensure endpoint URL is correct (staging vs production)

#### 3. Rate Limited (429)

**Symptoms:**
```
ExtractionError::RateLimited { retry_after_seconds: 60 }
```

**Solutions:**
- Implement exponential backoff in your code
- Reduce request frequency
- Contact support for rate limit increase
- Consider batching requests

#### 4. Timeout Errors

**Symptoms:**
```
ExtractionError::Timeout { timeout_ms: 30000 }
```

**Solutions:**
- Check network latency
- Reduce input text size
- Consider chunking large documents
- The default timeout is 30 seconds

#### 5. Parse Errors

**Symptoms:**
```
ExtractionError::ParseError("Failed to parse extraction JSON from assistant text")
```

**Solutions:**
- This usually indicates a model output issue
- Try with different input text
- Check if the model is returning valid JSON
- Enable debug logging to see raw responses

#### 6. Model Not Found / Invalid Model

**Symptoms:**
```
ExtractionError::ApiError { message: "model not found", status_code: Some(404) }
```

**Solutions:**
- Verify model name format: `provider/model` (e.g., `zai-coding-plan/glm-5`)
- Check if routing provider is set correctly
- Ensure you have access to the specified model

#### 7. Quota Exceeded (402)

**Symptoms:**
```
ExtractionError::QuotaExceeded("usage limit exceeded")
```

**Solutions:**
- Check your API usage limits
- Wait for quota reset (usually monthly)
- Upgrade your plan if needed

### Debug Logging

Enable debug logging to troubleshoot issues:

```rust
// Set RUST_LOG environment variable
std::env::set_var("RUST_LOG", "clarity_web=debug");

// Initialize logging
env_logger::init();
```

### Health Check Script

Use this script to verify your configuration:

```bash
#!/bin/bash
# health_check.sh

ENDPOINT="${OPENCODE_ENDPOINT:-https://api.opencode.ai/v1}"
SESSION_ID="${OPENCODE_SESSION_ID:-$(uuidgen)}"

echo "Checking OpenCode health at: $ENDPOINT"
echo "Session ID: $SESSION_ID"

# Try primary health endpoint
RESPONSE=$(curl -s -w "\n%{http_code}" \
    -H "X-Session-ID: $SESSION_ID" \
    "$ENDPOINT/health")

HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
BODY=$(echo "$RESPONSE" | sed '$d')

if [ "$HTTP_CODE" = "200" ]; then
    echo "Health check passed (primary endpoint)"
    exit 0
fi

# Try fallback health endpoint
RESPONSE=$(curl -s -w "\n%{http_code}" \
    -H "X-Session-ID: $SESSION_ID" \
    "$ENDPOINT/global/health")

HTTP_CODE=$(echo "$RESPONSE" | tail -n1)

if [ "$HTTP_CODE" = "200" ]; then
    echo "Health check passed (fallback endpoint)"
    exit 0
fi

echo "Health check failed with HTTP $HTTP_CODE"
exit 1
```

### Verifying Model Configuration

```rust
use clarity_web::providers::{OpenCodeProvider, OpenCodeProviderOptions};

fn verify_configuration() -> Result<(), Box<dyn std::error::Error>> {
    let provider = OpenCodeProvider::new_with_options(
        "https://api.opencode.ai/v1".to_string(),
        "test-session".to_string(),
        OpenCodeProviderOptions {
            model: Some("zai-coding-plan/glm-5".to_string()),
            routing_provider: Some("zai-coding-plan".to_string()),
        },
    )?;

    println!("Provider name: {}", provider.provider_name());
    println!("Endpoint: {}", provider.endpoint());
    println!("Session ID: {}", provider.session_id());
    println!("Model: {:?}", provider.model());
    println!("Routing provider: {:?}", provider.routing_provider());

    Ok(())
}
```

## Rollback Procedure

If you need to revert to the previous configuration:

1. Edit `~/.config/clarity/ai.toml`:

   ```toml
   [provider]
   provider = "opencode"
   endpoint = "https://api.opencode.ai/v1"
   session_id = ""
   model = "previous-model/provider"
   routing_provider = "previous-provider"
   ```

2. Or remove the model specification to use system defaults:

   ```toml
   [provider]
   provider = "opencode"
   endpoint = "https://api.opencode.ai/v1"
   session_id = ""
   # model and routing_provider removed
   ```

3. Restart Clarity processes.

## Performance Expectations

| Metric | Expected Value |
|--------|----------------|
| Latency (p50) | < 500ms |
| Latency (p99) | < 3s |
| Timeout | 30s |
| Extraction Quality | > 85% accuracy |

## Support

If you encounter issues not covered in this guide:

1. Check the logs for detailed error messages
2. Run the health check script
3. Verify your configuration matches the expected format
4. Contact support with:
   - Configuration (redact sensitive values)
   - Error messages
   - Request/response logs (if available)

## Related Documentation

- [OpenCode Provider Documentation](../clarity-web/src/providers/OPENCODE.md)
- [Discover Phase Architecture](./architecture/discover-phase.md)
- [AI Configuration Reference](./architecture/README.md)
