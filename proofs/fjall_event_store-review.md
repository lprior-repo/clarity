# Proof Review — `cl-kse` / `fjall_event_store`

| Field | Value |
|---|---|
| **Reviewer skill** | `proof-reviewer` |
| **Reviewer invocation ID** | `d8cfe57a-4c78-4293-b142-c725d84c334c` |
| **Review state** | `COMPLETED` |
| **Bead** | `cl-kse` (P0) |
| **Module** | `clarity-web/src/storage/fjall_event_store.rs` |
| **Date** | 2026-06-21 |
| **Lane(s) reviewed** | V (Verus) primary + K (Kani) secondary; T (TLA+) excluded by ownership |
| **Verdict** | **REJECTED** |

## Provenance

| Reviewed artifact | Path | SHA-256 |
|---|---|---|
| Verus spec/proof | `proofs/fjall_event_store_verus.rs` | `bf05f0e1813b3f2893bdb74098a1dc33c8335a2a77c175cbd709d99cfa6bb89f` |
| Kani harness source | `proofs/fjall_event_store_kani.rs` | `8e019ca142306b7b9a018755d1a8fdd0ec5579dc3580971b90bf273fd47c0fd7` |
| Writeup | `proofs/fjall_event_store-writeup.md` | `966c804d6b6dc9a6ffc76f44635b76050a6aadac65914def2f0f440a51d8ac74` |
| proof-writer report | `.beads/cl-kse/proof-writer-report.md` | `a83345e3374f2c295ecc4892e77c447dec191fcb119a4952bdc589b477af1d5a` |
| proof-evidence template | `.beads/cl-kse/proof-evidence.md` | `d27de4368a13bfa5da141b8fe6a7aa6660de5d3fb2bb1f0a33135e64a947b899` |
| Trusted-base ledger | `.beads/cl-kse/trusted-base-ledger.jsonl` | `b0e894d878784b116843b16f418de1521598502df65f853a859b4eeac1336481` |
| Supporting raw evidence | `/tmp/opencode/verus-fjall_event_store.txt` | `ec783f4423915fde38c7a4e83c88ae782f8feef365ca268da9001a4f6b6d6994` |
| Production source (not modified) | `clarity-web/src/storage/fjall_event_store.rs` | `3969c07213cce257fe1a9a773d91dbd7de796a42416dccaed624c63e2d2383e9` |

Producer: `proof-writer`. Independent reviewer: `proof-reviewer` (this run, invocation `d8cfe57a-…`).
Per `review-provenance.md`: writer ≠ reviewer invocation IDs; both pre-exist this review; no self-approval.

---

## TL;DR

**REJECTED.** The proof artifact is **not verifier-confirmed**: the raw `verus` log
(`/tmp/opencode/verus-fjall_event_store.txt`, `exit_status: 1`) shows three `E0308
mismatched types` errors at `fjall_event_store_verus.rs:264`, `:422`, and `:425`
(`Seq<char>` vs `Seq<u8>`). The proof-writer disclosed that one diagnostic
`verus` run was performed mid-session and the **final file was not re-verified**
(`proof-writer-report.md §3.1`).

Beyond the known type error, this review surfaces **four additional blocking
findings** the proof-writer did not flag:

1. **PO-V2 and PO-V3 proof bodies are empty** after a single helper call; the
   `forall |i| 0 <= i < 8 ⇒ key[16+i] == suffix[i]` quantifier that constitutes
   the actual obligation is **never discharged**. The "Verus's Seq theory
   handles this automatically" comment in the source is unsupported.
2. **`axiom_truncate_is_prefix` (TB-FJALL-002)** has an empty body and the
   forall quantifier in its postcondition (`forall|i| 0 <= i < 16 ⇒
   prefix[i] == sha256[i]`) is never established. The trust ledger marks this
   as "trusted" but the proof body suggests derivation. **Discrepancy.**
