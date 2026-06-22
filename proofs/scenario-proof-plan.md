# Proof Plan — `clarity-web/src/domain/scenario.rs`

| Field | Value |
|---|---|
| **Bead** | `cl-zup` (Phase 2 — domain expansion per `verification-targets.md §7`) |
| **Target** | `clarity-web/src/domain/scenario.rs` |
| **Primary lane** | **V** (Verus) — per `verification-targets.md §5.1` ("Scenario state machine. Verus for transitions") |
| **Secondary lane** | **P** (proptest) — per `verification-targets.md §5.1` ("proptest for round-trip on serialization") |
| **Tertiary lane (lightweight)** | **F** (Flux) — explicit fallback per `verification-targets.md §5.1` row, only if Verus proves too heavy for the refinement predicates |
| **Contract status** | **GAP — no `rust-contract` artifact exists for this module.** Clauses below are *inferred from source* and labelled `INFERRED`. The plan is gated on `rust-contract` ratifying (or correcting) these clauses before `proof-writer` runs. |
| **Module LOC** | ~634 (production ~325, `#[cfg(test)] mod tests` ~308) — the prompt says ~750; line-count drift is the test block |
| **Clippy hotspot** | **None.** The `#[cfg(test)] mod tests` block (lines 326–340) opens `clippy::unwrap_used`, `clippy::expect_used`, `clippy::panic`, etc. so it does not contribute to the `cl-2q6` baseline (verified against `formal-verification-report.md §4`). |
| **Planner** | `proof-planner` |
| **Date** | 2026-06-21 |

---

## 1. Module characterisation

`scenario.rs` is the **North Star Scenario state machine**: it captures the canonical 3-bullet user-journey field plus a 3-axis "hole punching" validation state. Pure data, zero I/O, zero concurrency, zero `unsafe` (`#![forbid(unsafe_code)]` at module top, line 6; workspace-level `Cargo.toml:10`; parent module `clarity-web/src/domain/mod.rs:12`). Zero arithmetic except `u8::clamp(1, 5)` and `severity >= 4`.

| Item | Lines | Kind | Risk surface |
|---|---|---|---|
| `enum HoleType { DiscoveryHole, EdgeCaseHole, MotivationDropOff }` | 21–32 | 3-variant enum; derives `Clone + Copy + Debug + PartialEq + Eq + Hash + Serialize + Deserialize` | Closed-enumeration invariant; serde naming |
| `HoleType::all()` | 37–43 | `const fn -> &'static [Self]` of length 3 | Slice length and exhaustiveness |
| `HoleType::label(self)` | 47–53 | `const fn -> &'static str` over the 3 variants | Total map, no panic |
| `HoleType::description(self)` | 57–63 | `const fn -> &'static str` over the 3 variants | Total map, no panic |
| `impl Display for HoleType` | 66–71 | Writes `label()` to formatter | Postcondition = label |
| `struct Hole { hole_type, description, severity }` | 77–85 | Owned; `severity: u8` (unconstrained at type level) | Constructor invariant (`new` → severity=3; `with_severity` → clamp(1,5)) |
| `Hole::new(ht, desc) -> Self` | 89–96 | `const fn`; severity = 3 | Default-severity contract |
| `Hole::with_severity(ht, desc, severity) -> Self` | 102–109 | `severity.clamp(1, 5)` | **Bounded-arithmetic invariant** |
| `Hole::is_high_severity(&self) -> bool` | 112–115 | `const fn`; predicate `severity >= 4` | Threshold semantics |
| `struct HolePunchingResults { discovery_hole, edge_case_hole, motivation_dropoff }` | 128–139 | 3 × `Option<String>`; derives `Default + Clone + Debug + PartialEq + Eq + Serialize + Deserialize` | None at struct level |
| `HolePunchingResults::new()` / `empty()` | 143–152 | Aliases for `default()` | Total |
| `HolePunchingResults::is_complete(&self) -> bool` | 158–172 | All 3 fields `Some(s)` with `!s.trim().is_empty()` | **State machine terminal predicate** |
| `HolePunchingResults::is_addressed(ht) -> bool` | 175–191 | Per-hole version of `is_complete`'s conjunct | **State machine per-axis guard** |
| `HolePunchingResults::address(self, ht, expl) -> Self` | 196–205 | Owned consumed-self builder; sets exactly one field via `normalize_explanation` | **Right-most-wins per axis**; **idempotence per axis** |
| `HolePunchingResults::explanation(ht) -> Option<&str>` | 208–215 | Observer | Total |
| `HolePunchingResults::unaddressed_holes() -> Vec<HoleType>` | 218–225 | Complement of addressed set | **Idempotence under repeated call**; `len ∈ 0..=3` |
| `HolePunchingResults::addressed_count() -> usize` | 228–234 | `0..=3` | **Monotonic non-decreasing w.r.t. successful address** |
| `HolePunchingResults::normalize_explanation(s) -> Option<String>` (private) | 237–243 | `None ⇔ s.trim().is_empty()`; else `Some(s)` verbatim | **Normalisation semantics** (zero-copy on non-empty input) |
| `HolePunchingResults::from_strings(d, e, m) -> Self` | 247–253 | Constructor applying `normalize_explanation` to each input | Equivalence to three `address` calls from `default()` |
| `struct ScenarioField { trigger, value_moment, feeling, hole_punching }` | 262–272 | All `String` × 3 + nested; derives `Default + Clone + Debug + PartialEq + Eq + Serialize + Deserialize` | None at struct level |
| `ScenarioField::new(trigger, value_moment, feeling)` | 277–284 | Empty `hole_punching`; bullets verbatim | Total |
| `ScenarioField::empty() -> Self` | 288–290 | Alias for `default()` | Total |
| `ScenarioField::is_complete(&self) -> bool` | 294–296 | `is_bullets_complete() ∧ hole_punching.is_complete()` | **State machine terminal predicate** |
| `ScenarioField::is_bullets_complete(&self) -> bool` | 300–304 | All 3 bullets non-empty trimmed | **State machine intermediate guard** |
| `ScenarioField::is_trigger_empty(&self) / is_value_moment_empty(&self) / is_feeling_empty(&self)` | 308–322 | Per-bullet emptiness predicates | Total |
| `#[cfg(test)] mod tests` | 325–633 | 18 hand-written tests | Test-only; not a proof target. Clippy gates opened in module lints (line 326–340). |

### 1.1 State machine analysis (FOCUS of this plan)

The module exposes a **two-axis state machine** for `ScenarioField`. The axes are independent:

