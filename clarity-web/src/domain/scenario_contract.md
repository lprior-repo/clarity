# Domain Contract — `clarity-web/src/domain/scenario.rs`

| Field | Value |
|---|---|
| **Bead** | `cl-zup` |
| **Module** | `clarity-web/src/domain/scenario.rs` |
| **Author** | `rust-contract` |
| **Date** | 2026-06-21 |
| **Status** | `AUTHORED` |

---

## §1 — Ubiquitous Language

### Core Types

| Term | Type | Definition |
|---|---|---|
| **HoleType** | 3-variant closed enum | Exactly `{DiscoveryHole, EdgeCaseHole, MotivationDropOff}`. No fourth variant exists or can be added without a breaking change. |
| **Hole** | struct | A single identified gap: `{ hole_type: HoleType, description: String, severity: u8 }`. Severity is always in `[1, 5]` after construction. |
| **HolePunchingResults** | struct | The 3-axis validation state: `{ discovery_hole: Option<String>, edge_case_hole: Option<String>, motivation_dropoff: Option<String> }`. |
| **ScenarioField** | struct | The North Star Scenario: `{ trigger: String, value_moment: String, feeling: String, hole_punching: HolePunchingResults }`. |

### Value Objects

| Term | Invariant |
|---|---|
| **Severity** | `u8` constrained to `[1, 5]` inclusive by `Hole::with_severity`. Default is `3`. |
| **Explanation** | `String` that is either empty/whitespace (normalised to `None`) or non-empty trimmed (stored verbatim as `Some(string)`). The `Some` branch carries the string **without any trim or copy modification**. |
| **Bullet** | Each of `trigger`, `value_moment`, `feeling` is a `String`. Completeness requires non-empty after `trim()`. Whitespace-only or empty strings fail `is_bullets_complete`. |

### Predicates

| Predicate | Definition |
|---|---|
| **HolePunchingResults::is_addressed(ht)** | `Some(s) ∧ !s.trim().is_empty()` — the field for `ht` is `Some` containing at least one non-whitespace character. |
| **HolePunchingResults::is_complete()** | `is_addressed(DiscoveryHole) ∧ is_addressed(EdgeCaseHole) ∧ is_addressed(MotivationDropOff)` |
| **ScenarioField::is_bullets_complete()** | `!trigger.trim().is_empty() ∧ !value_moment.trim().is_empty() ∧ !feeling.trim().is_empty()` |
| **ScenarioField::is_complete()** | `is_bullets_complete() ∧ hole_punching.is_complete()` |
| **Hole::is_high_severity()** | `severity >= 4` (inclusive on 4 and 5) |

---

## §2 — Typestates

### Two-Axis State Machine

`ScenarioField` is governed by two independent boolean axes:

**Axis A — Bullets** (3 fields, each independently empty or non-empty):

```
EmptyBullets  ⇔  any bullet is empty after trim
BulletsComplete  ⇔  all 3 bullets are non-empty after trim
```

**Axis B — Hole Punching** (3 independent addressed/unaddressed flags):

```
HolesEmpty       ⇔  all 3 fields are None
HolesPartial(n)  ⇔  addressed_count() == n ∈ {1, 2}
HolesComplete    ⇔  addressed_count() == 3 ∧ ∀ ht, is_addressed(ht)
```

### Terminal State

The **only terminal state** of the composed state machine is:

```
Complete  ≡  BulletsComplete  ∧  HolesComplete
```

There is **no rejected / failed state**. Predicates return `false` and the structure remains in a partial state. No `Result<ScenarioField, Error>` is ever constructed.

### Transitions

All transitions are functional (no in-place mutation):

| Trigger | From | To | Guard |
|---|---|---|---|
| `ScenarioField::new(t, v, f)` with 3 non-empty-trimmed strings | EmptyBullets or partial | BulletsComplete | Each bullet `!s.trim().is_empty()` |
| `HolePunchingResults::address(ht, expl)` with non-empty-trimmed `expl` | HolesEmpty or HolesPartial(n) | HolesPartial(n+1) or HolesComplete | `normalize_explanation(expl) == Some(_)` |
| `HolePunchingResults::address(ht, expl)` with whitespace/empty `expl` | any | unchanged | `normalize_explanation` → `None`; field write is a no-op |

---

## §3 — Error Taxonomy

This module contains **no error types** (`Result<T, E>` does not appear). The module is intentionally partial in the following sense:

### Absent-Value Variants (not errors)

| Variant | Representation | Meaning |
|---|---|---|
| Hole unaddressed | `Option<String>::None` | The hole has not been addressed yet |
| Bullet empty | `String` (zero-length or whitespace-only) | The bullet has not been filled |
| Scenario incomplete | `is_complete() == false` | Either bullets are incomplete or holes are not all addressed |

