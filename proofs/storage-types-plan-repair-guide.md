# Repair Guide — `cl-5dp` proof plan

**Target artifacts:**
- `proofs/storage-types-proof-plan.md`
- `proofs/storage-types-obligations.planned.jsonl`

**Reviewer:** `proof-plan-reviewer` (invocation `proof-plan-reviewer::cl-5dp::2026-06-21T17:05Z`)

**Findings reference:** `proofs/storage-types-plan-findings.jsonl` (11 rows)
**Lane review reference:** `proofs/storage-types-verifier-lane-review.jsonl` (22 rows: 15 V/P `rejected`, 7 NA `accepted`)

---

## Smallest state to rerun

After applying the fixes below, the proof-planner (or a repair agent) reruns the **review pass only**. The reviewer's re-review will flip the 22 `verifier-lane-review/v1` rows from `rejected` to `accepted` and emit `STATUS: APPROVED` in an updated `proofs/storage-types-plan-review.md`. The plan's architecture does not need to change; the obligation set, lane selection, and bridge plan are kept as-is.

---

## Blockers (must fix before any re-review)

### F-ST-001 + F-ST-002 — Obligations JSONL schema re-render (one mechanical pass)

**File:** `proofs/storage-types-obligations.planned.jsonl`

**Action:** Re-render all 22 rows against the canonical `proof-obligation/v1` schema. Use `proofs/newtypes-obligations.planned.jsonl` row 1 as the reference shape (it is the only sibling that is schema-compliant).

**Diff against current cl-5dp rows:**

| Field | Current | Required |
|-------|---------|----------|
| `schema_version` | absent | `"proof-obligation/v1"` (add to all 22 rows) |
| `obligation_id` | absent (uses `id`) | add as alias of `id` |
| `lane` | absent (uses `verifier`) | add as alias of `verifier` (e.g. `"V"` for verus, `"P"` for proptest, `"K"` for kani, etc.) |
| `risk` | string scalar | rename to `risk_tags`, convert to single-element list |
| `target_module` | absent (combined with `target`) | add `target_module: "clarity-web/src/storage/types.rs"` |
| `target_function` | absent | add (e.g. for PO-V3: `"AnswerRecord::from_answer"`) |
| `target_signature` | absent | add (e.g. `"pub const fn from_answer(step_id: String, value: String, timestamp: String) -> Self"`) |
| `expected_evidence_command` | absent | add (e.g. `"cargo verus verify --manifest-path clarity-web/Cargo.toml --crate-type lib"`) |
| `workdir` | absent | add `"/home/lewis/src/clarity"` |
| `model_bounds` | absent | add (e.g. `"No loop unwinding required (no loops in any constructor body)"`) |
| `tool_metadata` | absent | add object: `{"verus_version": "0.2026.05.05", "target_module_loc": 401}` |
| `trusted_base_refs` | absent | add list referencing the planned trusted-base-ledger rows (see F-ST-005) |
| `behavior_affecting` | absent | add bool (see decision per-row below) |
| `reviewer_role` | absent | add `"proof-writer"` for `V` rows, `"test-writer"` for `P` rows |
| `verus_mode` | absent (V rows) | add `"exec+spec"` (or appropriate) |

**Per-row `behavior_affecting` decision (currently missing):**

| Row | behavior_affecting | Rationale |
|-----|--------------------|-----------|
| PO-V1 | false | Enum exhaustiveness is a structural fact, not user-visible behavior |
| PO-V2 | true | Serde encoding is on the storage boundary (redb_store, transcript_store) |
| PO-V3 | true | Typestate from_answer propagates to redb_store.rs:175 |
| PO-V4, PO-V5, PO-V6, PO-V8 | false | Trivial total-constructor postconditions |
| PO-V7, PO-V9 | true | Timestamp values affect key ordering and consumer invariants |
| PO-V10 | false | Const-fact; not user-visible |
| PO-P1..PO-P5 | true | Round-trip on storage boundary; corruption visible to consumers |
| PO-K1, PO-F1, PO-L1, PO-M1, PO-T1, PO-Z1, PO-X1 | false | Not_applicable rows; not behavior-affecting |

