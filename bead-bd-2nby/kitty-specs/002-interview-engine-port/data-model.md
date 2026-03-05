# Data Model: Intent-CLI Port

**Feature**: 002-interview-engine-port
**Date**: 2026-02-27

## Entity Relationship Diagram

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              SPEC LAYER                                      │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌──────────────┐     1:N      ┌──────────────┐     1:N    ┌─────────────┐ │
│  │     Spec     │─────────────▶│   Feature    │───────────▶│  Behavior   │ │
│  └──────────────┘              └──────────────┘            └─────────────┘ │
│         │                              │                          │         │
│         │ 1:N                          │                          │ 1:N     │
│         ▼                              │                          ▼         │
│  ┌──────────────┐                      │                   ┌─────────────┐ │
│  │  Invariant   │                      │                   │Verification │ │
│  └──────────────┘                      │                   └─────────────┘ │
│         │                              │                                   │
│         │ 1:N                          │                                   │
│         ▼                              │                                   │
│  ┌──────────────┐                      │                                   │
│  │ AntiPattern  │                      │                                   │
│  └──────────────┘                      │                                   │
│                                        │                                   │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│                           INTERVIEW LAYER                                    │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌──────────────────┐   1:N    ┌──────────────┐                             │
│  │ InterviewSession │─────────▶│    Answer    │                             │
│  └──────────────────┘          └──────────────┘                             │
│         │                              │                                    │
│         │ 1:N                          │                                    │
│         ▼                              ▼                                    │
│  ┌──────────────┐              ┌──────────────┐                             │
│  │     Gap      │              │   Conflict   │                             │
│  └──────────────┘              └──────────────┘                             │
│         │                              │                                    │
│         │ N:1                          │ N:2                                │
│         ▼                              ▼                                    │
│  ┌──────────────┐              ┌──────────────────┐                         │
│  │   Question   │              │ConflictResolution│                         │
│  └──────────────┘              └──────────────────┘                         │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│                            PLANNING LAYER                                    │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌───────────────┐    1:N    ┌───────────────┐    1:N    ┌───────────────┐ │
│  │ ExecutionPlan │──────────▶│    Phase      │──────────▶│   PlanBead    │ │
│  └───────────────┘           └───────────────┘           └───────────────┘ │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│                             BEAD LAYER                                       │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌───────────────┐    derived from    ┌──────────────────┐                  │
│  │  BeadRecord   │◀───────────────────│ InterviewSession │                  │
│  └───────────────┘                    └──────────────────┘                  │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Core Entities

### Spec Layer

#### Spec

The root specification entity containing all feature definitions.

| Field | Type | Description |
|-------|------|-------------|
| name | String | Spec name (required, non-empty) |
| description | String | Detailed description |
| audience | String | Target audience |
| version | String | Spec version |
| success_criteria | Vec\<String\> | List of success criteria |
| features | Vec\<Feature\> | Feature definitions |
| invariants | Vec\<Invariant\> | Global invariants |
| anti_patterns | Vec\<AntiPattern\> | Anti-patterns to avoid |
| ai_hints | AIHints | AI implementation hints |

#### Feature

A logical grouping of related behaviors.

| Field | Type | Description |
|-------|------|-------------|
| name | String | Feature name (required, non-empty) |
| description | String | Feature description |
| behaviors | Vec\<Behavior\> | Behavior definitions |

#### Behavior

A single testable behavior.

| Field | Type | Description |
|-------|------|-------------|
| name | String | Behavior name (identifier format) |
| intent | String | What the behavior accomplishes |
| notes | String | Additional notes |
| requires | Vec\<String\> | Dependencies on other behaviors |
| tags | Vec\<String\> | Categorization tags |
| preconditions | Vec\<String\> | Preconditions |
| postconditions | Vec\<String\> | Postconditions |
| verifications | Vec\<Verification\> | Verification methods |

#### Verification

How to verify a behavior works correctly.

| Field | Type | Description |
|-------|------|-------------|
| description | String | Verification description |
| criteria | Vec\<String\> | Success criteria |
| examples | Vec\<Json\> | Example data |

#### Invariant

A global constraint that must always hold.

| Field | Type | Description |
|-------|------|-------------|
| name | String | Invariant name |
| description | String | What the invariant ensures |
| criteria | Vec\<String\> | Invariant criteria |

#### AntiPattern

A pattern to avoid with examples.

| Field | Type | Description |
|-------|------|-------------|
| name | String | Anti-pattern name |
| description | String | Why it's problematic |
| bad_example | Json | Example of the anti-pattern |
| good_example | Json | Correct alternative |
| why | String | Explanation |

#### AIHints

AI implementation guidance.

| Field | Type | Description |
|-------|------|-------------|
| implementation | ImplementationHints | Stack suggestions |
| entities | HashMap\<String, EntityHint\> | Entity hints |
| security | SecurityHints | Security guidance |
| pitfalls | Vec\<String\> | Common mistakes |

### Interview Layer

#### InterviewSession

The central interview state machine.

