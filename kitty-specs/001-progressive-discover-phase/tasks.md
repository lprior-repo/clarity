# Progressive Discover Phase - Work Packages

**Feature**: 001-progressive-discover-phase
**Generated**: 2026-02-26
**Total Work Packages**: 11
**Estimated Total Effort**: 28.5 hours

---

## Overview

This document breaks down the Progressive Discover Phase feature into actionable work packages. Each work package contains 3-7 subtasks that can be implemented independently.

### Status Legend
- [ ] Not started
- [~] In progress
- [x] Complete

### Dependency Graph

```
WP01: Foundation (state types, storage)
  ├── WP02: Prompt Phase UI
  ├── WP03: Extracting Phase UI
  └── WP04: Validation Server Functions

WP02 + WP03 + WP04:
  └── WP05: Confirm Phase - Problem & Persona

WP05:
  └── WP06: Confirm Phase - Solution, Nonpersona, Scenario

WP02-WP06:
  └── WP07: Confirm Navigation & Container

WP07:
  └── WP08: Preview Phase UI

WP08:
  └── WP09: KIRK Compilation & Locked Phase

WP01-WP09:
  └── WP10: Main Container & Integration

WP10:
  └── WP11: Cleanup & Polish
```

---

## Phase 1: Foundation

### WP01: Foundation Types and Storage

**Priority**: P0 (Critical Path)
**Dependencies**: None
**Estimated Size**: ~400 lines
**Prompt File**: `tasks/WP01-foundation-types-storage.md`

**Summary**: Implement the core state machine types (ProgressiveDiscoverPhase, ConfirmSubPhase), data structures (AntithesisResponse, StrawManValidation, HolePunchingResults, ScenarioField, InterrogationTranscript), and the storage layer (TranscriptStore trait, RedbTranscriptStore).

**Included Subtasks**:
- [ ] T001: Define ProgressiveDiscoverPhase enum with all 6 phases
- [ ] T002: Define ConfirmSubPhase enum with 5 confirmation steps
- [ ] T003: Define AntithesisResponse struct with 3 points and quality score
- [ ] T004: Define StrawManTrap enum and StrawManValidation struct
- [ ] T005: Define HolePunchingResults with 3 hole types
- [ ] T006: Define ScenarioField with trigger, value_moment, feeling
- [ ] T007: Define InterrogationTranscript with all fields and timestamps
- [ ] T008: Create TranscriptStore trait with save/load/delete/list methods
- [ ] T009: Implement RedbTranscriptStore with ACID guarantees
- [ ] T010: Add auto-save hook on state transitions
- [ ] T011: Add crash recovery with transcript restoration

**Validation**:
- All types derive Clone, Debug, PartialEq, Serialize, Deserialize
- Storage tests pass: save, load, delete, list, crash recovery
- No panics or unwraps in storage layer

---

## Phase 2: Initial UI Phases

### WP02: Prompt Phase UI

**Priority**: P1
**Dependencies**: WP01
**Estimated Size**: ~350 lines
**Prompt File**: `tasks/WP02-prompt-phase-ui.md`

**Summary**: Build the Prompt phase component with scaffolding buttons, main textarea, character count, and extract button. This is the entry point for the Progressive Discover wizard.

**Included Subtasks**:
- [ ] T012: Create scaffolding prompt buttons (3 buttons with example prompts)
- [ ] T013: Create main textarea with 2000 char limit
- [ ] T014: Add live character count display
- [ ] T015: Create ExtractFieldsButton with disabled state <50 chars
- [ ] T016: Compose PromptPhase component with all elements
- [ ] T017: Wire up extraction trigger to server function

**Validation**:
- Buttons display correctly with Tailwind styling
- Character count updates in real-time
- Extract button disabled until 50 chars
- Extraction triggers on click

### WP03: Extracting Phase UI

**Priority**: P1
**Dependencies**: WP01
**Estimated Size**: ~250 lines
**Prompt File**: `tasks/WP03-extracting-phase-ui.md`

**Summary**: Build the Extracting phase component with animated progress bar, status messages, and auto-transition to Confirm phase.

**Included Subtasks**:
- [ ] T018: Create progress animation component (pulsing/spinner)
- [ ] T019: Create status message display (extracting problem, persona, etc.)
- [ ] T020: Compose ExtractingPhase component
- [ ] T021: Implement auto-transition on extraction completion