### Normalised-Absent Convention

- Empty strings `""`, whitespace-only strings `"   "`, and tab/newline-only strings `"\t\n"` are all **normalised to `None`** by `normalize_explanation`.
- They are treated identically to the absent state.
- This normalisation is applied at `address()` entry and at `from_strings()` entry.

### Validity Boundaries

| Field | Valid range | Invalid |
|---|---|---|
| `Hole::severity` (u8) | `[1, 5]` after `with_severity` | Any `u8` value is accepted by the struct; `with_severity` clamps it |
| `HolePunchingResults::addressed_count` | `0..=3` | N/A — always in range by construction |
| `ScenarioField` completeness | `bool` | N/A |

---

## §4 — Workflows

### Workflow: Hole Punching (per-axis)

```
1. Start with HolePunchingResults::new()  → all 3 fields are None
2. For each hole type ht, call .address(ht, explanation_string)
   - If explanation is empty/whitespace → field stays/writes None (no-op)
   - If explanation has non-whitespace content → field becomes Some(explanation) verbatim
3. is_complete() becomes true when all 3 axes have non-empty explanations
4. address() is RIGHT-MOST-WINS per axis: later calls overwrite earlier ones
5. address() is IDEMPOTENT per axis: calling with the same explanation twice is identical to once
```

### Workflow: Scenario Completion

```
1. Client constructs ScenarioField::new(trigger, value_moment, feeling)
   - All 3 bullet strings stored verbatim (no trim on storage)
   - hole_punching starts as HolePunchingResults::new() (all None)
2. Client calls hole_punching.address(ht, explanation) for each hole type
3. is_bullets_complete() checks !trim().is_empty() on all 3 bullets
4. is_complete() requires both axes to be complete
5. No commit/reject transition — the client decides when to treat the scenario as ready
```

### Algebraic Laws (L1–L9)

| Law | Expression | Significance |
|---|---|---|
| **L1** | `r.address(ht, e1).address(ht, e2) == r.address(ht, e2)` | Right-most-wins per axis |
| **L2** | `r.address(ht, e).address(ht, e) == r.address(ht, e)` | Idempotent per axis |
| **L3** | `addressed_count()` is monotonic non-decreasing | Count only goes up with successful addresses |
| **L4** | `unaddressed_holes()` is idempotent under repeated call | Pure observer |
| **L5** | `is_complete() == is_bullets_complete() ∧ hole_punching.is_complete()` | Terminal state definition |
| **L6** | `is_complete() == is_bullets_complete() ∧ addressed_count() == 3` | Terminal state equivalence |
| **L7** | `is_bullets_complete() == ¬is_trigger_empty ∧ ¬is_value_moment_empty ∧ ¬is_feeling_empty` | Per-bullet emptiness |
| **L8** | `HoleType::all()` is a canonical 3-element slice | Closed enumeration |
| **L9** | `with_severity(_, _, s).severity == s.clamp(1, 5)` | Bounded arithmetic |

---

## §5 — Hazards

### H1 — Right-Most-Wins Semantics on `address`

**Hazard:** Calling `.address(ht, "first").address(ht, "second")` silently discards the first explanation. If a client calls `address` twice on the same hole type (e.g. due to a UI multi-submit), the first explanation is lost.

**Mitigation:** Clients that need to preserve history must wrap `HolePunchingResults` in an owning accumulator type that collects all explanations rather than overwriting. The current module does not provide this.

### H2 — Whitespace-Only Strings Are Silent Absent Values

**Hazard:** `"   "` passed to `address()` is silently normalised to `None`. A client that intended to write a placeholder space `" "` will find the field is still `None`.

**Mitigation:** None in-module. Clients should pre-validate inputs or use a wrapper type that rejects whitespace-only strings before calling `address()`.

### H3 — Bullet Fields Stored Verbatim, Not Normalised

**Hazard:** `ScenarioField::new("  trigger  ", "  value  ", "  feeling  ")` stores strings with leading/trailing whitespace. `is_bullets_complete()` trims them for the check, but the original whitespace is preserved in the struct. A subsequent serialisation round-trip will preserve the whitespace.

**Mitigation:** None in-module. Clients that require normalised bullets must trim before calling `new()`. The `is_bullets_complete()` predicate uses `trim()` so completion is not affected by stored whitespace.

### H4 — Severity Clamp Is Silent

**Hazard:** `with_severity(ht, desc, 0)` returns a `Hole` with `severity == 1` — no error, no warning. The input `0` is silently adjusted.

**Mitigation:** Callers should validate severity before calling `with_severity` if they need to distinguish out-of-range inputs.

### H5 — Serialisation Round-Trip Preserves Raw `u8` for Severity

