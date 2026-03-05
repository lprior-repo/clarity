# Feature Specification: Progressive Discover Phase

**Feature ID**: 001-progressive-discover-phase
**Mission**: software-dev
**Created**: 2026-02-26
**Status**: DRAFT

---

## Problem Statement

Users starting a new project often struggle to clearly articulate their problem, audience, solution, and success criteria. This leads to poorly defined plans that fail during execution. Users need a guided wizard that forces rigorous thinking through adversarial validation before any planning begins.

---

## User Personas

### Primary: Product Builder
A developer or product manager who wants to create a well-defined plan for a new feature or project. They may have a vague idea but need help crystallizing it into something actionable.

### Secondary: Team Lead
A technical leader who wants to ensure their team's plans are well thought through before committing resources.

---

## Goals & Success Metrics

### Business Goals
- Reduce planning failures by ensuring all plans pass adversarial validation
- Increase plan quality scores across the platform
- Improve user confidence in their planning decisions

### Success Metrics
1. Users complete the full wizard in under 15 minutes
2. 90% of completed plans achieve a quality score of 0.7 or higher
3. 80% of users who complete the wizard proceed to implementation
4. Hole punching catches at least one gap in 60% of scenarios

---

## User Scenarios & Testing

### Primary Flow: Successful Plan Creation
1. User opens Progressive Discover
2. User enters problem statement (min 50 characters)
3. System extracts 5 fields: Problem, Persona, Solution, Nonpersona, Scenario
4. User reviews each field with adversarial coaching:
   - Problem: Provides 3 null hypothesis points (antithesis)
   - Persona: Checks for 4 straw man traps
   - Solution: Validates VORP (Value, Obvious, Real, Possible)
   - Nonpersona: Confirms who is excluded
   - Scenario: Hole punching for 3 types of gaps
5. User previews summary and checks Four Brutal Truths
6. User locks in the plan
7. System compiles to 16-section KIRK contract
8. Plan is locked and beads are generated

### Edge Cases
- User refines problem statement and restarts extraction
- Antithesis quality too low - user must improve
- Straw man trap detected - user must acknowledge and fix
- VORP validation fails - user must provide better justification
- Hole punching reveals gaps - user must address

### Error Scenarios
- Extraction fails - show error, allow retry
- Network timeout - preserve progress, allow resume
- Invalid input - inline validation with helpful messages

---

## Functional Requirements

### FR1: Prompt Phase
- FR1.1: System shall display 3 scaffolding prompt buttons to help users start
- FR1.2: System shall provide a textarea with 2000 character limit
- FR1.3: System shall show live character count
- FR1.4: System shall disable "Extract Fields" button until 50 characters entered
- FR1.5: System shall trigger AI extraction on button click

### FR2: Extracting Phase
- FR2.1: System shall display animated progress bar during extraction
- FR2.2: System shall show status messages indicating extraction progress
- FR2.3: System shall auto-transition to Confirm phase on completion

### FR3: Confirm Phase - Problem Field
- FR3.1: System shall display extracted problem statement in editable textarea
- FR3.2: System shall require 3 antithesis points (null hypothesis)
- FR3.3: System shall calculate and display antithesis quality score (0-1)
- FR3.4: System shall block progression if quality score < 0.5

### FR4: Confirm Phase - Persona Field
- FR4.1: System shall display extracted persona in editable textarea
- FR4.2: System shall check for 4 straw man traps: Irrational Actor, Manic Pixie Dream User, Stoic Monk, Your Clone
- FR4.3: System shall display trap explanations when detected
- FR4.4: System shall require acknowledgment of all traps before proceeding

### FR5: Confirm Phase - Solution Field
- FR5.1: System shall display extracted solution in editable textarea
- FR5.2: System shall require VORP justification input
- FR5.3: System shall validate VORP specificity
- FR5.4: System shall display validation feedback

### FR6: Confirm Phase - Nonpersona Field
- FR6.1: System shall display extracted nonpersona in editable textarea
- FR6.2: System shall allow free-form editing

