# Vision: Progressive Discover Phase

## Executive Summary

The Progressive Discover Phase replaces the current dual-mode (Express/Guided) architecture with a single **Progressive Express** flow that combines clean UX with adversarial AI coaching. Users experience a seamless journey from freeform prompt through AI extraction, field-by-field confirmation with rigorous interrogation, and finally KIRK contract compilation that feeds directly into the Bead Factory.

---

## Problem Statement

### Current Issues

1. **Wall of Questions**: Guided Mode shows 5 fields at once, creating cognitive overload before users start
2. **Superficial Coaching**: AI suggestions are too helpful - users aren't forced to defend their ideas
3. **Missing Rigor**: No null hypothesis validation, no VORP (Value Over Replacement Product) analysis
4. **Disconnected Flow**: Express and Guided modes don't share state or learning
5. **Weak Handoff**: No direct path from discovery to executable KIRK contracts

### User Quotes (Hypothetical Research)

- *"I feel overwhelmed seeing all these questions at once"*
- *"The AI just agrees with everything - what's the point?"*
- *"I get through the questions but my plan still falls apart"*
- *"What do I do after I answer these questions?"*

---

## Solution: Progressive Discover

### Core Philosophy

**Adversarial AI Coaching Following the Double Diamond Process**

The AI doesn't help users build their product—it forces users to **prove their product won't fail** before allowing them to build it. This is "Double Diamond" meets "Devil's Advocate."

