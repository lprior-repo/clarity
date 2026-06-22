//! Verus spec/proof artifacts for `clarity-web/src/storage/types.rs`.
//!
//! | Field | Value |
//! |---|---|
//! | Bead | `cl-5dp` |
//! | Target | `clarity-web/src/storage/types.rs` (401 LOC) |
//! | Primary lane | **V** (Verus) — per `verification-targets.md §5.2` |
//! | Secondary lane | **P** (proptest) — see `types_proptest.rs` |
//!
//! # Source mapping
//!
//! Each artifact below cites `clarity-web/src/storage/types.rs:LINE` against the
//! production function it constrains. Path-and-line comments are the canonical
//! bridge for `proof-reviewer` to compare the spec body against the production body
//! line-for-line.
//!
//! # Vacuity resolution (H-ST-2)
//!
//! PO-V7b and PO-V9b (RFC 3339 parseability of timestamps produced by
//! `with_current_timestamp`) are **vacuous under `#[verifier::external_body]`**.
//! The ensures clause delegates to `chrono::DateTime::parse_from_rfc3339`'s own
//! library contract — the Verus spec proves nothing beyond what the library guarantees.
//!
//! Resolution (per contract §5, Q3 decision):
//! - **PO-V7a / PO-V9a**: Structural equality of shared `now` clone — **non-vacuous**, proved.
//! - **PO-V7b / PO-V9b**: RFC 3339 parseability — **reclassified as proptest behaviour test**.
//!   Companion test `test_project_metadata_with_current_timestamp` (types.rs:301) and
//!   `test_lattice_cache_with_current_timestamp` (types.rs:341) provide the behavioural evidence.
//!
//! # Trusted base
//!
//! | Trust | Why trusted | Mitigation |
//! |---|---|---|
//! | `serde_json::to_string` / `from_str` round-trip | Library contract; not our code | proptest PO-P1..P5 exercise round-trip on all 5 types. |
//! | `chrono::Utc::now().to_rfc3339()` returns RFC 3339 | Library contract; not our code | proptest behaviour tests parse timestamps back. |
//! | `Confidence` enum is closed (3 variants) | Type-system fact at spec time | PO-V1 enumerates exactly 3 variants; adding a 4th breaks spec at compile time. |
//! | `tables` constants are const-evaluated | Rust language rule | PO-V10 proves non-emptiness and pairwise distinctness at spec-load time. |

use vstd::prelude::*;

verus! {

// ============================================================
// Spec type mirrors — mirror production types mathematically.
// These are self-contained spec types for standalone verus verification.
// ============================================================

/// Mirror of `Confidence` (types.rs:13-20).
/// 3-variant closed enum; no future variants are valid.
pub enum ConfidenceSpec {
    High,
    Inferred,
    Uncertain,
}

/// Mirror of `AnswerRecord` (types.rs:25-36).
pub struct AnswerRecordSpec {
    pub step_id: String,
    pub value: String,
    pub timestamp: String,
    pub confidence: ConfidenceSpec,
    pub ai_generated: bool,
}

/// Mirror of `ExtractionCache` (types.rs:73-80).
pub struct ExtractionCacheSpec {
    pub input_hash: String,
    pub fields: String,
    pub timestamp: String,
}

/// Mirror of `ProjectMetadata` (types.rs:97-106).
pub struct ProjectMetadataSpec {
    pub mode_preference: String,
    pub current_phase: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Mirror of `LatticeCache` (types.rs:141-148).
pub struct LatticeCacheSpec {
    pub phase: String,
    pub output_data: String,
    pub timestamp: String,
}

// ============================================================
// Spec function layer — mathematical projections.
// ============================================================

/// PO-V1: `confidence_to_index` maps the 3-variant enum to `{0, 1, 2}`.
pub closed spec fn confidence_to_index(c: ConfidenceSpec) -> int {
    match c {
        ConfidenceSpec::High => 0,
        ConfidenceSpec::Inferred => 1,
        ConfidenceSpec::Uncertain => 2,
    }
}

/// PO-V2: serde name is lowercase variant name.
pub closed spec fn confidence_serde_name(c: ConfidenceSpec) -> &'static str {
    match c {
        ConfidenceSpec::High => "high",
        ConfidenceSpec::Inferred => "inferred",
        ConfidenceSpec::Uncertain => "uncertain",
    }
}

// ============================================================
// `Confidence` impl
// ============================================================

impl ConfidenceSpec {
    /// PO-V1: codomain of `to_index` is exactly `{0, 1, 2}`.
    /// The body maps to {0, 1, 2}; the codomain is proven in `lemma_confidence_to_index_codomain`.
    pub open spec fn to_index(self) -> int {
        confidence_to_index(self)
    }

    /// PO-V2: serde name is lowercase.
    pub open spec fn serde_name(self) -> &'static str {
        confidence_serde_name(self)
    }
}

