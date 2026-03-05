# Interview Session API Contract

**Version**: 1.0.0
**Feature**: 002-interview-engine-port

## Session Lifecycle

### Create Session

```rust
/// Create a new interview session
///
/// # Arguments
/// * `id` - Unique session identifier
/// * `profile` - System profile type
/// * `timestamp` - ISO 8601 creation timestamp
///
/// # Returns
/// New session in Discovery stage with empty collections
///
/// # Example
/// ```
/// let session = InterviewSession::new(
///     "sess-001".to_string(),
///     Profile::Api,
///     "2026-02-27T00:00:00Z".to_string(),
/// );
/// assert_eq!(session.stage, InterviewStage::Discovery);
/// assert!(session.answers.is_empty());
/// ```
pub fn new(id: String, profile: Profile, timestamp: String) -> Self;
```

### Add Answer

```rust
/// Add an answer to the session
///
/// # Arguments
/// * `answer` - The answer to add
///
/// # Returns
/// Updated session (builder pattern)
///
/// # Postconditions
/// - Answer appended to answers list
/// - updated_at timestamp refreshed
/// - Gap detection triggered for critical questions
///
/// # Example
/// ```
/// let session = session.add_answer(Answer {
///     question_id: "q-base-url".to_string(),
///     response: "https://api.example.com".to_string(),
///     ..Default::default()
/// });
/// assert_eq!(session.answers.len(), 1);
/// ```
pub fn add_answer(&mut self, answer: Answer) -> &mut Self;
```

### Detect Gaps

```rust
/// Detect missing information blocking spec completion
///
/// # Returns
/// Vector of detected gaps based on:
/// - Profile-specific required fields
/// - Critical questions with brief answers (< 10 chars)
/// - Missing field extractions
///
/// # Example
/// ```
/// let gaps = session.detect_gaps();
/// for gap in gaps.iter().filter(|g| g.blocking) {
///     println!("Blocking gap: {}", gap.description);
/// }
/// ```
pub fn detect_gaps(&self) -> Vec<Gap>;
```

### Detect Conflicts

```rust
/// Detect contradictions between answers
///
/// # Returns
/// Vector of detected conflicts including:
/// - CAP theorem conflicts (fast + consistent)
/// - Anonymous + audit conflicts
/// - Perspective conflicts (Developer vs Ops)
///
/// # Example
/// ```
/// let conflicts = session.detect_conflicts();
/// for conflict in &conflicts {
///     println!("Conflict: {}", conflict.description);
///     for option in &conflict.options {
///         println!("  - {}", option.description);
///     }
/// }
/// ```
pub fn detect_conflicts(&self) -> Vec<Conflict>;
```

### Can Proceed

```rust
/// Check if session can proceed to next stage
///
/// # Returns
/// Ok(()) if no blocking gaps or unresolved conflicts
/// Err(reason) if blocked
///
/// # Example
/// ```
/// match session.can_proceed() {
///     Ok(()) => session.complete_round(),
///     Err(reason) => println!("Blocked: {}", reason),
/// }
/// ```
pub fn can_proceed(&self) -> Result<(), String>;
```

### Complete Round

```rust
/// Mark current round as complete and advance
///
/// # Preconditions
/// - can_proceed() must return Ok(())
///
/// # Postconditions
/// - rounds_completed incremented
/// - Stage may advance (Discovery → Refinement → Validation → Complete)
///
/// # Returns
/// Updated session (builder pattern)
pub fn complete_round(&mut self) -> &mut Self;
```

## Data Types

### Profile

```rust
pub enum Profile {
    Api,      // REST/GraphQL API
    Cli,      // Command-line interface
    Event,    // Event-driven system
    Data,     // Data processing
    Workflow, // Workflow automation
    Ui,       // User interface
}
```

### InterviewStage

```rust
pub enum InterviewStage {
    Discovery,   // Initial information gathering
    Refinement,  // Detail refinement
    Validation,  // Validation of collected info
    Complete,    // Interview complete
    Paused,      // Paused for later resumption
}
```

### Answer

```rust
pub struct Answer {
    pub question_id: String,
    pub question_text: String,
    pub perspective: Perspective,
    pub round: u32,
    pub response: String,
    pub extracted: HashMap<String, String>,
    pub confidence: f64,  // 0.0 - 1.0
    pub notes: String,
    pub timestamp: String,  // ISO 8601
}
```

### Gap

```rust
pub struct Gap {
    pub id: String,
    pub field: String,
    pub description: String,
    pub blocking: bool,
    pub suggested_default: String,
    pub why_needed: String,
    pub round: u32,
    pub resolved: bool,
    pub resolution: String,
}
```

### Conflict

```rust
pub struct Conflict {
    pub id: String,
    pub between: (String, String),  // Answer IDs
    pub description: String,
    pub impact: String,
    pub options: Vec<ConflictResolution>,
    pub chosen: Option<i32>,  // Selected option index
}
```

### ConflictResolution

```rust
pub struct ConflictResolution {
    pub option: String,
    pub description: String,
    pub tradeoffs: String,
    pub recommendation: String,
}
```

## Error Types

```rust
pub enum InterviewError {
    SessionNotFound(String),
    InvalidStageTransition { from: InterviewStage, to: InterviewStage },
    BlockingGaps(Vec<Gap>),
    UnresolvedConflicts(Vec<Conflict>),
    InvalidAnswer { question_id: String, reason: String },
    SerializationError(String),
    IoError(String),
}
```