### The User Journey

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        PROGRESSIVE DISCOVER FLOW                            │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  PHASE 1: PROMPT                       ┌─────────────────────────────────┐  │
│  ┌─────────────────────────────────┐  │ Describe your idea...           │  │
│  │ [I want to build...]            │  │ [Users are struggling with...]  │  │
│  │ [Users are struggling with...]  │  │ [...]                          │  │
│  │ [...]                           │  │                                 │  │
│  │                                 │  │ ┌─────────────────────────────┐ │  │
│  │ ┌─────────────────────────────┐ │  │ │                             │ │  │
│  │ │ (freeform textarea)         │ │  │ │ (user types description)    │ │  │
│  │ └─────────────────────────────┘ │  │ └─────────────────────────────┘ │  │
│  │                                 │  │                                 │  │
│  │          [Extract Fields]       │  │            [Extract Fields]      │  │
│  └─────────────────────────────────┘  └─────────────────────────────────┘  │
│                                                                             │
│                                     ▼                                       │
│                                                                             │
│  PHASE 2: EXTRACTING                ┌─────────────────────────────────┐  │
│  ┌─────────────────────────────────┐  │  Extracting insights...         │  │
│  │  ████████████░░░░░░░░░░  67%    │  │  ████████████░░░░░░░░░░  67%   │  │
│  │                                 │  │                                 │  │
│  │  "Parsing problem statement..." │  │  "Parsing problem statement..." │  │
│  └─────────────────────────────────┘  └─────────────────────────────────┘  │
│                                                                             │
│                                     ▼                                       │
│                                                                             │
│  PHASE 3: CONFIRMING_FIELDS (5 sub-phases)                                 │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ Field 1/5: Problem                                                 │   │
│  │ ┌───────────────────────────────────────────────────────────────┐  │   │
│  │ │ "Based on what you wrote, here's the problem I see:"         │  │   │
│  │ │ ┌─────────────────────────────────────────────────────────┐   │  │   │
│  │ │ │ [AI-extracted problem text - editable]                  │   │  │   │
│  │ │ └─────────────────────────────────────────────────────────┘   │  │   │
│  │ │                                                               │  │   │
│  │ │ "Now the hard part - the null hypothesis:"                   │  │   │
│  │ │ "Give me 3 realistic reasons why users will ignore this:"    │  │   │
│  │ │ ┌─────────────────────────────────────────────────────────┐   │  │   │
│  │ │ │ 1. [antithesis point 1]                                 │   │  │   │
│  │ │ │ 2. [antithesis point 2]                                 │   │  │   │
│  │ │ │ 3. [antithesis point 3]                                 │   │  │   │
│  │ │ └─────────────────────────────────────────────────────────┘   │  │   │
│  │ └───────────────────────────────────────────────────────────────┘  │   │
│  │                                    [← Back] [Next →]              │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  │ Field 2/5: Target User (with Straw Man traps)                          │
│  │ Field 3/5: Solution (with VORP test)                                   │
│  │ Field 4/5: Nonpersonas                                                  │
│  │ Field 5/5: North Star Scenario (3 bullets + hole punching)             │
│                                                                             │
│                                     ▼                                       │
│                                                                             │
│  PHASE 4: PREVIEW                   ┌─────────────────────────────────┐  │
│  ┌─────────────────────────────────┐  │  Preview Your Plan              │  │
│  │ Problem:     [summary]          │  │  ┌─────────────────────────────┐ │  │
│  │ Antithesis:  [3 points]         │  │  │ Problem: Remote teams...    │ │  │
│  │ Solution:    [summary]          │  │  │ Antithesis: Email, Slack... │ │  │
│  │ User:        [summary]          │  │  │ Solution: Unified inbox...  │ │  │
│  │ Nonpersona:  [summary]          │  │  │ VORP: 2x faster than...    │ │  │
│  │ Scenario:    [summary]          │  │  └─────────────────────────────┘ │  │
│  │ VORP:        [justification]    │  │                                 │  │
│  │                                 │  │  Four Brutal Truths Check:      │  │
│  │ ☐ Scale - Survives 10k users?   │  │  ☐ Scale                       │  │
│  │ ☐ Back-loaded Value             │  │  ☐ Back-loaded Value           │  │
│  │ ☐ VORP                          │  │  ☐ VORP                        │  │
│  │ ☐ Sustaining                    │  │  ☐ Sustaining                  │  │
│  │                                 │  │                                 │  │
│  │ [← Refine] [Lock In →]          │  │  [← Refine] [Lock In →]        │  │
│  └─────────────────────────────────┘  └─────────────────────────────────┘  │
│                                                                             │
│                                     ▼                                       │
│                                                                             │
│  PHASE 5: KIRK COMPILATION           Compiling to KIRK Contracts           │
│  ┌─────────────────────────────────┐  ┌─────────────────────────────────┐  │
│  │ Extracting from your answers:  │  │  Extracting from your answers:  │  │
│  │                                 │  │                                 │  │
│  │ ✓ Invariants                    │  │  ✓ Invariants                   │  │
│  │ ✓ Preconditions                │  │  ✓ Preconditions                │  │
│  │ ✓ Postconditions               │  │  ✓ Postconditions               │  │
│  │ ✓ EARS Syntax                   │  │  ✓ EARS Syntax                  │  │
│  │ ✓ Negative Test Cases          │  │  ✓ Negative Test Cases          │  │
│  │                                 │  │                                 │  │
│  │ ████████████████████░░░░░░░░░░ │  │  ████████████████████░░░░░░░░░░ │  │
│  │            67%                  │  │             67%                 │  │
│  └─────────────────────────────────┘  └─────────────────────────────────┘  │
│                                                                             │
│                                     ▼                                       │
│                                                                             │
│  PHASE 6: LOCKED                    ┌─────────────────────────────────┐  │
│  ┌─────────────────────────────────┐  │  ✓ Plan Locked                  │  │
│  │ Generated:                      │  │                                 │  │
│  │ • 12 Beads atomized from KIRK   │  │  Generated:                     │  │
│  │ • CUE schema validated          │  │  • 12 Beads atomized           │  │
│  │ • Ready for deterministic exec  │  │  • CUE schema validated        │  │
│  │                                 │  │  • Ready for execution         │  │
│  │ [View Plan] [View Graph] [View  │  │                                 │  │
│  │  State]                         │  │  [View Plan] [View Graph]...   │  │
│  └─────────────────────────────────┘  └─────────────────────────────────┘  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Key Innovations

### 1. Adversarial AI Coaching

Traditional AI coaching is supportive. Progressive Discover coaching is **interrogative**:

| Traditional | Adversarial (Progressive) |
|-------------|---------------------------|
| "Your idea is great!" | "Why won't this fail?" |
| "Here's how to build it" | "What makes you think anyone will use this?" |
| "Let me suggest a solution" | "Give me 3 reasons this doesn't exist already" |
| Accepts vague claims | Demands concrete evidence |

### 2. Field-by-Field Confirmation with Depth

Each field has a specific adversarial pattern:

| Field | Adversarial Pattern |
|-------|-------------------|
| **Problem** | **Antithesis**: 3 null hypothesis points |
| **Persona** | **Straw Man Traps**: Irrational Actor, Manic Pixie, Stoic Monk, Your Clone |
| **Solution** | **VORP Test**: Why switch? What's 10x better? |
| **Nonpersona** | Explicit exclusion definition |
| **Scenario** | **Hole Punching**: Discovery, Edge Case, Motivation drop-offs |

### 3. KIRK Contract Generation

The adversarial interrogation feeds directly into KIRK (Keep Invariants Regular and Known) contracts:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    INTERROGATION → KIRK CONTRACT MAPPING                     │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  DISCOVERY FIELD         →    KIRK CONTRACT ELEMENT                          │
│  ─────────────────────   →    ─────────────────────────                     │
│  Problem                 →    EARS Ubiquitous Requirements                   │
│  Antithesis (3 points)   →    Inversions (Security, Usability failures)     │
│  Persona                 →    EARS Event-Driven ("WHEN user... THEN...")    │
│  Straw Man Traps         →    Unwanted behaviors (EARS Unwanted)            │
│  Solution                →    Implementation Tasks (decomposed)             │
│  Nonpersona              →    EARS Unwanted ("IF nonpersona... SHALL NOT")  │
│  Scenario (3 bullets)    →    ATDD Tests (happy paths, error paths)         │
│  Hole Punching           →    Failure Modes (symptoms, causes, debugging)   │
│  VORP                    →    Context (related patterns, similar features)  │
│  Four Brutal Truths      →    Verification Checkpoints                      │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 4. Direct Bead Generation

KIRK contracts → Bead Template → Atomic Beads

```
KIRK JSON (with all constraints)
    ↓
[fill 16-section bead template]
    ↓
Atomic Beads (max 4hr each)
    ↓
br create (bead database)
```

---

## Technical Architecture

### State Machine

```
PROMPT → EXTRACTING → CONFIRMING_FIELDS → PREVIEW → KIRK_COMPILATION → LOCKED

CONFIRMING_FIELDS sub-states:
  CONFIRM_PROBLEM → CONFIRM_PERSONA → CONFIRM_SOLUTION
    → CONFIRM_NONPERSONA → CONFIRM_SCENARIO
```

### Component Structure

```
clarity-web/src/components/discover/
├── progressive_discover.rs     # Main container (state machine)
├── phases/
│   ├── prompt_phase.rs         # Phase 1: Scaffolding + textarea
│   ├── extracting_phase.rs     # Phase 2: Loading animation
│   ├── confirm_phase.rs        # Phase 3: Chat-style wizard
│   ├── preview_phase.rs        # Phase 4: Summary + brutal truths
│   ├── kirk_compilation_phase.rs  # Phase 5: KIRK generation
│   └── locked_phase.rs         # Phase 6: Collapsed/handoff
├── state.rs                    # State types & InterrogationTranscript
└── mod.rs                      # Module exports
```

### Server Functions

```
clarity-web/src/server.rs
├── extract_fields_progressive   # Extract all 5 fields at once
├── validate_antithesis           # Check null hypothesis quality
├── validate_straw_man_traps      # Check for persona traps
├── validate_vorp                 # Check VORP specificity
├── validate_hole_punching        # Check scenario completeness
├── compile_to_kirk              # Generate KIRK JSON from transcript
└── generate_beads_from_kirk     # Atomize into beads, validate CUE
```

---

## Data Structures

### InterrogationTranscript

