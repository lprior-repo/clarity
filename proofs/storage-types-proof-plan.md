# Proof Plan — `clarity-web/src/storage/types.rs`

| Field | Value |
|---|---|
| **Bead** | `cl-5dp` |
| **Target** | `clarity-web/src/storage/types.rs` |
| **Primary lane** | **V** (Verus) — per `verification-targets.md §5.2` |
| **Secondary lane** | **P** (proptest) — round-trip on Serialize/Deserialize |
| **Contract status** | **GAP — no `rust-contract` artifact exists for this module.** Clauses below are *inferred from source* and labelled `INFERRED`. The plan is gated on `rust-contract` ratifying (or correcting) these clauses before `proof-writer` runs. |
| **Planner** | `proof-planner` |
| **Date** | 2026-06-21 |

---

## 1. Module characterisation

`storage/types.rs` is **pure data**: zero I/O, zero concurrency, zero `unsafe` (`#![forbid(unsafe_code)]` at module top, line 6). It declares five `serde::{Serialize, Deserialize}`-bearing types plus a `tables` module of string constants, and a `#[cfg(test)] mod tests` block of 10 hand-written tests (lines 188–400).

| Item | Lines | Kind | Risk surface |
|---|---|---|---|
| `enum Confidence { High, Inferred, Uncertain }` | 11–20 | 3-variant bounded enum; `serde(rename_all = "lowercase")` | Rust-local invariant (serde naming) |
| `struct AnswerRecord { step_id, value, timestamp, confidence, ai_generated }` | 24–36 | Owned `String` × 3 + enum + `bool` | Constructor invariant on `from_answer` |
| `AnswerRecord::new` (const) | 41–55 | Total constructor | None (already total) |
| `AnswerRecord::from_answer` (const) | 59–67 | Pre-filled constructor | **Typestate**: `confidence == High ∧ ai_generated == false` |
| `struct ExtractionCache { input_hash, fields, timestamp }` | 72–80 | Owned `String` × 3 | None (no invariants enforced) |
| `struct ProjectMetadata { mode_preference, current_phase, created_at, updated_at }` | 96–106 | Owned `String` × 4 | None at struct level; `with_current_timestamp` enforces RFC 3339 + `created_at == updated_at` |
| `ProjectMetadata::new` (const) | 111–123 | Total constructor | None |
| `ProjectMetadata::with_current_timestamp` | 127–135 | Wraps `chrono::Utc::now().to_rfc3339()` for both fields | RFC 3339 parseability + equality of timestamps |
| `struct LatticeCache { phase, output_data, timestamp }` | 140–148 | Owned `String` × 3 | None at struct level; `with_current_timestamp` enforces RFC 3339 |
| `LatticeCache::new` (const) | 153–159 | Total constructor | None |
| `LatticeCache::with_current_timestamp` | 163–169 | Wraps `chrono::Utc::now().to_rfc3339()` for `timestamp` | RFC 3339 parseability |
| `mod tables { ANSWERS, EXTRACTIONS, PROJECT_METADATA, LATTICE_CACHE }` | 174–186 | Four `&'static str` table-name constants | Non-empty, pairwise distinct |
| `#[cfg(test)] mod tests` | 188–400 | 10 unit tests using `serde_json` round-trip on fixed inputs | Test-only; not a proof target |

**Refinement-style properties** that are *documented* but *not enforced* by the type system:

- `AnswerRecord.step_id`, `ExtractionCache.input_hash`, `LatticeCache.phase`, `ProjectMetadata.mode_preference`, `ProjectMetadata.current_phase` are described as "unique identifier" / "identifier" / "non-empty implicitly" but are plain `String`. Verus can pin these as spec invariants on `new`/`from_answer`/`with_current_timestamp` if `rust-contract` ratifies them as part of the contract.
- `timestamp` / `created_at` / `updated_at` are documented as ISO 8601 / RFC 3339 but the type is `String`. Only `with_current_timestamp` constructors can be proven to produce RFC 3339 (because they call `chrono::Utc::now().to_rfc3339()`); arbitrary `String` timestamps cannot be proven RFC 3339 without a parser spec. This is marked as an `OPEN` clause for `rust-contract`.

## 2. Contract gap (honest disclosure)

The bead `cl-5dp` has **no upstream `rust-contract` artifact** under `clarity-web/src/storage/contract.md` (or any path matching `**/contract.md` in the workspace). All clauses used in this plan are *inferred by direct reading of the source*, not authored.

**Action required before `proof-writer` runs:** `rust-contract` must ratify or correct the clauses marked `INFERRED` below. If the inferred clauses are wrong, `proof-writer` will write specs against the wrong contract. The obligations JSONL sets `requires_contract: true` on every inferred-clause row so this gate is visible.

`rust-contract` should produce (at minimum):