| Field | Type | Description |
|-------|------|-------------|
| id | String | Unique session identifier |
| profile | Profile | System type being specified |
| created_at | String | ISO 8601 timestamp |
| updated_at | String | ISO 8601 timestamp |
| completed_at | Option\<String\> | Completion timestamp |
| stage | InterviewStage | Current state machine state |
| rounds_completed | u32 | Number of completed rounds |
| answers | Vec\<Answer\> | Collected answers |
| gaps | Vec\<Gap\> | Detected gaps |
| conflicts | Vec\<Conflict\> | Detected conflicts |
| raw_notes | String | Unstructured notes |
| current_phase | u32 | Current phase number |
| completed_phases | Vec\<u32\> | Completed phase numbers |

#### Profile (Enum)

System type being specified.

| Variant | Description |
|---------|-------------|
| Api | REST/GraphQL API |
| Cli | Command-line interface |
| Event | Event-driven system |
| Data | Data processing system |
| Workflow | Workflow automation |
| Ui | User interface |

#### InterviewStage (Enum)

Session lifecycle states.

| Variant | Description |
|---------|-------------|
| Discovery | Initial information gathering |
| Refinement | Detail refinement |
| Validation | Validation of collected info |
| Complete | Interview complete |
| Paused | Paused for later resumption |

#### Answer

A single question answer.

| Field | Type | Description |
|-------|------|-------------|
| question_id | String | Question identifier |
| question_text | String | Question text |
| perspective | Perspective | Answer perspective |
| round | u32 | Round number |
| response | String | User response |
| extracted | HashMap\<String, String\> | Extracted fields |
| confidence | f64 | Confidence score (0-1) |
| notes | String | Additional notes |
| timestamp | String | ISO 8601 timestamp |

#### Gap

Missing information blocking completion.

| Field | Type | Description |
|-------|------|-------------|
| id | String | Gap identifier |
| field | String | Missing field |
| description | String | Gap description |
| blocking | bool | Blocks progression |
| suggested_default | String | Suggested value |
| why_needed | String | Why this is needed |
| round | u32 | Detection round |
| resolved | bool | Resolution status |
| resolution | String | Resolution value |

#### Conflict

Contradiction between answers.

| Field | Type | Description |
|-------|------|-------------|
| id | String | Conflict identifier |
| between | (String, String) | Conflicting answer IDs |
| description | String | Conflict description |
| impact | String | Impact if unresolved |
| options | Vec\<ConflictResolution\> | Resolution options |
| chosen | Option\<i32\> | Selected option index |

### Planning Layer

#### ExecutionPlan

Execution plan with phases.

| Field | Type | Description |
|-------|------|-------------|
| session_id | String | Source session |
| phases | Vec\<Phase\> | Execution phases |
| blockers | Vec\<String\> | Blocking issues |
| total_beads | u32 | Total bead count |

#### Phase

A single execution phase.

| Field | Type | Description |
|-------|------|-------------|
| phase_number | u32 | Phase number |
| name | String | Phase name |
| beads | Vec\<PlanBead\> | Phase beads |
| status | PhaseStatus | Phase status |

#### PlanBead

A work item in the plan.

| Field | Type | Description |
|-------|------|-------------|
| id | String | Bead identifier |
| title | String | Bead title |
| description | String | Detailed description |
| requires | Vec\<String\> | Dependencies |
| effort | Effort | Effort estimate |
| status | BeadStatus | Current status |

### Bead Layer

#### BeadRecord

Generated work item record.

| Field | Type | Description |
|-------|------|-------------|
| title | String | Work item title |
| description | String | Detailed description |
| profile_type | String | Source profile |
| priority | u8 | Priority (0-4) |
| issue_type | String | Type classification |
| labels | Vec\<String\> | Tags |
| ai_hints | String | AI implementation hints |
| acceptance_criteria | Vec\<String\> | Definition of done |
| dependencies | Vec\<String\> | Dependencies |

## State Transitions

### InterviewSession State Machine

```
┌───────────┐     start      ┌───────────┐
│   New     │───────────────▶│ Discovery │
└───────────┘                └───────────┘
                                  │
                      round_complete │
                                  ▼
                            ┌───────────┐
                            │ Refinement│
                            └───────────┘
                                  │
                      gaps_resolved │
                                  ▼
                            ┌───────────┐
                            │ Validation│
                            └───────────┘
                                  │
                      validated   │
                                  ▼
                            ┌───────────┐
                            │ Complete  │
                            └───────────┘
                                  ▲
                                  │
                      resume      │
                            ┌───────────┐
                            │  Paused   │
                            └───────────┘
```

## Storage Schema

### JSONL Session File

```jsonl
{"id":"sess-001","profile":"api","stage":"discovery","answers":[...],"gaps":[...],"conflicts":[]}
{"id":"sess-002","profile":"cli","stage":"complete","answers":[...],"gaps":[],"conflicts":[]}
```

### Session History File

```jsonl
{"session_id":"sess-001","timestamp":"2026-02-27T00:00:00Z","description":"Initial creation","snapshot":{...}}
{"session_id":"sess-001","timestamp":"2026-02-27T01:00:00Z","description":"Added answer for q-auth","snapshot":{...}}
```

## Index Recommendations

For efficient queries on in-memory collections:

1. **answers_by_question_id**: HashMap\<String, &Answer\>
2. **gaps_by_field**: HashMap\<String, &Gap\>
3. **conflicts_by_answer**: HashMap\<String, Vec\<&Conflict\>\>
4. **behaviors_by_name**: HashMap\<String, &Behavior\>
