# Storage API Contract

**Version**: 1.0.0
**Feature**: 002-interview-engine-port

## JSONL Operations

### Session Serialization

```rust
/// Serialize session to JSONL line
///
/// # Arguments
/// * `session` - Session to serialize
///
/// # Returns
/// Single-line JSON string
///
/// # Format
/// ```json
/// {"id":"sess-001","profile":"api","stage":"discovery",...}
/// ```
///
/// # Errors
/// - SerializationError if session contains unserializable data
pub fn session_to_jsonl_line(session: &InterviewSession) -> Result<String, IntentError>;
```

### Session Append

```rust
/// Append session to JSONL file
///
/// # Arguments
/// * `session` - Session to append
/// * `path` - Path to .jsonl file
///
/// # Postconditions
/// - Session appended as new line
/// - File created if doesn't exist
/// - Parent directories created if needed
///
/// # Errors
/// - IoError if file cannot be created/written
/// - SerializationError if session cannot be serialized
pub fn append_session_to_jsonl(
    session: &InterviewSession,
    path: &Path,
) -> Result<(), IntentError>;
```

### Session Listing

```rust
/// List all sessions from JSONL file
///
/// # Arguments
/// * `path` - Path to .jsonl file
///
/// # Returns
/// Vector of all sessions in file order
///
/// # Errors
/// - FileNotFound if file doesn't exist
/// - ParseError if any line is invalid JSON
/// - ValidationError if any session is invalid
///
/// # Example
/// ```
/// let sessions = list_sessions_from_jsonl(path)?;
/// for session in &sessions {
///     println!("{}: {} ({})", session.id, session.profile, session.stage);
/// }
/// ```
pub fn list_sessions_from_jsonl(path: &Path) -> Result<Vec<InterviewSession>, IntentError>;
```

### Session Retrieval

```rust
/// Get specific session by ID
///
/// # Arguments
/// * `path` - Path to .jsonl file
/// * `session_id` - Session ID to find
///
/// # Returns
/// Session if found
///
/// # Errors
/// - NotFoundError if session not in file
/// - FileNotFound if file doesn't exist
pub fn get_session_from_jsonl(
    path: &Path,
    session_id: &str,
) -> Result<InterviewSession, IntentError>;
```

## Session Diffing

### Diff Sessions

```rust
/// Compute diff between two sessions
///
/// # Arguments
/// * `from` - Original session
/// * `to` - New session
///
/// # Returns
/// SessionDiff containing:
/// - Added answers
/// - Modified answers
/// - Removed answers
/// - Stage changes
/// - Gap changes
/// - Conflict changes
///
/// # Example
/// ```
/// let diff = diff_sessions(&old, &new);
/// println!("Changes:");
/// for change in &diff.answer_changes {
///     match change.change_type {
///         AnswerChangeType::Added => println!("+ {}", change.answer_id),
///         AnswerChangeType::Modified => println!("~ {}", change.answer_id),
///         AnswerChangeType::Removed => println!("- {}", change.answer_id),
///     }
/// }
/// ```
pub fn diff_sessions(from: &InterviewSession, to: &InterviewSession) -> SessionDiff;
```

### Session Snapshot

```rust
/// Create snapshot for history tracking
///
/// # Arguments
/// * `session` - Session to snapshot
/// * `description` - Human-readable description
///
/// # Returns
/// SessionSnapshot with timestamp and serialized state
pub fn create_snapshot(session: &InterviewSession, description: &str) -> SessionSnapshot;
```

### Append History

```rust
/// Append snapshot to history file
///
/// # Arguments
/// * `session` - Current session state
/// * `description` - Change description
/// * `history_path` - Path to history JSONL file
///
/// # Postconditions
/// - Snapshot appended to history
/// - File created if doesn't exist
pub fn append_to_history(
    session: &InterviewSession,
    description: &str,
    history_path: &Path,
) -> Result<(), IntentError>;
```

### List History

```rust
/// List all history entries for a session
///
/// # Arguments
/// * `history_path` - Path to history JSONL file
/// * `session_id` - Session ID to filter
///
/// # Returns
/// Vector of snapshots in chronological order
pub fn list_session_history(
    history_path: &Path,
    session_id: &str,
) -> Result<Vec<SessionSnapshot>, IntentError>;
```

## Data Types

### SessionDiff

```rust
pub struct SessionDiff {
    pub session_id: String,
    pub from_timestamp: String,
    pub to_timestamp: String,
    pub stage_changed: bool,
    pub old_stage: InterviewStage,
    pub new_stage: InterviewStage,
    pub answer_changes: Vec<AnswerDiff>,
    pub gaps_added: Vec<Gap>,
    pub gaps_resolved: Vec<Gap>,
    pub conflicts_added: Vec<Conflict>,
    pub conflicts_resolved: Vec<Conflict>,
}
```

### AnswerDiff

```rust
pub struct AnswerDiff {
    pub answer_id: String,
    pub change_type: AnswerChangeType,
    pub old_response: Option<String>,
    pub new_response: Option<String>,
    pub field_changes: HashMap<String, (Option<String>, Option<String>)>,
}
```

### AnswerChangeType

```rust
pub enum AnswerChangeType {
    Added,
    Modified,
    Removed,
}
```

### SessionSnapshot

```rust
pub struct SessionSnapshot {
    pub session_id: String,
    pub timestamp: String,
    pub description: String,
    pub stage: InterviewStage,
    pub answer_count: u32,
    pub gap_count: u32,
    pub conflict_count: u32,
    pub serialized_state: String,  // JSON
}
```

## File Paths

### Default Paths

```
.intent/
├── sessions.jsonl         # All sessions
├── history.jsonl          # Session history
├── session-{id}.cue       # Individual session (CUE format)
└── beads.jsonl            # Generated beads
```

### Path Utilities

```rust
/// Get default sessions file path
pub fn default_sessions_path() -> PathBuf;

/// Get default history file path
pub fn default_history_path() -> PathBuf;

/// Get session-specific CUE file path
pub fn session_cue_path(session_id: &str) -> PathBuf;

/// Ensure .intent directory exists
pub fn ensure_intent_dir() -> Result<PathBuf, IntentError>;
```