- `contract.md` for `clarity-web/src/storage/` with clauses keyed to the requirement IDs in §3.
- Explicit decision on whether `step_id`, `input_hash`, `phase`, `mode_preference`, `current_phase` are *part of the contract* as non-empty. If they are, `proof-writer` must specify a refinement predicate; if not, the obligation set shrinks.
- Explicit decision on whether `timestamp` fields are part of the contract as RFC 3339 outside of `with_current_timestamp`. If yes, an additional `is_rfc3339` spec fn is needed (likely through a parser spec — Kani or Flux candidate). If no, the round-trip property is unconditional and the obligations below are sufficient.

## 3. Requirements & inferred contract clauses

The requirements are derived from the doc comments and the `#[serde(...)]` attributes. Each row carries a `clause_origin` of either `INFERRED` (this plan) or `AUTHORED` (rust-contract — currently none).

| Req ID | Source | Inferred clause | Origin |
|---|---|---|---|
| REQ-ST-1 | `Confidence` variants (lines 14–19) | `Confidence` is exactly `{High, Inferred, Uncertain}`; no future-added variants break serde mapping. | INFERRED |
| REQ-ST-2 | `Confidence` `#[serde(rename_all = "lowercase")]` (line 12) | `serde_json::to_string(Confidence::v) == "\"v\""` for each variant `v`, and the inverse round-trips. | INFERRED |
| REQ-ST-3 | `AnswerRecord::from_answer` (lines 59–67) | `from_answer(step_id, value, ts).confidence == High ∧ …ai_generated == false`. | INFERRED |
| REQ-ST-4 | `AnswerRecord::new` (lines 41–55) | `new` is total: any 5-tuple of accepted types produces a valid `AnswerRecord`. | INFERRED |
| REQ-ST-5 | `ExtractionCache::new` (lines 85–91) | `new` is total. | INFERRED |
| REQ-ST-6 | `ProjectMetadata::new` (lines 111–123) | `new` is total. | INFERRED |
| REQ-ST-7 | `ProjectMetadata::with_current_timestamp` (lines 127–135) | Both `created_at` and `updated_at` equal `chrono::Utc::now().to_rfc3339()` evaluated at the same instant; both parse as RFC 3339. | INFERRED |
| REQ-ST-8 | `LatticeCache::new` (lines 153–159) | `new` is total. | INFERRED |
| REQ-ST-9 | `LatticeCache::with_current_timestamp` (lines 163–169) | `timestamp` equals `chrono::Utc::now().to_rfc3339()` and parses as RFC 3339. | INFERRED |
| REQ-ST-10 | `tables::ANSWERS/EXTRACTIONS/PROJECT_METADATA/LATTICE_CACHE` (lines 174–186) | Each constant is non-empty and pairwise distinct. | INFERRED |
| REQ-ST-11 | `Serialize` + `Deserialize` derives on all four record types | For any `T` of the four record types and any valid `T` value, `serde_json::from_str::<T>(&serde_json::to_string(&t).unwrap_or_default()) == Ok(t)`. | INFERRED |
| REQ-ST-12 | `Confidence` derived traits (line 11) | `Clone + Copy + PartialEq + Eq + Serialize + Deserialize`. | INFERRED |

**Open questions for `rust-contract`** (need answers before proof-writer starts):

1. Are `step_id`, `input_hash`, `phase`, `mode_preference`, `current_phase` part of the contract as non-empty? (Likely yes for `step_id` and `phase`; less clear for the others.)
2. Is the `fields` field of `ExtractionCache` contractually a JSON object, or arbitrary string? (Doc says "JSON object" but the type is `String`.)
3. Is `mode_preference` enumerated (e.g., "waterfall"/"agile" only)? Or open string?

## 4. Verifier lane decisions

Per `verifier-trigger-matrix.md`, classify the proof seeds across Verus / Kani / Flux / Loom / proptest / fuzz.

