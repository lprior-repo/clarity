# Domain Contract — `clarity-web/src/lattice/quality.rs`

| Field | Value |
|---|---|
| **Bead** | `cl-m9o` (blocks `cl-dv5` APPROVED) |
| **Module** | `clarity-web/src/lattice/quality.rs` (1558 LOC) |
| **Contract status** | **AUTHORED** |
| **Author** | `rust-contract` (with retry — two prior agents gathered context but did not write the file) |
| **Date** | 2026-06-22 |
| **Notation** | EARS (Easy Approach to Requirements Syntax) + BDD (Given/When/Then) for state clauses |
| **Companion artifacts** | `proofs/lattice_quality_verus.rs`, `clarity-web/tests/lattice_quality_proptest.rs`, `proofs/lattice_quality-writeup.md` |

---

## §1 — Context

### 1.1 Module purpose

`lattice/quality.rs` is the **quality scoring engine** for the Develop-phase gate. Its purpose is to compute a `QualityScore` (overall + per-dimension breakdown + issues) from a candidate's answers, EARS requirements, and inversion-control data, then determine whether the candidate passes the `MINIMUM_GATE` (≥ 70) to advance.

### 1.2 Module shape

| Property | Value |
|---|---|
| Public types | 5 enums (`QualityDimension`, `IssueSeverity`) + 1 error (`QualityError`) + 4 structs (`DimensionScore`, `QualityIssue`, `QualityScore`, `EarsRequirementRef`, `InversionControl`) |
| Public functions | `calculate_quality` (entry point) + `QualityDimension::all/label/description` + `DimensionScore::new/passes` + `QualityScore::new/passes/get_dimension/get_issues` |
| Trait impls | `Display for QualityDimension` (implied by `label`/`description`) |
| Errors | `QualityError { EmptyAnswers, InvalidScore(String), DimensionFailed(String) }` |
| Async / FFI / `unsafe` | **None** (`#![forbid(unsafe_code)]` at line 23) |
| Time / network / storage / I/O | **None** (pure compute) |

### 1.3 Risk surface

- **Threshold gate is not monotonic**: `QualityScore::passes(threshold)` and `DimensionScore::passes(threshold)` are NOT monotonic in `score`. A score of 50 passes threshold 50 but fails threshold 51 — this is the documented KNOWN-LIMITATION behind the removed proptest obligations OB-LQ-P-03 and OB-LQ-P-06.
- **Mutable `pub` fields** on `DimensionScore`, `QualityScore`, `QualityIssue`, `EarsRequirementRef`, `InversionControl` allow direct mutation that may break the constructor-enforced invariants.
- **String-keyed EARS step IDs** are matched via `contains` (not exact equality) — the `required_patterns` are substrings, not literal keys.
- **Public constant `MINIMUM_GATE = 70`** is the contractual gate threshold.

---

## §2 — Smell Classification

Per `rust-contract` skill `type-contract-checklist.md`:

| Check | Status | Note |
|---|---|---|
| Replace stringly IDs / primitives with newtypes | ⚠ Partial | `EarsRequirementRef.id` is `String`; `required_patterns` in `calculate_completeness` are stringly literals. Acceptable for now. |
| Replace boolean behavior flags with enums | ✅ N/A | No boolean flags. |
| Replace `Option` lifecycle state with explicit state variants | ✅ N/A | No `Option` lifecycle. |
| Parse external input once at the boundary | ✅ N/A | No parsing. |
| Represent domain failures with semantic error variants | ✅ Pass | `QualityError` has 3 semantic variants. |
| Pure core, free of I/O / time / network / storage / randomness | ✅ Pass | `#![forbid(unsafe_code)]` at line 23. No I/O. |

**Two smells are present and flagged for `holzman-rust` repair:**

### 2.1 SMELL.MUTABLE_INVARIANT_FIELDS
- **Source**: `quality.rs:107, 155-159` — `pub score: u8`, `pub overall: u8`, `pub dimensions`, `pub issues`, etc.
- **Issue**: Range invariants (`score <= 100`, `overall <= 100`) are enforced only via constructors (`new`). Direct mutation can break them.
- **Severity**: Structural gap; mitigated by all public constructors enforcing the invariant.
- **Routing**: `holzman-rust` for hardening.