3. **`axiom_sha256_length`** has an empty body and depends on Verus's
   automatic unfolding of `Seq::new(SHA256_DIGEST_BYTES, …)`. This works in
   practice but is implicit; a `#![trigger]` annotation or explicit `assert`
   would harden it.
4. **Brief violation:** final file is not re-verified after the diagnostic
   edit. Per `proof-reviewer` skill rule §9 and `evidence-standards.md`,
   `PENDING_FORMAL_EXECUTION` without cheap smoke/typecheck evidence is a
   blocker.

The Kani side is clean: 9 harnesses, all call production functions directly,
zero anti-verification-laundering (`#[kani::stub]`, `assert(true)`, hardcoded
inputs). The trusted-base ledger is well-formed with 10 honest entries
covering the genuine external dependencies (sha2, `format!("{:x}", …)`,
`serde_json::to_vec`, BE encoding, execution gates).

---

## Findings (ordered by severity)

Schema: `finding/v1`. Disposition values are canonical
(`fixed_with_evidence`, `owner_approved_debt`, `owner_approved_no_action`).
A `blocker` disposition forces `STATUS: REJECTED`.

### F-1 — `Seq<char>` vs `Seq<u8>` type error at lines 264, 422, 425 (BLOCKER)

- **Code:** `E_VERUS_DISCONNECTED_SPEC`
- **Artifact:** `proofs/fjall_event_store_verus.rs`
- **Obligations:** PO-V7 (`append_event_key_invariant`), `lemma_append_event_key`
- **Severity:** **critical / blocker**
- **Evidence:** `/tmp/opencode/verus-fjall_event_store.txt:1-50` — three `E0308 mismatched types`
  errors at lines 264 (`spec_event_key(event.session_id, event.seq)`), 422, 425.
  The `spec_event_key` signature (line 122) is `(Seq<u8>, u64) -> Seq<u8>` but
  `EventEnvelopeView.session_id` (line 242) is `Seq<char>`.
- **Disposition:** `owner_approved_no_action` — known; tracked as bead `cl-55u`.
  Repair deferred to `proof-writer`.
- **Required fix:** Either (a) change `EventEnvelopeView.session_id: Seq<char>` →
  `Seq<u8>` (with a `seq_to_bytes` spec helper that mirrors `String::as_bytes()`),
  OR (b) thread `Seq<u8>` through the spec graph and re-introduce a
  `seq_chars_to_bytes` adapter at the `EventEnvelopeView` boundary. Option (a) is
  faithful to production (`session_key_prefix(session_id.as_bytes())`),
  consistent with the Kani harnesses (which call `event_key(&session_id: String,
  seq)` so the production-side `as_bytes()` is implicit), and removes the
  type-class mismatch entirely.
- **Verification status:** `verus` exit code **1** (recorded in
  `verification-ledger.jsonl:30`, `classification: FAIL_LOCAL`). The artifact on
  disk is the same file that produced these errors. **Not re-verified** after the
  brief-violating diagnostic run.

### F-2 — PO-V2 proof body is empty after helper call; forall not discharged (BLOCKER)

- **Code:** `E_VERUS_DISCONNECTED_SPEC`
- **Artifact:** `proofs/fjall_event_store_verus.rs`
- **Obligation:** PO-V2 (`key_suffix_invariant`)
- **Severity:** **critical / blocker**
- **Evidence:** `fjall_event_store_verus.rs:328-337`
  ```rust
  pub proof fn lemma_key_suffix(input: Seq<u8>, seq: u64)
      ensures
          key_suffix_invariant(input, seq),
  {
      lemma_key_length(input, seq);
      // The concatenation distributes indices linearly: position k in
      // the concatenation is the (k - prefix_len)-th element of the
      // suffix when k >= prefix_len. Verus's Seq theory handles this
      // automatically.
  }
  ```
  `key_suffix_invariant` (line 183) requires proving `forall |i: int| 0 <= i <
  U64_BE_BYTES ==> key[SESSION_HASH_BYTES as int + i] == suffix[i]`. The body
  contains zero `assert`, zero `forall`-intro, zero SMT hint. The comment
  "Verus's Seq theory handles this automatically" is not a Verus rule;
  Verus generally requires explicit `assert … by { … }` blocks for forall
  quantifiers on `Seq::add`.
