# Progressive Discover Phase - Atomic Bead Decomposition

## Overview

This document breaks down the Progressive Discover Phase into **51 atomic beads**, each designed to be completed in **1-2 hours maximum**. Smaller beads enable:
- Faster feedback cycles
- Easier parallelization
- Clearer acceptance criteria
- Reduced cognitive load
- Better progress tracking

---

## Bead Size Guidelines

| Size | Duration | Complexity | When to Use |
|------|----------|------------|-------------|
| **XS** | 15-30 min | Trivial | Type definitions, constants |
| **S** | 30-60 min | Simple | Single component, single function |
| **M** | 1-2 hours | Moderate | Multi-part component, integration |
| **L** | 2-4 hours | Complex | Multi-component feature |

**Target**: Most beads should be XS-S, with some M. Avoid L beads.

---

## Full Bead List (51 Beads)

### GROUP 1: State Machine Foundation (6 beads, ~3h)

#### 1.1 `state: define ProgressiveDiscoverPhase enum` [XS, 15min]
```rust
// File: clarity-web/src/components/discover/state.rs
pub enum ProgressiveDiscoverPhase {
    Prompt,
    Extracting,
    ConfirmingFields,
    Preview,
    KirkCompilation,
    Locked,
}
```
- Add Default impl (Prompt)
- Add Display impl
- Add unit tests for all variants

#### 1.2 `state: define ConfirmSubPhase enum` [XS, 15min]
```rust
pub enum ConfirmSubPhase {
    Problem,
    Persona,
    Solution,
    Nonpersona,
    Scenario,
}
```
- Add Default impl (Problem)
- Add Display impl
- Add next()/prev() methods
- Add unit tests

#### 1.3 `state: define AntithesisResponse struct` [XS, 15min]
```rust
pub struct AntithesisResponse {
    pub points: [String; 3],
    pub quality_score: f64,
}
```
- Add Default impl
- Add validation (quality_score 0-1)
- Add unit tests

#### 1.4 `state: define StrawMan types` [XS, 15min]
```rust
pub enum StrawManTrap {
    IrrationalActor,
    ManicPixieDreamUser,
    StoicMonk,
    YourClone,
}

pub struct StrawManValidation {
    pub traps_detected: Vec<StrawManTrap>,
    pub passed: bool,
}
```
- Add Display impl for trap names
- Add unit tests

#### 1.5 `state: define HolePunchingResults struct` [XS, 15min]
```rust
pub struct HolePunchingResults {
    pub discovery_hole: Option<String>,
    pub edge_case_hole: Option<String>,
    pub motivation_dropoff: Option<String>,
}

impl HolePunchingResults {
    pub fn is_complete(&self) -> bool { ... }
}
```
- Add unit tests

#### 1.6 `state: define InterrogationTranscript struct` [S, 30min]
```rust
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

pub struct ScenarioField {
    pub trigger: String,
    pub value_moment: String,
    pub feeling: String,
    pub hole_punching: HolePunchingResults,
}
```
- Add Default impl
- Add builder pattern
- Add Serialize/Deserialize
- Add unit tests

---

### GROUP 2: Storage Layer (4 beads, ~2h)

#### 2.1 `storage: create TranscriptStore trait` [S, 30min]
```rust
// File: clarity-web/src/storage/transcript_store.rs
#[async_trait]
pub trait TranscriptStore: Send + Sync {
    async fn save(&self, session_id: &str, transcript: &InterrogationTranscript) -> Result<()>;
    async fn load(&self, session_id: &str) -> Result<Option<InterrogationTranscript>>;
    async fn delete(&self, session_id: &str) -> Result<()>;
}
```
- Define trait
- Add mock implementation for testing
- Add unit tests

#### 2.2 `storage: implement RedbTranscriptStore` [M, 1h]
```rust
pub struct RedbTranscriptStore {
    db: Arc<Database>,
}

impl TranscriptStore for RedbTranscriptStore {
    // Implementation using redb
}
```
- Implement all trait methods
- Handle serialization
- Add error handling
- Add unit tests

#### 2.3 `storage: add auto-save hook` [S, 30min]
```rust
pub struct AutoSavingTranscript {
    inner: InterrogationTranscript,
    store: Arc<dyn TranscriptStore>,
    session_id: String,
    dirty: bool,
}
```
- Auto-save on drop
- Manual save method
- Add unit tests

