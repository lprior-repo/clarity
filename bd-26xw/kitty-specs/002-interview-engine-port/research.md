# Research: Gleam → Rust Porting Guide

**Feature**: 002-interview-engine-port
**Date**: 2026-02-27

## Source Code Analysis

### Repository Structure

```
intent-cli/
├── src/
│   ├── intent.gleam           # Main CLI entry point (447 lines)
│   └── intent/                # Core modules (44 files, 15,242 lines)
├── schema/                    # CUE schemas (8 files, 2,138 lines)
├── test/                      # Test files (39 files)
└── gleam.toml                 # Project configuration
```

### Key Findings

#### 1. Type System Mapping

**Gleam Enums → Rust Enums**

```gleam
// Gleam
pub type Profile {
  Api
  Cli
  Event
  Data
  Workflow
  Ui
}
```

```rust
// Rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Profile {
    Api,
    Cli,
    Event,
    Data,
    Workflow,
    Ui,
}
```

**Gleam Records → Rust Structs**

```gleam
// Gleam
pub type Answer {
  Answer(
    question_id: String,
    question_text: String,
    perspective: Perspective,
    round: Int,
    response: String,
    extracted: Dict(String, String),
    confidence: Float,
    notes: String,
    timestamp: String,
  )
}
```

```rust
// Rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Answer {
    pub question_id: String,
    pub question_text: String,
    pub perspective: Perspective,
    pub round: u32,
    pub response: String,
    pub extracted: HashMap<String, String>,
    pub confidence: f64,
    pub notes: String,
    pub timestamp: String,
}
```

#### 2. Error Handling Patterns

**Gleam Pattern:**

```gleam
pub fn validate_file_path(path: String) -> Result(String, SecurityError) {
  check_shell_metacharacters(path)
  |> and_then(check_literal_traversal)
  |> and_then(check_url_encoded)
  // ...
}
```

**Rust Pattern:**

```rust
pub fn validate_file_path(path: &str) -> Result<String, SecurityError> {
    check_shell_metacharacters(path)?;
    check_literal_traversal(path)?;
    check_url_encoded(path)?;
    // ...
    Ok(path.to_string())
}
```

#### 3. List Operations

| Gleam | Rust Equivalent |
|-------|-----------------|
| `list.map(items, fn)` | `items.iter().map(fn).collect()` |
| `list.filter(items, fn)` | `items.iter().filter(fn).collect()` |
| `list.fold(items, from:, with:)` | `items.iter().fold(init, fn)` |
| `list.find(items, fn)` | `items.iter().find(fn)` |
| `list.any(items, fn)` | `items.iter().any(fn)` |
| `list.all(items, fn)` | `items.iter().all(fn)` |
| `list.flat_map(items, fn)` | `items.iter().flat_map(fn).collect()` |
| `list.try_map(items, fn)` | `items.iter().map(fn).collect::<Result<Vec<_>, _>>()` |

#### 4. Dictionary Operations

| Gleam | Rust Equivalent |
|-------|-----------------|
| `dict.new()` | `HashMap::new()` |
| `dict.insert(d, k, v)` | `d.insert(k, v)` |
| `dict.get(d, k)` | `d.get(k)` |
| `dict.from_list(list)` | `list.into_iter().collect()` |
| `dict.to_list(d)` | `d.into_iter().collect()` |
| `dict.map_values(d, fn)` | `d.into_iter().map(fn).collect()` |

#### 5. String Operations

| Gleam | Rust Equivalent |
|-------|-----------------|
| `string.split(s, sep)` | `s.split(sep).collect()` |
| `string.contains(s, sub)` | `s.contains(sub)` |
| `string.replace(s, from, to)` | `s.replace(from, to)` |
| `string.trim(s)` | `s.trim()` |
| `string.lowercase(s)` | `s.to_lowercase()` |
| `string.length(s)` | `s.len()` (bytes) or `s.chars().count()` |

#### 6. JSON Handling

**Gleam (gleam_json):**

```gleam
case json.decode(json_str, dynamic.dynamic) {
  Ok(data) -> parse_spec(data)
  Error(e) -> Error(JsonParseError(format_json_error(e)))
}
```

**Rust (serde_json):**

```rust
match serde_json::from_str::<Spec>(json_str) {
    Ok(spec) => Ok(spec),
    Err(e) => Err(IntentError::JsonParse(e.to_string())),
}
```