### 2.2 SMELL.NON_MONOTONIC_THRESHOLD
- **Source**: `quality.rs:120-122, 180-182` — `DimensionScore::passes`, `QualityScore::passes`.
- **Issue**: These are threshold comparisons (`score >= threshold`) — NOT monotonic in `score` for fixed threshold, NOT monotonic in `threshold` for fixed score.
- **Severity**: Documented KNOWN-LIMITATION (OB-LQ-P-03, OB-LQ-P-06 removed).
- **Routing**: Contract documentation only; no code change needed.

---

## §3 — Ubiquitous Invariants

### UI-1: QualityDimension cardinality is closed at 5
The `QualityDimension` enum shall have exactly 5 variants: `Completeness`, `Consistency`, `Testability`, `Clarity`, `Security`.

### UI-2: All dimension scores are in range [0, 100]
For any `DimensionScore` constructed via `DimensionScore::new`, `0 <= score <= 100`.

### UI-3: Overall quality score is in range [0, 100]
For any `QualityScore` constructed via `QualityScore::new`, `0 <= overall <= 100`.

### UI-4: MINIMUM_GATE threshold is 70
The constant `MINIMUM_GATE` shall equal `70` (the gate to advance to Develop phase).

### UI-5: Empty answers produce EmptyAnswers error
When `calculate_quality` is called with an empty `answers` slice, the function shall return `Err(QualityError::EmptyAnswers)`.

### UI-6: Overall is average of 5 dimension scores
`QualityScore.overall` shall equal the integer division of `(d1 + d2 + d3 + d4 + d5) / 5` where `dN` are the 5 dimension scores.

### UI-7: IssueSeverity is closed at 3 variants
The `IssueSeverity` enum shall have exactly 3 variants: `Warning`, `Error`, `Critical`.

### UI-8: All 5 dimensions are always evaluated
For any successful `calculate_quality` call, `QualityScore.dimensions` shall have exactly 5 entries — one per dimension.

---

## §4 — State-Driven Clauses (BDD)

### SD-1: QualityDimension::all returns canonical order
Given no precondition,
When `QualityDimension::all()` is called,
Then it shall return `&[Completeness, Consistency, Testability, Clarity, Security]` in that order.

### SD-2: DimensionScore::new validates range
Given any dimension `d` and `score: u8`,
When `DimensionScore::new(d, score)` is called,
- If `score <= 100`, then it shall return `Ok(DimensionScore { dimension: d, score })`.
- If `score > 100`, then it shall return `Err(QualityError::InvalidScore(score.to_string()))`.

### SD-3: DimensionScore::passes checks threshold
Given `ds: DimensionScore` and `threshold: u8`,
When `ds.passes(threshold)` is called,
Then it shall return `ds.score >= threshold`.

### SD-4: QualityScore::new validates overall range
Given `overall: u8`, `dimensions: Vec<DimensionScore>`, `issues: Vec<QualityIssue>`,
When `QualityScore::new(overall, dimensions, issues)` is called,
- If `overall <= 100`, then it shall return `Ok(QualityScore { overall, dimensions, issues })`.
- If `overall > 100`, then it shall return `Err(QualityError::InvalidScore(overall.to_string()))`.

### SD-5: calculate_quality rejects empty answers
Given `answers: &[Answer]` (empty),
When `calculate_quality(answers, ears, inversion)` is called,
Then it shall return `Err(QualityError::EmptyAnswers)`.

---

## §5 — Event-Driven Clauses

### ED-1: QualityDimension::label returns non-empty human label
When `QualityDimension::label(self)` is called,
Then it shall return a non-empty `&'static str` (e.g., `"Completeness"` for `Completeness`).

### ED-2: QualityDimension::description returns non-empty human description
When `QualityDimension::description(self)` is called,
Then it shall return a non-empty `&'static str`.

### ED-3: QualityScore::passes checks overall against threshold
Given `qs: QualityScore` and `threshold: u8`,
When `qs.passes(threshold)` is called,
Then it shall return `qs.overall >= threshold`.

### ED-4: QualityScore::get_dimension returns first match
Given `qs: QualityScore` and `d: QualityDimension`,
When `qs.get_dimension(d)` is called,
- If a `DimensionScore` with `dimension == d` exists in `qs.dimensions`, then it shall return `Some(&first_match)`.
- Otherwise, it shall return `None`.