- **Axis A — Bullets** (3 field predicates `is_trigger_empty`, `is_value_moment_empty`, `is_feeling_empty`):
  - `EmptyBullets` ⇔ any one of the 3 bullets is empty trimmed.
  - `BulletsComplete` ⇔ all 3 bullets non-empty trimmed (`is_bullets_complete()`).
- **Axis B — Hole Punching** (3 per-axis predicates `is_addressed(ht)` for each `HoleType`):
  - `HolesEmpty` ⇔ all 3 fields are `None` after `normalize_explanation`.
  - `HolesPartial(n)` ⇔ `addressed_count() == n ∈ 1..=2`.
  - `HolesComplete` ⇔ `addressed_count() == 3 ∧ ∀ ht, is_addressed(ht)` (`is_complete()`).

The composed `ScenarioField` state is the cartesian product; the only **terminal state** is `Complete ≡ BulletsComplete × HolesComplete`. There is **no rejected/failed state** — predicates return `false` and the structure stays in a partial state, which is a deliberate design choice (no `Result<ScenarioField, _>` ever).

**Transitions** (functional only — no in-place mutator on `ScenarioField`):

| Trigger | From | To | Guard |
|---|---|---|---|
| `ScenarioField::new(t, v, f)` (or struct-literal) with all 3 non-empty trimmed | EmptyBullets or partial | `BulletsComplete` (axis A flips) | bullet predicates per field |
| `HolePunchingResults::address(ht, expl)` with non-empty trimmed `expl` | HolesEmpty or `HolesPartial(n)` | `HolesPartial(n+1)` or `HolesComplete` | `normalize_explanation(expl) == Some(_)` |
| `HolePunchingResults::address(ht, expl)` with empty/whitespace `expl` | any | unchanged | `normalize_explanation` collapses to `None` ⇒ no-op field-write |

**Algebraic / monoid properties observed in source** (the FOCUS list):

- **L1 — `HolePunchingResults::address` is right-most-wins per axis.** For any `ht` and any two explanations `e1, e2`, `r.address(ht, e1).address(ht, e2) == r.address(ht, e2)` on the field corresponding to `ht`. The other two fields are byte-equal between the two expressions (line 199–203: only one field is touched per call).
- **L2 — `HolePunchingResults::address` is idempotent per axis.** `r.address(ht, e).address(ht, e) == r.address(ht, e)` for any `ht, e` (consequence of L1 with `e1 == e2`).
- **L3 — `addressed_count ∈ 0..=3` and is monotonic non-decreasing w.r.t. successful `address` calls** (where "successful" means the input normalises to `Some(_)`). No sequence of `address` calls can reduce `addressed_count`. This is the list-shaped (not multiset, not set) semantics of `Vec<HoleType>::filter` over `is_addressed`.
- **L4 — `unaddressed_holes()` is idempotent under repeated call.** Two calls without intervening mutation produce equal `Vec<HoleType>`s (pure observer; lines 219–225 are pure).
- **L5 — `is_complete ⇔ is_bullets_complete ∧ hole_punching.is_complete`** (line 294–296). This is the conjunction law; `is_complete` is *defined* as the conjunction, so it is trivially a stable predicate.
- **L6 — `is_complete ⇔ addressed_count() == 3 ∧ is_bullets_complete()`.** Consequence of `addressed_count == 3 ⇔ ∀ ht, is_addressed(ht) ⇔ hole_punching.is_complete()` (lines 229–234 + 159–172).
- **L7 — `ScenarioField::is_bullets_complete ⇔ ¬is_trigger_empty ∧ ¬is_value_moment_empty ∧ ¬is_feeling_empty`** (lines 300–304 vs 308–322). Direct logical equivalence.
- **L8 — `HoleType::all()` is the canonical enumerated list.** `all().len() == 3` (line 38–42); for every `ht ∈ HoleType`, `all().contains(&ht)` is `true`; for any `ht ∉ HoleType` (vacuously, the enum is closed), `all().contains(&ht)` is `false`.
- **L9 — `Hole::with_severity` is a clamp to `[1, 5]`.** `with_severity(_, _, s).severity == s.clamp(1, 5)` (line 107). Boundary: `s == 0 → 1`; `s == 5 → 5`; `s == 6 → 5`; `s == 255 → 5`.

**Involution / involutory properties**: none observed. `address` and `with_severity` are *not* involutions; they overwrite and have no inverse.

**Serde round-trip**: every type derives `Serialize + Deserialize` (lines 21, 77, 128, 262); the `serde_json` round-trip must be an identity on well-formed values. This is a **behavioural property** and is covered by `P` (proptest) per `verification-targets.md §5.1`.

---

## 2. Contract gap (honest disclosure)

The bead `cl-zup` has **no upstream `rust-contract` artifact** under `clarity-web/src/domain/contract.md` (or any path matching `**/contract.md` in the workspace — verified by `find`). All clauses used in this plan are *inferred by direct reading of the source*, not authored.

**Action required before `proof-writer` runs:** `rust-contract` must ratify or correct the clauses marked `INFERRED` in §3. If the inferred clauses are wrong, `proof-writer` will write specs against the wrong contract. The obligations JSONL sets `requires_contract: true` on every inferred-clause row so this gate is visible.

`rust-contract` should produce (at minimum):

- `contract.md` for `clarity-web/src/domain/scenario.rs` (or `clarity-web/src/domain/`) with clauses keyed to the requirement IDs in §3.
- Explicit decision on **what "addressed" means** for `HolePunchingResults`. Current source: `Some(s) ∧ !s.trim().is_empty()` (lines 161–171). Alternatives: any `Some(_)` (including whitespace-only); only `Some(s) ∧ s != ""` (no trim). This drives REQ-SC-11, REQ-SC-12, REQ-SC-14, REQ-SC-15, REQ-SC-16.
- Explicit decision on **whether `ScenarioField`'s bullet fields are part of the contract as "non-empty" or "any string including whitespace"**. Current source: `!s.trim().is_empty()` (lines 301–303). Drives REQ-SC-21, REQ-SC-22.
- Explicit decision on **`Hole::severity` clamp range**: current source `[1, 5]` (line 107). Drives REQ-SC-7.
- Explicit decision on **`Hole::with_severity` returning `Self` vs `Result<Self, _>`**: current source is `Self` (line 102). Drives whether REQ-SC-7 needs a refinement on the clamped value alone.
- Explicit decision on **`Hole::is_high_severity` threshold**: current source `>= 4` (line 113). Drives REQ-SC-8.
- Explicit decision on **`HolePunchingResults::address` shape**: current source is `fn address(mut self, ht, expl) -> Self` (line 197) — owned consumed-self builder. Drives REQ-SC-14 and the L1/L2 algebraic laws.

