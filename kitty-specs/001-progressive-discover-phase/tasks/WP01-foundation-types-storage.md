---
lane: "done"
dependencies: []
agent: claude
shell_pid: '392507'
reviewed_by: "Lewis Prior"
review_status: "approved"
---
# WP01: Foundation Types and Storage

---
work_package_id: "WP01"
title: "Foundation Types and Storage"
lane: "planned"
dependencies: []
subtasks: ["T001", "T002", "T003", "T004", "T005", "T006", "T007", "T008", "T009", "T010", "T011"]
---

## Objective

Implement the core state machine types and storage layer for the Progressive Discover feature. This work package establishes the foundation that all other components depend on.

## Context

The Progressive Discover feature is a multi-phase wizard that guides users through problem definition with adversarial validation. The state machine must track which phase the user is in, what data they've entered, and persist this state for crash recovery.

**Key Files**:
- `clarity-web/src/components/discover/state.rs` - State machine enums
- `clarity-web/src/components/discover/types.rs` - Data structures
- `clarity-web/src/storage/transcript_store.rs` - Storage trait
- `clarity-web/src/storage/redb_transcript_store.rs` - Redb implementation

## Implementation Guidance

### T001: Define ProgressiveDiscoverPhase Enum

**Purpose**: Track the current phase of the wizard.

**Location**: `clarity-web/src/components/discover/state.rs`

**Implementation**:
```rust
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProgressiveDiscoverPhase {
    #[default]
    Prompt,
    Extracting,
    ConfirmingFields,
    Preview,
    KirkCompilation,
    Locked,
}
```

**Requirements**:
- Must derive Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize
- Default should be `Prompt`
- Implement Display for UI display
- Add `fn all() -> &'static [Self]` for iteration

### T002: Define ConfirmSubPhase Enum

**Purpose**: Track which field is being confirmed in the ConfirmingFields phase.

**Location**: `clarity-web/src/components/discover/state.rs`

**Implementation**:
```rust
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConfirmSubPhase {
    #[default]
    Problem,
    Persona,
    Solution,
    Nonpersona,
    Scenario,
}
```

**Requirements**:
- Must derive Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize
- Add navigation methods: `fn next(&self) -> Option<Self>`, `fn prev(&self) -> Option<Self>`
- Add `fn index(&self) -> usize` for progress calculation

### T003: Define AntithesisResponse Struct

**Purpose**: Store the user's 3 null hypothesis points with quality score.

**Location**: `clarity-web/src/components/discover/types.rs` or `transcript_store.rs`

**Implementation**:
```rust
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AntithesisResponse {
    pub points: [String; 3],
    pub quality_score: f64,
}

impl AntithesisResponse {
    pub fn new(point1: String, point2: String, point3: String, quality_score: f64) -> Self {
        Self {
            points: [point1, point2, point3],
            quality_score: quality_score.clamp(0.0, 1.0),
        }
    }

    pub fn empty() -> Self {
        Self {
            points: [String::new(), String::new(), String::new()],
            quality_score: 0.0,
        }
    }
}
```

### T004: Define StrawManTrap and StrawManValidation

**Purpose**: Detect and track persona straw man argument traps.

**Location**: `clarity-web/src/components/discover/types.rs`

**Implementation**:
```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StrawManTrap {
    IrrationalActor,
    ManicPixieDreamUser,
    StoicMonk,
    YourClone,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct StrawManValidation {
    pub traps_detected: Vec<StrawManTrap>,
    pub passed: bool,
}

impl StrawManValidation {
    pub fn new(traps_detected: Vec<StrawManTrap>) -> Self {
        Self {
            passed: traps_detected.is_empty(),
            traps_detected,
        }
    }

    pub fn passed() -> Self {
        Self {
            traps_detected: Vec::new(),
            passed: true,
        }
    }
}
```

### T005: Define HolePunchingResults

**Purpose**: Track the 3 types of gaps in scenarios.

**Location**: `clarity-web/src/components/discover/types.rs`

**Implementation**:
```rust
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HolePunchingResults {
    pub discovery_hole: Option<String>,
    pub edge_case_hole: Option<String>,
    pub motivation_dropoff: Option<String>,
}

impl HolePunchingResults {
    pub fn is_complete(&self) -> bool {
        self.discovery_hole.is_some()
            && self.edge_case_hole.is_some()
            && self.motivation_dropoff.is_some()
    }

    pub fn addressed_count(&self) -> usize {
        [self.discovery_hole.is_some(), self.edge_case_hole.is_some(), self.motivation_dropoff.is_some()]
            .iter()
            .filter(|&&x| x)
            .count()
    }
}
```