- **Disposition:** `owner_approved_no_action` — requires proof-writer repair
  (notably the proof body must do real work; the comment is not a proof).
- **Required fix:** Either (a) introduce a `lemma_seq_add_distributes_indices`
  lemma that proves the linear distribution property of `Seq::add` (8 indices,
  one per byte) and call it from `lemma_key_suffix` and `lemma_key_prefix`, OR
  (b) inline `assert forall |i: int| 0 <= i < 8 implies key[16+i] == suffix[i]
  by { reveal(spec_event_key); reveal(seq_to_be_bytes_spec); }`. Either way,
  the proof body cannot be empty.

### F-3 — PO-V3 proof body is empty after helper call; forall not discharged (BLOCKER)

- **Code:** `E_VERUS_DISCONNECTED_SPEC`
- **Artifact:** `proofs/fjall_event_store_verus.rs`
- **Obligation:** PO-V3 (`key_prefix_invariant`)
- **Severity:** **critical / blocker**
- **Evidence:** `fjall_event_store_verus.rs:341-349`
  ```rust
  pub proof fn lemma_key_prefix(input: Seq<u8>, seq: u64)
      ensures
          key_prefix_invariant(input, seq),
  {
      lemma_key_length(input, seq);
      // Positions [0, SESSION_HASH_BYTES) in the concatenation are the
      // prefix positions directly. Verus's Seq theory handles this
      // automatically.
  }
  ```
  Same structural failure as F-2. `key_prefix_invariant` (line 195) requires
  `forall |i: int| 0 <= i < SESSION_HASH_BYTES ==> key[i] == prefix[i]`; the
  body does not establish it.
- **Disposition:** `owner_approved_no_action` — requires proof-writer repair.
- **Required fix:** Same as F-2: explicit `assert forall |i| … by { … }` block
  referencing `Seq::add` distribution, or a helper lemma.

### F-4 — `axiom_truncate_is_prefix` (TB-FJALL-002) has empty body; forall not discharged (BLOCKER)

- **Code:** `E_VERUS_DISCONNECTED_SPEC`, also `E_TRUST_LEDGER_INCOMPLETE`
- **Artifact:** `proofs/fjall_event_store_verus.rs:284-293`
- **Obligation:** TB-FJALL-002 (`axiom_truncate_is_prefix`)
- **Severity:** **high / blocker**
- **Evidence:**
  ```rust
  pub proof fn axiom_truncate_is_prefix(input: Seq<u8>)
      ensures
          spec_session_key_prefix(input).len() == SESSION_HASH_BYTES as nat,
          forall|i: int|
              #![trigger spec_session_key_prefix(input)[i]]
              0 <= i < SESSION_HASH_BYTES as int
                  ==> spec_session_key_prefix(input)[i] == spec_sha256(input)[i],
  {
      axiom_sha256_length(input);
  }
  ```
  The body calls only `axiom_sha256_length`. The first conjunct (length = 16)
  is discharged transitively. The forall quantifier (prefix equality at every
  index) is **never established** in the body. The trusted-base ledger entry
  TB-FJALL-002 claims this is "derived from TB-FJALL-001 plus the definition
  of `Seq::take`" but the proof body does not actually unfold `Seq::take`.
- **Disposition:** `owner_approved_no_action` — the trust record and the proof
  body disagree.
- **Required fix:** Either (a) prove the forall with an explicit
  `assert forall |i: int| 0 <= i < 16 implies spec_session_key_prefix(input)[i]
  == spec_sha256(input)[i] by { reveal(spec_session_key_prefix); }`, OR
  (b) update TB-FJALL-002 to honestly classify the forall as **trusted** (not
  "derived") and add `admit()` to the proof body. Option (a) is preferred; the
  property follows from `Seq::take`'s definition by SMT.