---

## 3. Requirements & inferred contract clauses

Each row carries a `clause_origin` of either `INFERRED` (this plan) or `AUTHORED` (rust-contract — currently none). Algebraic-law rows are tagged with their law identifier (L1–L9) for traceability to §1.1.

| Req ID | Source | Inferred clause | Law | Origin |
|---|---|---|---|---|
| REQ-SC-1 | `HoleType` enum + `#[derive(Hash + Eq + PartialEq + Copy + Clone)]` (lines 21–32) | `HoleType` is exactly the 3-variant closed enum `{DiscoveryHole, EdgeCaseHole, MotivationDropOff}`. No variant is hidden. | — | INFERRED |
| REQ-SC-2 | `HoleType::all()` (lines 37–43) | `all().len() == 3` ∧ `all() == &[DiscoveryHole, EdgeCaseHole, MotivationDropOff]` ∧ `∀ ht ∈ HoleType, all().contains(&ht)` | L8 | INFERRED |
| REQ-SC-3 | `HoleType::label(self)` (lines 47–53) | Total map: `label(DiscoveryHole) == "Discovery Hole"`, `label(EdgeCaseHole) == "Edge Case Hole"`, `label(MotivationDropOff) == "Motivation Drop-off"`. Output is non-empty for each variant. | — | INFERRED |
| REQ-SC-4 | `HoleType::description(self)` (lines 57–63) | Total map: non-empty output for each of the 3 variants. | — | INFERRED |
| REQ-SC-5 | `impl Display for HoleType` (lines 66–71) | `format!("{}", ht) == ht.label()` for each variant. | — | INFERRED |
| REQ-SC-6 | `Hole::new` (lines 89–96) | `new(ht, d).hole_type == ht ∧ new(ht, d).description == d ∧ new(ht, d).severity == 3`. | — | INFERRED |
| REQ-SC-7 | `Hole::with_severity` (lines 102–109) | `with_severity(ht, d, s).severity == s.clamp(1, 5)`. Boundary cases: `s == 0 → 1`; `s == 5 → 5`; `s == 6 → 5`; `s == 255 → 5`. Other fields are byte-equal to `(ht, d)` after construction. | L9 | INFERRED |
| REQ-SC-8 | `Hole::is_high_severity` (lines 112–115) | `is_high_severity() ⇔ self.severity >= 4`. Threshold inclusive on the upper end. | — | INFERRED |
| REQ-SC-9 | `HolePunchingResults::default` (line 128, derive) | `default() == HolePunchingResults { discovery_hole: None, edge_case_hole: None, motivation_dropoff: None }`. | — | INFERRED |
| REQ-SC-10 | `HolePunchingResults::new` / `empty` (lines 143–152) | Both are equivalent to `default()` (L_total). | — | INFERRED |
| REQ-SC-11 | `HolePunchingResults::normalize_explanation` (lines 237–243, private) | `normalize_explanation(s) == None ⇔ s.trim().is_empty()`; `normalize_explanation(s) == Some(s) ⇔ !s.trim().is_empty()`. The `Some(_)` branch is verbatim (no trim, no copy beyond the move). | — | INFERRED |
| REQ-SC-12 | `HolePunchingResults::is_addressed(ht)` (lines 175–191) | `is_addressed(DiscoveryHole) ⇔ self.discovery_hole.as_ref().is_some_and(|s| !s.trim().is_empty())`; symmetric for the other two variants using `edge_case_hole` and `motivation_dropoff` respectively. | — | INFERRED |
| REQ-SC-13 | `HolePunchingResults::is_complete` (lines 158–172) | `is_complete() ⇔ is_addressed(DiscoveryHole) ∧ is_addressed(EdgeCaseHole) ∧ is_addressed(MotivationDropOff)`. | L5 | INFERRED |
| REQ-SC-14 | `HolePunchingResults::address(self, ht, expl)` (lines 196–205) | `r.address(ht, e)` returns `r` with exactly one field updated: the field corresponding to `ht` is set to `normalize_explanation(e)`; the other two fields are byte-equal to those of `r`. | L1 | INFERRED |
| REQ-SC-15 | `HolePunchingResults::address` is right-most-wins per axis | `r.address(ht, e1).address(ht, e2) == r.address(ht, e2)` for any `ht, e1, e2`. (Consequence of REQ-SC-14 applied twice.) | L1 | INFERRED |
| REQ-SC-16 | `HolePunchingResults::address` is idempotent per axis | `r.address(ht, e).address(ht, e) == r.address(ht, e)`. (Consequence of REQ-SC-15 with `e1 == e2`.) | L2 | INFERRED |
| REQ-SC-17 | `HolePunchingResults::explanation(ht)` (lines 208–215) | Observer: `explanation(ht) == self.<field for ht>.as_deref()`. The non-`None` branch returns the inner `&str` verbatim. | — | INFERRED |
| REQ-SC-18 | `HolePunchingResults::unaddressed_holes()` (lines 218–225) | `unaddressed_holes() == HoleType::all().iter().filter(|&&ht| !self.is_addressed(ht)).copied().collect()`. Output length in `0..=3`. Idempotent under repeated call. | L4 | INFERRED |
| REQ-SC-19 | `HolePunchingResults::addressed_count()` (lines 228–234) | `addressed_count() == HoleType::all().iter().filter(|&&ht| self.is_addressed(ht)).count()` ∧ `0 <= addressed_count() <= 3`. | L3 | INFERRED |
| REQ-SC-20 | `HolePunchingResults::from_strings(d, e, m)` (lines 247–253) | `from_strings(d, e, m) == HolePunchingResults { discovery_hole: normalize_explanation(d), edge_case_hole: normalize_explanation(e), motivation_dropoff: normalize_explanation(m) }`. Equivalence to three `address` calls from `default()`. | — | INFERRED |
| REQ-SC-21 | `ScenarioField::new(t, v, f)` (lines 277–284) | `new(t, v, f).trigger == t ∧ new(t, v, f).value_moment == v ∧ new(t, v, f).feeling == f ∧ new(t, v, f).hole_punching == HolePunchingResults::default()`. Bullet fields stored verbatim (no trim). | — | INFERRED |
| REQ-SC-22 | `ScenarioField::is_bullets_complete(&self)` (lines 300–304) | `is_bullets_complete() ⇔ !self.trigger.trim().is_empty() ∧ !self.value_moment.trim().is_empty() ∧ !self.feeling.trim().is_empty()`. | L7 | INFERRED |
| REQ-SC-23 | `ScenarioField::is_trigger_empty(&self)` etc. (lines 308–322) | `is_trigger_empty() ⇔ self.trigger.trim().is_empty()`; symmetric for the other two. | L7 | INFERRED |
| REQ-SC-24 | `ScenarioField::is_complete(&self)` (lines 294–296) | `is_complete() ⇔ is_bullets_complete() ∧ self.hole_punching.is_complete()`. Equivalent to `is_bullets_complete() ∧ addressed_count() == 3` per L6. | L5, L6 | INFERRED |
| REQ-SC-25 | `ScenarioField::empty()` (lines 288–290) | `empty() == default()`. | — | INFERRED |
| REQ-SC-26 | `#[derive(Serialize + Deserialize)]` on all 4 types (lines 21, 77, 128, 262) | For each `T ∈ {HoleType, Hole, HolePunchingResults, ScenarioField}` and any well-formed `t: T`, `serde_json::from_str::<T>(&serde_json::to_string(&t).unwrap_or_default()) == Ok(t)`. | — | INFERRED |

