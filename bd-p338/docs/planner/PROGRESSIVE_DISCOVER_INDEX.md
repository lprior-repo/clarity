# Progressive Discover Phase - Master Index

## Document Suite

This index provides navigation to all documentation for the Progressive Discover Phase implementation.

---

## Core Documents

### 1. Vision Document
**Location**: `/home/lewis/src/clarity/docs/VISION-ProgressiveDiscover.md`

**Contains**:
- Executive Summary
- Problem Statement (current issues)
- Solution: Progressive Discover
- Complete User Journey (visual flow)
- Key Innovations (Adversarial AI, KIRK mapping)
- Technical Architecture
- Adversarial Coaching Patterns
- Success Metrics
- Glossary

**Read First**: Yes - This is the primary reference document.

---

### 2. Master Implementation Plan
**Location**: `/home/lewis/src/clarity/docs/PROGRESSIVE_DISCOVER_PLAN.md`

**Contains**:
- Task Dependency Graph (visual)
- Complete 14-task list
- Each task with: ID, hours, complexity, files, dependencies
- Summary Statistics
- 4-Week Implementation Schedule
- File Structure After Implementation

**Read Second**: Yes - This is the execution roadmap.

---

### 3. Task Definitions Directory
**Location**: `/home/lewis/src/clarity/docs/planner/tasks/`

**Contains**: 14 JSON task files

| File | Hours | Complexity | Description |
|------|-------|------------|-------------|
| `progressive-discover-state-machine.json` | 4h | medium | Core state machine types |
| `storage-integration.json` | 6h | medium | State persistence (redb) |
| `prompt-phase-component.json` | 6h | medium | Freeform input + extraction |
| `extracting-phase-component.json` | 4h | low | Loading animation |
| `confirm-phase-component.json` | 16h | **high** | Adversarial coaching wizard |
| `preview-phase-component.json` | 8h | medium | Summary + brutal truths |
| `kirk-compilation-phase-component.json` | 10h | medium | KIRK generation progress |
| `locked-phase-component.json` | 6h | low | Collapsed completion view |
| `progressive-discover-main-container.json` | 10h | medium | State machine orchestration |
| `validation-server-functions.json` | 8h | medium | 4 adversarial validators |
| `compile-to-kirk-server-function.json` | 12h | **high** | KIRK contract generation |
| `delete-old-express-guided-components.json` | 2h | low | Cleanup old components |
| `e2e-tests.json` | 16h | medium | Playwright scenarios |
| `INDEX.json` | - | - | Dependency graph metadata |

**Use For**: Individual task reference when implementing.

---

### 4. Task Navigation Guide
**Location**: `/home/lewis/src/clarity/docs/planner/TASK_NAVIGATION.md`

**Contains**:
- How to read task JSON files
- Dependency explanations
- Critical path identification
- Parallelization opportunities

**Use For**: Understanding task relationships.

---

## Quick Reference Tables

### State Machine States

| State | Description | Next States |
|-------|-------------|-------------|
| `PROMPT` | User enters freeform description | `EXTRACTING` |
| `EXTRACTING` | AI extracts 5 fields | `CONFIRMING_FIELDS` |
| `CONFIRMING_FIELDS` | Field-by-field adversarial coaching | `PREVIEW` |
| `PREVIEW` | Summary + Four Brutal Truths | `KIRK_COMPILATION` or `PROMPT` (refine) |
| `KIRK_COMPILATION` | Generate KIRK contracts | `LOCKED` |
| `LOCKED` | Handoff to Bead Factory | - (terminal) |

### Confirm Sub-Phases

| Sub-Phase | Adversarial Pattern | Validation |
|-----------|-------------------|------------|
| `CONFIRM_PROBLEM` | Antithesis (3 null hypotheses) | Quality score > 0.7 |
| `CONFIRM_PERSONA` | Straw Man traps (4 types) | No traps detected |
| `CONFIRM_SOLUTION` | VORP test | Concrete improvement quantified |
| `CONFIRM_NONPERSONA` | Explicit exclusion | Non-empty description |
| `CONFIRM_SCENARIO` | Hole Punching (3 checks) | All holes addressed |

### Straw Man Traps

| Trap | Description | Example |
|------|-------------|---------|
| `IrrationalActor` | Acts against stated motivations | Environmentalist who loves single-use plastics |
| `ManicPixieDreamUser` | Magically loves everything | Busy mom wanting hours of learning time |
| `StoicMonk` | Tolerates immense friction | Developer loving CLI debugging for fun |
| `YourClone` | Has your system knowledge | Non-technical user knowing Git workflows |

### Four Brutal Truths

| Truth | Question |
|-------|----------|
| **Scale** | Survives 10,000 users, not just 10? |
| **Back-loaded Value** | All "checkbox" features included? |
| **VORP** | Significantly better than workaround? |
| **Sustaining** | Won't break down in a month? |

### Hole Punching Checks

| Hole | Question |
|------|----------|
| **Discovery** | How did they discover the feature? |
| **Edge Case** | What if internet drops, they mistype, etc.? |
| **Motivation** | Why continue at high-friction steps? |

---