- **Anti-laundering note:** `axiom_truncate_is_prefix` is **not** marked
  `admit()` in source, yet its postcondition is only partially discharged.
  This is a **partial trust without ledger acknowledgement**.

### F-5 — `axiom_sha256_length` empty body relies on implicit unfolding (BLOCKER at closure)

- **Code:** `E_VERUS_DISCONNECTED_SPEC`
- **Artifact:** `proofs/fjall_event_store_verus.rs:274-280`
- **Obligation:** TB-FJALL-001
- **Severity:** **medium / blocker at closure**
- **Evidence:**
  ```rust
  pub proof fn axiom_sha256_length(input: Seq<u8>)
      ensures
          spec_sha256(input).len() == SHA256_DIGEST_BYTES as nat,
  {
      // The spec_sha256 body constructs `Seq::new(SHA256_DIGEST_BYTES, ...)`,
      // which has length exactly SHA256_DIGEST_BYTES by definition.
  }
  ```
  Empty body. In Verus the postcondition holds if and only if the SMT can
  unfold `spec_sha256` and resolve the `Seq::new(N, …).len() == N` lemma. This
  usually works but is fragile to ordering of `reveal`s. An
  `assert(spec_sha256(input).len() == SHA256_DIGEST_BYTES as nat);` would
  surface any unfolding issue at proof time.
- **Disposition:** `owner_approved_no_action` — TB-FJALL-001 is well-grounded
  in NIST FIPS 180-4 but the proof body should harden.
- **Required fix:** Add explicit `assert` of the postcondition before the
  closing `}`.

### F-6 — `lemma_key_distinct` logic gap on `else` branch (HIGH)

- **Code:** `E_VERUS_DISCONNECTED_SPEC`
- **Artifact:** `proofs/fjall_event_store_verus.rs:353-374`
- **Obligation:** PO-V4 (`key_distinct_invariant`)
- **Severity:** **high / blocker at closure**
- **Evidence:**
  ```rust
  pub proof fn lemma_key_distinct(input: Seq<u8>, seq_a: u64, seq_b: u64)
      requires
          seq_a != seq_b,
      ensures
          spec_event_key(input, seq_a) != spec_event_key(input, seq_b),
  {
      if spec_event_key(input, seq_a) == spec_event_key(input, seq_b) {
          // ...
          axiom_be_bytes_injective(seq_a, seq_b);
          assert(seq_to_be_bytes_spec(seq_a) != seq_to_be_bytes_spec(seq_b));
          assert(false);
      }
  }
  ```
  The `if` branch drives the SMT into contradiction via `assert(false)`. The
  `else` branch is implicit. Verus's SMT is generally able to discharge
  `b ==> b`-style tautologies after a contradictory `if`, but this construction
  relies on `axiom_be_bytes_injective` having its postcondition established
  (it does, via `admit()`). The proof body is structurally fragile:
  - If `axiom_be_bytes_injective`'s `admit()` were ever removed and the
    injectivity proved, the SMT would still need an explicit forall elimination
    of the equality chain `key_a == key_b ⇒ equal-prefix ⇒ equal-suffix ⇒
    BE(seq_a) == BE(seq_b)`.
  - The proof does **not** explicitly derive `key_a == key_b ⇒ BE(seq_a) ==
    BE(seq_b)`; it relies on `assert(false)` to terminate the branch.
- **Disposition:** `owner_approved_no_action`.
- **Required fix:** Use the `assert(…) by { … }` pattern with named lemmas
  rather than `if … assert(false)`. Concretely:
  ```rust
  assert(spec_event_key(input, seq_a) != spec_event_key(input, seq_b)) by {
      lemma_key_length(input, seq_a);
      lemma_key_length(input, seq_b);
      lemma_key_suffix(input, seq_a);
      lemma_key_suffix(input, seq_b);
      axiom_be_bytes_injective(seq_a, seq_b);
  };
  ```
  Then no `if` branch and no `assert(false)`.