**Open questions for `rust-contract`** (need answers before proof-writer starts):

1. Is "non-empty trimmed" (current source) the full definition of "addressed" for `HolePunchingResults`, or should `is_addressed` collapse on any `Some(_)` (including whitespace-only)? Affects REQ-SC-11, REQ-SC-12, REQ-SC-14 (the `address` semantics on whitespace inputs).
2. Is `ScenarioField`'s bullet-completeness predicate based on `!trim().is_empty()` (current) or `!is_empty()` (without trim)? Affects REQ-SC-21, REQ-SC-22.
3. Is the `Hole::severity` clamp range `[1, 5]` fixed by the contract, or implementation-detail? Affects whether REQ-SC-7 specifies `1..=5` as literals or as a named bound.
4. Is `Hole::is_high_severity`'s threshold `>= 4` part of the contract, or `> 4` (strict, current source is inclusive on 4)? Affects REQ-SC-8.
5. Is `HolePunchingResults::address(self, …)` consuming `self` (current source) the public signature, or may it change to `&mut self` or to `&self -> Self`? Affects REQ-SC-14 and the L1/L2 algebraic laws (L1/L2 hold under both signatures but the proof shape changes).

---

## 4. Verifier lane decisions

Per `verifier-trigger-matrix.md`, classify the proof seeds across Verus / Kani / Flux / Loom / proptest / fuzz.

| Lane | Decision | Evidence / rationale |
|---|---|---|
| **V** (Verus) | **REQUIRED — primary** | Module is pure data + ~20 pure functions/methods; the state-machine transitions (axis A and B), the algebraic laws (L1–L9), and the constructor/validator predicates are exactly what Verus specs. Per `verification-targets.md §5.1`. Verus is installed (`/home/lewis/.local/bin/verus`, v0.2026.05.05, profile release). |
| **P** (proptest) | **REQUIRED — secondary** | All four types derive `Serialize`/`Deserialize`; serde round-trip on `ScenarioField` and `HolePunchingResults` is the headline property. The algebraic laws (L1–L7) over generated inputs exercise boundary conditions that hand-written unit tests cannot easily reach (e.g. `addressed_count` after many `address` calls; bullet-trimming on Unicode whitespace). proptest is a workspace `dev-dependency` (`clarity-web/Cargo.toml`). |
| **F** (Flux) | **NOT APPLICABLE (with explicit downgrade reasoning)** | `verification-targets.md §5.1` lists **F** as a tertiary lane for `scenario.rs`. Downgrade to `not_applicable`: Verus covers the refinement predicates (non-empty trimmed, clamp `[1, 5]`, `0..=3` range on `addressed_count`) with more rigour at the same author cost. Flux would require tagging `String` or `u8` with refinement indices (`{ s: String | !s.trim().is_empty() }`), which does not match the `Option<String>` representation in `HolePunchingResults` or the unconstrained `String` representation in `ScenarioField`'s bullet fields. `cargo-flux` is installed but unused for this module. |
| **K** (Kani) | **NOT APPLICABLE** | Kani's strength (bounded model check of `unsafe`, fixed-width arithmetic, parser bounds) does not apply. The only numeric surface is `u8::clamp(1, 5)` (line 107), which is provable in Verus in one `ensures` clause. No `#[kani::proof]` harness is warranted. Cite: `#![forbid(unsafe_code)]` (line 6), no parser, no fixed-width index space. Kani is **not installed** per `verification-targets.md §3`; even if applicable, install would be required first. |
| **L** (Loom) | **NOT APPLICABLE** | No concurrency in this module — no threads, channels, atomics, `Send + Sync` interactions, async, or spawn calls. The `address(mut self, …)` consumed-self builder cannot race with itself within one thread. |
| **M** (Miri) | **NOT APPLICABLE** | `#![forbid(unsafe_code)]` at module top (line 6) and workspace level (`Cargo.toml:10`). No `unsafe` blocks anywhere in `scenario.rs`. |
| **T** (TLA+) | **NOT APPLICABLE** | While the module exposes a *predicate-driven* state machine (Empty → BulletsComplete → Complete), the transitions are **driven by client code calling constructors and `address`/`new` methods** — there is no in-module temporal workflow, no retries, no leases, no batch ordering. The state machine is **stateless from the module's perspective**: each `ScenarioField` is an immutable value; no internal mutable state, no event log, no scheduler. A TLA+ spec would be vacuous. TLA+ is reserved for `intent/beads`, `intent/batch`, and `storage/fjall_event_store` per `verification-targets.md §5.2–§5.3`. |
| **Z** (fuzz) | **NOT APPLICABLE** | The deserialisation boundary is structured `serde_json` of typed Rust records (REQ-SC-26) — covered by proptest. No hand-written parser, no regex, no codec, no frame decoder. The untrusted-input parser boundary is `intent/parser.rs` (per `verification-targets.md §5.3`: Z+P lane for `intent/parser.rs`). `cargo-fuzz` is **not installed** per `verification-targets.md §3`; even if it were, no fuzz target exists in this module. |
| **X** (exercise-only) | **NOT APPLICABLE** | The whole module is in scope for V + P coverage. The `#[cfg(test)] mod tests` block (lines 325–633) is 18 hand-written tests covering the same properties as §3 — these are referenced by the bridge (§8) but are not themselves proof targets. No glue-code block needs to be relegated to `X`. |