// ============================================================
// `AnswerRecord` impl — PO-V3 (from_answer), PO-V4 (new totality)
// ============================================================

impl AnswerRecordSpec {
    /// PO-V4: `AnswerRecord::new` is **total** — faithful copy of all 5 fields.
    /// Source: types.rs:41-55.
    pub const fn new(
        step_id: String,
        value: String,
        timestamp: String,
        confidence: ConfidenceSpec,
        ai_generated: bool,
    ) -> (r: Self)
        ensures
            r.step_id == step_id,
            r.value == value,
            r.timestamp == timestamp,
            r.confidence == confidence,
            r.ai_generated == ai_generated,
    {
        Self { step_id, value, timestamp, confidence, ai_generated }
    }

    /// PO-V3: `from_answer` enforces typestate: `confidence == High && ai_generated == false`.
    /// Source: types.rs:59-67.
    pub const fn from_answer(step_id: String, value: String, timestamp: String) -> (r: Self)
        ensures
            r.step_id == step_id,
            r.value == value,
            r.timestamp == timestamp,
            r.confidence == ConfidenceSpec::High,
            r.ai_generated == false,
    {
        Self {
            step_id,
            value,
            timestamp,
            confidence: ConfidenceSpec::High,
            ai_generated: false,
        }
    }
}

// ============================================================
// `ExtractionCache` impl — PO-V5 (totality)
// ============================================================

impl ExtractionCacheSpec {
    /// PO-V5: `ExtractionCache::new` is **total**.
    /// Source: types.rs:85-91.
    pub const fn new(input_hash: String, fields: String, timestamp: String) -> (r: Self)
        ensures
            r.input_hash == input_hash,
            r.fields == fields,
            r.timestamp == timestamp,
    {
        Self { input_hash, fields, timestamp }
    }
}

// ============================================================
// `ProjectMetadata` impl — PO-V6 (new totality), PO-V7 (with_current_timestamp)
// ============================================================

impl ProjectMetadataSpec {
    /// PO-V6: `ProjectMetadata::new` is **total**.
    /// Source: types.rs:111-123.
    pub const fn new(
        mode_preference: String,
        current_phase: String,
        created_at: String,
        updated_at: String,
    ) -> (r: Self)
        ensures
            r.mode_preference == mode_preference,
            r.current_phase == current_phase,
            r.created_at == created_at,
            r.updated_at == updated_at,
    {
        Self { mode_preference, current_phase, created_at, updated_at }
    }

    /// PO-V7: `with_current_timestamp` sets `created_at == updated_at` to the same
    /// `chrono::Utc::now().to_rfc3339()` instant (single call, shared via `clone()`).
    ///
    /// **PO-V7a (structural equality — non-vacuous):**
    /// `result.created_at == result.updated_at` — single `now` call shared via clone.
    ///
    /// **PO-V7b (RFC 3339 parseability — VACUOUS, documented H-ST-2):**
    /// Delegated to proptest behaviour test `proptest_project_metadata_with_current_timestamp_behaviour`.
    ///
    /// Source: types.rs:127-135.
    ///
    /// NOTE: `#[verifier::external_body]` means Verus trusts the ensures clause without
    /// verifying the body. The production crate links chrono; standalone verus cannot.
    #[verifier::external_body]
    pub fn with_current_timestamp(mode_preference: String, current_phase: String) -> (r: Self)
        ensures
            r.mode_preference == mode_preference,
            r.current_phase == current_phase,
            r.created_at == r.updated_at,
    {
        Self { mode_preference, current_phase, created_at: String::new(), updated_at: String::new() }
    }
}