#### 2.4 `storage: add recovery from crash` [S, 30min]
- Load existing transcript on init
- Resume from last saved state
- Add integration tests

---

### GROUP 3: Prompt Phase UI (4 beads, ~2h)

#### 3.1 `ui-prompt: create scaffolding prompt buttons` [S, 30min]
```rust
// File: clarity-web/src/components/discover/phases/prompt_phase.rs
fn ScaffoldingPrompts() -> Element {
    // "I want to build..."
    // "Users are struggling with..."
    // "I'm trying to solve..."
}
```
- 3 clickable prompt buttons
- Click fills textarea
- Add unit tests

#### 3.2 `ui-prompt: create main textarea` [S, 30min]
```rust
fn PromptTextarea(on_change: Callback<String>) -> Element {
    // Large textarea
    // Character counter
    // Placeholder text
}
```
- Max 2000 chars
- Live character count
- Add unit tests

#### 3.3 `ui-prompt: create ExtractFieldsButton` [S, 30min]
```rust
fn ExtractFieldsButton(on_click: Callback<()>, disabled: bool) -> Element {
    // "Extract Fields" button
    // Loading state
    // Min 50 chars requirement
}
```
- Disabled until 50 chars
- Loading spinner
- Add unit tests

#### 3.4 `ui-prompt: compose PromptPhase component` [S, 30min]
```rust
#[component]
pub fn PromptPhase(on_extract: Callback<String>) -> Element {
    // Compose scaffolding + textarea + button
    // Handle extraction trigger
}
```
- Wire all pieces together
- Add integration tests

---

### GROUP 4: Extracting Phase UI (2 beads, ~1h)

#### 4.1 `ui-extracting: create progress animation` [S, 30min]
```rust
// File: clarity-web/src/components/discover/phases/extracting_phase.rs
fn ExtractionProgress(phase: ExtractionPhase) -> Element {
    // Progress bar
    // Phase-specific message
}
```
- Animated progress bar
- Phase messages
- Add unit tests

#### 4.2 `ui-extracting: compose ExtractingPhase component` [S, 30min]
```rust
#[component]
pub fn ExtractingPhase(progress: f64, message: String) -> Element {
    // Full extracting UI
}
```
- Progress percentage
- Current action message
- Add unit tests

---

### GROUP 5: Confirm Phase - Problem (4 beads, ~2h)

#### 5.1 `ui-confirm-problem: create problem display` [S, 30min]
```rust
// File: clarity-web/src/components/discover/phases/confirm/problem_field.rs
fn ProblemDisplay(problem: Signal<String>) -> Element {
    // Editable textarea
    // "Based on what you wrote..." header
}
```
- Editable extracted text
- Add unit tests

#### 5.2 `ui-confirm-problem: create antithesis input` [M, 1h]
```rust
fn AntithesisInput(points: Signal<[String; 3]>) -> Element {
    // 3 input fields
    // "Give me 3 reasons why users will ignore this..."
    // Quality indicator
}
```
- 3 text inputs
- Quality scoring
- Add unit tests

#### 5.3 `ui-confirm-problem: create validation indicator` [S, 30min]
```rust
fn AntithesisQuality(score: f64) -> Element {
    // Visual quality indicator
    // Red/Yellow/Green
    // Tooltip with tips
}
```
- Visual feedback
- Improvement tips
- Add unit tests

#### 5.4 `ui-confirm-problem: compose ProblemConfirm component` [S, 30min]
```rust
#[component]
pub fn ProblemConfirm(
    problem: Signal<String>,
    antithesis: Signal<AntithesisResponse>,
    on_next: Callback<()>,
) -> Element {
    // Full problem confirmation UI
}
```
- Wire all pieces
- Add integration tests

---

### GROUP 6: Confirm Phase - Persona (4 beads, ~2h)

#### 6.1 `ui-confirm-persona: create persona display` [S, 30min]
```rust
// File: clarity-web/src/components/discover/phases/confirm/persona_field.rs
fn PersonaDisplay(persona: Signal<String>) -> Element {
    // Editable textarea
    // "Who is this for?" header
}
```