Two infrastructure gaps block adjacent lanes (`K`, `Z`) but those lanes are `not_applicable` to this module regardless, so the gaps are not blockers for *this* plan. They are noted for the landing-skill pre-flight in `verification-targets.md §4`.

---

## 5. Proof coverage matrix

| Req ID | Law | Lane | Obligation ID | Targets | Evidence | Verus mode |
|---|---|---|---|---|---|---|
| REQ-SC-1 | — | V | PO-SC-V-01 | `HoleType` enum (whole) | Verus spec fn `hole_type_to_spec(ht: HoleType) -> nat` with exact 3-element codomain; match is exhaustive. | exec + spec |
| REQ-SC-2 | L8 | V | PO-SC-V-02 | `HoleType::all` | Verus ensures: `result.len() == 3` and `result` equals the explicit slice literal. | exec |
| REQ-SC-3 | — | V | PO-SC-V-03 | `HoleType::label` | Verus ensures via `match`: each variant returns the literal string from §3. | exec + spec |
| REQ-SC-4 | — | V | PO-SC-V-04 | `HoleType::description` | Verus ensures via `match`: each variant returns a non-empty literal. | exec + spec |
| REQ-SC-5 | — | V | PO-SC-V-05 | `impl Display for HoleType` | Verus `#[verifier::external_body]` on the `fmt` body; spec asserts the postcondition: bytes written equal `self.label()` as bytes. | exec + external_body |
| REQ-SC-6 | — | V | PO-SC-V-06 | `Hole::new` | Verus ensures: `result.hole_type == ht ∧ result.description == d ∧ result.severity == 3`. | exec |
| REQ-SC-7 | L9 | V | PO-SC-V-07 | `Hole::with_severity` | Verus ensures: `1 <= result.severity <= 5 ∧ result.severity == s.clamp(1, 5)` and other fields byte-equal to `(ht, d)`. Boundary cases asserted by `ensures` conjuncts. | exec + spec |
| REQ-SC-8 | — | V | PO-SC-V-08 | `Hole::is_high_severity` | Verus ensures: `result == (self.severity >= 4)`. | exec |
| REQ-SC-9 | — | V | PO-SC-V-09 | `HolePunchingResults::default` | Verus ensures: `result.discovery_hole.is_none() ∧ result.edge_case_hole.is_none() ∧ result.motivation_dropoff.is_none()`. | exec |
| REQ-SC-10 | — | V | PO-SC-V-10 | `HolePunchingResults::new` / `empty` | Verus ensures: both return `HolePunchingResults::default()` (extensional equality on 3 fields). | exec |
| REQ-SC-11 | — | V | PO-SC-V-11 | `HolePunchingResults::normalize_explanation` (private) | Verus ensures: `result.is_none() ⇔ s.trim().is_empty()`; `result == Some(s) ⇔ !s.trim().is_empty()`. Spec fn `normalize_spec(s: String) -> Option<String>`. Body remains private; spec asserted on the visible `from_strings` (REQ-SC-20) call site. | exec + spec (via from_strings) |
| REQ-SC-12 | — | V | PO-SC-V-12 | `HolePunchingResults::is_addressed` | Verus ensures via `match`: each variant returns the corresponding field's `Some(_).with_non_empty_trim` predicate. | exec + spec |
| REQ-SC-13 | L5 | V | PO-SC-V-13 | `HolePunchingResults::is_complete` | Verus ensures: `result == (is_addressed(DiscoveryHole) ∧ is_addressed(EdgeCaseHole) ∧ is_addressed(MotivationDropOff))`. | exec |
| REQ-SC-14 | L1 | V | PO-SC-V-14 | `HolePunchingResults::address` (single call) | Verus ensures: `result.<field-for-ht> == normalize_explanation(e)` and the other two fields are byte-equal to `self.<same-field>`. | exec + spec |
| REQ-SC-15 | L1 | V | PO-SC-V-15 | `address` right-most-wins per axis | Verus proves for any `r, ht, e1, e2`: `r.address(ht, e1).address(ht, e2) == r.address(ht, e2)` on all 3 fields. | exec + spec |
| REQ-SC-16 | L2 | V | PO-SC-V-16 | `address` idempotent per axis | Verus proves `r.address(ht, e).address(ht, e) == r.address(ht, e)` (consequence of PO-SC-V-15 with `e1 == e2`). | exec + spec |
| REQ-SC-17 | — | V | PO-SC-V-17 | `HolePunchingResults::explanation` | Verus ensures via `match`: each variant returns the corresponding field's `as_deref()`. | exec |
| REQ-SC-18 | L4 | V | PO-SC-V-18 | `HolePunchingResults::unaddressed_holes` | Verus ensures: result length in `0..=3` and `result == HoleType::all().iter().filter(|&&ht| !self.is_addressed(ht)).copied().collect()`. Idempotence: calling twice without mutation produces equal `Vec`s. | exec + spec |
| REQ-SC-19 | L3 | V | PO-SC-V-19 | `HolePunchingResults::addressed_count` | Verus ensures: `0 <= result <= 3 ∧ result == is_addressed(Discovery) as usize + is_addressed(EdgeCase) as usize + is_addressed(Motivation) as usize`. | exec + spec |
| REQ-SC-20 | — | V | PO-SC-V-20 | `HolePunchingResults::from_strings` | Verus ensures: 3-field struct literal with each field equal to `normalize_explanation(input)`. Equivalence to 3 sequential `address` calls from `default()`. | exec + spec |
| REQ-SC-21 | — | V | PO-SC-V-21 | `ScenarioField::new` | Verus ensures: `result.trigger == t ∧ result.value_moment == v ∧ result.feeling == f ∧ result.hole_punching == HolePunchingResults::default()`. Bullets stored verbatim (no trim). | exec |
| REQ-SC-22 | L7 | V | PO-SC-V-22 | `ScenarioField::is_bullets_complete` | Verus ensures: `result == (!trigger.trim().is_empty() ∧ !value_moment.trim().is_empty() ∧ !feeling.trim().is_empty())`. | exec + spec |
| REQ-SC-23 | L7 | V | PO-SC-V-23 | `ScenarioField::is_trigger_empty / is_value_moment_empty / is_feeling_empty` | Verus ensures (one obligation covering all 3): each predicate returns `self.<field>.trim().is_empty()`. | exec |
| REQ-SC-24 | L5, L6 | V | PO-SC-V-24 | `ScenarioField::is_complete` | Verus ensures: `result == (is_bullets_complete() ∧ self.hole_punching.is_complete())`. | exec |
| REQ-SC-25 | — | V | PO-SC-V-25 | `ScenarioField::empty` | Verus ensures: `result == default()`. | exec |
| REQ-SC-26 | — | P | PO-SC-P-01 | `HoleType` serde round-trip | proptest over `HoleType::all()` (exhaustive — only 3 variants): `serde_json::from_str::<HoleType>(&serde_json::to_string(&ht).unwrap_or_default()).unwrap() == ht`. | n/a (Rust test) |
| REQ-SC-26 | — | P | PO-SC-P-02 | `Hole` serde round-trip | proptest: generate `(ht, desc, severity in 0..=255)` and assert round-trip preserves all 3 fields. Note: `with_severity` clamps but `serde` round-trip uses raw `u8`. | n/a |
| REQ-SC-26, REQ-SC-12, REQ-SC-14 | L2, L3 | P | PO-SC-P-03 | `HolePunchingResults` serde round-trip + `address` idempotence + monotonicity | proptest: generate arbitrary `(d, e, m)` `String` triples (incl. Unicode whitespace, BOM, RTL marks); round-trip; assert `address` is idempotent on each axis (`r.address(ht, s).address(ht, s) == r.address(ht, s)`); assert `addressed_count` is monotonic non-decreasing across a randomly-generated sequence of `address` calls. | n/a |
| REQ-SC-26, REQ-SC-22, REQ-SC-24 | L5, L6 | P | PO-SC-P-04 | `ScenarioField` serde round-trip + state-machine law L6 | proptest: generate `(t, v, f, hole_punching)` and round-trip; assert `is_complete ⇔ is_bullets_complete ∧ addressed_count == 3`; assert bullet-trimming semantics on arbitrary Unicode whitespace. | n/a |
| REQ-SC-26 | — | P | PO-SC-P-05 | `Display for HoleType` round-trip | proptest: `format!("{}", ht) == ht.label()` for all 3 variants. | n/a |
| REQ-SC-15 | L1 | P | PO-SC-P-06 | `address` right-most-wins per axis (property test on Rust) | proptest: for randomly-generated `(r, ht, e1, e2)`, assert `r.address(ht, e1).address(ht, e2) == r.address(ht, e2)` on all 3 axes. | n/a |
| REQ-SC-7 | L9 | P | PO-SC-P-07 | `Hole::with_severity` clamp boundaries | proptest: severity in `0..=255`; assert `with_severity(ht, d, s).severity == s.clamp(1, 5)`; focus on `s ∈ {0, 1, 5, 6, 254, 255}`. | n/a |
| REQ-SC-13, REQ-SC-19 | L3, L5 | P | PO-SC-P-08 | `is_complete ⇔ addressed_count == 3` (Law L6) | proptest: generate arbitrary `HolePunchingResults` via a sequence of `address` calls; assert `is_complete() == (addressed_count() == 3)`. | n/a |
| n/a | — | K | PO-SC-K1 | scenario.rs | n/a | not_applicable |
| n/a | — | F | PO-SC-F1 | scenario.rs | n/a | not_applicable (downgrade reasoning per §4) |
| n/a | — | L | PO-SC-L1 | scenario.rs | n/a | not_applicable |
| n/a | — | M | PO-SC-M1 | scenario.rs | n/a | not_applicable |
| n/a | — | T | PO-SC-T1 | scenario.rs | n/a | not_applicable |
| n/a | — | Z | PO-SC-Z1 | scenario.rs | n/a | not_applicable |
| n/a | — | X | PO-SC-X1 | scenario.rs | n/a | not_applicable |

