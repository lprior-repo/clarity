# Implementation Plan: Progressive Discover Phase

**Branch**: `main` | **Date**: 2026-02-26 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/kitty-specs/001-progressive-discover-phase/spec.md`

## Summary

A multi-phase wizard that guides users through problem definition with adversarial validation (antithesis, straw man traps, VORP test, hole punching), then compiles validated inputs into a 16-section KIRK contract. The feature consists of 62 atomic beads organized into 18 groups across UI components, server functions, and storage layers.

## Technical Context

**Language/Version**: Rust 1.75+
**Primary Dependencies**: Dioxus 0.7, Tailwind CSS, redb, serde
**Storage**: redb (local persistence for InterrogationTranscript)
**Testing**: cargo test, Playwright (E2E)
**Target Platform**: Web (WASM) + Desktop
**Project Type**: Web application (frontend + server functions)
**Performance Goals**: Field extraction <10s, state transitions <100ms, auto-save <500ms
**Constraints**: No data loss on crash, graceful network degradation
**Scale/Scope**: 6 phases, 5 confirmation sub-phases, 62 implementation beads

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

No constitution file found - skipping constitution check.

## Project Structure

### Documentation (this feature)

```
kitty-specs/001-progressive-discover-phase/
├── plan.md              # This file
├── spec.md              # Feature specification
├── meta.json            # Feature metadata
├── checklists/
│   └── requirements.md  # Spec quality checklist
├── research/            # Phase 0 output (if needed)
├── contracts/           # Phase 1 output (if needed)
└── tasks/               # Phase 2 output (from /spec-kitty.tasks)
```

### Source Code (repository root)

```
clarity-web/src/
├── components/
│   └── discover/
│       ├── progressive_discover.rs     # Main container
│       ├── state.rs                    # State machine types
│       ├── phases/
│       │   ├── prompt_phase.rs
│       │   ├── extracting_phase.rs
│       │   ├── confirm_phase.rs
│       │   ├── preview_phase.rs
│       │   ├── kirk_compilation_phase.rs
│       │   └── locked_phase.rs
│       └── mod.rs
├── storage/
│   ├── transcript_store.rs             # NEW
│   └── mod.rs                          # MODIFIED
├── server.rs                           # MODIFIED (add 5 new functions)
└── kirk.rs                             # NEW (KIRK contract types)

