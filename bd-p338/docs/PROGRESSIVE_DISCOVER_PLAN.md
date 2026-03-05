# Progressive Discover Phase - Master Implementation Plan

## Session: progressive-discover

**Created**: 2026-02-25
**Status**: PLANNED
**Total Estimated Hours**: 108
**Total Beads**: 14

---

## Overview

This plan decomposes the Progressive Discover Phase into 14 atomic beads following the 16-section bead template. Each bead is designed to be implementable in a single focused session (max 16 hours), with clear acceptance criteria, dependencies, and testing requirements.

---

## Task Dependency Graph

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                         DEPENDENCY GRAPH - CRITICAL PATH                        │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                 │
│  1. progressive-discover-state-machine (4h) [FOUNDATION]                       │
│     └─── BLOCKS: prompt-phase, extracting-phase, confirm-phase                 │
│                                                                                 │
│  2. storage-integration (6h) [PARALLEL]                                        │
│     └─── BLOCKS: main-container                                                │
│                                                                                 │
│  ┌───────────────────────────────────────────────────────────────────────┐     │
│  │  PARALLEL TRACK 1: UI PHASES                                          │     │
│  ├───────────────────────────────────────────────────────────────────────┤     │
│  │                                                                       │     │
│  │  3. prompt-phase-component (6h)                                      │     │
│  │      │                                                               │     │
│  │  4. extracting-phase-component (4h)                                  │     │
│  │      │                                                               │     │
│  │  5. confirm-phase-component (16h) [HIGH COMPLEXITY]                  │     │
│  │      │                                                               │     │
│  │  6. preview-phase-component (8h)                                     │     │
│  │      │                                                               │     │
│  │  7. kirk-compilation-phase-component (10h)                           │     │
│  │      │                                                               │     │
│  │  8. locked-phase-component (6h)                                      │     │
│  │      │                                                               │     │
│  │  9. progressive-discover-main-container (10h)                        │     │
│  │      └─── DEPENDS ON: ALL UI PHASES + storage-integration            │     │
│  │                                                                       │     │
│  └───────────────────────────────────────────────────────────────────────┘     │
│                                                                                 │
│  ┌───────────────────────────────────────────────────────────────────────┐     │
│  │  PARALLEL TRACK 2: SERVER FUNCTIONS                                  │     │
│  ├───────────────────────────────────────────────────────────────────────┤     │
│  │                                                                       │     │
│  │  10. validation-server-functions (8h)                                │     │
│  │       │                                                              │     │
│  │  11. compile-to-kirk-server-function (12h) [HIGH COMPLEXITY]          │     │
│  │       │                                                              │     │
│  └───────────────────────────────────────────────────────────────────────┘     │
│                                                                                 │
│  ┌───────────────────────────────────────────────────────────────────────┐     │
│  │  TRACK 3: TESTING & CLEANUP                                          │     │
│  ├───────────────────────────────────────────────────────────────────────┤     │
│  │                                                                       │     │
│  │  12. delete-old-express-guided-components (2h)                        │     │
│  │       └─── DEPENDS ON: main-container                                  │     │
│  │                                                                       │     │
│  │  13. e2e-tests (16h)                                                 │     │
│  │       └─── DEPENDS ON: main-container, validation, compile-to-kirk    │     │
│  │                                                                       │     │
│  └───────────────────────────────────────────────────────────────────────┘     │
│                                                                                 │
└─────────────────────────────────────────────────────────────────────────────────┘
```

---

## Task List (All 14 Beads)

### Foundation Tasks

#### 1. progressive-discover-state-machine (4h)
**ID**: `progressive-discover-state-machine`
**Phase**: discover
**Priority**: P0
**Complexity**: medium

**Description**: Define the core state machine types and data structures for the Progressive Discover flow.

**Files**:
- `clarity-web/src/components/discover/state.rs` [CREATE]

**Key Types**:
- `ProgressiveDiscoverPhase` enum (6 states)
- `ConfirmSubPhase` enum (5 sub-states)
- `InterrogationTranscript` struct
- `AntithesisResponse`, `StrawManValidation`, `VORP` types
- `ScenarioField` with `HolePunchingResults`

**Blocks**: prompt-phase, extracting-phase, confirm-phase

---

#### 2. storage-integration (6h)
**ID**: `storage-integration`
**Phase**: discover
**Priority**: P1
**Complexity**: medium

**Description**: Implement state persistence using redb for the InterrogationTranscript.

**Files**:
- `clarity-web/src/storage/transcript_store.rs` [CREATE]
- `clarity-web/src/storage/mod.rs` [MODIFY]

**Key Features**:
- Save/load InterrogationTranscript
- Auto-save on state transitions
- Recovery from crash

**Blocks**: main-container

---

### UI Component Tasks

#### 3. prompt-phase-component (6h)
**ID**: `prompt-phase-component`
**Phase**: discover
**Priority**: P0
**Complexity**: medium

**Description**: Implement the PROMPT phase with scaffolding prompts and freeform textarea.

**Files**:
- `clarity-web/src/components/discover/phases/prompt_phase.rs` [CREATE]

**Key Features**:
- 3 scaffolding prompt buttons
- Large textarea with character counter
- Extract Fields button
- Integration with AI extraction

**Depends**: state-machine

---

#### 4. extracting-phase-component (4h)
**ID**: `extracting-phase-component`
**Phase**: discover
**Priority**: P0
**Complexity**: low

**Description**: Implement the EXTRACTING phase with loading animation.

**Files**:
- `clarity-web/src/components/discover/phases/extracting_phase.rs` [CREATE]

**Key Features**:
- Progress bar
- Status messages ("Parsing problem statement...", etc.)
- Auto-transition on completion

**Depends**: state-machine, prompt-phase

---

#### 5. confirm-phase-component (16h) [HIGH COMPLEXITY]
**ID**: `confirm-phase-component`
**Phase**: discover
**Priority**: P0
**Complexity**: high

**Description**: Implement the field-by-field confirmation wizard with adversarial coaching for all 5 fields.

**Files**:
- `clarity-web/src/components/discover/phases/confirm_phase.rs` [CREATE]

**Key Features**:
- Chat-style interface
- Problem + Antithesis (3 null hypothesis points)
- Persona + Straw Man traps
- Solution + VORP test
- Nonpersona
- Scenario + Hole Punching (3 bullets)
- Back/Next navigation

**Depends**: state-machine, extracting-phase, validation-server-functions

---

#### 6. preview-phase-component (8h)
**ID**: `preview-phase-component`
**Phase**: discover
**Priority**: P0
**Complexity**: medium

**Description**: Implement the PREVIEW phase with summary and Four Brutal Truths check.

**Files**:
- `clarity-web/src/components/discover/phases/preview_phase.rs` [CREATE]

**Key Features**:
- Summary of all fields
- Four Brutal Truths checklist
- Refine button (back to PROMPT)
- Lock In button (to KIRK_COMPILATION)

**Depends**: confirm-phase

---

#### 7. kirk-compilation-phase-component (10h)
**ID**: `kirk-compilation-phase-component`
**Phase**: discover
**Priority**: P0
**Complexity**: medium

**Description**: Implement the KIRK_COMPILATION phase with progress animation.

**Files**:
- `clarity-web/src/components/discover/phases/kirk_compilation_phase.rs` [CREATE]

**Key Features**:
- 16-section progress tracking
- Shows what's being extracted
- Auto-transition on completion

**Depends**: preview-phase, compile-to-kirk-server-function

---

#### 8. locked-phase-component (6h)
**ID**: `locked-phase-component`
**Phase**: discover
**Priority**: P0
**Complexity**: low

**Description**: Implement the LOCKED phase with collapsed summary.

**Files**:
- `clarity-web/src/components/discover/phases/locked_phase.rs` [CREATE]

**Key Features**:
- Collapsed view
- Bead count display
- View Plan/Graph/State buttons

**Depends**: kirk-compilation-phase

---

#### 9. progressive-discover-main-container (10h)
**ID**: `progressive-discover-main-container`
**Phase**: discover
**Priority**: P0
**Complexity**: medium

**Description**: Create the main ProgressiveDiscover component orchestrating all phases.

**Files**:
- `clarity-web/src/components/discover/progressive_discover.rs` [CREATE]
- `clarity-web/src/components/discover/mod.rs` [MODIFY]

**Key Features**:
- State machine orchestration
- Phase rendering
- Navigation logic
- Integration with extraction provider

**Depends**: all UI phases + storage-integration

---

### Server Function Tasks

#### 10. validation-server-functions (8h)
**ID**: `validation-server-functions`
**Phase**: discover
**Priority**: P0
**Complexity**: medium

**Description**: Implement 4 adversarial validation server functions.

**Files**:
- `clarity-web/src/server.rs` [MODIFY]

**Functions**:
- `validate_antithesis` - Check null hypothesis quality
- `validate_straw_man_traps` - Check for persona traps
- `validate_vorp` - Check VORP specificity
- `validate_hole_punching` - Check scenario completeness

---

#### 11. compile-to-kirk-server-function (12h) [HIGH COMPLEXITY]
**ID**: `compile-to-kirk-server-function`
**Phase**: discover
**Priority**: P0
**Complexity**: high

**Description**: Implement KIRK contract generation from interrogation transcript.

**Files**:
- `clarity-web/src/server.rs` [MODIFY]

**Function**: `compile_to_kirk(transcript) -> KirkContract`

**Output**: Complete 16-section KIRK JSON

---

### Cleanup & Testing Tasks

#### 12. delete-old-express-guided-components (2h)
**ID**: `delete-old-express-guided-components`
**Phase**: discover
**Priority**: P1
**Complexity**: low

**Description**: Remove deprecated dual-mode components.

**Files**:
- `clarity-web/src/components/discover/discover_flow.rs` [DELETE]
- `clarity-web/src/components/discover/express_flow.rs` [DELETE]
- `clarity-web/src/components/discover/guided_flow.rs` [DELETE]
- `clarity-web/src/components/discover/mode_toggle.rs` [DELETE]

**Depends**: main-container

---

#### 13. e2e-tests (16h)
**ID**: `e2e-tests`
**Phase**: discover
**Priority**: P1
**Complexity**: medium

**Description**: Comprehensive Playwright E2E tests.

**Files**:
- `playwright-tests/progressive-discover/` [CREATE]

**Scenarios**:
- Successful plan creation
- Antithesis quality rejection
- Straw Man trap detection
- VORP failure
- Hole punching failure
- Refine cycle

**Depends**: main-container, validation, compile-to-kirk

---

## Summary Statistics

| Metric | Value |
|--------|-------|
| **Total Tasks** | 14 |
| **Total Hours** | 108 |
| **Critical Path Length** | 10 tasks |
| **High Complexity** | 2 tasks (confirm-phase, compile-to-kirk) |
| **Medium Complexity** | 6 tasks |
| **Low Complexity** | 6 tasks |
| **Foundation Tasks** | 2 |
| **UI Component Tasks** | 7 |
| **Server Function Tasks** | 2 |
| **Testing/Cleanup** | 2 |
| **Max Parallelization** | 3 tracks (UI, Server, Storage) |

---

## Implementation Order (Recommended)

### Week 1: Foundation
1. Day 1-2: `progressive-discover-state-machine`
2. Day 3-4: `storage-integration` (parallel)
3. Day 5: `prompt-phase-component`

### Week 2: Core UI Flow
4. Day 1-2: `extracting-phase-component`
5. Day 3-6: `confirm-phase-component` (HIGH COMPLEXITY)
6. Day 7: `validation-server-functions` (parallel with confirm-phase)

### Week 3: Completion Flow
7. Day 1-2: `preview-phase-component`
8. Day 3-5: `compile-to-kirk-server-function` (HIGH COMPLEXITY, can start earlier)
9. Day 5-6: `kirk-compilation-phase-component`
10. Day 7: `locked-phase-component`

### Week 4: Integration & Testing
11. Day 1-3: `progressive-discover-main-container`
12. Day 4: `delete-old-express-guided-components`
13. Day 5-10: `e2e-tests` (can start earlier in parallel)

---

## File Structure After Implementation

```
clarity-web/src/components/discover/
├── progressive_discover.rs     # Main container
├── state.rs                    # State machine types
├── phases/
│   ├── prompt_phase.rs
│   ├── extracting_phase.rs
│   ├── confirm_phase.rs
│   ├── preview_phase.rs
│   ├── kirk_compilation_phase.rs
│   └── locked_phase.rs
└── mod.rs

clarity-web/src/storage/
├── transcript_store.rs         # NEW
└── mod.rs                      # MODIFIED

clarity-web/src/server.rs       # MODIFIED (add 5 new functions)

# DELETED:
# - discover_flow.rs
# - express_flow.rs
# - guided_flow.rs
# - mode_toggle.rs
```

---

## Next Steps

1. **Review this plan** - Confirm task breakdown and dependencies
2. **Start with foundation** - Begin with `progressive-discover-state-machine`
3. **Generate beads** - Use `br create` for each task as needed
4. **Track progress** - Update task status as beads are completed

---

*Generated by Planner Skill v1.0*
*Session: progressive-discover*
*Date: 2026-02-25*
