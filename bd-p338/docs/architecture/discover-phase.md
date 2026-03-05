# Discover Phase Architecture

## Overview

The Discover phase is the initial entry point for users to capture their problem context and requirements. It implements a dual-mode interface designed to reduce cognitive load while maintaining engineering rigor through the Mental Lattice analysis framework.

### Design Goals

1. **Reduce Input Friction**: Allow users to provide input in the way that suits their thinking style
2. **Maintain Engineering Rigor**: Ensure all captured data meets quality standards through EARS parsing, inversion analysis, and quality scoring
3. **Enable AI-Assistance**: Provide intelligent extraction and suggestions while keeping humans in control
4. **Ensure Data Persistence**: All state is immediately persisted to redb for full recovery

## Mode Flows

### Express Mode

Express mode enables freeform input with AI-powered field extraction for users who prefer to write naturally without structured prompts.

```
┌─────────────────────────────────────────────────────────────┐
│  User Input (Freeform Textarea)                             │
│  - Guided placeholder with example                         │
│  - 2000 character soft limit                                │
│  - Character counter displayed                             │
└──────────────┬──────────────────────────────────────────────┘
               │
               ▼
┌─────────────────────────────────────────────────────────────┐
│  Extraction Trigger                                         │
│  - Manual: "Extract Details" button                        │
│  - Auto: After 3s typing pause                             │
└──────────────┬──────────────────────────────────────────────┘
               │
               ▼
┌─────────────────────────────────────────────────────────────┐
│  ExtractionProvider::extract_fields()                       │
│  - Sends text to AI provider                               │
│  - Returns structured fields with confidence scores        │
└──────────────┬──────────────────────────────────────────────┘
               │
               ▼
┌─────────────────────────────────────────────────────────────┐
│  Field Review Cards                                         │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐      │
│  │ Problem  │ │Antithesis│ │ Solution │ │   User   │      │
│  │          │ │          │ │          │ │          │      │
│  │ [Conf]   │ │ [Conf]   │ │ [Conf]   │ │ [Conf]   │      │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘      │
│  ┌──────────┐                                            │
│  │ Scenario │                                            │
│  │          │                                            │
│  │ [Conf]   │                                            │
│  └──────────┘                                            │
└──────────────┬──────────────────────────────────────────────┘
               │
               ▼
┌─────────────────────────────────────────────────────────────┐
│  User Confirmation                                          │
│  - Edit individual fields                                  │
│  - Lock individual fields                                  │
│  - "Confirm All" to lock all                               │
└──────────────┬──────────────────────────────────────────────┘
               │
               ▼
┌─────────────────────────────────────────────────────────────┐
│  Quality Gate                                               │
│  - Score ≥ 70%: Enable "Continue to Define"               │
│  - Score < 70%: Show improvement suggestions               │
└─────────────────────────────────────────────────────────────┘
```

**Express Flow Fields:**

| Field ID | Title | Type | Description |
|----------|-------|------|-------------|
| `problem` | Problem Statement | TextArea | Core problem being solved |
| `user` | Target User | Text | Who experiences this problem |
| `context` | Context & Background | TextArea | Relevant background information |
| `constraints` | Constraints | TextArea | Limitations and boundaries |
| `goals` | Goals & Success Metrics | TextArea | What success looks like |

### Guided Mode

Guided mode provides sequential questions with AI suggestions for users who prefer structured prompts.