## Implementation Sequence

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           WEEK 1: FOUNDATION                               │
├─────────────────────────────────────────────────────────────────────────────┤
│ Day 1-2: progressive-discover-state-machine                                │
│ Day 3-4: storage-integration (PARALLEL)                                    │
│ Day 5:   prompt-phase-component                                             │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│                           WEEK 2: CORE UI FLOW                             │
├─────────────────────────────────────────────────────────────────────────────┤
│ Day 1-2: extracting-phase-component                                        │
│ Day 3-6: confirm-phase-component (HIGH COMPLEXITY)                         │
│ Day 7:   validation-server-functions (PARALLEL)                            │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│                           WEEK 3: COMPLETION FLOW                          │
├─────────────────────────────────────────────────────────────────────────────┤
│ Day 1-2: preview-phase-component                                           │
│ Day 3-5: compile-to-kirk-server-function (HIGH COMPLEXITY, EARLY START)    │
│ Day 5-6: kirk-compilation-phase-component                                  │
│ Day 7:   locked-phase-component                                            │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│                           WEEK 4: INTEGRATION                              │
├─────────────────────────────────────────────────────────────────────────────┤
│ Day 1-3: progressive-discover-main-container                               │
│ Day 4:   delete-old-express-guided-components                              │
│ Day 5-10: e2e-tests (EARLY START)                                          │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Server Functions

### New Functions

| Function | Purpose | Input | Output |
|----------|---------|-------|--------|
| `extract_fields_progressive` | Extract all 5 fields from freeform | `String` | `ExtractedFields` |
| `validate_antithesis` | Score null hypothesis quality | `[String; 3]` | `f64` (0-1) |
| `validate_straw_man_traps` | Detect persona traps | `String` | `StrawManValidation` |
| `validate_vorp` | Check VORP specificity | `String` | `ValidationResult` |
| `validate_hole_punching` | Check scenario completeness | `ScenarioField` | `HolePunchingResults` |
| `compile_to_kirk` | Generate KIRK contracts | `InterrogationTranscript` | `KirkContract` |
| `generate_beads_from_kirk` | Atomize into beads | `KirkContract` | `Vec<Bead>` |

---

## KIRK Contract Mapping

| Discovery Field | Bead Template Section | Output |
|-----------------|----------------------|--------|
| Problem | EARS Requirements | Ubiquitous requirements |
| Antithesis | Inversions | Security, usability failures |
| Solution | Implementation Tasks | Phase 0-4 decomposition |
| Persona | EARS Event-Driven | "WHEN user... THEN system..." |
| Nonpersona | EARS Unwanted | "IF nonpersona... SHALL NOT..." |
| Scenario | ATDD Tests | Happy paths, error paths |
| Hole Punching | Failure Modes | Symptoms, causes, debugging |
| VORP | Context | Related files, similar patterns |
| Four Brutal Truths | Verification Checkpoints | Research, tests, gates |

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

### E2E Tests (Playwright)
- Successful plan creation
- Antithesis quality rejection
- Straw Man trap detection
- VORP failure
- Hole punching failure
- Refine cycle

---

## Risk Register

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| AI extraction quality | High | Medium | Fallback to manual entry |
| User abandonment | Medium | Low | Conversational coaching tone |
| KIRK schema mismatch | High | Low | Early validation |
| Performance (large transcripts) | Medium | Medium | Streaming responses |

---

## Glossary

| Term | Definition |
|------|------------|
| **Antithesis** | Null hypothesis - 3 reasons the product will fail |
| **VORP** | Value Over Replacement Product - why this beats current workaround |
| **Straw Man Trap** | Unrealistic user archetype that sounds plausible |
| **Hole Punching** | Finding gaps in the happy path scenario |
| **KIRK** | Keep Invariants Regular and Known |
| **Four Brutal Truths** | Scale, Back-loaded Value, VORP, Sustaining |
| **Double Diamond** | Discover, Define, Develop, Deliver |

---

## Success Metrics

| Metric | Current | Target | How to Measure |
|--------|---------|--------|----------------|
| Time to Locked Plan | N/A | < 10 min | avg(prompt → locked) |
| Field Edit Rate | N/A | < 30% | edits / total confirms |
| Completion Rate | N/A | > 70% | locked / started |
| Antithesis Quality | N/A | > 0.7 | specificity score |
| VORP Specificity | N/A | > 0.8 | concrete / vague ratio |
| Generated Beads | 0 | 8-15 | per locked plan |

---

## Commands Reference

```bash
# View vision
cat /home/lewis/src/clarity/docs/VISION-ProgressiveDiscover.md

# View implementation plan
cat /home/lewis/src/clarity/docs/PROGRESSIVE_DISCOVER_PLAN.md

# List all tasks
ls /home/lewis/src/clarity/docs/planner/tasks/

# View specific task
cat /home/lewis/src/clarity/docs/planner/tasks/confirm-phase-component.json

# Find high complexity tasks
grep -l '"complexity": "high"' /home/lewis/src/clarity/docs/planner/tasks/*.json

# Find all dependencies
grep -h '"blocks"' /home/lewis/src/clarity/docs/planner/tasks/*.json | jq -r '.[][]' | sort -u
```

---

*Progressive Discover Phase - Master Index*
*Last Updated: 2026-02-25*