**Validation**:
- Animation displays smoothly
- Status messages update during extraction
- Auto-transitions to Confirm phase

---

## Phase 3: Server Functions

### WP04: Validation Server Functions

**Priority**: P1
**Dependencies**: WP01
**Estimated Size**: ~400 lines
**Prompt File**: `tasks/WP04-validation-server-functions.md`

**Summary**: Implement server-side validation functions for antithesis quality, straw man trap detection, VORP validation, and hole punching completeness.

**Included Subtasks**:
- [ ] T022: Implement validate_antithesis server function
- [ ] T023: Implement validate_straw_man_traps server function
- [ ] T024: Implement validate_vorp server function
- [ ] T025: Implement validate_hole_punching server function
- [ ] T026: Create ValidationResponse types for each function
- [ ] T027: Add error handling with typed errors

**Validation**:
- All functions accessible via Dioxus server functions
- Antithesis validation returns quality score 0-1
- Straw man detection identifies all 4 trap types
- VORP validation checks specificity
- Hole punching tracks all 3 gap types

---

## Phase 4: Confirm Phase UI

### WP05: Confirm Phase - Problem & Persona

**Priority**: P1
**Dependencies**: WP02, WP03, WP04
**Estimated Size**: ~500 lines
**Prompt File**: `tasks/WP05-confirm-problem-persona.md`

**Summary**: Build the first two confirmation sub-phases: Problem (with antithesis input and quality indicator) and Persona (with straw man trap checklist).

**Included Subtasks**:
- [ ] T028: Create ProblemDisplay component (editable textarea)
- [ ] T029: Create AntithesisInput component (3 input fields)
- [ ] T030: Create AntithesisQuality indicator (score display)
- [ ] T031: Compose ProblemConfirm component
- [ ] T032: Create PersonaDisplay component
- [ ] T033: Create StrawManTrap checklist with explanations
- [ ] T034: Create trap explanation modal/tooltips
- [ ] T035: Compose PersonaConfirm component

**Validation**:
- Problem field editable with extracted content
- 3 antithesis inputs with quality score
- Persona displays with trap checklist
- All 4 straw man traps detected and explained

### WP06: Confirm Phase - Solution, Nonpersona, Scenario

**Priority**: P1
**Dependencies**: WP05
**Estimated Size**: ~500 lines
**Prompt File**: `tasks/WP06-confirm-solution-scenario.md`

**Summary**: Build the remaining three confirmation sub-phases: Solution (with VORP input), Nonpersona (simple display), and Scenario (with hole punching checklist).

**Included Subtasks**:
- [ ] T036: Create SolutionDisplay component
- [ ] T037: Create VORP input (Value, Obvious, Real, Possible)
- [ ] T038: Compose SolutionConfirm component
- [ ] T039: Create NonpersonaDisplay component
- [ ] T040: Compose NonpersonaConfirm component
- [ ] T041: Create ScenarioField inputs (trigger, value_moment, feeling)
- [ ] T042: Create HolePunching checklist (3 gap types)
- [ ] T043: Compose ScenarioConfirm component

**Validation**:
- Solution field with VORP justification
- Nonpersona editable
- Scenario has 3 bullet inputs
- Hole punching tracks all 3 gaps

### WP07: Confirm Navigation & Container

**Priority**: P1
**Dependencies**: WP05, WP06
**Estimated Size**: ~350 lines
**Prompt File**: `tasks/WP07-confirm-navigation.md`

**Summary**: Build the navigation system for the confirm phase (progress indicator, back/next buttons) and the main ConfirmPhase container that routes between sub-phases.