#### 6.2 `ui-confirm-persona: create trap checklist` [M, 1h]
```rust
fn StrawManTrapChecklist(validation: Signal<StrawManValidation>) -> Element {
    // 4 trap checkboxes
    // Irrational Actor
    // Manic Pixie Dream User
    // Stoic Monk
    // Your Clone
}
```
- 4 trap types
- Visual indicators
- Add unit tests

#### 6.3 `ui-confirm-persona: create trap explanation modal` [S, 30min]
```rust
fn TrapExplanationModal(trap: StrawManTrap) -> Element {
    // Modal with trap explanation
    // Example
    // How to fix
}
```

#### 6.4 `ui-confirm-persona: compose PersonaConfirm component` [S, 30min]
```rust
#[component]
pub fn PersonaConfirm(
    persona: Signal<String>,
    validation: Signal<StrawManValidation>,
    on_next: Callback<()>,
) -> Element { ... }
```

---

### GROUP 7: Confirm Phase - Solution (3 beads, ~1.5h)

#### 7.1 `ui-confirm-solution: create solution display` [S, 30min]
```rust
// File: clarity-web/src/components/discover/phases/confirm/solution_field.rs
fn SolutionDisplay(solution: Signal<String>) -> Element { ... }
```

#### 7.2 `ui-confirm-solution: create VORP input` [M, 1h]
```rust
fn VORPInput(justification: Signal<String>) -> Element {
    // "Why will they switch?" textarea
    // "What makes this 10x better?"
    // Specificity indicator
}
```
- VORP questions
- Quality scoring
- Add unit tests

#### 7.3 `ui-confirm-solution: compose SolutionConfirm component` [S, 30min]
```rust
#[component]
pub fn SolutionConfirm(
    solution: Signal<String>,
    vorp: Signal<String>,
    on_next: Callback<()>,
) -> Element { ... }
```

---

### GROUP 8: Confirm Phase - Nonpersona (2 beads, ~1h)

#### 8.1 `ui-confirm-nonpersona: create nonpersona display` [S, 30min]
```rust
// File: clarity-web/src/components/discover/phases/confirm/nonpersona_field.rs
fn NonpersonaDisplay(nonpersona: Signal<String>) -> Element {
    // "Who are you NOT building for?"
    // Editable textarea
}
```

#### 8.2 `ui-confirm-nonpersona: compose NonpersonaConfirm component` [S, 30min]
```rust
#[component]
pub fn NonpersonaConfirm(
    nonpersona: Signal<String>,
    on_next: Callback<()>,
) -> Element { ... }
```

---

### GROUP 9: Confirm Phase - Scenario (5 beads, ~2.5h)

#### 9.1 `ui-confirm-scenario: create trigger input` [S, 30min]
```rust
// File: clarity-web/src/components/discover/phases/confirm/scenario_field.rs
fn TriggerInput(trigger: Signal<String>) -> Element {
    // "What triggers them to look for a solution?"
}
```

#### 9.2 `ui-confirm-scenario: create value moment input` [S, 30min]
```rust
fn ValueMomentInput(value_moment: Signal<String>) -> Element {
    // "What's the key moment of value?"
}
```

#### 9.3 `ui-confirm-scenario: create feeling input` [S, 30min]
```rust
fn FeelingInput(feeling: Signal<String>) -> Element {
    // "How do they feel after?"
}
```

#### 9.4 `ui-confirm-scenario: create hole punching checklist` [M, 1h]
```rust
fn HolePunchingChecklist(results: Signal<HolePunchingResults>) -> Element {
    // Discovery Hole checkbox
    // Edge Case Hole checkbox
    // Motivation Drop-off checkbox
    // Each with input field
}
```

#### 9.5 `ui-confirm-scenario: compose ScenarioConfirm component` [S, 30min]
```rust
#[component]
pub fn ScenarioConfirm(
    scenario: Signal<ScenarioField>,
    hole_punching: Signal<HolePunchingResults>,
    on_next: Callback<()>,
) -> Element { ... }
```

---

### GROUP 10: Confirm Phase - Navigation (2 beads, ~1h)