### T006: Define ScenarioField

**Purpose**: Store the 3 bullet scenario prompts.

**Location**: `clarity-web/src/components/discover/types.rs`

**Implementation**:
```rust
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScenarioField {
    pub trigger: String,
    pub value_moment: String,
    pub feeling: String,
    pub hole_punching: HolePunchingResults,
}

impl ScenarioField {
    pub fn is_complete(&self) -> bool {
        !self.trigger.trim().is_empty()
            && !self.value_moment.trim().is_empty()
            && !self.feeling.trim().is_empty()
            && self.hole_punching.is_complete()
    }
}
```

### T007: Define InterrogationTranscript

**Purpose**: The complete record of the user's discovery session.

**Location**: `clarity-web/src/storage/transcript_store.rs`

**Implementation**:
```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InterrogationTranscript {
    pub original_prompt: String,
    pub problem: ExtractedField,
    pub antithesis: AntithesisResponse,
    pub persona: ExtractedField,
    pub straw_man_validation: StrawManValidation,
    pub solution: ExtractedField,
    pub vorp_justification: String,
    pub nonpersona: ExtractedField,
    pub scenario: ScenarioField,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExtractedField {
    pub content: String,
    pub confidence: f64,
    pub source: String,
    pub extracted_at: String,
}
```

### T008: Create TranscriptStore Trait

**Purpose**: Abstract storage backend for transcript persistence.

**Location**: `clarity-web/src/storage/transcript_store.rs`

**Implementation**:
```rust
#[async_trait]
pub trait TranscriptStore: Send + Sync {
    async fn save(&self, session_id: &str, transcript: &InterrogationTranscript) -> TranscriptResult<()>;
    async fn load(&self, session_id: &str) -> TranscriptResult<Option<InterrogationTranscript>>;
    async fn delete(&self, session_id: &str) -> TranscriptResult<()>;
    async fn list_sessions(&self) -> TranscriptResult<Vec<String>>;
}
```

**Requirements**:
- Must provide ACID guarantees
- Async trait for non-blocking IO
- Typed error handling with StorageError

### T009: Implement RedbTranscriptStore

**Purpose**: Redb-based persistent storage implementation.

**Location**: `clarity-web/src/storage/redb_transcript_store.rs`

**Implementation**:
- Use `redb::Database` for persistence
- Store transcripts as JSON in a single table
- Implement all TranscriptStore methods
- Add `open(path)` and `open_in_memory()` constructors

**Requirements**:
- Atomic transactions for save/delete
- Graceful error handling (no panics)
- Tests for all operations

### T010: Add Auto-Save Hook

**Purpose**: Automatically save transcript on state transitions.

**Implementation**:
- Add `auto_save` method to transcript management
- Call on every phase transition
- Debounce to avoid excessive writes (500ms)

### T011: Add Crash Recovery

**Purpose**: Restore transcript on page reload/crash.

**Implementation**:
- Store current session ID in localStorage
- On mount, check for existing session
- Load transcript if found
- Allow user to resume or start fresh

## Test Strategy

Run existing tests:
```bash
cargo test --lib state -- --nocapture
cargo test --lib transcript_store -- --nocapture
cargo test --lib redb_transcript_store -- --nocapture
```

Verify:
- [ ] All enums serialize/deserialize correctly
- [ ] Storage save/load round-trip works
- [ ] Crash recovery restores state
- [ ] No panics or unwraps

## Definition of Done

- [ ] All 11 subtasks complete
- [ ] Tests pass
- [ ] Code compiles without warnings
- [ ] No unwrap/expect in production code
- [ ] Types documented with doc comments

## Risks

| Risk | Mitigation |
|------|------------|
| Redb doesn't work on WASM | Use conditional compilation, localStorage fallback |
| Serialization fails | Comprehensive tests for all types |
| Race conditions in auto-save | Debounce saves, use async properly |

## Reviewer Guidance

Focus on:
1. Are all required traits derived correctly?
2. Does storage handle errors gracefully?
3. Is crash recovery robust?
4. Are there any panics lurking?

## Activity Log

- 2026-02-26T15:08:46Z – claude – shell_pid=392507 – lane=doing – Assigned agent via workflow command
- 2026-02-26T17:46:08Z – claude – shell_pid=392507 – lane=for_review – T010 Auto-save and T011 crash recovery implemented
- 2026-02-26T17:55:39Z – claude – shell_pid=392507 – lane=done – Review passed: tests fixed