**Unwind bounds:** none of the obligations target loops. `HoleType::all()` is a static slice; `is_addressed`, `is_complete`, `unaddressed_holes`, `addressed_count` all have trivially-bounded iteration (`HoleType::all().iter()` walks a 3-element slice). No `#[verifier::loop_isolation]` or `#[kani::unwind]` needed.

**Verus mode summary:**
- 22 Verus obligations use `exec + spec` mode (the original function body, with `requires`/`ensures`).
- 1 Verus obligation (`PO-SC-V-05`, `Display for HoleType`) uses `exec + #[verifier::external_body]` — the `fmt::Formatter` trait surface is not Verus-friendly to spec inline; the spec asserts the postcondition on a ghost witness and trusts the fmt machinery.
- 8 proptest obligations run as ordinary `#[test]` functions in a `#[cfg(test)] mod tests_proptest` submodule added by `proof-writer` to `clarity-web/src/domain/scenario.rs` (no production body changes).
- 7 `not_applicable` rows cite concrete evidence in §4.

---

## 6. Trusted base plan

These are the assumptions the proofs lean on. Each is either explicitly trusted or has its own obligation. This section is the source for `trusted-base-ledger/v1` rows in the downstream ledger.

| Trust | Why trusted | Mitigation in obligations |
|---|---|---|
| `String::trim()` and `String::trim().is_empty()` are spec-correct in Verus stdlib | Verus distribution ships verified specs for these methods | PO-SC-V-03, V-04, V-07, V-11, V-12, V-13, V-18, V-19, V-20, V-22, V-23, V-24 assume the stdlib spec is correct; if Verus ships a known-bad spec, the entire plan fails and that is a Verus toolchain defect, not our defect. |
| `u8::clamp(1, 5)` returns a value in `[1, 5]` | Rust stdlib contract | PO-SC-V-07 asserts the postcondition. The stdlib spec is trusted. |
| `serde_json::to_string` and `from_str` are total on the well-formed types | serde library contract | PO-SC-P-01…P-04 exercise the round-trip directly. If serde silently corrupts values, the property fails. |
| `Vec::push` is order-preserving and appends (no dedup) | Rust stdlib contract | PO-SC-V-18, V-19 prove `Vec` semantics via the `HoleType::all()` filter pattern; the spec references the `Vec` API directly. |
| `Display::fmt` for `&str` writes the inner bytes | Rust stdlib | PO-SC-V-05 asserts the postcondition on a ghost witness. The trait method body is `#[verifier::external_body]` — the Rust stdlib contract is trusted. |
| `HoleType` is a closed 3-variant enum | Type-system fact at the time the spec is written | PO-SC-V-01, V-02, V-03, V-04, V-05 enumerate the 3 variants explicitly via `match`. Adding a variant would require updating the spec — caught at compile time of the spec file. |
| The 4 struct literals (`Hole`, `HolePunchingResults`, `ScenarioField`, and the 5-field structs) are closed (no future field added without spec update) | Type-system fact at spec-write time | PO-SC-V-06, V-09, V-14, V-21 enumerate all fields explicitly; adding a field is a breaking change that requires spec update — caught at compile time of the spec file. |
| `address(self, ht, e)` is `#[must_use]` (line 196 doc-comment) — but the function is total and side-effect-free on the consumed `self` | Source inspection; no `#[must_use]` attribute is actually present at line 196 (only the doc comment), but the function is observably total | PO-SC-V-14, V-15, V-16 prove total postconditions on the returned `Self`. |
| `HolePunchingResults` has no `Hash`, no `Ord`, no `PartialOrd` derives (line 128) — only `Default + Clone + Debug + PartialEq + Eq + Serialize + Deserialize` | Source inspection at line 128 | PO-SC-V-09…V-20 do not depend on `Hash` or `Ord`. If `rust-contract` adds them, the spec set is unchanged. |
| `ScenarioField` has no `Hash`, no `Ord`, no `PartialOrd` derives (line 262) — only `Default + Clone + Debug + PartialEq + Eq + Serialize + Deserialize` | Source inspection at line 262 | PO-SC-V-21, V-24, V-25 do not depend on `Hash` or `Ord`. |