```
┌─────────────────────────────────────────────────────────────┐
│  Progress Indicator                                         │
│  "2/5 answered"                                             │
│  [✓] [✓] [○] [○] [○]                                       │
└─────────────────────────────────────────────────────────────┘
               │
               ▼
┌─────────────────────────────────────────────────────────────┐
│  Question 1                                                 │
│  ┌───────────────────────────────────────────────────────┐ │
│  │ 1. Problem Statement (required) [Answered ✓]          │ │
│  │                                                        │ │
│  │ What problem are you trying to solve?                 │ │
│  │ Hint: Focus on the core pain point                    │ │
│  │                                                        │ │
│  │ ┌──────────────────────────────────────────────────┐ │ │
│  │ │ [textarea - pre-filled with draft or empty]      │ │ │
│  │ │                                          [AI Suggest] │ │ │
│  │ └──────────────────────────────────────────────────┘ │ │
│  │                                                        │ │
│  │ 123 chars                                    [Submitted] │ │
│  │                                                        │ │
│  │ ┌──────────────────────────────────────────────────┐ │ │
│  │ │ Follow-up: Have you considered who else...       │ │ │
│  │ └──────────────────────────────────────────────────┘ │ │
│  └───────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
               │
               ▼ (repeat for all 5 questions)
┌─────────────────────────────────────────────────────────────┐
│  All Questions Answered                                     │
│  - Quality score calculated                                 │
│  - Mental lattice analysis triggered                        │
│  - "Continue to Define" enabled (if quality ≥ 70%)         │
└─────────────────────────────────────────────────────────────┘
```

**Guided Flow Features:**

1. **Sequential Questions**: Questions appear one at a time in order
2. **AI Suggest Button**: Provides contextual suggestions based on previous answers
3. **Follow-up Questions**: Appears after submission to deepen understanding
4. **Context Carry-Forward**: AI uses all previous answers when generating suggestions
5. **Express Pre-fill**: If user switches from Express mode, extracted content pre-fills the first question

## Component Hierarchy

```
DiscoverPhase
├── ModeToggle
│   ├── Express Mode Button
│   └── Guided Mode Button
├── ExpressFlow (when Express active)
│   ├── FreeformTextarea
│   ├── ExtractButton
│   ├── FieldReviewGrid
│   │   ├── FieldCard (Problem)
│   │   ├── FieldCard (Target User)
│   │   ├── FieldCard (Context)
│   │   ├── FieldCard (Constraints)
│   │   └── FieldCard (Goals)
│   ├── ConfirmAllButton
│   └── ContinueCTA
└── GuidedFlow (when Guided active)
    ├── ProgressIndicator
    ├── QuestionCard (1..5)
    │   ├── QuestionText
    │   ├── HintText
    │   ├── Textarea
    │   ├── AISuggestButton
    │   ├── SubmitButton
    │   └── FollowUpPrompt
    └── CompletionBanner
```

## Data Flow Diagram

### Express Mode Data Flow

```
User Input
    │
    ▼
Freeform Textarea (input_text signal)
    │
    ├─► [typing pause 3s] ──► AutoExtractionTrigger
    │
    └─► [button click] ────► ManualExtractionTrigger
                                │
                                ▼
                    ExtractionProvider::extract_fields()
                                │
                                ├─► Build Schema (5 fields)
                                ├─► Call AI Provider
                                └─► Return ExtractedFields
                                        │
                                        ▼
                        ┌───────────────────────────────┐
                        │ Update Field Signals         │
                        │ - Set content                │
                        │ - Set confidence             │
                        │ - Mark as unconfirmed        │
                        └───────────────────────────────┘
                                        │
                                        ▼
                        ┌───────────────────────────────┐
                        │ User Edits & Confirms        │
                        │ - Edit field content         │
                        │ - Lock individual fields     │
                        └───────────────────────────────┘
                                        │
                                        ▼
                        ┌───────────────────────────────┐
                        │ on_complete callback         │
                        │ - Validate all locked        │
                        │ - Trigger quality score      │
                        └───────────────────────────────┘
                                        │
                                        ▼
                            QualityScoring::calculate()
                                        │
                                        ├─► Parse EARS requirements
                                        ├─► Run inversion analysis
                                        ├─► Calculate dimensions
                                        └─► Return QualityScore
                                                │
                                                ▼
                                    ┌───────────────────────────┐
                                    │ Score ≥ 70%?             │
                                    ├───────────────────────────┤
                                    │ Yes: Show Continue CTA   │
                                    │ No: Show improvements     │
                                    └───────────────────────────┘
```