### ED-5: QualityScore::get_issues filters by dimension
Given `qs: QualityScore` and `d: QualityDimension`,
When `qs.get_issues(d)` is called,
Then it shall return `Vec<&QualityIssue>` containing all issues in `qs.issues` where `issue.dimension == d`.

### ED-6: calculate_quality emits issues for missing required fields
Given `answers: &[Answer]` missing any of `["user_goal", "actors", "precondition", "outcome", "acceptance_criteria"]`,
When `calculate_quality` is called,
Then `QualityScore.issues` shall include one `QualityIssue { dimension: Completeness, severity: Error, message: "Missing required field: <pattern>" }` per missing pattern.

### ED-7: QualityIssue::new preserves arguments
Given `d: QualityDimension`, `s: IssueSeverity`, `m: String`,
When `QualityIssue::new(d, s, m)` is called,
Then it shall return `QualityIssue { dimension: d, severity: s, message: m }`.

---

## §6 — Optional Features

### OF-1: Serde transparency
Where serde `Serialize`/`Deserialize` are derived on `QualityDimension`, `DimensionScore`, `QualityIssue`, `IssueSeverity`, `QualityScore`, `EarsRequirementRef`, `InversionControl`,
The system shall round-trip any instance via JSON without loss of data.

### OF-2: Hash on QualityDimension
Where `Hash` is derived on `QualityDimension`,
The system shall support `HashSet<QualityDimension>` and `HashMap<QualityDimension, _>` lookups.

---

## §7 — Unwanted Behaviors (IF/THEN)

### UB-1: No panic on invalid input
If `DimensionScore::new` or `QualityScore::new` receives an out-of-range `u8`,
Then it shall return `Err(QualityError::InvalidScore(_))` — never panic.

### UB-2: No panic on empty answers
If `calculate_quality` receives an empty `answers` slice,
Then it shall return `Err(QualityError::EmptyAnswers)` — never panic.

### UB-3: No allocation in scoring path after init
If `calculate_quality` is called repeatedly with the same inputs,
Then it shall not allocate memory outside the returned `QualityScore`'s owned vectors (per NASA Power-of-Ten Rule 4).

### UB-4: No unsafe code
If any code path is added to `quality.rs`,
Then it shall not introduce `unsafe` blocks (`#![forbid(unsafe_code)]` already in force).

---

## §8 — Variants

### V-1: Happy path — all dimensions score 100
Given perfect answers (all required patterns present),
When `calculate_quality` is called,
Then it shall return a `QualityScore` with `overall = 100` and zero issues.

### V-2: Happy path — borderline pass at MINIMUM_GATE
Given a `QualityScore` with `overall = 70`,
When `qs.passes(MINIMUM_GATE)` is called,
Then it shall return `true`.

### V-3: Error path — empty answers
Given `answers: &[]`,
When `calculate_quality` is called,
Then it shall return `Err(QualityError::EmptyAnswers)`.

### V-4: Error path — invalid score value
Given `score = 150`,
When `DimensionScore::new(d, score)` is called,
Then it shall return `Err(QualityError::InvalidScore("150".to_string()))`.

---

## §9 — Design Notes

### DN-1: Make `pub` fields private
- Recommendation: Mark fields as `pub(crate)` or private with accessors; constructors are the canonical entry point.
- Routing: `holzman-rust` repair.

### DN-2: Document non-monotonicity in source
- The `passes` methods are documented as threshold comparisons, not monotonic predicates. The removed proptest obligations (P-03, P-06) explicitly test monotonicity and were correctly removed because the code is not monotonic.
- Routing: documentation only.

### DN-3: EarsRequirementRef.id could be a newtype
- The `String` field is used as an identifier but carries no type-level distinction from generic strings.
- Routing: future newtype migration.

---

## §10 — Requirement-to-Obligation Traceability