No `axiom`, `admit`, or `#[verifier::trusted]` is required **inside** `scenario.rs` — every trust boundary is either an `extern_spec` on a stdlib method (`String::trim`, `u8::clamp`, `Vec::push` semantics) or a `#[verifier::external_body]` on a trait impl (`Display for HoleType`). Both are honest and have explicit `compensating_evidence` (proptest rows for the same property).

---

## 7. Waiver candidates

**None.** All in-scope behaviour is provable under the chosen lanes. The non-applicable lanes (K, F, L, M, T, Z, X) have concrete evidence in §4 and do not require waivers — they are genuinely not needed.

If `rust-contract` decides that "addressed" should mean `Some(_)` (any, including whitespace) instead of `Some(s) ∧ !s.trim().is_empty()` (current source), the `is_addressed` spec (REQ-SC-12) changes shape, and REQ-SC-11, REQ-SC-14, REQ-SC-15, REQ-SC-16 must be re-derived against the weaker predicate. That is an **additive change to the spec**, not a waiver of behaviour — the obligation set grows or contracts but does not get waived.

If `rust-contract` decides that the `severity` clamp range `[1, 5]` is an implementation detail rather than a contract, the `with_severity` spec (REQ-SC-7) becomes `with_severity(_, _, s).severity == s.clamp(MIN, MAX)` with `MIN, MAX` as named constants — proof-writer authors a `const` spec. That is also additive, not a waiver.

If `rust-contract` decides that `address` should change from consumed-self to `&mut self` or to `&self -> Self`, the L1/L2 algebraic laws hold under all three signatures; the proof-writer adjusts the proof shape. Additive, not a waiver.

---

## 8. Bridge input for `proof-to-implementation`

The proof-writer will produce Verus specs and proptest properties. The bridge agent maps them to:

| Proof claim | Rust source ref | Independent behaviour test |
|---|---|---|
| PO-SC-V-01 | `HoleType` enum (lines 21–32) | Existing `test_hole_type_all_returns_three_types` (line 347) covers `all().len() == 3` and variant containment. |
| PO-SC-V-02 | `HoleType::all` (lines 37–43) | Existing `test_hole_type_all_returns_three_types` (line 347). |
| PO-SC-V-03 | `HoleType::label` (lines 47–53) | Existing `test_hole_type_labels` (line 356). |
| PO-SC-V-04 | `HoleType::description` (lines 57–63) | Existing `test_hole_type_descriptions` (line 363). |
| PO-SC-V-05 | `Display for HoleType` (lines 66–71) | Existing `test_hole_type_display` (line 370). |
| PO-SC-V-06 | `Hole::new` (lines 89–96) | Existing `test_hole_new_has_default_severity` (line 383). |
| PO-SC-V-07 | `Hole::with_severity` (lines 102–109) | Existing `test_hole_with_severity_clamps_high` (line 391) and `test_hole_with_severity_clamps_low` (line 397) cover the boundary at `0` and `10`. New `test_hole_with_severity_clamps_max_u8` adds `severity == 255 → 5`. |
| PO-SC-V-08 | `Hole::is_high_severity` (lines 112–115) | Existing `test_hole_is_high_severity` (line 403). |
| PO-SC-V-09 | `HolePunchingResults::default` (line 128, derive) | Existing `test_default_holes_are_none` (line 413). |
| PO-SC-V-10 | `HolePunchingResults::new` / `empty` (lines 143–152) | New `test_hole_punching_new_equals_default` and `test_hole_punching_empty_equals_default`. |
| PO-SC-V-11 | `normalize_explanation` (private, lines 237–243) | Tested transitively via `from_strings` (existing `test_empty_string_treated_as_none`, line 453) and `address` (existing `test_address_method_empty_normalizes`, line 479). |
| PO-SC-V-12 | `is_addressed` (lines 175–191) | Existing `test_is_addressed` (line 485). |
| PO-SC-V-13 | `is_complete` (lines 158–172) | Existing `test_is_complete_requires_all_holes` (line 422) and `test_partial_holes_incomplete` (line 432). |
| PO-SC-V-14 | `address` (lines 196–205) | Existing `test_address_method` (line 463) and `test_address_method_empty_normalizes` (line 479). |
| PO-SC-V-15 | `address` right-most-wins (L1) | New `test_address_right_most_wins` calls `r.address(ht, "x").address(ht, "y")` and asserts equality with `r.address(ht, "y")` on all 3 axes. |
| PO-SC-V-16 | `address` idempotent (L2) | New `test_address_idempotent` calls `r.address(ht, "x").address(ht, "x")` and asserts equality with `r.address(ht, "x")`. |
| PO-SC-V-17 | `explanation` (lines 208–215) | Existing `test_explanation` (line 497). |
| PO-SC-V-18 | `unaddressed_holes` (lines 218–225) | Existing `test_unaddressed_holes` (line 511). New `test_unaddressed_holes_idempotent` calls twice and asserts equality. |
| PO-SC-V-19 | `addressed_count` (lines 228–234) | Existing `test_partial_holes_incomplete` (line 432) asserts `addressed_count() == 1` and `== 2`. |
| PO-SC-V-20 | `from_strings` (lines 247–253) | Existing `test_empty_string_treated_as_none` (line 453). |
| PO-SC-V-21 | `ScenarioField::new` (lines 277–284) | Existing `test_scenario_field_bullets_complete_but_holes_not` (line 558) covers the positive case; extend with all 3 fields explicit equality assertions. |
| PO-SC-V-22 | `is_bullets_complete` (lines 300–304) | Existing `test_scenario_field_default_is_incomplete` (line 550) and `test_whitespace_treated_as_empty` (line 585). |
| PO-SC-V-23 | `is_trigger_empty` etc. (lines 308–322) | Existing `test_individual_field_empty_checks` (line 599) and `test_whitespace_treated_as_empty` (line 585). |
| PO-SC-V-24 | `is_complete` (lines 294–296) | Existing `test_scenario_field_complete_when_all_filled_and_holes_addressed` (line 569). |
| PO-SC-V-25 | `ScenarioField::empty` (lines 288–290) | New `test_scenario_field_empty_equals_default`. |
| PO-SC-P-01 | `HoleType` serde | Existing `test_hole_type_serialization` (line 375). |
| PO-SC-P-02 | `Hole` serde | New proptest `proptest_hole_serde_roundtrip` in `#[cfg(test)] mod tests_proptest`. |
| PO-SC-P-03 | `HolePunchingResults` serde + idempotence + monotonicity | New proptest `proptest_hole_punching_address_laws` + `proptest_hole_punching_serde_roundtrip`. |
| PO-SC-P-04 | `ScenarioField` serde + L6 | New proptest `proptest_scenario_field_serde_roundtrip` + `proptest_scenario_field_is_complete_law`. |
| PO-SC-P-05 | `Display for HoleType` round-trip | Existing `test_hole_type_display` (line 370). |
| PO-SC-P-06 | `address` right-most-wins (property test) | Same property as PO-SC-V-15, but generated over random inputs. |
| PO-SC-P-07 | `with_severity` clamp boundaries | Existing `test_hole_with_severity_clamps_high` (line 391) and `test_hole_with_severity_clamps_low` (line 397); extend with `severity == 255`. |
| PO-SC-P-08 | `is_complete ⇔ addressed_count == 3` (Law L6) | New proptest `proptest_hole_punching_is_complete_law` (property of the conjunction law). |