**Verification:** after re-render, `python -c "import json; [json.loads(l) for l in open('proofs/storage-types-obligations.planned.jsonl')]"` should succeed and each row should have all 21 required fields.

---

### F-ST-003 — Emit `verifier-lane-decisions.jsonl`

**File:** `proofs/verifier-lane-decisions.jsonl` (new)

**Action:** Produce one `verifier-lane-decision/v1` row per obligation. Each row's `id` becomes the `lane_decision_id` key in the matching `verifier-lane-review/v1` row. Currently the reviewer used `PO-V1` etc. as the `lane_decision_id`, but the canonical schema expects the planner to define the decision ID and the reviewer to reference it. The fix is to emit decision rows with IDs like `VLD-ST-1` ... `VLD-ST-22` and update the lane-review rows.

**Required fields per `verifier-lane-decision/v1`:** `schema_version`, `id`, `requirement_id`, `contract_clause`, `proof_seed_id`, `verifier`, `risk_tags`, `applicability`, `decision_reason`, `required_obligation_ids`, `non_applicability_evidence_refs`, `limitation_kind`, `owner_state`, `status`.

**Reference shape (example for PO-V1):**
```json
{"schema_version":"verifier-lane-decision/v1","id":"VLD-ST-1","requirement_id":"REQ-ST-1","contract_clause":"Confidence is exactly {High, Inferred, Uncertain}","proof_seed_id":"REQ-ST-1","verifier":"verus","risk_tags":["rust_local_invariant"],"applicability":"required","decision_reason":"Module is pure; Verus spec fn over match is the cheapest exhaustiveness check","required_obligation_ids":["PO-V1"],"non_applicability_evidence_refs":[],"limitation_kind":null,"owner_state":6,"status":"planned"}
```

**Verification:** the file should have 22 rows; each row's `id` is unique; the IDs in `storage-types-verifier-lane-review.jsonl` should reference these decision IDs (not the obligation IDs).

---

### F-ST-004 — Split PO-V7 and PO-V9 to address non-vacuity

**File:** `proofs/storage-types-proof-plan.md` §3, §5, and the obligations JSONL

**Action:** The current PO-V7 and PO-V9 are vacuous because `#[verifier::external_body]` is used on the same `.parse::<DateTime<Utc>>()` call that the postcondition claims succeeds. Choose ONE of the following repair paths:

**Path A (recommended): Split the obligations.**
- PO-V7a: `ProjectMetadata::with_current_timestamp` produces `result.created_at == result.updated_at`. Provable by referencing the literal `let now = chrono::Utc::now().to_rfc3339(); created_at: now.clone(), updated_at: now` pattern in the spec.
- PO-V7b: `ProjectMetadata::with_current_timestamp` produces a `result.created_at` that parses as `Ok(DateTime<Utc>)`. NOT provable in Verus without an extern_spec. Reclassify as: (i) a Kani obligation with a chrono extern_spec linkage (requires kani installed per cl-u04), or (ii) a Flux refinement obligation (`#[refined_by]` with `is_rfc3339` predicate), or (iii) a behavior-test obligation that lifts `test_project_metadata_with_current_timestamp` out of `#[cfg(test)] mod tests` into a non-test path.

Same split for PO-V9 (LatticeCache::with_current_timestamp).

**Path B: Cite a chrono extern_spec.**
If a chrono extern_spec already exists in the workspace (check `proofs/verus/chrono_extern_spec.rs` or similar), link to it. If not, this is path A. Search the workspace for `extern_spec` markers to confirm.

**Path C: Lift the hand-written tests.**
Move `test_project_metadata_with_current_timestamp` (line 301) and `test_lattice_cache_with_current_timestamp` (line 341) from `#[cfg(test)] mod tests` to a dedicated `#[cfg(test)] mod timestamp_proof` module that is referenced by PO-V7b/PO-V9b. Update plan §10 to acknowledge these tests as proof targets, not just exercise coverage.