#### 10.1 `ui-confirm-nav: create field progress indicator` [S, 30min]
```rust
// File: clarity-web/src/components/discover/phases/confirm/confirm_nav.rs
fn FieldProgressIndicator(current: ConfirmSubPhase) -> Element {
    // 1/5, 2/5, etc.
    // Progress dots
}
```

#### 10.2 `ui-confirm-nav: create back/next buttons` [S, 30min]
```rust
fn ConfirmNavigation(
    current: ConfirmSubPhase,
    on_back: Callback<()>,
    on_next: Callback<()>,
) -> Element { ... }
```

---

### GROUP 11: Confirm Phase - Main Container (2 beads, ~1h)

#### 11.1 `ui-confirm: create ConfirmPhase router` [M, 1h]
```rust
// File: clarity-web/src/components/discover/phases/confirm_phase.rs
#[component]
pub fn ConfirmPhase(
    sub_phase: Signal<ConfirmSubPhase>,
    transcript: Signal<InterrogationTranscript>,
    on_complete: Callback<()>,
) -> Element {
    // Route to correct sub-phase
}
```

#### 11.2 `ui-confirm: add state persistence` [S, 30min]
- Save progress on each field completion
- Restore on navigation
- Add integration tests

---

### GROUP 12: Preview Phase UI (4 beads, ~2h)

#### 12.1 `ui-preview: create summary display` [M, 1h]
```rust
// File: clarity-web/src/components/discover/phases/preview_phase.rs
fn TranscriptSummary(transcript: &InterrogationTranscript) -> Element {
    // Problem summary
    // Antithesis points
    // Solution summary
    // Persona summary
    // Nonpersona summary
    // Scenario summary
    // VORP justification
}
```

#### 12.2 `ui-preview: create Four Brutal Truths checklist` [S, 30min]
```rust
fn BrutalTruthsChecklist(checked: Signal<[bool; 4]>) -> Element {
    // Scale checkbox
    // Back-loaded Value checkbox
    // VORP checkbox
    // Sustaining checkbox
}
```

#### 12.3 `ui-preview: create action buttons` [S, 30min]
```rust
fn PreviewActions(on_refine: Callback<()>, on_lock: Callback<()>) -> Element {
    // "Refine" button
    // "Lock In" button
}
```

#### 12.4 `ui-preview: compose PreviewPhase component` [S, 30min]
```rust
#[component]
pub fn PreviewPhase(
    transcript: Signal<InterrogationTranscript>,
    on_refine: Callback<()>,
    on_lock: Callback<()>,
) -> Element { ... }
```

---

### GROUP 13: Kirk Compilation Phase UI (3 beads, ~1.5h)

#### 13.1 `ui-kirk: create compilation progress` [M, 1h]
```rust
// File: clarity-web/src/components/discover/phases/kirk_compilation_phase.rs
fn KirkProgress(step: KirkStep, progress: f64) -> Element {
    // Step indicator
    // Progress bar
    // Current action message
}
```

#### 13.2 `ui-kirk: create completion indicators` [S, 30min]
```rust
fn KirkCompletionIndicators(completed: Vec<KirkSection>) -> Element {
    // ✓ Invariants
    // ✓ Preconditions
    // ✓ Postconditions
    // etc.
}
```

#### 13.3 `ui-kirk: compose KirkCompilationPhase component` [S, 30min]
```rust
#[component]
pub fn KirkCompilationPhase(
    progress: Signal<f64>,
    current_step: Signal<KirkStep>,
    on_complete: Callback<KirkContract>,
) -> Element { ... }
```

---

### GROUP 14: Locked Phase UI (3 beads, ~1.5h)

#### 14.1 `ui-locked: create completion summary` [S, 30min]
```rust
// File: clarity-web/src/components/discover/phases/locked_phase.rs
fn LockedSummary(bead_count: usize) -> Element {
    // "Plan Locked" header
    // Bead count
    // CUE validation status
}
```

#### 14.2 `ui-locked: create navigation buttons` [S, 30min]
```rust
fn LockedNavigation(on_view_plan: Callback<()>, on_view_graph: Callback<()>, on_view_state: Callback<()>) -> Element {
    // "View Plan" button
    // "View Graph" button
    // "View State" button
}
```