The proof-writer must **not** modify any production function body. Verus `#[verifier::external_body]` is the correct tool for `Display for HoleType` (line 66–71). The `format!("{}", ht)` call's trust on `fmt::Formatter` machinery is the only `external_body` site in this plan.

---

## 9. Blockers for proof-writer

1. **Contract ratification (BLOCKING).** `rust-contract` must author `contract.md` (or equivalent) for `clarity-web/src/domain/` and either ratify or correct the 26 clauses in §3. All 33 `planned` obligations carry `requires_contract: true` to make this gate visible (25 Verus + 8 proptest).
2. **Open refinement decisions (NON-BLOCKING but recommended).** §3 lists five open questions. Defaulting them to "current source behaviour" yields the obligation set above. Defaulting them differently shrinks or grows the set; still tractable but not yet planned.
3. **No tooling gaps for this plan.** Verus is installed (v0.2026.05.05); `cargo verus` is installed; proptest is bundled via `clarity-web/Cargo.toml`. Kani / cargo-fuzz gaps exist but do not block this module.
4. **Clippy debt (`cl-2q6`) is independent of this plan.** `domain/scenario.rs` does **not** contribute to the 64-site lint baseline (verified against `formal-verification-report.md §4`). The `#[cfg(test)] mod tests` block opens `clippy::unwrap_used`, `clippy::expect_used`, `clippy::panic`, etc. (lines 326–340) so test additions are clippy-exempt from the workspace lint set. However, the **production function bodies must remain clippy-clean** under `cargo clippy --workspace --all-targets -- -D warnings`; the proptest additions added by `proof-writer` in `#[cfg(test)] mod tests_proptest` must also stay clippy-clean (use `#[allow(...)]` on the proptest submodule if necessary, mirroring the existing test module).

---

## 10. Non-targets (explicit)

Per `verification-targets.md §8`, the following are NOT in this plan:

- **Line-by-line proofs.** Refused; not cost-effective for the trivial extractors (e.g. `explanation(ht) -> self.<field>.as_deref()`). They are spec'd because the spec cost is near-zero and the contract gap is fully closed for the type, but no per-line proof ceremony is implied.
- **Miri, Loom, TLA+, fuzz, Flux, Kani.** All not applicable (§4). No `not_applicable` obligation rows for these will be promoted to `waived`.
- **Production `unsafe`** — `forbid` at workspace level (`Cargo.toml:10`) and module level (`scenario.rs:6`); no obligation needed.
- **The 18 hand-written tests in `#[cfg(test)] mod tests`** (lines 325–633) are exercised behaviour, not proof targets. They are referenced by the bridge (§8) but not themselves proved.
- **A TLA+ spec for `scenario.rs`'s state machine.** The state machine is **predicate-driven and stateless from the module's perspective**: each `ScenarioField` is an immutable value; transitions are driven by client code. A TLA+ spec would describe the client code's sequencing, not the module itself. The module is fully covered by Verus `requires`/`ensures` clauses. TLA+ is reserved for `intent/beads`, `intent/batch`, and `storage/fjall_event_store` per `verification-targets.md §5.2–§5.3`.
- **The 18 hand-written tests' clippy debt** — they are clippy-exempt via `#[allow(...)]` at line 326–340.

---

## 11. Pre-flight checklist for landing this plan

- [ ] `rust-contract` produces contract clauses for the 26 inferred items in §3 (or supersedes them).
- [ ] `proof-plan-reviewer` reviews this file and the obligations JSONL (`scenario-obligations.planned.jsonl`) and writes `verifier-lane-review.jsonl` with independent disposition.
- [ ] Verus invocation command confirmed: `cargo verus verify --manifest-path clarity-web/Cargo.toml --crate-type lib` from `/home/lewis/src/clarity`. (Alternative single-file form: `verus clarity-web/src/domain/scenario.rs` — to be confirmed by `formal-verifier` based on `cargo verus` integration tests.)
- [ ] proptest invocation confirmed: `cargo test -p clarity-web --lib domain::scenario -- --nocapture` (or the specific `proptest_<name>` filter), with the new property functions added in `#[cfg(test)] mod tests_proptest`.
- [ ] `cl-2q6` clippy gate is independent of this plan (the module's `#[cfg(test)] mod tests` already opens the relevant lints at lines 326–340; the production function bodies contribute zero to the clippy baseline).

---

*End of plan.*