```rust
pub struct InterrogationTranscript {
    // Original user input
    pub original_prompt: String,

    // AI-extracted fields
    pub problem: ExtractedField,
    pub persona: ExtractedField,
    pub solution: ExtractedField,
    pub nonpersona: ExtractedField,
    pub scenario: ScenarioField,

    // Adversarial responses
    pub antithesis: AntithesisResponse,
    pub straw_man_validation: StrawManValidation,
    pub vorp_justification: String,
    pub hole_punching_results: HolePunchingResults,

    // Timestamps
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

pub struct AntithesisResponse {
    pub points: [String; 3],  // Exactly 3 null hypothesis points
    pub quality_score: f64,   // 0-1, higher = more specific
}

pub struct StrawManValidation {
    pub traps_detected: Vec<StrawManTrap>,
    pub passed: bool,
}

pub enum StrawManTrap {
    IrrationalActor,
    ManicPixieDreamUser,
    StoicMonk,
    YourClone,
}

pub struct ScenarioField {
    pub trigger: String,
    pub value_moment: String,
    pub feeling: String,
    pub hole_punching: HolePunchingResults,
}

pub struct HolePunchingResults {
    pub discovery_hole: Option<String>,
    pub edge_case_hole: Option<String>,
    pub motivation_dropoff: Option<String>,
}
```

### KIRK Contract Output

```rust
pub struct KirkContract {
    pub clarifications: ClarificationsSection,
    pub ears: EarsRequirements,
    pub kirk_contracts: KirkConstraints,
    pub research_requirements: ResearchRequirements,
    pub inversions: Vec<Inversion>,
    pub atdd_tests: AtddTests,
    pub e2e_tests: E2ETests,
    pub verification_checkpoints: Vec<Checkpoint>,
    pub implementation_tasks: Vec<ImplementationTask>,
    pub failure_modes: Vec<FailureMode>,
    pub anti_hallucination: AntiHallucinationChecks,
    pub context_survival: ContextSurvival,
    pub completion_checklist: CompletionChecklist,
    pub context: ContextSection,
    pub ai_hints: Vec<AIHint>,
}
```

---

## Adversarial Coaching Patterns

### Pattern 1: Antithesis (Problem Field)

**Goal**: Force users to articulate why their product might fail

```
AI: "Based on what you wrote, the problem is: [summary]. Now prove to me this
    problem actually exists. Give me 3 realistic reasons why your target
    customer will ignore or reject this solution."

User: [must provide 3 specific, non-generic reasons]

AI Quality Check:
- Generic: "People might not like it" → LOW QUALITY
- Specific: "Email already handles this, and switching costs are high" → HIGH
```

### Pattern 2: Straw Man Traps (Persona Field)

**Goal**: Catch unrealistic user archetypes

```
AI: "Who is this for? You described: [persona]. Let me check for traps..."

□ Irrational Actor - Acting against own stated motivations?
  Example: "Environmentalist who loves single-use plastics"

□ Manic Pixie Dream User - Magically loves everything?
  Example: "Busy mom who wants to spend hours learning new software"

□ Stoic Monk - Tolerates immense friction without complaint?
  Example: "Developer who enjoys debugging CLI tools for fun"

□ Your Clone - Has your system knowledge?
  Example: "Non-technical user who understands Git workflows"
```

### Pattern 3: VORP Test (Solution Field)

**Goal**: Quantify value over existing workarounds

```
AI: "You want to build: [solution]. Prove it's better than what people do now."
    ""
    "VORP = Value Over Replacement Product"
    "What is the 'Replacement Product' (current workaround)?"
    "What makes your solution 10x better?"
    "Why will they switch? (Switching costs > perceived gain)"

User: [must quantify improvement]
```

### Pattern 4: Hole Punching (Scenario Field)

**Goal**: Find holes in the happy path

```
AI: "You painted a nice picture. Now let me punch holes in it:"
    ""
    "□ Discovery Hole - How did they discover the feature?"
    "□ Edge Case Hole - What if internet drops? They mistype?"
    "□ Motivation Drop-off - Why continue at high-friction steps?"

User: [must address each hole or revise scenario]
```

---

## Success Metrics