**Verification:** After the split, the postcondition in PO-V7a/PO-V9a is provable by Verus on a faithful-copy analysis of the constructor body. PO-V7b/PO-V9b has either a non-vacuous Verus spec, a Kani harness, a Flux refinement, or a behavior-test obligation with a captured PASS row.

**Add a non-vacuity section to the plan** (currently absent). Suggested location: between §6 (Trusted base plan) and §7 (Waiver candidates). Name it `§6.5 Non-vacuity evidence` and list the postconditions that are non-trivially proved vs. delegated to library contracts.

---

### F-ST-005 — Emit `trusted-base-ledger.jsonl`

**File:** `proofs/storage-types-trusted-base.jsonl` (new)

**Action:** Produce one `trusted-base-ledger/v1` row per trust marker. The plan §6 lists 5 trust assumptions; the obligations have 3 `#[verifier::external_body]` markers; the total is 6 rows. Required fields per `trusted-base-ledger/v1`: `schema_version`, `id`, `obligation_id`, `artifact`, `location`, `marker`, `trusted_kind`, `reason`, `scope`, `impact`, `behavior_affecting`, `compensating_evidence`, `owner`, `expiry`, `reviewer_disposition`, `status`.

**Reference rows:**

| ID | obligation_id | marker | trusted_kind | reason | scope |
|----|---------------|--------|--------------|--------|-------|
| TB-ST-1 | PO-V2 | `external_body` on `serde_json::to_string` | `external_body` | library contract not our code | Confidence serde encoding |
| TB-ST-2 | PO-V7, PO-V9 | `external_body` on `chrono::Utc::now().to_rfc3339` | `external_body` | library contract | RFC 3339 timestamp generation |
| TB-ST-3 | PO-V7, PO-V9 | `external_body` on `chrono::DateTime::parse_from_rfc3339` | `external_body` | library contract; OR linked to F-ST-004 split | RFC 3339 timestamp parseability |
| TB-ST-4 | PO-V1, PO-V2 | `match` exhaustiveness | `type_system` | Confidence enum is closed | Enum variants |
| TB-ST-5 | PO-P1..PO-P5 | arbitrary UTF-8 input | `stdlib` | Rust stdlib String is total over UTF-8 | Serde round-trip inputs |
| TB-ST-6 | PO-V10 | const-evaluation | `const_expr` | const expressions evaluated at spec load | tables::* values |

**Verification:** the file should have 6 rows; each `obligation_id` matches a row in the obligations JSONL; `reviewer_disposition: pending` for all (the reviewer will set this on re-review).

---

### F-ST-006 — Reclassify map-key non-emptiness as BLOCKING

**File:** `proofs/storage-types-proof-plan.md` §3, §9

**Action:** The current §9 marks all three open refinement questions (non-emptiness, ExtractionCache.fields JSON object, mode_preference enumerated) as "NON-BLOCKING but recommended". The first of these is BEHAVIOR-AFFECTING because:
- `redb_store.rs:106`: `table.insert(hash, json.as_str())` uses `ExtractionCache::input_hash` as the redb table key.
- `redb_store.rs:141`: `table.insert(phase, json.as_str())` uses `LatticeCache::phase` as the key.
- `redb_store.rs:188`: `table.insert(answer.step_id.as_str(), json.as_str())` uses `AnswerRecord::step_id` as the key.

Empty keys are reachable through the public `new(...)` constructors (which take `String` without validation). An empty-key insertion would be retrievable only by passing the empty string, which is a real correctness issue (data corruption / lookup miss).

**Fix:** Elevate Q1 (non-emptiness on `step_id`, `input_hash`, `phase`) to BLOCKING in §9. Either:
- (a) Have rust-contract ratify non-emptiness as part of the contract, and add 1-3 Verus obligations (PO-V11a/b/c) that refine the `new` constructors to assert non-empty inputs.
- (b) Mark as `owner_approved_debt` with explicit `debt_ref: cl-5dp-debt-001` and a written rationale that the existing production callers never pass empty strings (cite `redb_store.rs:175` which constructs via `from_answer` and the upstream `Answer` validator in `domain/types.rs`).

Q2 (ExtractionCache.fields as JSON object) and Q3 (mode_preference enumerated) remain non-blocking observations since no consumer currently parses them as structured types.