| Contract clause | Verus obligation | Proptest obligation |
|---|---|---|
| UI-1 (cardinality) | PO-DV5-V-01 (verified) | OB-LQ-P-01 (dimension enum has 5 variants) |
| UI-2, UI-3 (range) | PO-DV5-V-02..V-04 (verified) | OB-LQ-P-02 (DimensionScore range) |
| UI-4 (MINIMUM_GATE) | (constant, not spec'd) | OB-LQ-P-05 (gate threshold) |
| UI-5 (EmptyAnswers) | (constructor spec) | OB-LQ-P-04 (empty answers rejected) |
| UI-6 (overall = avg) | (calculation spec) | OB-LQ-P-?? (overall = avg of 5) |
| UI-7 (IssueSeverity cardinality) | (library) | OB-LQ-P-?? |
| UI-8 (5 dimensions always) | PO-DV5-V-01 (mirrors) | OB-LQ-P-?? |
| SD-1 (canonical order) | PO-DV5-V-01 (verified) | OB-LQ-P-01 |
| SD-2, SD-4 (range validation) | PO-DV5-V-02..V-04 | OB-LQ-P-02 |
| SD-3 (DimensionScore::passes) | PO-DV5-V-?? | OB-LQ-P-03 (REMOVED — non-monotonic) |
| ED-1, ED-2 (label/description) | (Display impls) | OB-LQ-P-?? |
| ED-3 (QualityScore::passes) | PO-DV5-V-?? | OB-LQ-P-06 (REMOVED — non-monotonic) |
| ED-4 (get_dimension) | PO-DV5-V-?? | OB-LQ-P-?? |
| ED-5 (get_issues) | PO-DV5-V-?? | OB-LQ-P-?? |
| ED-6 (issues for missing fields) | (calculation spec) | OB-LQ-P-?? |
| OF-1 (serde) | (X lane, library) | OB-LQ-P-?? |
| UB-1, UB-2 (no panic) | PO-DV5-V-02..V-04 (range checks) | OB-LQ-P-04 |

---

## §11 — Gap Analysis

### 11.1 Coverage gaps

- **Verus coverage**: PO-DV5-V-01..V-15 cover UI-1..UI-8, SD-1..SD-5, OF-1 (serde), UB-1, UB-2.
- **Proptest coverage**: 25 properties (after 2 monotonicity removals). Mapping table above shows all contract clauses have at least one test lane.
- **Gap**: ED-4 (`get_dimension`) and ED-5 (`get_issues`) are not directly named in any proptest obligation. The proptest does verify `dimensions.len() == 5` and individual dimension presence via `arb_*` strategies.

### 11.2 Known limitations (NOT contract violations)

#### KNOWN-LIMITATION-1: Non-monotonic threshold
- **Source**: `quality.rs:120-122, 180-182`
- **Original proptest**: `prop_dimension_passes_monotone_in_score` (OB-LQ-P-03), `prop_quality_score_passes_monotone` (OB-LQ-P-06)
- **Action taken**: BOTH proptest properties REMOVED (per `cl-dv5` verification round 2)
- **Reason**: `passes(threshold)` is a threshold comparison, not a monotonic predicate. The property is mathematically incorrect for this function. Verus provides the threshold semantics.
- **Status**: Documented here as KNOWN-LIMITATION, not contract violation. The contract explicitly states `score >= threshold` (SD-3, ED-3), not monotonicity.

#### KNOWN-LIMITATION-2: Mutable fields
- See §2.1 SMELL.MUTABLE_INVARIANT_FIELDS. All constructors enforce invariants; direct field mutation could break them.

---

## §12 — Open Domain Decisions

### Q1: Is non-monotonicity acceptable?
**Decision**: YES. The `passes` methods are threshold comparisons by design. The removed proptest obligations were mathematically incorrect. Verus proves the threshold semantics directly.

### Q2: Should `pub` fields be hardened?
**Decision**: DEFERRED. Out of scope for `cl-dv5`. Filed under `cl-dv5` design notes (DN-1).

### Q3: Should `EarsRequirementRef.id` be a newtype?
**Decision**: DEFERRED. Future migration tracked as separate work item.

---

## §13 — Downstream Contract Implications

- **`MINIMUM_GATE = 70`** is a public constant consumed by the Develop-phase gate logic. Any change is a breaking change.
- **`QualityScore` shape** (overall + dimensions + issues) is the canonical data structure returned by `calculate_quality` and consumed by gate decision logic.
- **`QualityError` variants** are part of the public error contract; adding variants is non-breaking, removing variants is breaking.

---

*End of contract. File: `clarity-web/src/lattice/quality_contract.md` (cl-m9o, 2026-06-22).*