**Hazard:** `Hole` serialises with the raw `u8` severity value. A `Hole` with `severity = 0` (if constructed via struct literal bypassing `with_severity`) round-trips as `severity = 0`. The `with_severity` clamp is not applied on deserialisation.

**Mitigation:** Use `with_severity` when constructing `Hole` values; avoid direct struct literals with out-of-range severity.

---

## §6 — Proof Seeds

Proof seeds are the key contract clauses that map to the 33 planned obligations (25 Verus + 8 proptest). Each seed carries its requirement ID and the open question decision that gates it.

### Proof Seeds: HoleType

```
PS-SC-01: HoleType ≡ closed 3-variant enum { DiscoveryHole, EdgeCaseHole, MotivationDropOff }
           → REQ-SC-1 → PO-SC-V-01
           → Q-DECISION: none required

PS-SC-02: HoleType::all() → slice of length 3, exactly those 3 variants
           → REQ-SC-2 → PO-SC-V-02
           → Q-DECISION: none required

PS-SC-03: ∀ ht ∈ HoleType: label(ht) == specific non-empty static str
           → REQ-SC-3 → PO-SC-V-03

PS-SC-04: ∀ ht ∈ HoleType: description(ht) == specific non-empty static str
           → REQ-SC-4 → PO-SC-V-04

PS-SC-05: Display(ht) postcondition: writes exactly label(ht) bytes to formatter
           → REQ-SC-5 → PO-SC-V-05
```

### Proof Seeds: Hole

```
PS-SC-06: Hole::new(ht, d) → { hole_type: ht, description: d, severity: 3 }
           → REQ-SC-6 → PO-SC-V-06

PS-SC-07: Hole::with_severity(ht, d, s) → severity == s.clamp(1, 5) ∈ [1, 5]
           → REQ-SC-7 → PO-SC-V-07
           → Q-DECISION: Q3 (ratified as [1, 5] inclusive)

PS-SC-08: Hole::is_high_severity() == (severity >= 4)
           → REQ-SC-8 → PO-SC-V-08
           → Q-DECISION: Q4 (ratified as >= 4)
```

### Proof Seeds: HolePunchingResults

```
PS-SC-09: HolePunchingResults::default() → all 3 fields None
           → REQ-SC-9 → PO-SC-V-09

PS-SC-10: HolePunchingResults::new() == empty() == default()
           → REQ-SC-10 → PO-SC-V-10

PS-SC-11: normalize_explanation(s) == None ⇔ s.trim().is_empty()
                               == Some(s) ⇔ !s.trim().is_empty()
           → REQ-SC-11 → PO-SC-V-11
           → Q-DECISION: Q1 (ratified — whitespace-only → None; verbatim on Some)

PS-SC-12: is_addressed(ht) == self.<field>.as_ref().is_some_and(|s| !s.trim().is_empty())
           → REQ-SC-12 → PO-SC-V-12
           → Q-DECISION: Q1 (ratified — non-empty trimmed required)

PS-SC-13: is_complete() == is_addressed(DiscoveryHole) ∧ is_addressed(EdgeCaseHole) ∧ is_addressed(MotivationDropOff)
           → REQ-SC-13 → PO-SC-V-13
           → Q-DECISION: Q1 (inherited from PS-SC-12)

PS-SC-14: address(self, ht, e) → result.<field-for-ht> == normalize_explanation(e); other 2 fields unchanged
           → REQ-SC-14 → PO-SC-V-14
           → Q-DECISION: Q1 (ratified — normalize_explanation semantics apply)

PS-SC-15: r.address(ht, e1).address(ht, e2) == r.address(ht, e2)  [L1: right-most-wins]
           → REQ-SC-15 → PO-SC-V-15
           → Q-DECISION: Q1 (ratified — normalize semantics unchanged)

PS-SC-16: r.address(ht, e).address(ht, e) == r.address(ht, e)  [L2: idempotent]
           → REQ-SC-16 → PO-SC-V-16
           → Q-DECISION: Q1 (ratified — normalize semantics unchanged)

PS-SC-17: explanation(ht) == self.<field>.as_deref()
           → REQ-SC-17 → PO-SC-V-17

PS-SC-18: unaddressed_holes() == HoleType::all().filter(|&&ht| !is_addressed(ht)).collect(); len ∈ 0..=3
           → REQ-SC-18 → PO-SC-V-18

PS-SC-19: addressed_count() == count of ht where is_addressed(ht); result ∈ 0..=3
           → REQ-SC-19 → PO-SC-V-19

PS-SC-20: from_strings(d, e, m) == { discovery_hole: normalize(d), edge_case_hole: normalize(e), motivation_dropoff: normalize(m) }
           → REQ-SC-20 → PO-SC-V-20
           → Q-DECISION: Q1 (ratified — normalize_explanation applies)
```