### F-7 — Final file not re-verified after diagnostic edit (BLOCKER)

- **Code:** `E_PROOF_SMOKE_MISSING`, `E_FORMAL_PENDING_AT_CLOSURE`
- **Artifact:** `proofs/fjall_event_store_verus.rs` (post-edit)
- **Severity:** **high / blocker**
- **Evidence:**
  - `proof-writer-report.md §3.1` (lines 73-93): "The writer ran `verus
    proofs/fjall_event_store_verus.rs` **once** during this session to surface
    syntax errors … The errors were fixed in the source. **No further runs
    were performed**; the file was not executed against the final version."
  - `proofs/fjall_event_store-writeup.md §7` row 1: "`cargo verus` or `verus`
    actually run on this artifact: **No** — proof-writer brief says 'Do NOT
    run verus, kani, or cargo.' Source written; execution owned by
    `formal-verifier`."
  - `verification-ledger.jsonl:30` records the **intermediate** diagnostic
    run with `exit_status: 1`. There is no row for the **final** file.
- **Disposition:** `owner_approved_no_action` — owner (proof-writer) explicitly
  states "execution owned by formal-verifier". The brief violation is
  documented; the file's verifier status is **PENDING_FORMAL_EXECUTION** with
  no post-edit smoke evidence.
- **Required fix:** Either (a) `formal-verifier` runs `verus proofs/
  fjall_event_store_verus.rs` on the **final** file (post-F-1..F-6 repair)
  and the ledger row at `verification-ledger.jsonl:30` is updated, OR (b) the
  bead remains open until `proof-writer` does so and patches `proof-evidence.md`.

### F-8 — TLA+ (Lane T) deferred; write-batch durability unverified at this closure (MEDIUM)

- **Code:** `E_BEHAVIOR_WAIVER`
- **Artifact:** `proofs/fjall_event_store-writeup.md §4` (lines 162-174)
- **Severity:** **medium / requires owner-approved-debt at closure**
- **Evidence:** `verification-targets.md §5.2` lists TLA+ as the **secondary**
  lane for `storage/fjall_event_store.rs` covering "write-batch ack,
  recovery, snapshot-consistency temporal property". The writeup §4
  (lines 162-174) defers TLA+ to `tla-plus` as a separate bead. The Verus
  artifact explicitly does **not** cover `append_event_sync_all`'s I/O
  success path (`fjall_event_store_verus.rs:55-58`, writeup §2 PO-V7).
- **Disposition:** `owner_approved_debt` — acceptable per the lane
  ownership model (T is explicitly secondary, separate bead); but P0 closure
  of `cl-kse` requires the owner to acknowledge that write-batch
  durability remains unverified until TLA+ lands.
- **Required fix:** Add to `verification-ledger.jsonl` an
  `owner_approved_debt` row referencing `TB-FJALL-008` (model
  simplification) and the TLA+ bead once it exists.

### F-9 — Kani PO-K6 / PO-K7 input bounded to 64 bytes (LOW / documented)

- **Code:** `E_KANI_COVER_ONLY` (false positive; documented bound)
- **Artifact:** `proofs/fjall_event_store_kani.rs:247, 262`
- **Severity:** **low / not blocker**
- **Evidence:** Both harnesses use `kani::bounded_any::<_, 64>()`. TB-FJALL-009
  records this bound (64 bytes) and notes the property generalizes because
  `Sha256::digest` is total. Compensating: `kani::cover!(bytes.len() == 64)`
  is present (non-vacuity). Acceptable.
- **Disposition:** `owner_approved_no_action` — bound is documented and the
  `kani::cover!` calls prove the bound is reached.

### F-10 — `seq_to_be_bytes_spec` shift expression (LOW)

- **Code:** n/a (informational)
- **Artifact:** `proofs/fjall_event_store_verus.rs:127-131`
- **Severity:** **low**
- **Evidence:** The shift amount `((U64_BE_BYTES - 1 - i) * 8) as u64` is in
  range `{0, 8, 16, …, 56}` for `i ∈ 0..U64_BE_BYTES`. Safe. No `admit()`
  needed. The cast is necessary because the shift amount must be `u64` in
  Verus's spec context.
- **Disposition:** `owner_approved_no_action` — correct, no fix.

### F-11 — Kani harnesses PO-K4 / PO-K5 use `kani::assume` (LOW / legitimate)

- **Code:** `E_KANI_ASSUMPTION_VACUITY` (false positive; legitimate input-space constraint)
- **Artifact:** `proofs/fjall_event_store_kani.rs:200, 225`
- **Severity:** **low**
- **Evidence:** `kani::assume(seq_a != seq_b)` (PO-K4) and
  `kani::assume(seq_a < seq_b)` (PO-K5) constrain the input space to the
  non-trivial region. Without these the assertions (`assert_ne!`, `assert! <
  `) would be trivially true on the diagonal. Both harnesses pair the
  assumption with `kani::cover!` proving the non-trivial region is reached.
  **This is the correct Kani idiom.**
- **Disposition:** `owner_approved_no_action` — correct, no fix.

### F-12 — Trusted-base ledger entries TB-FJALL-006 / 007 / 010 have `reviewer_disposition: pending` (LOW)

- **Code:** `E_TRUST_PENDING_AT_CLOSURE`
- **Artifact:** `.beads/cl-kse/trusted-base-ledger.jsonl` (rows 6, 7, 10)
- **Severity:** **low / not blocker (bead is not closing in this review)**
- **Evidence:** TB-FJALL-006, TB-FJALL-007, TB-FJALL-010 carry
  `reviewer_disposition: "pending"` and are themselves blockers at closure.
- **Disposition:** `owner_approved_no_action` — appropriate; these are
  closure gates that will be flipped by `formal-verifier` after F-7 is
  resolved and by `rust-contract` after TB-FJALL-010 is closed.

---

## Cross-lane consistency

The writeup §6 specifies five cross-lane consistency pairs
(PO-V1 ↔ PO-K1, PO-V4 ↔ PO-K4, PO-V5 ↔ PO-K8, PO-V6 ↔ PO-K6, PO-V6 ↔ PO-K7).
None can be verified until the Verus artifact passes and the Kani harnesses
are wired and run. **Both gates are open.** No agreement / disagreement can
be asserted at this time.

---

## Anti-verification-laundering audit (mandatory)

| Pattern | Verus file | Kani file |
|---|---|---|
| `#[verifier::external_body]` | **0** (one match is a comment on line 32) | n/a |
| `assume(` | **0** | **2** — both `kani::assume(seq_a != seq_b)` and `kani::assume(seq_a < seq_b)`, legitimate input-space constraints (F-11) |
| `admit(` | **1** — `axiom_be_bytes_injective` at line 308, TB-FJALL-005 | n/a |
| `assert(true)` | n/a | **0** (no `assert!` calls use `true`) |
| `#[kani::stub]` / `#[kani::stub_verified]` | n/a | **0** |
| `#[kani::should_panic]` | n/a | **0** |
| Hardcoded structural inputs | n/a | **0** — all harnesses use `kani::any()` or `kani::bounded_any` |
| `cover!`-only proof | n/a | **0** — every cover is paired with an assertion |