# DELETED after implementation:
# - discover_flow.rs
# - express_flow.rs
# - guided_flow.rs
# - mode_toggle.rs
```

**Structure Decision**: Single web application with Dioxus frontend components and server functions. State management via Dioxus Signals. Persistence via redb.

## Bead Breakdown (62 Beads, 18 Groups)

### GROUP 1: State Machine Foundation (6 beads, ~1.5h)

| ID | Bead | Size | Hours | File |
|----|------|------|-------|------|
| 1.1 | define ProgressiveDiscoverPhase enum | XS | 0.25h | state.rs |
| 1.2 | define ConfirmSubPhase enum | XS | 0.25h | state.rs |
| 1.3 | define AntithesisResponse struct | XS | 0.25h | state.rs |
| 1.4 | define StrawMan types | XS | 0.25h | state.rs |
| 1.5 | define HolePunchingResults struct | XS | 0.25h | state.rs |
| 1.6 | define InterrogationTranscript struct | S | 0.5h | state.rs |

### GROUP 2: Storage Layer (4 beads, ~2h)

| ID | Bead | Size | Hours | File |
|----|------|------|-------|------|
| 2.1 | create TranscriptStore trait | S | 0.5h | transcript_store.rs |
| 2.2 | implement RedbTranscriptStore | M | 1h | transcript_store.rs |
| 2.3 | add auto-save hook | S | 0.5h | transcript_store.rs |
| 2.4 | add recovery from crash | S | 0.5h | transcript_store.rs |

### GROUP 3: Prompt Phase UI (4 beads, ~2h)

| ID | Bead | Size | Hours | File |
|----|------|------|-------|------|
| 3.1 | create scaffolding prompt buttons | S | 0.5h | prompt_phase.rs |
| 3.2 | create main textarea | S | 0.5h | prompt_phase.rs |
| 3.3 | create ExtractFieldsButton | S | 0.5h | prompt_phase.rs |
| 3.4 | compose PromptPhase component | S | 0.5h | prompt_phase.rs |

### GROUP 4: Extracting Phase UI (2 beads, ~1h)

| ID | Bead | Size | Hours | File |
|----|------|------|-------|------|
| 4.1 | create progress animation | S | 0.5h | extracting_phase.rs |
| 4.2 | compose ExtractingPhase component | S | 0.5h | extracting_phase.rs |

### GROUP 5: Confirm Phase - Problem (4 beads, ~2h)

| ID | Bead | Size | Hours | File |
|----|------|------|-------|------|
| 5.1 | create problem display | S | 0.5h | confirm/problem_field.rs |
| 5.2 | create antithesis input | M | 1h | confirm/problem_field.rs |
| 5.3 | create validation indicator | S | 0.5h | confirm/problem_field.rs |
| 5.4 | compose ProblemConfirm component | S | 0.5h | confirm/problem_field.rs |

### GROUP 6: Confirm Phase - Persona (4 beads, ~2h)

| ID | Bead | Size | Hours | File |
|----|------|------|-------|------|
| 6.1 | create persona display | S | 0.5h | confirm/persona_field.rs |
| 6.2 | create trap checklist | M | 1h | confirm/persona_field.rs |
| 6.3 | create trap explanation modal | S | 0.5h | confirm/persona_field.rs |
| 6.4 | compose PersonaConfirm component | S | 0.5h | confirm/persona_field.rs |

### GROUP 7: Confirm Phase - Solution (3 beads, ~1.5h)

| ID | Bead | Size | Hours | File |
|----|------|------|-------|------|
| 7.1 | create solution display | S | 0.5h | confirm/solution_field.rs |
| 7.2 | create VORP input | M | 1h | confirm/solution_field.rs |
| 7.3 | compose SolutionConfirm component | S | 0.5h | confirm/solution_field.rs |

### GROUP 8: Confirm Phase - Nonpersona (2 beads, ~1h)

| ID | Bead | Size | Hours | File |
|----|------|------|-------|------|
| 8.1 | create nonpersona display | S | 0.5h | confirm/nonpersona_field.rs |
| 8.2 | compose NonpersonaConfirm component | S | 0.5h | confirm/nonpersona_field.rs |

### GROUP 9: Confirm Phase - Scenario (5 beads, ~2.5h)

| ID | Bead | Size | Hours | File |
|----|------|------|-------|------|
| 9.1 | create trigger input | S | 0.5h | confirm/scenario_field.rs |
| 9.2 | create value moment input | S | 0.5h | confirm/scenario_field.rs |
| 9.3 | create feeling input | S | 0.5h | confirm/scenario_field.rs |
| 9.4 | create hole punching checklist | M | 1h | confirm/scenario_field.rs |
| 9.5 | compose ScenarioConfirm component | S | 0.5h | confirm/scenario_field.rs |

### GROUP 10: Confirm Navigation (2 beads, ~1h)

| ID | Bead | Size | Hours | File |
|----|------|------|-------|------|
| 10.1 | create field progress indicator | S | 0.5h | confirm/confirm_nav.rs |
| 10.2 | create back/next buttons | S | 0.5h | confirm/confirm_nav.rs |

### GROUP 11: Confirm Main Container (2 beads, ~1h)

| ID | Bead | Size | Hours | File |
|----|------|------|-------|------|
| 11.1 | create ConfirmPhase router | M | 1h | confirm_phase.rs |
| 11.2 | add state persistence | S | 0.5h | confirm_phase.rs |

### GROUP 12: Preview Phase UI (4 beads, ~2h)

| ID | Bead | Size | Hours | File |
|----|------|------|-------|------|
| 12.1 | create summary display | M | 1h | preview_phase.rs |
| 12.2 | create Four Brutal Truths checklist | S | 0.5h | preview_phase.rs |
| 12.3 | create action buttons | S | 0.5h | preview_phase.rs |
| 12.4 | compose PreviewPhase component | S | 0.5h | preview_phase.rs |

### GROUP 13: Kirk Compilation UI (3 beads, ~1.5h)

| ID | Bead | Size | Hours | File |
|----|------|------|-------|------|
| 13.1 | create compilation progress | M | 1h | kirk_compilation_phase.rs |
| 13.2 | create completion indicators | S | 0.5h | kirk_compilation_phase.rs |
| 13.3 | compose KirkCompilationPhase component | S | 0.5h | kirk_compilation_phase.rs |

### GROUP 14: Locked Phase UI (3 beads, ~1.5h)

| ID | Bead | Size | Hours | File |
|----|------|------|-------|------|
| 14.1 | create completion summary | S | 0.5h | locked_phase.rs |
| 14.2 | create navigation buttons | S | 0.5h | locked_phase.rs |
| 14.3 | compose LockedPhase component | S | 0.5h | locked_phase.rs |

### GROUP 15: Main Container (4 beads, ~2h)

| ID | Bead | Size | Hours | File |
|----|------|------|-------|------|
| 15.1 | create state machine hook | M | 1h | progressive_discover.rs |
| 15.2 | create phase router | S | 0.5h | progressive_discover.rs |
| 15.3 | create navigation handler | S | 0.5h | progressive_discover.rs |
| 15.4 | compose ProgressiveDiscover component | S | 0.5h | progressive_discover.rs |

### GROUP 16: Validation Servers (4 beads, ~2h)

| ID | Bead | Size | Hours | File |
|----|------|------|-------|------|
| 16.1 | implement validate_antithesis | S | 0.5h | server.rs |
| 16.2 | implement validate_straw_man_traps | M | 1h | server.rs |
| 16.3 | implement validate_vorp | S | 0.5h | server.rs |
| 16.4 | implement validate_hole_punching | S | 0.5h | server.rs |

### GROUP 17: KIRK Servers (4 beads, ~2.5h)

| ID | Bead | Size | Hours | File |
|----|------|------|-------|------|
| 17.1 | create KirkContract types | S | 0.5h | kirk.rs |
| 17.2 | implement EARS extraction | M | 1h | kirk.rs |
| 17.3 | implement KIRK constraints extraction | M | 1h | kirk.rs |
| 17.4 | implement compile_to_kirk | M | 1h | server.rs |

### GROUP 18: Cleanup (2 beads, ~1h)

| ID | Bead | Size | Hours | File |
|----|------|------|-------|------|
| 18.1 | delete old components | S | 0.5h | (multiple) |
| 18.2 | update mod.rs exports | S | 0.5h | mod.rs |

## Dependency Graph

```
FOUNDATION (must complete first)
├── Group 1: State Machine (blocks all UI groups)
└── Group 2: Storage (blocks Group 15)

