//! proptest properties for `clarity-web/src/storage/types.rs`.
//!
//! | Field | Value |
//! |---|---|
//! | Bead | `cl-5dp` |
//! | Target | `clarity-web/src/storage/types.rs` (401 LOC) |
//! | Primary lane | **P** (proptest) — round-trip on Serialize/Deserialize |
//! | Secondary lane | **V** (Verus) — see `types_verus.rs` |
//!
//! # Vacuity resolution (contract H-ST-2)
//!
//! PO-V7b (RFC 3339 parseability of `ProjectMetadata.created_at` / `updated_at` produced by
//! `with_current_timestamp`) and PO-V9b (same for `LatticeCache.timestamp`) are **vacuous
//! under `#[verifier::external_body]`** in the Verus spec.  This file provides the
//! **behavioural evidence** for those properties via `chrono::DateTime::parse_from_rfc3339`.
//!
//! The structural equality properties PO-V7a and PO-V9a (`created_at == updated_at`) are
//! covered by the Verus spec `types_verus.rs`.  The companion unit tests at
//! `types.rs:301` and `types.rs:341` additionally exercise the chrono parse as a runtime
//! sanity check.
//!
//! # Wiring note
//!
//! This file lives at `proofs/types_proptest.rs` per the proof-writer brief.
//! To execute it as a `cargo test` integration test, copy or move the file to
//! `clarity-web/tests/types_proptest.rs`.  The expected command then becomes:
//!
//! ```text
//! cargo test -p clarity-web --test types_proptest
//! ```
//!
//! Wiring requires no changes to production code; the existing
//! `[dev-dependencies] proptest = "1.10.0"` in `clarity-web/Cargo.toml` covers
//! the import.  The `#![allow(...)]` line below matches the convention used by
//! other integration-test files in `clarity-web/tests/`.
//!
//! # Source mapping
//!
//! Each property cites `clarity-web/src/storage/types.rs:LINE` against the
//! production function it constrains.
//!
//! # Anti-verification-laundering
//!
//! Every property invokes the production API via `use clarity_web::storage::types::*;`.
//! No property rewrites or shadows the production functions; no production
//! mutation; no test-of-a-test.
//!
//! # Trusted base
//!
//! | Trust | Why trusted | Mitigation |
//! |---|---|---|
//! | `serde_json` round-trip preserves values | Library contract; not our code | Each round-trip property re-parses the same JSON and asserts equality. |
//! | `proptest!` macro | Library contract; not our code | Standard proptest machinery; no custom shrinkers. |
//! | `chrono::DateTime::parse_from_rfc3339` | Library contract; not our code | Companion behaviour tests parse timestamps and assert `is_ok()`. |

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::float_cmp,
    clippy::needless_collect,
    clippy::match_same_arms
)]

use proptest::prelude::*;

use clarity_web::storage::types::{
    Confidence, AnswerRecord, ExtractionCache, ProjectMetadata, LatticeCache,
};

// ============================================================
// Generators
// ============================================================

/// Generate any `Confidence` variant uniformly.
fn arb_confidence() -> impl Strategy<Value = Confidence> {
    prop_oneof![
        Just(Confidence::High),     // types.rs:14
        Just(Confidence::Inferred),  // types.rs:17
        Just(Confidence::Uncertain), // types.rs:19
    ]
}

/// Generate an arbitrary `String` — includes empty, non-ASCII, and very long strings.
fn arb_string() -> impl Strategy<Value = String> {
    "[\\x00-\\x10FFFF]{0,256}".prop_map(|s| {
        let mut r = String::new();
        let mut chars: Vec<char> = s.chars().collect();
        for c in chars.drain(..) {
            r.push(c);
        }
        r
    })
}

/// Generate an arbitrary non-empty `String`.
fn arb_non_empty_string() -> impl Strategy<Value = String> {
    "[\\x00-\\x10FFFF]{1,256}".prop_map(|s| {
        let mut r = String::new();
        for c in s.chars() {
            r.push(c);
        }
        r
    })
}

/// Generate an `AnswerRecord` via the production `new` constructor.
fn arb_answer_record_via_new() -> impl Strategy<Value = AnswerRecord> {
    (
        arb_non_empty_string(), // step_id — non-empty per C-ST-4
        arb_string(),            // value
        arb_string(),            // timestamp (arbitrary string per C-ST-2)
        arb_confidence(),
        any::<bool>(),
    )
        .prop_map(|(step_id, value, timestamp, confidence, ai_generated)| {
            AnswerRecord::new(step_id, value, timestamp, confidence, ai_generated)
        })
}

/// Generate an `AnswerRecord` via `from_answer` constructor.
fn arb_answer_record_via_from_answer() -> impl Strategy<Value = AnswerRecord> {
    (
        arb_non_empty_string(), // step_id — non-empty per C-ST-4
        arb_string(),            // value
        arb_string(),            // timestamp
    )
        .prop_map(|(step_id, value, timestamp)| AnswerRecord::from_answer(step_id, value, timestamp))
}