**Verdict:** No anti-verification-laundering. The single `admit()` (TB-FJALL-005)
is honestly recorded. The two `kani::assume` calls constrain input space and
do not encode the desired result.

---

## Trusted-base ledger audit

All 10 entries (`TB-FJALL-001` through `TB-FJALL-010`) are present, schema
`trusted-base-ledger/v1`, and referenceable from the spec/proof artifact.
Per-entry disposition:

| TB ID | Honesty | Notes |
|---|---|---|
| TB-FJALL-001 (sha256 length) | ✅ | External trust, NIST FIPS 180-4. F-5: proof body should add explicit assert. |
| TB-FJALL-002 (truncate is prefix) | ⚠️ | Discrepancy between ledger ("derived") and proof body (effectively trusted). F-4. |
| TB-FJALL-003 (serde_json::to_vec) | ✅ | External trust, scope-limited to value side. |
| TB-FJALL-004 (format!("{:x}", …) hex encoding) | ✅ | External trust, stdlib. |
| TB-FJALL-005 (BE encoding injective) | ✅ | External trust + single `admit()`. Compensating Kani PO-K4/PO-K5. |
| TB-FJALL-006 (Verus not run final) | ✅ | Honestly recorded `BLOCKED_TOOLING`. |
| TB-FJALL-007 (Kani not run) | ✅ | Honestly recorded `BLOCKED_TOOLING`. |
| TB-FJALL-008 (View type drops fields) | ✅ | Documented model simplification. F-8: P0 closure requires debt acknowledgement. |
| TB-FJALL-009 (Kani 64-byte bound) | ✅ | Documented bound. |
| TB-FJALL-010 (no rust-contract) | ✅ | Honestly recorded. Closure gate for rust-contract. |