### Guided Mode Data Flow

```
Question Rendered
    │
    ▼
User Types Answer (draft signal)
    │
    ├─► [AI Suggest clicked] ──► ServerSuggestionProvider
    │                               │
    │                               ├─► Build context from previous answers
    │                               ├─► Call get_coach_guidance()
    │                               └─► Update draft signal
    │
    ▼
[Submit clicked]
    │
    ▼
Create Answer {
    step_id: question.id,
    value: draft.trim(),
    timestamp: Utc::now().to_rfc3339()
}
    │
    ▼
Update answers signal
    │
    ├─► Mark question as answered
    ├─► Clear draft
    ├─► Show follow-up (if exists)
    └─► Update progress indicator
            │
            ▼
All questions answered?
    │
    ├─► No: Show next question
    │
    └─► Yes: Trigger completion
            │
            ▼
    Mental Lattice Analysis
            │
            ├─► ears::parse_requirements()
            ├─► inversion::invert()
            ├─► effects::trace_effects()
            └─► quality::calculate_quality()
                    │
                    ▼
            QualityScore displayed
                    │
                    ▼
        Show "Continue to Define" (if ≥ 70%)
```

## Extraction Provider Architecture

### Trait Definition

```rust
pub trait ExtractionProvider: Send + Sync {
    /// Extract structured fields from freeform text
    async fn extract_fields(
        &self,
        input: &str,
        context: &ExtractionContext
    ) -> Result<ExtractedFields, ExtractionError>;

    /// Generate suggestion for a specific field
    async fn suggest_field(
        &self,
        field: FieldType,
        context: &ExtractionContext
    ) -> Result<String, ExtractionError>;
}
```

### Implementations

1. **OpenCodeProvider** (Initial)
   - Uses OpenCode API via OpenAI-compatible interface
   - Session-based context for consistency
   - Configurable endpoint and model

2. **ClaudeProvider** (Future)
   - Direct Anthropic API integration
   - Native function calling support
   - Extended context window

3. **MockProvider** (Testing)
   - Returns deterministic mock data
   - No external API calls
   - Used for unit and integration tests

### Extraction Context

```rust
pub struct ExtractionContext {
    pub document_type: Option<String>,      // e.g., "express_flow_input"
    pub locale: Option<String>,              // e.g., "en_US"
    pub schema: Option<Vec<SchemaField>>,    // Target fields to extract
    pub extra: serde_json::Value,            // Provider-specific metadata
}

pub struct SchemaField {
    pub name: String,                        // Field identifier
    pub field_type: FieldType,               // Text | TextArea | Select
    pub required: bool,                      // Whether extraction is required
    pub description: Option<String>,         // Human-readable description
    pub options: Option<Vec<String>>,        // For Select fields
}
```

### Confidence Levels

| Level | Threshold | Meaning |
|-------|-----------|---------|
| High | ≥ 0.8 | Direct extraction from input text |
| Medium | 0.5 - 0.79 | Logical inference, moderate certainty |
| Low | < 0.5 | Guesswork, requires user verification |

## Quality Scoring Integration

### Dimension Calculation

Quality scoring is triggered at two points:

1. **During Express Mode**: Each time a field is confirmed
2. **During Guided Mode**: When all 5 questions are answered

### Five Dimensions

| Dimension | Calculation | Weight |
|-----------|-------------|--------|
| Completeness | % of required fields filled | 20% |
| Consistency | Absence of contradictions | 20% |
| Testability | % of EARS with acceptance criteria | 20% |
| Clarity | Sentence complexity + jargon density | 20% |
| Security | Coverage of auth/encryption/validation | 20% |

### Gate Behavior

```
Score ≥ 70%
    │
    ├─► Enable "Continue to Define" CTA
    ├─► Show green indicator
    └─► Allow progression to next phase

Score < 70%
    │
    ├─► Disable "Continue to Define"
    ├─► Show red indicator
    ├─► Display improvement suggestions
    └─► Highlight missing required fields
```

