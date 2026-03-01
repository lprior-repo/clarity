# Clarity Web

A Rust web application for requirements engineering and specification management.

## Overview

Clarity Web provides:

- **Intent CLI** - A planning and bead generation tool for capturing requirements
- **Lattice Patterns** - Pattern recognition for requirements analysis (EARS, Design by Contract, etc.)
- **PME (Product Management Engine)** - Double Diamond framework implementation
- **Providers** - AI extraction provider integrations (OpenCode)

## Architecture

```
clarity-web/
├── src/
│   ├── intent/        # Spec parsing, interview, plan, beads
│   ├── lattice/       # Pattern recognition (EARS, conflicts, gaps)
│   ├── pme/           # Product management (Discover, Define, Develop)
│   ├── providers/     # AI extraction providers
│   ├── storage/       # Data persistence (JSONL, redb)
│   ├── config/        # AI configuration management
│   └── ui/            # Dioxus UI components
```

## Usage

### As a Library

```rust
use clarity_web::intent::parser::parse_spec;
use clarity_web::intent::validation::validate_spec;

let json = r#"{"name": "my-spec", "features": [...]}"#;
let spec = parse_spec(json)?;
let result = validate_spec(&spec);
```

### With OpenCode Provider

```rust
use clarity_web::providers::{OpenCodeProvider, ExtractionProvider};

let provider = OpenCodeProvider::new(
    "https://api.opencode.ai/v1".to_string(),
    "session-id".to_string(),
)?;

let result = provider.extract_fields(text, &context).await?;
```

## Configuration

AI configuration is stored in `~/.config/clarity/ai.toml`:

```toml
[provider]
provider = "opencode"
endpoint = "https://api.opencode.ai/v1"
model = "zai-coding-plan/glm-5"

[quality]
min_score = 70
```

## Development

```bash
# Build
cargo build --package clarity-web

# Test
cargo test --package clarity-web

# Clippy
cargo clippy --package clarity-web
```

## License

MIT License - See project root for details.
