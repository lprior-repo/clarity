# Progressive Discover Phase - Work Packages

**Feature**: 001-progressive-discover-phase
**Generated**: 2026-02-26
**Total Work Packages**: 10
**Total Beads**: 29 (Progressive Discover specific)

---

## Overview

This document maps work packages to existing beads in the br system. Each work package groups related beads for parallel implementation.

### Bead Status Tracking

Use `br` commands to manage beads:
```bash
br ready              # Show beads ready to work
br show bd-XXXX       # View bead details
br claim bd-XXXX      # Claim a bead
br close bd-XXXX      # Close completed bead
```

### Dependency Graph

```
WP01: Server Functions (no UI deps)
  │
WP02: Prompt Phase UI ──┐
WP03: Extracting Phase  │
                        ├──► WP04: Confirm Phase UI
WP01 (server funcs) ────┘
                              │
                              ▼
                        WP05: Preview Phase UI
                              │
                              ▼
                        WP06: Kirk + Locked Phases
                              │
                              ▼
                        WP07: Main Container
                              │
                              ▼
                        WP08: Cleanup
```

---

## Work Packages

### WP01: Server Functions

**Beads**: 7
**Dependencies**: None (foundation)
**Can Parallelize**: Yes - all server functions are independent

| Bead ID | Title | Status |
|---------|-------|--------|
| bd-378l | server: implement validate_antithesis | [ ] |
| bd-28v1 | server: implement validate_straw_man_traps | [ ] |
| bd-2mcc | server: implement validate_vorp | [ ] |
| bd-13yb | server: implement validate_hole_punching | [ ] |
| bd-2uci | server: create KirkContract types | [ ] |
| bd-zf68 | server: implement EARS extraction | [ ] |
| bd-l1qq | server: implement compile_to_kirk | [ ] |

**Implementation Command**: `spec-kitty implement WP01`

**Details**: See `tasks/WP01-server-functions.md`

---

### WP02: Prompt Phase UI

**Beads**: 4
**Dependencies**: None
**Can Parallelize**: Yes - components are independent

| Bead ID | Title | Status |
|---------|-------|--------|
| bd-1fpe | ui-prompt: create scaffolding prompt buttons | [ ] |
| bd-3po6 | ui-prompt: create main textarea | [ ] |
| bd-2x35 | ui-prompt: create ExtractFieldsButton | [ ] |
| bd-khid | ui-prompt: compose PromptPhase component | [ ] |

**Implementation Command**: `spec-kitty implement WP02`

**Details**: See `tasks/WP02-prompt-phase-ui.md`

---

### WP03: Extracting Phase UI

**Beads**: 2
**Dependencies**: None
**Can Parallelize**: Yes

| Bead ID | Title | Status |
|---------|-------|--------|
| bd-23qy | ui-extracting: create progress animation | [ ] |
| bd-xz68 | ui-extracting: compose ExtractingPhase component | [ ] |

**Implementation Command**: `spec-kitty implement WP03`

**Details**: See `tasks/WP03-extracting-phase-ui.md`

---

### WP04: Confirm Phase UI

**Beads**: 6
**Dependencies**: WP01 (validation functions), WP02 (for context)
**Can Parallelize**: Partially - display components can be parallel, composition depends on them

| Bead ID | Title | Status |
|---------|-------|--------|
| bd-1hkh | ui-confirm: create ProblemDisplay component | [ ] |
| bd-24dz | ui-confirm: create AntithesisInput component | [ ] |
| bd-36lz | ui-confirm: create AntithesisQuality indicator | [ ] |
| bd-fskg | ui-confirm: create StrawManTrap checklist | [ ] |
| bd-2smr | ui-confirm: create PersonaDisplay component | [ ] |
| bd-2jjb | ui-confirm: compose ProblemConfirm component | [ ] |

**Implementation Command**: `spec-kitty implement WP04 --base WP01`

**Details**: See `tasks/WP04-confirm-phase-ui.md`

---

### WP05: Preview Phase UI

**Beads**: 3
**Dependencies**: WP04 (confirm phase complete)
**Can Parallelize**: Partially

| Bead ID | Title | Status |
|---------|-------|--------|
| bd-3h2v | ui-preview: create summary display | [ ] |
| bd-2k1q | ui-preview: create Four Brutal Truths checklist | [ ] |
| bd-3fz2 | ui-preview: compose PreviewPhase component | [ ] |

**Implementation Command**: `spec-kitty implement WP05 --base WP04`

**Details**: See `tasks/WP05-preview-phase-ui.md`

---

### WP06: Kirk Compilation + Locked Phase