| Lane | Decision | Evidence / rationale |
|---|---|---|
| **V** (Verus) | **REQUIRED — primary** | Module is pure; invariants on constructors and serde naming are exactly what Verus specs. Per `verification-targets.md §5.2`. Verus is installed (`/home/lewis/.local/bin/verus`, v0.2026.05.05). |
| **P** (proptest) | **REQUIRED — secondary** | All four record types + `Confidence` derive `Serialize`/`Deserialize`. Round-trip property is the natural proptest target; cheaper than Verus for arbitrary `String` contents. proptest is bundled with cargo (no install needed). |
| **K** (Kani) | **NOT APPLICABLE** | Kani's strength (bounded model check of unsafe, fixed-width arithmetic, parser bounds) does not apply. `Confidence` is a 3-variant enum — Verus spec fn over `match` covers exhaustiveness more cheaply. No `#[kani::proof]` harness is warranted. Cite: `#![forbid(unsafe_code)]` (line 6), no arithmetic, no parser, no index ops. Kani not installed (see `verification-targets.md §3`) — even if applicable, install would be required first. |
| **F** (Flux) | **NOT APPLICABLE** | Flux is the lightweight refinement-type alternative to Verus. Since Verus covers the refinement properties (RFC 3339, non-empty) with more rigour and the same author burden, Flux would be redundant. `cargo-flux` is installed but unused for this module. |
| **L** (Loom) | **NOT APPLICABLE** | No concurrency in this module — no threads, channels, atomics, `Send + Sync` interactions, async, or spawn calls. |
| **M** (Miri) | **NOT APPLICABLE** | `#![forbid(unsafe_code)]` at module top (line 6). No `unsafe` blocks anywhere in `types.rs`. |
| **T** (TLA+) | **NOT APPLICABLE** | No temporal workflow — no state machine transitions, no retries, no leases, no batch ordering. The `tables` constants are static. A TLA+ spec for `fjall_event_store.rs` is mentioned in §5.2 of the verification targets but is out of scope here. |
| **Z** (fuzz) | **NOT APPLICABLE** | The deserialisation boundary is structured `serde_json` of typed Rust records — there is no hand-written parser, regex, codec, or frame decoder. Adversarial input funnels through `serde_json`, which has its own robustness story. `cargo-fuzz` is not installed (per `verification-targets.md §3`); even if it were, no harness target exists in this module. |
| **X** (exercise-only) | **NOT APPLICABLE** | The whole module is in scope for V + P coverage. The `#[cfg(test)] mod tests` block is covered by behaviour tests and does not need a separate `X` lane. |

Two infrastructure gaps block adjacent lanes (`K`, `Z`) but those lanes are `not_applicable` to this module regardless, so the gaps are not blockers for *this* plan. They are noted for the landing-skill pre-flight in `verification-targets.md §4`.

## 5. Proof coverage matrix

| Req ID | Lane | Obligation ID | Targets | Evidence |
|---|---|---|---|---|
| REQ-ST-1 | V | PO-V1 | `Confidence` (whole enum) | Verus spec `confidence_to_spec(c: Confidence) -> nat` with exact 3-element codomain. |
| REQ-ST-2 | V | PO-V2 | `Confidence` serde encoding | Verus spec `confidence_serde_name(c) == serde_json::to_string(c).unwrap_or_default()` — proved via exec `#[verifier::external_body]` to the serde_json call. |
| REQ-ST-3 | V | PO-V3 | `AnswerRecord::from_answer` | Verus ensures/requires on the constructor. |
| REQ-ST-4 | V | PO-V4 | `AnswerRecord::new` | Verus ensures on `new`. |
| REQ-ST-5 | V | PO-V5 | `ExtractionCache::new` | Verus ensures on `new`. |
| REQ-ST-6 | V | PO-V6 | `ProjectMetadata::new` | Verus ensures on `new`. |
| REQ-ST-7 | V | PO-V7 | `ProjectMetadata::with_current_timestamp` | Verus ensures on the timestamp constructor; uses `chrono::Utc::now()` as a trusted opaque value with the postcondition that the result `.parse::<DateTime<Utc>>()` returns `Ok`. |
| REQ-ST-8 | V | PO-V8 | `LatticeCache::new` | Verus ensures on `new`. |
| REQ-ST-9 | V | PO-V9 | `LatticeCache::with_current_timestamp` | Verus ensures on the timestamp constructor. |
| REQ-ST-10 | V | PO-V10 | `tables::*` constants | Verus spec on the const expressions at compile time. |
| REQ-ST-11 | P | PO-P1 … PO-P5 | All five serializable types | proptest round-trip properties. |

REQ-ST-12 is implicit in the derives and exercised transitively by REQ-ST-11; no separate obligation.

## 6. Trusted base plan

These are the assumptions the proofs lean on. Each is either explicitly trusted or has its own obligation.

| Trust | Why trusted | Mitigation in obligations |
|---|---|---|
| `serde_json` round-trip preserves values | Library contract; not our code | PO-P1…P5 explicitly exercise the round-trip on our types — if serde silently corrupted values, the property tests fail. |
| `chrono::Utc::now().to_rfc3339()` returns an RFC 3339 string | Library contract; not our code | PO-V7, PO-V9 use this as the trusted opaque return value; the postcondition parses it back. If the library broke the contract, the parse would fail and the ensures would fail. |
| The `Confidence` enum is closed (no future variant added) | Type-system fact at the time the spec is written | PO-V1, PO-V2 enumerate the 3 variants explicitly via `match`. Adding a variant would require updating the spec — caught at compile time of the spec file. |
| `String` is a total type over arbitrary UTF-8 | Rust stdlib | No mitigation needed; the round-trip property must hold for arbitrary bytes. |
| The `tables` constants do not change at runtime | `const` declarations | PO-V10 proves the constant values at spec-load time. |