**Included Subtasks**:
- [ ] T044: Create ConfirmProgressBar component (1/5, 2/5, etc.)
- [ ] T045: Create ConfirmNavButtons (Back, Next)
- [ ] T046: Create ConfirmPhase router (switches between sub-phases)
- [ ] T047: Add state persistence on navigation
- [ ] T048: Implement validation blocking (can't proceed if validation fails)

**Validation**:
- Progress indicator shows current step
- Navigation works between all 5 sub-phases
- State persists on each transition
- Can't proceed if validation fails

---

## Phase 5: Final UI Phases

### WP08: Preview Phase UI

**Priority**: P1
**Dependencies**: WP07
**Estimated Size**: ~400 lines
**Prompt File**: `tasks/WP08-preview-phase.md`

**Summary**: Build the Preview phase component with summary display, Four Brutal Truths checklist, and action buttons (Refine, Lock In).

**Included Subtasks**:
- [ ] T049: Create PreviewSummary component (all confirmed fields)
- [ ] T050: Create FourBrutalTruths checklist (Scale, Back-loaded Value, VORP, Sustaining)
- [ ] T051: Create RefineButton (returns to Prompt phase)
- [ ] T052: Create LockInButton (proceeds to compilation)
- [ ] T053: Compose PreviewPhase component
- [ ] T054: Add confirmation dialog for Lock In action

**Validation**:
- Summary shows all 5 confirmed fields
- Four Brutal Truths display with checkboxes
- Refine returns to Prompt phase
- Lock In proceeds to KIRK compilation

### WP09: KIRK Compilation & Locked Phase

**Priority**: P1
**Dependencies**: WP08
**Estimated Size**: ~450 lines
**Prompt File**: `tasks/WP09-kirk-locked-phase.md`

**Summary**: Build the KIRK Compilation phase (with progress indicators for 16 sections) and the Locked phase (completion summary, navigation buttons).

**Included Subtasks**:
- [ ] T055: Create KirkCompilationProgress component (16 sections)
- [ ] T056: Create section completion indicators
- [ ] T057: Compose KirkCompilationPhase component
- [ ] T058: Create LockedSummary component (artifact stats)
- [ ] T059: Create LockedNavButtons (Plan, Graph, State views)
- [ ] T060: Compose LockedPhase component
- [ ] T061: Implement compile_to_kirk server function

**Validation**:
- Compilation shows progress for all 16 sections
- Locked phase shows bead count
- Navigation buttons work

---

## Phase 6: Integration

### WP10: Main Container & Integration

**Priority**: P0
**Dependencies**: WP01-WP09
**Estimated Size**: ~500 lines
**Prompt File**: `tasks/WP10-main-container.md`

**Summary**: Build the main ProgressiveDiscover container component that orchestrates all phases, manages global state, and handles phase transitions.

**Included Subtasks**:
- [ ] T062: Create use_progressive_discover hook (state machine)
- [ ] T063: Create use_progressive_discover_actions hook (actions)
- [ ] T064: Create phase router (switches between phases)
- [ ] T065: Create navigation handler (phase transitions)
- [ ] T066: Compose ProgressiveDiscover component
- [ ] T067: Wire up storage persistence
- [ ] T068: Add crash recovery on mount
- [ ] T069: Integrate with app routing

**Validation**:
- All phases render correctly
- State transitions work
- Persistence works across reloads
- Crash recovery restores state

### WP11: Cleanup & Polish

**Priority**: P2
**Dependencies**: WP10
**Estimated Size**: ~250 lines
**Prompt File**: `tasks/WP11-cleanup-polish.md`

**Summary**: Remove old Express/Guided flow components, update module exports, and polish the UI.

**Included Subtasks**:
- [ ] T070: Delete discover_flow.rs
- [ ] T071: Delete express_flow.rs
- [ ] T072: Delete guided_flow.rs
- [ ] T073: Delete mode_toggle.rs
- [ ] T074: Update mod.rs exports
- [ ] T075: Remove unused imports
- [ ] T076: Run clippy and fix warnings
- [ ] T077: Update e2e tests for Progressive Discover

**Validation**:
- No references to old components
- Clippy passes with no warnings
- Tests pass

---

## Summary Statistics

| Metric | Value |
|--------|-------|
| Total Work Packages | 11 |
| Total Subtasks | 77 |
| Estimated Effort | 28.5 hours |
| Average WP Size | ~400 lines |
| Max WP Size | ~500 lines (WP05, WP06, WP10) |

### Size Distribution
- Small (250-350 lines): WP03, WP07, WP11
- Medium (350-450 lines): WP01, WP02, WP04, WP08, WP09
- Large (450-500 lines): WP05, WP06, WP10

### Parallelization Opportunities
- WP02, WP03, WP04 can run in parallel after WP01
- WP05 and WP06 are sequential (confirm phases)
- Server functions (WP04) can be developed in parallel with UI

### MVP Scope
**Minimum Viable Product**: WP01-WP07
This delivers the core wizard flow from Prompt through Confirm phases.

---

## Next Steps

1. Run `spec-kitty implement WP01` to start with foundation
2. After WP01 completes, run WP02, WP03, WP04 in parallel
3. Continue sequentially through remaining WPs
4. Run `spec-kitty analyze` to verify implementation quality