/// Generate an `ExtractionCache` via `new`.
fn arb_extraction_cache_via_new() -> impl Strategy<Value = ExtractionCache> {
    (
        arb_non_empty_string(), // input_hash — non-empty per C-ST-6
        arb_string(),            // fields — arbitrary string per C-ST-7
        arb_string(),            // timestamp — arbitrary string per C-ST-5
    )
        .prop_map(|(input_hash, fields, timestamp)| ExtractionCache::new(input_hash, fields, timestamp))
}

/// Generate a `ProjectMetadata` via `new`.
fn arb_project_metadata_via_new() -> impl Strategy<Value = ProjectMetadata> {
    (
        arb_non_empty_string(), // mode_preference — non-empty per C-ST-10
        arb_non_empty_string(), // current_phase — non-empty per C-ST-10
        arb_string(),            // created_at — arbitrary string per C-ST-8
        arb_string(),            // updated_at — arbitrary string per C-ST-8
    )
        .prop_map(|(mode_preference, current_phase, created_at, updated_at)| {
            ProjectMetadata::new(mode_preference, current_phase, created_at, updated_at)
        })
}

/// Generate a `LatticeCache` via `new`.
fn arb_lattice_cache_via_new() -> impl Strategy<Value = LatticeCache> {
    (
        arb_non_empty_string(), // phase — non-empty per C-ST-13
        arb_string(),            // output_data — arbitrary string per C-ST-11
        arb_string(),            // timestamp — arbitrary string per C-ST-11
    )
        .prop_map(|(phase, output_data, timestamp)| LatticeCache::new(phase, output_data, timestamp))
}

// ============================================================
// PO-P1: Confidence round-trip (serde_json)
// Source: types.rs:11-20
// ============================================================

proptest! {
    /// PO-P1: `Confidence` round-trips through `serde_json::to_string` and
    /// `serde_json::from_str` for all three variants.
    /// Source: types.rs:11-20; contract C-ST-1.
    #[test]
    fn proptest_confidence_roundtrip(c in arb_confidence()) {
        let json = serde_json::to_string(&c);
        prop_assert!(json.is_ok(), "serialization failed for {c:?}");
        let json_str = json.unwrap();

        let parsed: Result<Confidence, _> = serde_json::from_str(&json_str);
        prop_assert!(parsed.is_ok(), "deserialization failed for {json_str}");
        prop_assert_eq!(parsed.unwrap(), c);
    }

    /// PO-P1 variant: serde name is lowercase.
    /// Source: types.rs:12 (`#[serde(rename_all = "lowercase")]`).
    #[test]
    fn proptest_confidence_serde_name_lowercase(c in arb_confidence()) {
        let json = serde_json::to_string(&c).unwrap();
        let expected_name = match c {
            Confidence::High => "\"high\"",
            Confidence::Inferred => "\"inferred\"",
            Confidence::Uncertain => "\"uncertain\"",
        };
        prop_assert_eq!(json.as_str(), expected_name);
    }
}

// ============================================================
// PO-P2: AnswerRecord round-trip (serde_json)
// Source: types.rs:24-68
// ============================================================

proptest! {
    /// PO-P2: `AnswerRecord` (via `new`) round-trips through serde_json.
    /// Source: types.rs:41-55; contract C-ST-2.
    #[test]
    fn proptest_answer_record_roundtrip(record in arb_answer_record_via_new()) {
        let json_result = serde_json::to_string(&record);
        prop_assert!(json_result.is_ok(), "serialization failed");
        let json_str = json_result.unwrap();

        let parsed: Result<AnswerRecord, _> = serde_json::from_str(&json_str);
        prop_assert!(parsed.is_ok(), "deserialization failed for {json_str}");
        prop_assert_eq!(parsed.unwrap(), record);
    }

    /// PO-P2: `from_answer` round-trips and preserves typestate.
    /// Source: types.rs:59-67; contract C-ST-3.
    #[test]
    fn proptest_answer_record_from_answer_roundtrip(record in arb_answer_record_via_from_answer()) {
        // Typestate: confidence == High && ai_generated == false
        prop_assert!(record.confidence == Confidence::High);
        prop_assert!(!record.ai_generated);

        let json_result = serde_json::to_string(&record);
        prop_assert!(json_result.is_ok());
        let json_str = json_result.unwrap();

        let parsed: Result<AnswerRecord, _> = serde_json::from_str(&json_str);
        prop_assert!(parsed.is_ok());
        prop_assert_eq!(parsed.unwrap(), record);
    }
}

// ============================================================
// PO-P3: ExtractionCache round-trip (serde_json)
// Source: types.rs:72-92
// ============================================================

proptest! {
    /// PO-P3: `ExtractionCache` round-trips through serde_json.
    /// Source: types.rs:85-91; contract C-ST-5.
    #[test]
    fn proptest_extraction_cache_roundtrip(cache in arb_extraction_cache_via_new()) {
        let json_result = serde_json::to_string(&cache);
        prop_assert!(json_result.is_ok(), "serialization failed");
        let json_str = json_result.unwrap();

        let parsed: Result<ExtractionCache, _> = serde_json::from_str(&json_str);
        prop_assert!(parsed.is_ok(), "deserialization failed for {json_str}");
        prop_assert_eq!(parsed.unwrap(), cache);
    }
}