| Metric | Current | Target | Measurement |
|--------|---------|--------|-------------|
| Time to Locked Plan | N/A | < 10 min | avg(prompt → locked) |
| Field Edit Rate | N/A | < 30% | edits during confirm / total confirms |
| Completion Rate | N/A | > 70% | locked / started |
| Antithesis Quality | N/A | > 0.7 | specificity score (0-1) |
| VORP Specificity | N/A | > 0.8 | concrete vs vague ratio |
| Generated Beads | 0 | 8-15 | per locked plan |

---

## Non-Goals (Out of Scope)

- Changes to Plan/Graph/State views (right panel)
- CUE schema definition (assumed to exist)
- User authentication/persistence
- Multi-language support
- Real-time collaboration
- Version history for plans

---

## Dependencies

### External Dependencies (Existing)

- `dioxus` ^0.7 - Frontend framework
- `dioxus-fullstack` - Server functions
- `chrono` - Timestamps
- `serde` - Serialization
- `itertools` - Iteration utilities

### Internal Dependencies

- `crate::providers::ExtractionProvider` - AI field extraction
- `crate::lattice::quality` - Quality scoring
- `crate::types::Answer` - User answer type
- `crate::server::*` - Existing server functions

### New Dependencies

None planned - using existing infrastructure.

---

## Implementation Phases

### Phase 0: Foundation (Research & Setup)
- Research existing state machine patterns in codebase
- Confirm CUE schema location and format
- Verify `br create` integration point

### Phase 1: State Machine & Types
- Define `ProgressiveDiscoverPhase` enum
- Define `InterrogationTranscript` struct
- Define all adversarial response types

### Phase 2: Prompt Phase
- Implement scaffolding prompts
- Implement textarea with extraction trigger
- Wire AI extraction

### Phase 3: Confirm Phase (The Core)
- Implement Problem + Antithesis
- Implement Persona + Straw Man validation
- Implement Solution + VORP
- Implement Nonpersona
- Implement Scenario + Hole Punching

### Phase 4: Preview & Brutal Truths
- Implement summary view
- Implement Four Brutal Truths check
- Wire Refine/Next navigation

### Phase 5: KIRK Compilation
- Implement `compile_to_kirk` server function
- Map interrogation → KIRK sections
- Implement progress animation

### Phase 6: Locked & Handoff
- Implement collapsed view
- Implement bead generation
- Wire to Bead Factory

### Phase 7: Cleanup
- Delete Express/Guided components
- Update module exports
- Integration tests

---

## Testing Strategy

### Unit Tests
- State machine transitions
- Adversarial validation logic
- KIRK contract generation

### Integration Tests
- Full user flow (PROMPT → LOCKED)
- Server function end-to-end
- Bead generation validation

### E2E Tests
- Playwright scenarios for:
  - Successful plan creation
  - Antithesis quality rejection
  - Straw Man trap detection
  - VORP failure
  - Hole punching failure
  - Refine cycle

---

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| AI extraction quality | High | Fallback to manual entry, quality scoring |
| User abandonment on adversarial flow | Medium | Make coaching conversational, not punitive |
| KIRK schema mismatch | High | Validate against actual CUE schema early |
| Performance (large transcripts) | Medium | Streaming responses, incremental validation |

---

## Glossary

- **Antithesis**: Null hypothesis - 3 reasons the product will fail
- **VORP**: Value Over Replacement Product - why this beats current workaround
- **Straw Man Trap**: Unrealistic user archetype that sounds plausible but isn't
- **Hole Punching**: Finding gaps in the happy path scenario
- **KIRK**: Keep Invariants Regular and Known - contract specification format
- **Four Brutal Truths**: Scale, Back-loaded Value, VORP, Sustaining
- **Double Diamond**: Discover, Define, Develop, Deliver design process

---

## References

- PRD: /home/lewis/src/clarity/docs/PRD-ProgressiveDiscover.md
- Existing code: clarity-web/src/components/discover/
- Server functions: clarity-web/src/server.rs
- Providers: clarity-web/src/providers/

---

*Vision Document v1.0 | Progressive Discover Phase | Last Updated: 2026-02-25*
