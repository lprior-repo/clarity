# Discover Phase UX Redesign - Design Document

## Context

Clarity Planner's Discover phase currently shows all 5 questions at once, creating a cognitive "wall" that front-loads mental demand. The goal is to reduce input friction while preserving (and enhancing) engineering rigor through:

1. **Express/Guided mode toggle** - Let users choose their path
2. **AI-assisted field extraction** - Turn freeform input into structured fields
3. **Native Mental Lattice integration** - EARS, inversion, effects, quality scoring in Rust
4. **redb persistence** - Per-project databases for full state recovery

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                        Clarity Web App                          │
├─────────────────────────────────────────────────────────────────┤
│  Discover Phase                                                 │
│  ┌──────────────────┐    ┌──────────────────┐                  │
│  │   ExpressFlow    │    │   GuidedFlow     │                  │
│  │  - Freeform input│    │  - Sequential    │                  │
│  │  - AI extraction │    │  - AI suggest    │                  │
│  │  - Field review  │    │  - Per-question  │                  │
│  └────────┬─────────┘    └────────┬─────────┘                  │
│           │                       │                             │
│           └───────────┬───────────┘                             │
│                       ▼                                         │
│           ┌───────────────────────┐                             │
│           │   ExtractionProvider  │◄── OpenCode (initial)       │
│           │   (trait)             │◄── Claude (future)          │
│           └───────────┬───────────┘                             │
│                       ▼                                         │
│  ┌─────────────────────────────────────────────┐                │
│  │            Mental Lattice (Rust)            │                │
│  │  ┌──────┐ ┌─────────┐ ┌─────────┐ ┌───────┐ │                │
│  │  │ EARS │ │Inversion│ │ Effects │ │Quality│ │                │
│  │  └──────┘ └─────────┘ └─────────┘ └───────┘ │                │
│  │  ┌──────────┐ ┌──────────┐ ┌─────────────┐  │                │
│  │  │ Premortem│ │ Coverage │ │   Compact   │  │                │
│  │  └──────────┘ └──────────┘ └─────────────┘  │                │
│  └─────────────────────────────────────────────┘                │
│                       │                                         │
│                       ▼                                         │
│           ┌───────────────────────┐                             │
│           │   redb Storage        │                             │
│           │   (per-project db)    │                             │
│           └───────────────────────┘                             │
└─────────────────────────────────────────────────────────────────┘
```

## Mode Flows

### Express Mode

1. **Freeform Input**
   - Single large textarea with guided placeholder
   - Hint: "The more specific you are, the better AI can extract. Aim for 2-3 paragraphs."
   - Character counter (soft limit 2000)

2. **Extraction**
   - "Extract Details" button (manual trigger)
   - Auto-extraction runs after 3s typing pause (background)
   - Smart diff: if manual and auto differ, show review badge

3. **Field Review**
   - 5 cards: Problem, Antithesis, Solution, Target User, North Star
   - Each shows: content (editable), confidence badge
   - Low-confidence cards expanded by default
   - Lock individual fields or "Confirm All"

4. **Progression**
   - Quality score updates live
   - All 5 locked → "Continue to Define" CTA

### Guided Mode (Enhanced)

1. **Sequential questions** (preserved from current)
2. **AI Suggest button** on each question
   - Uses previous answers + partial input
   - Inserts draft into textarea
3. **Context carry-forward**
   - AI references earlier answers in suggestions
   - Switch from Express mid-way → pre-fills with extracted content
4. **Progress**: "2/5 answered" with checkmarks

## Mental Lattice Integration

### Phase Triggers

| Phase Complete | Lattice Functions Called |
|----------------|--------------------------|
| Discover | `ears()`, `inversion()`, `effects()` |
| Define | `quality()` (gate ≥70%), `premortem()`, `owasp()` |
| Develop | `compact()`, `coverage()` |

### Quality Gate

- Quality score visible from first Discover answer
- 5 dimensions: Completeness, Consistency, Testability, Clarity, Security
- Develop tab disabled until score ≥70%
- Score updates live as fields confirmed

### Right Panel Mapping

| Tab | Lattice Output |
|-----|----------------|
| Plan | EARS requirements + KIRK contracts (human-readable) |
| Graph | Bead dependency DAG (from effects traces) |
| State | KIRK invariants layer |

## Data Persistence (redb)

### Database Location
```
~/.local/share/clarity/projects/{project_id}/data.redb
```

### Tables

| Table | Key | Value |
|-------|-----|-------|
| `answers` | step_id | { value, timestamp, confidence, ai_generated } |
| `extraction_cache` | input_hash | ExtractedFields |
| `metadata` | - | { mode_preference, current_phase, created_at, updated_at } |
| `lattice_cache` | phase | LatticeOutput |

### Write Pattern
- Immediate writes (no batching)
- On load: restore from redb
- On project switch: close current, open new

## Component Structure

```
clarity-web/src/
├── components/
│   └── discover/
│       ├── ModeToggle.rs         # Express | Guided toggle
│       ├── ExpressFlow.rs        # Freeform → extraction → review
│       ├── GuidedFlow.rs         # Enhanced sequential
│       ├── FieldCard.rs          # Single field with confidence
│       ├── ExtractionDiff.rs     # Auto vs manual diff
│       └── QualityScore.rs       # Live score bar
├── providers/
│   ├── mod.rs
│   ├── trait.rs                  # ExtractionProvider trait
│   └── opencode.rs               # OpenCode implementation
├── lattice/                      # Native Mental Lattice
│   ├── mod.rs
│   ├── ears.rs
│   ├── inversion.rs
│   ├── effects.rs
│   ├── quality.rs
│   ├── premortem.rs
│   ├── coverage.rs
│   └── compact.rs
└── storage/
    ├── mod.rs
    └── redb_store.rs
```

## AI Provider Trait

```rust
pub trait ExtractionProvider: Send + Sync {
    async fn extract_fields(&self, input: &str) -> Result<ExtractedFields, ExtractionError>;
    async fn suggest_field(&self, field: FieldType, context: &ExtractionContext) -> Result<String, ExtractionError>;
}

pub struct ExtractedFields {
    problem: Option<FieldExtraction>,
    antithesis: Option<FieldExtraction>,
    solution: Option<FieldExtraction>,
    persona: Option<FieldExtraction>,
    scenario: Option<FieldExtraction>,
}

pub struct FieldExtraction {
    content: String,
    confidence: Confidence,  // High, Inferred, Uncertain
    source_text: Vec<String>,
}

pub enum Confidence {
    High,       // Direct extraction from input
    Inferred,   // Logical inference, needs verification
    Uncertain,  // Guesswork, needs user input
}
```

## Configuration

Location: `~/.config/clarity/ai.toml`

```toml
[provider]
type = "opencode"
endpoint = "http://localhost:3000"
session_id = "..."

[quality]
minimum_score = 70  # Gate for Develop phase
```

## Success Criteria

1. User can switch between Express and Guided modes
2. Express mode extracts all 5 fields from freeform input
3. Confidence badges accurately reflect extraction certainty
4. Quality score updates live and gates Develop phase
5. All state persists to redb and recovers on app restart
6. OpenCode integration works for extraction