---

## Repair priority for `proof-writer`

| Order | Finding | Required action |
|---:|---|---|
| 1 | F-1 | Fix `Seq<char>` → `Seq<u8>` mismatch on lines 264, 422, 425. |
| 2 | F-2 / F-3 | Add explicit forall discharge in `lemma_key_suffix` and `lemma_key_prefix`. |
| 3 | F-4 | Either prove the forall in `axiom_truncate_is_prefix` or update TB-FJALL-002 to "trusted" + `admit()`. |
| 4 | F-5 | Add explicit `assert` in `axiom_sha256_length` body. |
| 5 | F-6 | Restructure `lemma_key_distinct` to use named lemma calls. |
| 6 | F-7 | Re-run `verus proofs/fjall_event_store_verus.rs` after repairs; capture `exit 0` and `verification-ledger.jsonl` row. |
| 7 | F-8 | Open or reference the TLA+ bead; add `owner_approved_debt` row for write-batch durability. |

---

## Counts

| Disposition | Count |
|---|---:|
| `blocker` (forces REJECT) | **7** — F-1, F-2, F-3, F-4, F-5 (closure-time), F-6 (closure-time), F-7 |
| High (non-blocker at this review) | **0** (folded into blockers) |
| Medium / requires debt acknowledgement | **1** — F-8 |
| Low / informational | **4** — F-9, F-10, F-11, F-12 |
| **Total findings** | **12** |
| **Blocking findings** | **7** |
| **Non-blocking findings** | **5** |

---

## Recommendation

**REJECT** the proof artifact. The seven blocking findings — five in the
Verus spec/proof structure (F-1..F-5, F-6) plus the missing post-edit
verification (F-7) — must be repaired and re-verified before the bead can
advance toward closure. The trust-base ledger is honest except for the
TB-FJALL-002 discrepancy (F-4). The Kani side is clean and the anti-
verification-laundering audit passes. The TLA+ lane (F-8) is acceptable
under the documented lane ownership model but must be acknowledged as
debt at P0 closure.

Re-submission after repair should include:

1. `verus proofs/fjall_event_store_verus.rs` exit 0 raw log in
   `/tmp/opencode/verus-fjall_event_store.retry.txt`.
2. Updated `verification-ledger.jsonl` row overwriting line 30 (or a
   sibling row) with `exit_status: 0` and `classification: PASS`.
3. TB-FJALL-002 either proven (with explicit forall) or honestly re-labeled
   as trusted with `admit()`.
4. F-8 owner-approved-debt row.

---

STATUS: REJECTED