PARALLEL TRACKS after foundation:
├── TRACK A: UI Components (Groups 3-14)
│   └── Sequential within confirm phases (5-9)
├── TRACK B: Server Functions (Groups 16-17)
│   └── Group 16 before Group 5 (validation needed)
└── TRACK C: Main Container (Group 15)
    └── After all phases complete

CLEANUP (Group 18)
└── After Group 15 complete
```

## Execution Order (Recommended)

### Sprint 1: Foundation (Day 1)
Groups 1, 2 → State machine + Storage

### Sprint 2: Prompt + Extracting (Day 2)
Groups 3, 4 → First two phases

### Sprint 3-4: Confirm Phases (Days 3-4)
Groups 5-11 → All confirmation sub-phases

### Sprint 5: Preview + Kirk + Locked (Day 5)
Groups 12-14 → Final phases

### Sprint 6: Main Container (Day 6)
Group 15 → Orchestration

### Sprint 7: Server Functions (Day 7, parallel)
Groups 16-17 → Backend validation

### Sprint 8: Cleanup (Day 8)
Group 18 → Remove old components

## Summary Statistics

| Metric | Value |
|--------|-------|
| Total Beads | 62 |
| Total Hours | 28.5h |
| Groups | 18 |
| XS (15min) | 5 |
| S (30min) | 37 |
| M (1h) | 20 |
| Max Parallelization | 3 tracks |

---

*Generated: 2026-02-26*
*Ready for `/spec-kitty.tasks` to generate work packages*