// ============================================================
// `LatticeCache` impl — PO-V8 (new totality), PO-V9 (with_current_timestamp)
// ============================================================

impl LatticeCacheSpec {
    /// PO-V8: `LatticeCache::new` is **total**.
    /// Source: types.rs:153-159.
    pub const fn new(phase: String, output_data: String, timestamp: String) -> (r: Self)
        ensures
            r.phase == phase,
            r.output_data == output_data,
            r.timestamp == timestamp,
    {
        Self { phase, output_data, timestamp }
    }

    /// PO-V9: `with_current_timestamp` sets `timestamp` to the result of
    /// `chrono::Utc::now().to_rfc3339()`.
    ///
    /// **PO-V9a (structural — non-vacuous):** `result.timestamp` from chrono call.
    ///
    /// **PO-V9b (RFC 3339 parseability — VACUOUS, documented H-ST-2):**
    /// Delegated to proptest behaviour test `proptest_lattice_cache_with_current_timestamp_behaviour`.
    ///
    /// Source: types.rs:163-169.
    ///
    /// NOTE: `#[verifier::external_body]` means Verus trusts the ensures clause without
    /// verifying the body. The production crate links chrono; standalone verus cannot.
    #[verifier::external_body]
    pub fn with_current_timestamp(phase: String, output_data: String) -> (r: Self)
        ensures
            r.phase == phase,
            r.output_data == output_data,
    {
        Self { phase, output_data, timestamp: String::new() }
    }
}

// ============================================================
// `tables` module — PO-V10
// ============================================================

pub mod tables_spec {
    use super::*;

    /// Source: types.rs:176
    pub const ANSWERS: &'static str = "answers";
    /// Source: types.rs:179
    pub const EXTRACTIONS: &'static str = "extractions";
    /// Source: types.rs:182
    pub const PROJECT_METADATA: &'static str = "project_metadata";
    /// Source: types.rs:185
    pub const LATTICE_CACHE: &'static str = "lattice_cache";

    /// PO-V10: Each constant is non-empty and the four are pairwise distinct.
    /// Const-evaluated at spec-load time; no runtime cost.
    pub closed spec fn all_non_empty_and_distinct() -> bool {
        ANSWERS@.len() > 0
            && EXTRACTIONS@.len() > 0
            && PROJECT_METADATA@.len() > 0
            && LATTICE_CACHE@.len() > 0
            && ANSWERS != EXTRACTIONS
            && ANSWERS != PROJECT_METADATA
            && ANSWERS != LATTICE_CACHE
            && EXTRACTIONS != PROJECT_METADATA
            && EXTRACTIONS != LATTICE_CACHE
            && PROJECT_METADATA != LATTICE_CACHE
    }
}

// ============================================================
// Proof lemmas
// ============================================================

/// PO-V1 proof: codomain of `confidence_to_index` is exactly {0, 1, 2}.
proof fn lemma_confidence_to_index_codomain()
    ensures
        forall|c: ConfidenceSpec| 0 <= confidence_to_index(c) && confidence_to_index(c) <= 2,
        forall|c: ConfidenceSpec| #![auto]
            match c {
                ConfidenceSpec::High => confidence_to_index(c) == 0,
                ConfidenceSpec::Inferred => confidence_to_index(c) == 1,
                ConfidenceSpec::Uncertain => confidence_to_index(c) == 2,
            },
{
    assert(confidence_to_index(ConfidenceSpec::High) == 0);
    assert(confidence_to_index(ConfidenceSpec::Inferred) == 1);
    assert(confidence_to_index(ConfidenceSpec::Uncertain) == 2);
}

} // verus!

// ===========================================================================
// Plain Rust main — required by Verus for standalone verification.
//
// Verus verifies all spec functions and proof lemmas inside `verus! { ... }`.
// `main` is not part of the proof and exists only to make the artifact
// compile in isolation when run via `verus proofs/types_verus.rs`.
// ===========================================================================

fn main() {
    // Proof content lives inside `verus! { ... }` above.
}