#### 14.3 `ui-locked: compose LockedPhase component` [S, 30min]
```rust
#[component]
pub fn LockedPhase(
    beads: Vec<Bead>,
    on_navigate: Callback<RightTab>,
) -> Element { ... }
```

---

### GROUP 15: Main Container (4 beads, ~2h)

#### 15.1 `ui-main: create state machine hook` [M, 1h]
```rust
// File: clarity-web/src/components/discover/progressive_discover.rs
fn use_progressive_discover_state() -> (
    Signal<ProgressiveDiscoverPhase>,
    Signal<ConfirmSubPhase>,
    Signal<InterrogationTranscript>,
) { ... }
```

#### 15.2 `ui-main: create phase router` [S, 30min]
```rust
fn PhaseRouter(phase: ProgressiveDiscoverPhase) -> Element {
    // Match on phase, render correct component
}
```

#### 15.3 `ui-main: create navigation handler` [S, 30min]
```rust
fn use_phase_navigation(
    phase: Signal<ProgressiveDiscoverPhase>,
    sub_phase: Signal<ConfirmSubPhase>,
) -> PhaseNavigation { ... }
```

#### 15.4 `ui-main: compose ProgressiveDiscover component` [S, 30min]
```rust
#[component]
pub fn ProgressiveDiscover(
    extraction_provider: Option<Arc<dyn ExtractionProvider>>,
    answers: Signal<Vec<Answer>>,
    mut_answers: Signal<Vec<Answer>>,
) -> Element { ... }
```

---

### GROUP 16: Server Functions - Validation (4 beads, ~2h)

#### 16.1 `server: implement validate_antithesis` [S, 30min]
```rust
// File: clarity-web/src/server.rs
#[server]
pub async fn validate_antithesis(points: [String; 3]) -> Result<f64, ServerFnError> {
    // Check each point for specificity
    // Return quality score 0-1
}
```

#### 16.2 `server: implement validate_straw_man_traps` [M, 1h]
```rust
#[server]
pub async fn validate_straw_man_traps(persona: String) -> Result<StrawManValidation, ServerFnError> {
    // Check for 4 trap types
    // Return detected traps
}
```

#### 16.3 `server: implement validate_vorp` [S, 30min]
```rust
#[server]
pub async fn validate_vorp(justification: String) -> Result<ValidationResult, ServerFnError> {
    // Check for concrete vs vague claims
    // Return specificity score
}
```

#### 16.4 `server: implement validate_hole_punching` [S, 30min]
```rust
#[server]
pub async fn validate_hole_punching(scenario: ScenarioField) -> Result<HolePunchingResults, ServerFnError> {
    // Check for 3 hole types
    // Return results
}
```

---

### GROUP 17: Server Functions - KIRK (4 beads, ~2h)

#### 17.1 `server: create KirkContract types` [S, 30min]
```rust
// File: clarity-web/src/kirk.rs (new file)
pub struct KirkContract {
    pub ears: EarsRequirements,
    pub invariants: Vec<String>,
    pub preconditions: Vec<String>,
    pub postconditions: Vec<String>,
    pub atdd_tests: AtddTests,
    // ... all 16 sections
}
```

#### 17.2 `server: implement EARS extraction` [M, 1h]
```rust
fn extract_ears(transcript: &InterrogationTranscript) -> EarsRequirements {
    // Map problem -> ubiquitous
    // Map persona -> event-driven
    // Map nonpersona -> unwanted
}
```

#### 17.3 `server: implement KIRK constraints extraction` [M, 1h]
```rust
fn extract_kirk_constraints(transcript: &InterrogationTranscript) -> KirkConstraints {
    // Map antithesis -> inversions
    // Extract invariants
    // Extract pre/postconditions
}
```

#### 17.4 `server: implement compile_to_kirk` [M, 1h]
```rust
#[server]
pub async fn compile_to_kirk(transcript: InterrogationTranscript) -> Result<KirkContract, ServerFnError> {
    // Call all extraction functions
    // Compose full KirkContract
}
```

---

### GROUP 18: Cleanup (2 beads, ~1h)

#### 18.1 `cleanup: delete old components` [S, 30min]
- Delete `discover_flow.rs`
- Delete `express_flow.rs`
- Delete `guided_flow.rs`
- Delete `mode_toggle.rs`