**Beads**: 4
**Dependencies**: WP01 (compile_to_kirk), WP05 (preview complete)
**Can Parallelize**: Kirk and Locked can be done in parallel

| Bead ID | Title | Status |
|---------|-------|--------|
| bd-1jie | ui-kirk: create compilation progress | [ ] |
| bd-3cpp | ui-kirk: compose KirkCompilationPhase component | [ ] |
| bd-1le0 | ui-locked: create completion summary | [ ] |
| bd-3nia | ui-locked: compose LockedPhase component | [ ] |

**Implementation Command**: `spec-kitty implement WP06 --base WP05`

**Details**: See `tasks/WP06-kirk-locked-phase.md`

---

### WP07: Main Container

**Beads**: 2
**Dependencies**: All phase components (WP02-WP06)
**Can Parallelize**: No - depends on all phases

| Bead ID | Title | Status |
|---------|-------|--------|
| bd-1vgj | ui-main: create state machine hook | [ ] |
| bd-3ja1 | ui-main: compose ProgressiveDiscover component | [ ] |

**Implementation Command**: `spec-kitty implement WP07 --base WP06`

**Details**: See `tasks/WP07-main-container.md`

---

### WP08: Cleanup

**Beads**: 1
**Dependencies**: WP07 (main container working)
**Can Parallelize**: No

| Bead ID | Title | Status |
|---------|-------|--------|
| bd-3ctz | cleanup: delete old Express/Guided components | [ ] |

**Implementation Command**: `spec-kitty implement WP08 --base WP07`

**Details**: See `tasks/WP08-cleanup.md`

---

## Summary Statistics

| Metric | Value |
|--------|-------|
| Total Work Packages | 8 |
| Total Progressive Discover Beads | 29 |
| Largest WP | WP01, WP04 (7 and 6 beads) |
| Smallest WP | WP03, WP07, WP08 (2, 2, 1 beads) |

### Parallelization Opportunities

**Phase 1 (Foundation)**: WP01, WP02, WP03 can all run in parallel
- WP01: Server functions (no UI)
- WP02: Prompt phase UI
- WP03: Extracting phase UI

**Phase 2 (Confirm)**: WP04 after WP01 completes

**Phase 3 (Final)**: WP05 → WP06 → WP07 → WP08 (sequential)

### Recommended Execution

```bash
# Phase 1: Run in parallel (3 agents)
spec-kitty implement WP01  # Agent 1: Server functions
spec-kitty implement WP02  # Agent 2: Prompt UI
spec-kitty implement WP03  # Agent 3: Extracting UI

# Phase 2: After WP01 completes
spec-kitty implement WP04 --base WP01

# Phase 3: Sequential
spec-kitty implement WP05 --base WP04
spec-kitty implement WP06 --base WP05
spec-kitty implement WP07 --base WP06
spec-kitty implement WP08 --base WP07
```

---

## Bead Quick Reference

All Progressive Discover beads with their current status:

```
ui-prompt (4 beads):
  bd-1fpe - scaffolding prompt buttons
  bd-3po6 - main textarea
  bd-2x35 - ExtractFieldsButton
  bd-khid - PromptPhase component

ui-extracting (2 beads):
  bd-23qy - progress animation
  bd-xz68 - ExtractingPhase component

ui-confirm (6 beads):
  bd-1hkh - ProblemDisplay component
  bd-24dz - AntithesisInput component
  bd-36lz - AntithesisQuality indicator
  bd-fskg - StrawManTrap checklist
  bd-2smr - PersonaDisplay component
  bd-2jjb - ProblemConfirm component

ui-preview (3 beads):
  bd-3h2v - summary display
  bd-2k1q - Four Brutal Truths checklist
  bd-3fz2 - PreviewPhase component

ui-kirk (2 beads):
  bd-1jie - compilation progress
  bd-3cpp - KirkCompilationPhase component

ui-locked (2 beads):
  bd-1le0 - completion summary
  bd-3nia - LockedPhase component

ui-main (2 beads):
  bd-1vgj - state machine hook
  bd-3ja1 - ProgressiveDiscover component

server (7 beads):
  bd-378l - validate_antithesis
  bd-28v1 - validate_straw_man_traps
  bd-2mcc - validate_vorp
  bd-13yb - validate_hole_punching
  bd-2uci - KirkContract types
  bd-zf68 - EARS extraction
  bd-l1qq - compile_to_kirk

cleanup (1 bead):
  bd-3ctz - delete old components
```

---

## Next Steps

1. Run `br ready` to see which beads are ready to claim
2. Start with WP01, WP02, WP03 in parallel
3. Use `br claim bd-XXXX` before starting each bead
4. Use `br close bd-XXXX` when complete
5. Run `cargo check` and `cargo test` after each WP