### FR7: Confirm Phase - Scenario Field
- FR7.1: System shall display 3 scenario components: trigger, value moment, feeling
- FR7.2: System shall require hole punching for 3 gap types: discovery hole, edge case hole, motivation dropoff
- FR7.3: System shall indicate completeness status

### FR8: Confirm Phase Navigation
- FR8.1: System shall display progress indicator (1/5, 2/5, etc.)
- FR8.2: System shall provide Back and Next navigation buttons
- FR8.3: System shall persist state on each field completion

### FR9: Preview Phase
- FR9.1: System shall display summary of all confirmed fields
- FR9.2: System shall display Four Brutal Truths checklist: Scale, Back-loaded Value, VORP, Sustaining
- FR9.3: System shall provide "Refine" button to return to Prompt phase
- FR9.4: System shall provide "Lock In" button to proceed to compilation

### FR10: KIRK Compilation Phase
- FR10.1: System shall display 16-section compilation progress
- FR10.2: System shall show completion indicators for each section
- FR10.3: System shall auto-transition to Locked phase on completion

### FR11: Locked Phase
- FR11.1: System shall display "Plan Locked" confirmation
- FR11.2: System shall show generated bead count
- FR11.3: System shall provide navigation to Plan, Graph, and State views

### FR12: State Persistence
- FR12.1: System shall auto-save transcript on each state transition
- FR12.2: System shall recover transcript on page reload
- FR12.3: System shall support crash recovery

---

## Non-Functional Requirements

### NFR1: Performance
- Field extraction completes in under 10 seconds
- State transitions render in under 100ms
- Auto-save completes in under 500ms

### NFR2: Accessibility
- All interactive elements keyboard accessible
- Screen reader compatible
- Color contrast meets WCAG AA

### NFR3: Reliability
- No data loss on browser crash
- Graceful degradation on network failure
- Progress recovery on session restore

---

## Key Entities

### InterrogationTranscript
The complete record of the user's discovery session, including original prompt, extracted fields, validation results, and timestamps.

### ProgressiveDiscoverPhase
Enumeration of wizard phases: Prompt, Extracting, ConfirmingFields, Preview, KirkCompilation, Locked.

### ConfirmSubPhase
Enumeration of confirmation steps: Problem, Persona, Solution, Nonpersona, Scenario.

### AntithesisResponse
Three null hypothesis points with quality score.

### StrawManValidation
Detected persona traps with pass/fail status.

### HolePunchingResults
Three gap types: discovery hole, edge case hole, motivation dropoff.

### KirkContract
16-section compiled planning document.

---

## Assumptions

- AI extraction service is available and reliable
- User has basic understanding of their problem domain
- Single-user sessions (no collaboration during discovery)
- Desktop-first UI (mobile support not required initially)
- Dioxus 0.7 + Tailwind CSS for frontend implementation
- redb for local state persistence

---

## Out of Scope

- Multi-user collaborative discovery sessions
- Mobile-responsive design (future enhancement)
- Import/export of transcripts
- Template-based discovery flows
- Integration with external AI providers beyond the default

---

## Dependencies

- AI extraction provider for field extraction
- Validation server functions for adversarial checks
- Storage layer for transcript persistence
- KIRK compilation service

---

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| AI extraction poor quality | Medium | High | Allow manual editing of all fields |
| Users skip validation | Medium | Medium | Block progression on failed checks |
| Session timeout | Low | Medium | Auto-save and crash recovery |
| Complex state management | Medium | High | Clear state machine with tests |

---

## References

- `docs/PROGRESSIVE_DISCOVER_PLAN.md` - Original implementation plan
- `docs/PROGRESSIVE_DISCOVER_BEADS.md` - 62 atomic bead breakdown
- `docs/BEAD_TRACKING_SHEET.md` - Progress tracking
- `docs/VISION-ProgressiveDiscover.md` - Feature vision document