## Module Dependency Graph

```
intent.gleam (CLI entry)
    ├── interview → interview_storage, plan_mode
    ├── plan_mode → interview, bead_templates
    ├── bead_templates → interview
    ├── quality_analyzer → types
    ├── spec_validator → parser, security
    ├── loader → parser, security
    ├── parser → types
    ├── security (standalone)
    ├── formats (standalone)
    └── errors (standalone)
```

## Critical Implementation Details

### 1. Gap Detection Algorithm

Profile-specific required fields checked during interview:

```rust
fn get_required_fields(profile: Profile) -> Vec<&'static str> {
    match profile {
        Profile::Api => vec!["base_url", "auth_method", "happy_path", "error_cases", "response_format"],
        Profile::Cli => vec!["command_name", "happy_path", "help_text", "exit_codes"],
        Profile::Event => vec!["event_type", "payload_schema", "trigger"],
        Profile::Data => vec!["data_model", "access_patterns", "retention"],
        Profile::Workflow => vec!["steps", "happy_path", "error_recovery"],
        Profile::Ui => vec!["user_flows", "happy_path", "states"],
    }
}
```

### 2. Conflict Detection Patterns

```rust
fn detect_cap_conflict(answers: &[Answer]) -> Option<Conflict> {
    let has_fast = answers.iter().any(|a|
        a.response.to_lowercase().contains("fast") ||
        a.response.to_lowercase().contains("latency")
    );
    let has_consistent = answers.iter().any(|a|
        a.response.to_lowercase().contains("consistent") ||
        a.response.to_lowercase().contains("accurate")
    );

    if has_fast && has_consistent {
        Some(Conflict::cap_theorem())
    } else {
        None
    }
}
```

### 3. JSONL Session Format

Each line is a complete JSON session:

```json
{"id":"session-123","profile":"api","created_at":"2026-02-27T00:00:00Z","stage":"discovery","answers":[...],"gaps":[...],"conflicts":[]}
```

### 4. Bead 16-Section Template

The enhanced CUE bead format includes:
1. id
2. title
3. description
4. profile_type
5. priority
6. issue_type
7. labels
8. ai_hints
9. acceptance_criteria
10. dependencies
11. preconditions
12. postconditions
13. verification_steps
14. rollback_plan
15. estimated_effort
16. risk_level

## Testing Strategy

### Unit Tests to Port

From `test/intent_test.gleam` (111k lines - largest test file):
- Spec parsing tests
- Interview session tests
- Gap detection tests
- Conflict detection tests
- Bead generation tests

### Property-Based Testing

Use `proptest` for:
- Round-trip serialization (Spec → JSON → Spec)
- Gap detection invariants
- Conflict detection invariants

### Integration Tests

- Load existing `.intent/sessions.jsonl` files
- Validate against CUE schemas
- Performance benchmarks

## Performance Considerations

### JSONL Performance

Target: < 10ms for 100 answers

```rust
// Use buffered I/O
use std::io::{BufRead, BufWriter};

// Stream large files
fn stream_sessions(path: &Path) -> impl Iterator<Item = Result<InterviewSession, Error>> {
    BufReader::new(File::open(path)?)
        .lines()
        .map(|line| serde_json::from_str(&line?))
}
```

### Gap Detection Performance

Target: < 5ms for 50 answers

```rust
// Use index-based lookup for O(1) field checks
fn detect_gaps(session: &InterviewSession) -> Vec<Gap> {
    let answered_fields: HashSet<&str> = session.answers
        .iter()
        .map(|a| a.question_id.as_str())
        .collect();

    REQUIRED_FIELDS[session.profile]
        .iter()
        .filter(|field| !answered_fields.contains(*field))
        .map(|field| Gap::missing(field))
        .collect()
}
```

## Alternatives Considered

### Alternative 1: FFI Bridge to Gleam

**Rejected because:**
- Requires Erlang runtime (defeats single-binary goal)
- Complex FFI boundary
- Performance overhead

### Alternative 2: Incremental Port with Adapter

**Rejected because:**
- Maintains dual codebases
- Adapter layer adds complexity
- Eventually need full port anyway

### Selected: Full Rust Port

**Rationale:**
- Single binary distribution
- Rust type safety
- No runtime dependencies
- Better performance characteristics
- Long-term maintainability