### Proof Seeds: ScenarioField

```
PS-SC-21: ScenarioField::new(t, v, f) → { trigger: t, value_moment: v, feeling: f, hole_punching: default() }
           bullets stored VERBATIM (no trim applied on storage)
           → REQ-SC-21 → PO-SC-V-21
           → Q-DECISION: Q2 (ratified — verbatim storage, trim only used for completeness predicate)

PS-SC-22: is_bullets_complete() == !trigger.trim().is_empty() ∧ !value_moment.trim().is_empty() ∧ !feeling.trim().is_empty()
           → REQ-SC-22 → PO-SC-V-22
           → Q-DECISION: Q2 (ratified — whitespace-only or empty strings fail completeness)

PS-SC-23: is_trigger_empty() == trigger.trim().is_empty() (and symmetric for value_moment, feeling)
           → REQ-SC-23 → PO-SC-V-23a, V-23b, V-23c
           → Q-DECISION: Q2 (ratified — trim-based emptiness check)

PS-SC-24: is_complete() == is_bullets_complete() ∧ hole_punching.is_complete()
           → REQ-SC-24 → PO-SC-V-24

PS-SC-25: ScenarioField::empty() == default()
           → REQ-SC-25 → PO-SC-V-25
```

### Proof Seeds: Serde Round-Trip

```
PS-SC-26: ∀ T ∈ {HoleType, Hole, HolePunchingResults, ScenarioField}:
          serde_json::from_str::<T>(&serde_json::to_string(&t).unwrap_or_default()) == Ok(t)
           → REQ-SC-26 → PO-SC-P-01, P-02, P-03, P-04, P-05
```

### Proof Seeds: Algebraic Property Tests

```
PS-SC-P-06: address right-most-wins (L1) as proptest property
           → PO-SC-P-06

PS-SC-P-07: with_severity clamp boundaries (s ∈ {0, 1, 5, 6, 254, 255})
           → PO-SC-P-07
           → Q-DECISION: Q3 (ratified [1, 5] inclusive)

PS-SC-P-08: is_complete() == (addressed_count() == 3) as proptest property (Law L6)
           → PO-SC-P-08
```

---

## §7 — Open Question Decisions

| Q | Question | Decision | Rationale |
|---|---|---|---|
| **Q1** (BLOCKING) | What does "addressed" mean? `Some(s) ∧ !s.trim().is_empty()` or `Some(s)` (accepting whitespace)? | **`Some(s) ∧ !s.trim().is_empty()`** — whitespace-only strings are NOT addressed; they normalise to `None`. | The `normalize_explanation` semantics at lines 237-243 and `is_addressed` implementation at lines 176-190 are consistent and intentional. A whitespace-only explanation does not constitute a meaningful response to a hole. This decision drives 7 obligations (PO-SC-V-11, V-12, V-13, V-14, V-15, V-16, PO-SC-P-03). |
| **Q2** (BLOCKING) | What is the whitespace semantics for bullet fields? Is `"  text  "` accepted verbatim? | **Accepted verbatim for storage; `trim()` used for completeness predicate only.** `ScenarioField::new` stores bullets without modification. `is_bullets_complete()` calls `trim()` on the stored values. This means whitespace-padded strings pass completeness but preserve the whitespace on serialisation. | The asymmetry (verbatim storage, trimmed predicate) is consistent with `HolePunchingResults` where `normalize_explanation` discards whitespace at the *entry* point, but `ScenarioField` has no such entry normalisation. This is a design choice. The plan calls it "deliberate-or-accidental" — we ratify it as deliberate with explicit documentation. |
| **Q3** | Is severity clamp range `[1, 5]` inclusive or exclusive? | **`[1, 5]` inclusive** — `s.clamp(1, 5)` maps `s ∈ {0, 1, 5, 6, 255}` to `{1, 1, 5, 5, 5}`. | The `with_severity` doc-comment says "Panics: This function does not panic." The clamp is the intended behaviour. No change to proof shape. |
| **Q4** | Is `is_high_severity` threshold `>= 4` or `> 4`? | **`>= 4`** — both severity 4 and severity 5 are high severity. | The source at line 113 uses `>= 4`. Changing to `> 4` would demote severity-4 holes from high severity, which changes the predicate's meaning. Ratified as-is. |
| **Q5** | May `address` change from `fn address(mut self, ...)` to `&mut self` or `&self → Self`? | **`fn address(mut self, ...) → Self` is the canonical signature.** The consumed-self builder form is ratified. | L1 (right-most-wins) and L2 (idempotent) hold under all three signatures. The `mut self` form makes the ownership explicit and prevents accidental reuse of the consumed value. No obligation needs to change shape. |

---

*End of contract.*