## 7. Waiver candidates

**None.** All in-scope behaviour is provable under the chosen lanes. The non-applicable lanes (K, F, L, M, T, Z, X) have concrete evidence in §4 and do not require waivers — they are genuinely not needed.

If `rust-contract` decides that `step_id` / `input_hash` / `phase` / `mode_preference` / `current_phase` *are* contractually non-empty, an additional refinement predicate obligation may need to be added (likely Flux or Verus). That is an additive change, not a waiver of behaviour.

## 8. Bridge input for `proof-to-implementation`

The proof-writer will produce Verus specs and proptest properties. The bridge agent maps them to:

| Proof claim | Rust source ref | Independent behaviour test |
|---|---|---|
| PO-V1, V2 | `Confidence` enum + `#[serde(rename_all = "lowercase")]` | Extend existing `test_confidence_serialization` (line 209) with property cases. |
| PO-V3 | `AnswerRecord::from_answer` (lines 59–67) | Existing `test_answer_record_from_answer` (line 254) covers the positive case; add an `assert_eq!(r.confidence, Confidence::High)` and `assert!(!r.ai_generated)` regression. |
| PO-V4 | `AnswerRecord::new` (lines 41–55) | Existing `test_answer_record_with_ai_generated` (line 373) covers a non-default combination. |
| PO-V5 | `ExtractionCache::new` (lines 85–91) | Existing `test_extraction_cache_serialization` (line 268). |
| PO-V6 | `ProjectMetadata::new` (lines 111–123) | Existing `test_project_metadata_serialization` (line 284). |
| PO-V7 | `ProjectMetadata::with_current_timestamp` (lines 127–135) | Existing `test_project_metadata_with_current_timestamp` (line 301) parses both timestamps and checks the range. |
| PO-V8 | `LatticeCache::new` (lines 153–159) | Existing `test_lattice_cache_serialization` (line 325). |
| PO-V9 | `LatticeCache::with_current_timestamp` (lines 163–169) | Existing `test_lattice_cache_with_current_timestamp` (line 341). |
| PO-V10 | `tables::*` constants (lines 174–186) | Existing `test_table_constants` (line 395). |
| PO-P1…P5 | All five serializable types | New proptest functions in `#[cfg(test)] mod tests` (extend the existing module). |

The proof-writer must **not** modify any production function body. Verus `#[verifier::external_body]` is the correct tool for the serde_json / chrono calls that should not be re-verified.

## 9. Blockers for proof-writer

1. **Contract ratification (BLOCKING).** `rust-contract` must author `contract.md` (or equivalent) for `clarity-web/src/storage/` and either ratify or correct the 12 clauses in §3. All 15 `planned` obligations carry `requires_contract: true` to make this gate visible (10 Verus + 5 proptest).
2. **Open refinement decisions (NON-BLOCKING but recommended).** §3 lists three open questions. Defaulting them to "documented but unenforced" yields the obligation set above. Defaulting them to "enforced by spec" adds 2–3 Flux-or-Verus refinement obligations; still tractable but not yet planned.
3. **No tooling gaps for this plan.** Verus is installed; proptest is bundled with cargo. Kani / cargo-fuzz gaps exist but do not block this module.

## 10. Non-targets (explicit)

Per `verification-targets.md §8`, the following are NOT in this plan:

- **Line-by-line proofs.** Refused; not cost-effective for glue/serde code.
- **The 10 hand-written tests in `#[cfg(test)] mod tests`** (lines 188–400) are exercised behaviour, not proof targets. They are referenced by the bridge (§8) but not themselves proved.
- **Miri, Loom, TLA+, fuzz.** All not applicable (§4). No `not_applicable` obligation rows for these will be promoted to `waived`.
- **Production `unsafe`** — `forbid` at workspace level (`Cargo.toml` line 10) and module level (line 6); no obligation needed.

## 11. Pre-flight checklist for landing this plan

- [ ] `rust-contract` produces contract clauses for the 12 inferred items in §3.
- [ ] `proof-plan-reviewer` reviews this file and the obligations JSONL.
- [ ] Verus invocation command confirmed: `cargo verus verify --crate clarity-web --module storage::types` (or whatever the project's Verus integration looks like — to be confirmed by `formal-verifier`).
- [ ] proptest invocation confirmed: `cargo test -p clarity-web --lib storage::types::tests` extended with the new property functions.
- [ ] `cl-2q6` clippy gate is independent of this plan (the module's `#[cfg(test)] mod tests` uses `clippy::unwrap_used` etc. under `#[allow(...)]` so it does not contribute to the lint debt).

---

*End of plan.*