## Mental Lattice Triggers

### Discover Phase

Upon completion (either mode), the following lattice functions execute:

```rust
// Parse requirements from answers
let ears_output = ears::parse_requirements(&answers_text);

// Generate challenges to assumptions
let inversion_output = inversion::invert(&problem, &solution)?;

// Trace causal dependencies
let effects_output = effects::trace_effects(&solution_text);

// Calculate quality score
let quality_score = quality::calculate_quality(&answers, &ears_reqs, &inversion)?;
```

### Define Phase

After use cases and constraints are defined:

```rust
// Generate failure scenarios
let premortem_output = premortem::generate_premortem(&solution, &constraints);

// Re-calculate quality with additional context
let updated_quality = quality::calculate_quality(&all_answers, &ears, &inversion)?;
```

## State Persistence

### Write Pattern

All state changes trigger immediate writes to redb:

```rust
// User submits answer
store.save_answer(&answer)?;

// Extraction completes
store.save_extraction_cache(&input_hash, &cache)?;

// Quality score calculated
store.save_lattice_cache("discover", &lattice_output)?;

// Mode preference changes
let metadata = ProjectMetadata {
    mode_preference: "express".to_string(),
    current_phase: "discover".to_string(),
    created_at: project.created_at,
    updated_at: Utc::now().to_rfc3339(),
};
store.save_metadata(&metadata)?;
```

### Recovery Pattern

On app load or project switch:

```rust
// Open project database
let store = RedbStore::open(project_path)?;

// Load previous state
let metadata = store.get_metadata()?;
let answers = store.load_answers()?;
let extraction_cache = store.get_extraction_cache(&hash)?;
let lattice_cache = store.get_lattice_cache("discover")?;

// Restore UI signals
active_phase.set(metadata.current_phase);
mode_preference.set(metadata.mode_preference);
answers.set(answers);
```

## Extension Points

### Adding New Extraction Providers

1. Implement `ExtractionProvider` trait
2. Add configuration to `~/.config/clarity/ai.toml`
3. Register provider in provider factory

```rust
pub struct CustomProvider;

#[async_trait]
impl ExtractionProvider for CustomProvider {
    async fn extract_fields(
        &self,
        input: &str,
        context: &ExtractionContext
    ) -> Result<ExtractedFields, ExtractionError> {
        // Custom extraction logic
    }
}
```

### Adding New Fields

To add a new field to Express mode:

1. Add field definition to `EXPRESS_FIELDS` constant
2. Update `ExtractionContext` schema
3. Add field card to render loop
4. Update quality scoring dimensions if needed

### Custom Quality Dimensions

Add new quality dimension:

1. Add variant to `QualityDimension` enum
2. Implement calculation function
3. Add to `calculate_quality()` aggregation
4. Update UI to display new dimension

## Configuration

### AI Provider Configuration

Location: `~/.config/clarity/ai.toml`

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

## Testing Strategy

### Unit Tests

- Test individual field extraction
- Test confidence calculation
- Test quality dimension scoring
- Test EARS parsing

### Integration Tests

- Test full Express flow with mock provider
- Test full Guided flow with mock provider
- Test mode switching with state preservation
- Test quality gate enforcement

### Provider Tests

- Test each provider implementation
- Test error handling
- Test timeout behavior
- Test cache invalidation

## Performance Considerations

### Extraction Optimization

- **Debouncing**: 3s delay prevents excessive API calls
- **Caching**: Input hash-based cache avoids re-extraction
- **Streaming**: Consider streaming responses for large inputs

### UI Responsiveness

- **Async Operations**: All provider calls are async
- **Optimistic UI**: Draft updates are immediate
- **Background Processing**: Quality scoring doesn't block input

### Storage Efficiency

- **Incremental Writes**: Only changed data is written
- **Compression**: Large text fields are compressed
- **Cache Eviction**: Old extraction cache entries are pruned