// ============================================================
// PO-P4: ProjectMetadata round-trip (serde_json)
// Source: types.rs:96-136
// ============================================================

proptest! {
    /// PO-P4: `ProjectMetadata` (via `new`) round-trips through serde_json.
    /// Source: types.rs:111-123; contract C-ST-8.
    #[test]
    fn proptest_project_metadata_roundtrip(meta in arb_project_metadata_via_new()) {
        let json_result = serde_json::to_string(&meta);
        prop_assert!(json_result.is_ok(), "serialization failed");
        let json_str = json_result.unwrap();

        let parsed: Result<ProjectMetadata, _> = serde_json::from_str(&json_str);
        prop_assert!(parsed.is_ok(), "deserialization failed for {json_str}");
        prop_assert_eq!(parsed.unwrap(), meta);
    }

    /// PO-P4 companion: `with_current_timestamp` produces parseable timestamps.
    /// **This is the behavioural evidence for PO-V7b** (vacuous under Verus external_body).
    /// Source: types.rs:127-135; contract C-ST-9.
    #[test]
    fn proptest_project_metadata_with_current_timestamp_behaviour(
        mode_preference in arb_non_empty_string(),
        current_phase in arb_non_empty_string(),
    ) {
        use chrono::{DateTime, Utc};

        let before = Utc::now();
        let meta = ProjectMetadata::with_current_timestamp(mode_preference.clone(), current_phase.clone());
        let after = Utc::now();

        prop_assert_eq!(meta.mode_preference, mode_preference);
        prop_assert_eq!(meta.current_phase, current_phase);

        // PO-V7b: RFC 3339 parseability — behavioural evidence (vacuous in Verus spec)
        let created: Result<DateTime<Utc>, _> = DateTime::parse_from_rfc3339(&meta.created_at)
            .map(|dt| dt.with_timezone(&Utc));
        prop_assert!(created.is_ok(), "created_at is not RFC 3339: {}", meta.created_at);
        let updated: Result<DateTime<Utc>, _> = DateTime::parse_from_rfc3339(&meta.updated_at)
            .map(|dt| dt.with_timezone(&Utc));
        prop_assert!(updated.is_ok(), "updated_at is not RFC 3339: {}", meta.updated_at);

        // Both timestamps within [before, after] range
        let c = created.unwrap();
        let u = updated.unwrap();
        prop_assert!(c >= before && c <= after, "created_at {c} outside range");
        prop_assert!(u >= before && u <= after, "updated_at {u} outside range");

        // PO-V7a: structural equality (documented vacuity resolution) — after borrows are done
        prop_assert_eq!(meta.created_at, meta.updated_at);
    }
}

// ============================================================
// PO-P5: LatticeCache round-trip (serde_json)
// Source: types.rs:140-170
// ============================================================

proptest! {
    /// PO-P5: `LatticeCache` (via `new`) round-trips through serde_json.
    /// Source: types.rs:153-159; contract C-ST-11.
    #[test]
    fn proptest_lattice_cache_roundtrip(cache in arb_lattice_cache_via_new()) {
        let json_result = serde_json::to_string(&cache);
        prop_assert!(json_result.is_ok(), "serialization failed");
        let json_str = json_result.unwrap();

        let parsed: Result<LatticeCache, _> = serde_json::from_str(&json_str);
        prop_assert!(parsed.is_ok(), "deserialization failed for {json_str}");
        prop_assert_eq!(parsed.unwrap(), cache);
    }

    /// PO-P5 companion: `with_current_timestamp` produces a parseable RFC 3339 timestamp.
    /// **This is the behavioural evidence for PO-V9b** (vacuous under Verus external_body).
    /// Source: types.rs:163-169; contract C-ST-12.
    #[test]
    fn proptest_lattice_cache_with_current_timestamp_behaviour(
        phase in arb_non_empty_string(),
        output_data: String,
    ) {
        use chrono::{DateTime, Utc};

        let before = Utc::now();
        let cache = LatticeCache::with_current_timestamp(phase.clone(), output_data.clone());
        let after = Utc::now();

        prop_assert_eq!(cache.phase, phase);
        prop_assert_eq!(cache.output_data, output_data);

        // PO-V9b: RFC 3339 parseability — behavioural evidence (vacuous in Verus spec)
        let parsed: Result<DateTime<Utc>, _> = DateTime::parse_from_rfc3339(&cache.timestamp)
            .map(|dt| dt.with_timezone(&Utc));
        prop_assert!(parsed.is_ok(), "timestamp is not RFC 3339: {}", cache.timestamp);

        // Timestamp within [before, after] range
        let t = parsed.unwrap();
        prop_assert!(t >= before && t <= after, "timestamp {t} outside range");
    }
}