**Verification:** §9 lists the reclassification; the plan or the obligations JSONL carry either a new obligation or an explicit debt row.

---

## Non-blocking observations (recommended fixes; do not block re-review)

### F-ST-007 — Cite cl-2vr in §11

**File:** `proofs/storage-types-proof-plan.md` §11

**Action:** Replace line 171 with a more nuanced statement that acknowledges cl-2vr as a project-wide gate. Suggested replacement:
> `types.rs` production code (lines 1-187) contains 0 panic-prone sites per the cl-2vr inventory (`rg -c '\.(unwrap|expect)\(|\bpanic!|\btodo!|\bunimplemented!' clarity-web/src/storage/types.rs` over the non-test range). The `#[cfg(test)] mod tests` module (lines 188-400) contains 11 `.expect()` sites under `#[allow(...)]` at lines 189-203. cl-2vr adds `[lints] workspace = true` to `clarity-web/Cargo.toml`; this module does not require remediation, but the module-scoped panic-freedom claim is contingent on cl-2vr closure as a project-wide gate per `formal-verification-report.md` §3.

### F-ST-008 — Test count 10 → 11

**File:** `proofs/storage-types-proof-plan.md` §1 line 33, §10 line 161

**Action:** Change "10 unit tests" to "11 unit tests" and "The 10 hand-written tests" to "The 11 hand-written tests".

### F-ST-009 — Bridge plan §8 informal → structured list

**File:** `proofs/storage-types-proof-plan.md` §8

**Action:** Replace the Markdown table with explicit `source_refs` and `behavior_test_refs` lists per obligation. Example for PO-V3:
- `source_refs`: `["clarity-web/src/storage/types.rs:59-67"]`
- `behavior_test_refs`: `["clarity-web/src/storage/types.rs:254 (test_answer_record_from_answer)"]`

This is the hand-off shape that `proof-to-implementation` will consume. The proof-to-implementation skill will produce the formal `rust-refinement-obligation/v1` rows; the plan's job is to provide the input shape.

### F-ST-010 — Strengthen non-applicability evidence for PO-Z1 and PO-F1

**File:** `proofs/storage-types-obligations.planned.jsonl`

**Action for PO-Z1:** Replace the current `assumptions` value with a single-paragraph citation of serde_json's documented contract and the absence of custom Deserialize impls in types.rs.

**Action for PO-F1:** Keep `not_applicable` but add a `notes` field stating: "Re-evaluate Flux applicability after PO-V7/PO-V9 are repaired per F-ST-004. If Verus's extern_spec story for chrono::DateTime::parse_from_rfc3339 cannot be made non-vacuous, Flux becomes the lighter tool."

### F-ST-011 — Add refinement_harness_refs for PO-V7/PO-V9

**File:** `proofs/storage-types-proof-plan.md` §8 (bridge plan)

**Action:** Add a `refinement_harness_refs` entry: `proofs/verus/storage_types_timestamp_refinement.rs` (planned), containing a Verus spec fn `is_rfc3339(s: &str) -> bool` with refinement linkage to `chrono::DateTime<Utc>`. Connected to F-ST-004; same fix path.

---

## Rerun instructions

After applying the blockers (F-ST-001 through F-ST-006) and any of the observations (F-ST-007 through F-ST-011) the planner chooses to address:

1. Re-invoke `proof-plan-reviewer` on the updated `proofs/storage-types-proof-plan.md` and `proofs/storage-types-obligations.planned.jsonl`.
2. The reviewer will read the new `proofs/verifier-lane-decisions.jsonl` (F-ST-003) and `proofs/storage-types-trusted-base.jsonl` (F-ST-005) and re-render `proofs/storage-types-verifier-lane-review.jsonl` with the 22 rows flipped from `rejected` to `accepted`.
3. The reviewer will emit an updated `proofs/storage-types-plan-review.md` with `STATUS: APPROVED` if all blockers are addressed, or `STATUS: REJECTED` with a new findings row if any blocker remains.

The repair pass should not require touching `types.rs` or any other production code. All artifacts are in `proofs/`.