#### 18.2 `cleanup: update mod.rs exports` [S, 30min]
- Update `discover/mod.rs`
- Export new components
- Remove old exports

---

## Summary Statistics

| Group | Beads | Total Hours | Description |
|-------|-------|-------------|-------------|
| 1. State Machine | 6 | 1.5h | Core types |
| 2. Storage | 4 | 2h | Persistence layer |
| 3. Prompt UI | 4 | 2h | Phase 1 |
| 4. Extracting UI | 2 | 1h | Phase 2 |
| 5. Problem Confirm | 4 | 2h | Phase 3.1 |
| 6. Persona Confirm | 4 | 2h | Phase 3.2 |
| 7. Solution Confirm | 3 | 1.5h | Phase 3.3 |
| 8. Nonpersona Confirm | 2 | 1h | Phase 3.4 |
| 9. Scenario Confirm | 5 | 2.5h | Phase 3.5 |
| 10. Confirm Nav | 2 | 1h | Navigation |
| 11. Confirm Main | 2 | 1h | Container |
| 12. Preview UI | 4 | 2h | Phase 4 |
| 13. Kirk UI | 3 | 1.5h | Phase 5 |
| 14. Locked UI | 3 | 1.5h | Phase 6 |
| 15. Main Container | 4 | 2h | Orchestration |
| 16. Validation Servers | 4 | 2h | Backend |
| 17. KIRK Servers | 4 | 2.5h | Backend |
| 18. Cleanup | 2 | 1h | Removal |
| **TOTAL** | **62** | **28.5h** | |

---

## Parallelization Opportunities

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        PARALLEL EXECUTION TRACKS                            │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  TRACK A: UI Components          TRACK B: Server Functions                 │
│  ─────────────────────          ─────────────────────────                  │
│  Groups 3-15 (UI)                Groups 16-17 (Backend)                   │
│  ~22h                            ~4.5h                                     │
│                                                                             │
│  TRACK C: Foundation (Sequential)                                          │
│  ─────────────────────────────────                                         │
│  Groups 1-2 (State + Storage)                                              │
│  ~3.5h                                                                     │
│                                                                             │
│  TRACK D: Testing (Continuous)                                             │
│  ─────────────────────────────                                              │
│  Unit tests with each bead                                                 │
│  Integration tests after groups                                            │
│                                                                             │
│  OPTIMAL PARALLEL:                                                          │
│  After Group 1 complete:                                                    │
│    - Track A can start Group 3                                              │
│    - Track B can start Group 16                                             │
│    - Track C continues Group 2                                              │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Bead Execution Order (Recommended)

### Sprint 1: Foundation (Day 1)
1.1 → 1.2 → 1.3 → 1.4 → 1.5 → 1.6 → 2.1 → 2.2

### Sprint 2: Storage + Prompt (Day 2)
2.3 → 2.4 → 3.1 → 3.2 → 3.3 → 3.4

### Sprint 3: Extracting + Problem (Day 3)
4.1 → 4.2 → 5.1 → 5.2 → 5.3 → 5.4

### Sprint 4: Persona + Solution (Day 4)
6.1 → 6.2 → 6.3 → 6.4 → 7.1 → 7.2 → 7.3

### Sprint 5: Nonpersona + Scenario (Day 5)
8.1 → 8.2 → 9.1 → 9.2 → 9.3 → 9.4 → 9.5

### Sprint 6: Confirm Navigation (Day 6)
10.1 → 10.2 → 11.1 → 11.2

### Sprint 7: Preview (Day 7)
12.1 → 12.2 → 12.3 → 12.4

### Sprint 8: Kirk + Locked (Day 8)
13.1 → 13.2 → 13.3 → 14.1 → 14.2 → 14.3

### Sprint 9: Main Container (Day 9)
15.1 → 15.2 → 15.3 → 15.4

### Sprint 10: Server Functions (Day 10, can start earlier in parallel)
16.1 → 16.2 → 16.3 → 16.4 → 17.1 → 17.2 → 17.3 → 17.4

### Sprint 11: Cleanup (Day 11)
18.1 → 18.2

---

*Generated: 2026-02-25*
*Total Beads: 62*
*Total Estimated Hours: 28.5h*
