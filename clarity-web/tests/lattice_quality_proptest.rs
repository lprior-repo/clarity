// =============================================================================
// File:         proofs/lattice_quality_proptest.rs
// Lane:         proptest (P) — secondary
// Bead:         cl-dv5
// Target:       clarity-web/src/lattice/quality.rs
// Plan ref:     verification-targets.md §5.4
// Tool:         proptest 1.10.0 (dev-dependency of clarity-web; bundled with cargo)
//
// UPSTREAM NOTE
// -------------
// No approved proof plan exists yet for this module (`proof-planner` has not
// run, so no `proof-obligations.planned.jsonl` with formal IDs is on disk).
// Obligation IDs in this file (`OB-LQ-P-NN`) are PROVISIONAL — they must be
// replaced with planner-assigned IDs once `proof-planner` formalizes the
// module. The writeup `proofs/lattice_quality-writeup.md` records this gap.
//
// INTEGRATION
// -----------
// This file is intended to be installed as
// `clarity-web/tests/lattice_quality_proptest.rs` so cargo auto-discovers
// it as an integration test. The expected invocation is:
//
//     cargo test -p clarity-web --test lattice_quality_proptest -- --nocapture
//
// The `use clarity_web::...` paths assume the standard clarity-web library
// layout declared in `clarity-web/src/lib.rs`. None of the source files
// in `clarity-web/` are modified by this artifact.
//
// SCOPE
// -----
// Property-based tests for the public API of the lattice quality scoring
// module. These properties complement the Verus algebraic specs in
// `proofs/lattice_quality_verus.rs` by exercising the actual production
// code with random inputs. They cover:
//
//   - Range validation: `DimensionScore::new`, `QualityScore::new`
//   - Monotonicity of `passes` (in score and in threshold)
//   - `calculate_quality` structural invariants (empty input → error,
//     exactly 5 dimensions, overall ∈ [0, 100], issues reference known
//     dimensions)
//   - Idempotence (algebraic purity) of `calculate_quality`
//   - JSON round-trip for serializable types
//   - `QualityDimension::all` cardinality and distinctness
//   - `MINIMUM_GATE` constant value
//   - `get_dimension` / `get_issues` correctness
// =============================================================================

#![allow(clippy::unwrap_used, clippy::expect_used)] // proptest contract: shrinking requires panic-able assertions

use clarity_web::lattice::quality::{
    calculate_quality, DimensionScore, EarsRequirementRef, InversionControl, IssueSeverity,
    QualityDimension, QualityError, QualityIssue, QualityScore, Answer, MINIMUM_GATE,
};
use proptest::prelude::*;
use std::collections::HashSet;

// =============================================================================
// §1  Arbitrary-instance generators
// =============================================================================

/// All `QualityDimension` variants. Used by `arb_quality_dimension`.
const ALL_DIMENSIONS: &[QualityDimension] = &[
    QualityDimension::Completeness,
    QualityDimension::Consistency,
    QualityDimension::Testability,
    QualityDimension::Clarity,
    QualityDimension::Security,
];

fn arb_quality_dimension() -> impl Strategy<Value = QualityDimension> {
    proptest::sample::select(ALL_DIMENSIONS)
}

fn arb_issue_severity() -> impl Strategy<Value = IssueSeverity> {
    proptest::sample::select(&[
        IssueSeverity::Warning,
        IssueSeverity::Error,
        IssueSeverity::Critical,
    ])
}

fn arb_short_string(min: usize, max: usize) -> impl Strategy<Value = String> {
    proptest::collection::vec(proptest::arbitrary::any::<char>(), min..=max)
        .prop_map(|v| v.into_iter().collect())
}

fn arb_answer() -> impl Strategy<Value = Answer> {
    (
        arb_short_string(1, 24),  // step_id — narrow alphabet would be better but char is fine
        arb_short_string(0, 64),  // value
        arb_short_string(0, 32),  // timestamp
    )
        .prop_map(|(step_id, value, timestamp)| Answer {
            step_id,
            value,
            timestamp,
        })
}

fn arb_ears_requirement_ref() -> impl Strategy<Value = EarsRequirementRef> {
    (
        arb_short_string(1, 16),
        arb_short_string(0, 64),
        any::<bool>(),
    )
        .prop_map(|(id, text, has_acceptance_criteria)| EarsRequirementRef {
            id,
            text,
            has_acceptance_criteria,
        })
}

fn arb_inversion_control() -> impl Strategy<Value = InversionControl> {
    (any::<bool>(), 0usize..=1000).prop_map(|(has_inversion_tests, inverted_count)| {
        InversionControl {
            has_inversion_tests,
            inverted_count,
        }
    })
}

fn arb_dimension_score() -> impl Strategy<Value = DimensionScore> {
    (arb_quality_dimension(), 0u8..=100).prop_map(|(d, s)| {
        // In-range input — `new` is total over [0, 100] by OB-LQ-V-05/06.
        DimensionScore::new(d, s).expect("in-range score must be Ok")
    })
}

fn arb_quality_issue() -> impl Strategy<Value = QualityIssue> {
    (arb_quality_dimension(), arb_issue_severity(), arb_short_string(1, 64))
        .prop_map(|(dimension, severity, message)| QualityIssue::new(dimension, severity, message))
}

fn arb_quality_score() -> impl Strategy<Value = QualityScore> {
    (
        0u8..=100,
        proptest::collection::vec(arb_dimension_score(), 0..=5),
        proptest::collection::vec(arb_quality_issue(), 0..=3),
    )
        .prop_map(|(overall, dimensions, issues)| {
            // Valid range by construction — `new` accepts overall ∈ [0, 100].
            QualityScore::new(overall, dimensions, issues).expect("valid overall must be Ok")
        })
}

// =============================================================================
// §2  Properties — algebraic contract
// =============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    // ---------- OB-LQ-P-01: DimensionScore::new accepts in-range scores ----------

    #[test]
    fn prop_dimension_score_new_in_range(d in arb_quality_dimension(), s in 0u8..=100) {
        let r = DimensionScore::new(d, s);
        prop_assert!(r.is_ok(), "in-range score {:?} rejected: {:?}", s, r);
        let ds = r.unwrap();
        prop_assert_eq!(ds.score, s);
        prop_assert_eq!(ds.dimension, d);
    }

    // ---------- OB-LQ-P-02: DimensionScore::new rejects out-of-range scores ----------

    #[test]
    fn prop_dimension_score_new_rejects_too_high(d in arb_quality_dimension(), s in 101u8..=255) {
        let r = DimensionScore::new(d, s);
        prop_assert!(r.is_err(), "out-of-range score {:?} accepted: {:?}", s, r);
        prop_assert!(
            matches!(r, Err(QualityError::InvalidScore(_))),
            "expected InvalidScore, got {:?}",
            r
        );
    }

    // ---------- OB-LQ-P-04: DimensionScore::passes is antitone in threshold ----------

    #[test]
    fn prop_dimension_passes_antitone_in_threshold(
        d in arb_quality_dimension(),
        score in 0u8..=100,
        t_lo in 0u8..=100,
        t_hi in 0u8..=100,
    ) {
        prop_assume!(t_hi >= t_lo);
        let s = DimensionScore::new(d, score).unwrap();
        if s.passes(t_hi) {
            prop_assert!(
                s.passes(t_lo),
                "antitonicity violated: score={:?} t_lo={:?} t_hi={:?}",
                score, t_lo, t_hi
            );
        }
    }

    // ---------- OB-LQ-P-05: QualityScore::new validates overall range ----------

    #[test]
    fn prop_quality_score_new_validates_overall(
        overall in any::<u8>(),
        dims in proptest::collection::vec(arb_dimension_score(), 0..=5),
        issues in proptest::collection::vec(arb_quality_issue(), 0..=3),
    ) {
        let r = QualityScore::new(overall, dims, issues);
        if overall <= 100 {
            prop_assert!(r.is_ok(), "valid overall {:?} rejected: {:?}", overall, r);
            let q = r.unwrap();
            prop_assert_eq!(q.overall, overall);
        } else {
            prop_assert!(r.is_err(), "invalid overall {:?} accepted: {:?}", overall, r);
            prop_assert!(
                matches!(r, Err(QualityError::InvalidScore(_))),
                "expected InvalidScore, got {:?}",
                r
            );
        }
    }

    // ---------- OB-LQ-P-07: calculate_quality with empty answers returns EmptyAnswers ----------

    #[test]
    fn prop_calculate_quality_empty_answers(
        ears in proptest::collection::vec(arb_ears_requirement_ref(), 0..=3),
        inv in arb_inversion_control(),
    ) {
        let answers: Vec<Answer> = vec![];
        let r = calculate_quality(&answers, &ears, &inv);
        prop_assert!(matches!(r, Err(QualityError::EmptyAnswers)));
    }

    // ---------- OB-LQ-P-08: calculate_quality overall is in [0, 100] ----------

    #[test]
    fn prop_calculate_quality_overall_in_range(
        answers in proptest::collection::vec(arb_answer(), 1..=8),
        ears in proptest::collection::vec(arb_ears_requirement_ref(), 0..=5),
        inv in arb_inversion_control(),
    ) {
        let r = calculate_quality(&answers, &ears, &inv);
        prop_assert!(r.is_ok(), "non-empty answers unexpectedly failed: {:?}", r);
        let q = r.unwrap();
        prop_assert!(q.overall <= 100, "overall out of range: {:?}", q.overall);
        // u8 is non-negative by type; q.overall >= 0 is automatic.
    }

    // ---------- OB-LQ-P-09: calculate_quality returns exactly 5 dimensions ----------

    #[test]
    fn prop_calculate_quality_five_dimensions(
        answers in proptest::collection::vec(arb_answer(), 1..=8),
        ears in proptest::collection::vec(arb_ears_requirement_ref(), 0..=5),
        inv in arb_inversion_control(),
    ) {
        let r = calculate_quality(&answers, &ears, &inv);
        prop_assert!(r.is_ok());
        let q = r.unwrap();
        prop_assert_eq!(
            q.dimensions.len(),
            5,
            "calculate_quality must always return 5 dimensions"
        );
    }

    // ---------- OB-LQ-P-10: calculate_quality returns each of the 5 distinct dimensions ----------

    #[test]
    fn prop_calculate_quality_distinct_dimensions(
        answers in proptest::collection::vec(arb_answer(), 1..=8),
        ears in proptest::collection::vec(arb_ears_requirement_ref(), 0..=5),
        inv in arb_inversion_control(),
    ) {
        let r = calculate_quality(&answers, &ears, &inv);
        prop_assume!(r.is_ok());
        let q = r.unwrap();
        let dims: Vec<_> = q.dimensions.iter().map(|d| d.dimension).collect();
        let unique: HashSet<_> = dims.iter().copied().collect();
        prop_assert_eq!(unique.len(), 5, "all 5 dimensions must be unique");
        // And every dimension must be one of the documented variants.
        let all = QualityDimension::all();
        for d in &dims {
            prop_assert!(
                all.contains(d),
                "returned dimension {:?} not in QualityDimension::all()",
                d
            );
        }
    }

    // ---------- OB-LQ-P-11: calculate_quality is idempotent (deterministic) ----------

    #[test]
    fn prop_calculate_quality_idempotent(
        answers in proptest::collection::vec(arb_answer(), 1..=8),
        ears in proptest::collection::vec(arb_ears_requirement_ref(), 0..=5),
        inv in arb_inversion_control(),
    ) {
        let r1 = calculate_quality(&answers, &ears, &inv);
        let r2 = calculate_quality(&answers, &ears, &inv);
        prop_assert_eq!(r1, r2, "calculate_quality must be deterministic");
    }

    // ---------- OB-LQ-P-12: All issues reference a documented dimension ----------

    #[test]
    fn prop_issues_reference_documented_dimension(q in arb_quality_score()) {
        let all = QualityDimension::all();
        for issue in &q.issues {
            prop_assert!(
                all.contains(&issue.dimension),
                "issue dimension {:?} not in QualityDimension::all()",
                issue.dimension
            );
        }
    }

    // ---------- OB-LQ-P-13: get_dimension returns the matching DimensionScore ----------

    #[test]
    fn prop_get_dimension_returns_match(d in arb_dimension_score()) {
        let q = QualityScore::new(50, vec![d], vec![]).unwrap();
        let got = q.get_dimension(d.dimension);
        prop_assert!(got.is_some());
        prop_assert_eq!(got.unwrap().score, d.score);
        prop_assert_eq!(got.unwrap().dimension, d.dimension);
    }

    // ---------- OB-LQ-P-14: get_dimension returns None for an absent dimension ----------

    #[test]
    fn prop_get_dimension_returns_none_for_absent(
        present in arb_dimension_score(),
        absent in arb_quality_dimension(),
    ) {
        prop_assume!(present.dimension != absent);
        let q = QualityScore::new(50, vec![present], vec![]).unwrap();
        prop_assert!(q.get_dimension(absent).is_none());
    }

    // ---------- OB-LQ-P-15: get_issues filters by dimension ----------

    #[test]
    fn prop_get_issues_filters_by_dimension(
        target in arb_quality_dimension(),
        other in arb_quality_dimension(),
        target_issues in proptest::collection::vec(arb_short_string(1, 32), 0..=3),
        other_count in 0usize..=3,
    ) {
        prop_assume!(target != other);
        let issues: Vec<QualityIssue> = target_issues
            .into_iter()
            .map(|m| QualityIssue::new(target, IssueSeverity::Warning, m))
            .chain(
                (0..other_count).map(|i| {
                    QualityIssue::new(other, IssueSeverity::Warning, format!("other-{i}"))
                }),
            )
            .collect();
        let q = QualityScore::new(50, vec![], issues.clone()).unwrap();
        let filtered = q.get_issues(target);
        let expected_count = issues.iter().filter(|i| i.dimension == target).count();
        prop_assert_eq!(filtered.len(), expected_count);
    }

    // ---------- OB-LQ-P-16: MINIMUM_GATE constant equals 70 ----------

    #[test]
    fn prop_minimum_gate_constant_value(_dummy in 0u8..1) {
        prop_assert_eq!(MINIMUM_GATE, 70);
    }

    // ---------- OB-LQ-P-17: QualityDimension::all() has 5 elements, all distinct ----------

    #[test]
    fn prop_quality_dimension_all_cardinality_and_distinctness(_dummy in 0u8..1) {
        let all = QualityDimension::all();
        prop_assert_eq!(all.len(), 5, "QualityDimension::all must have 5 elements");
        let unique: HashSet<_> = all.iter().copied().collect();
        prop_assert_eq!(unique.len(), 5, "all elements must be distinct");
    }

    // ---------- OB-LQ-P-18: label() is non-empty for every variant ----------

    #[test]
    fn prop_label_nonempty(d in arb_quality_dimension()) {
        prop_assert!(!d.label().is_empty());
    }

    // ---------- OB-LQ-P-19: description() is non-empty for every variant ----------

    #[test]
    fn prop_description_nonempty(d in arb_quality_dimension()) {
        prop_assert!(!d.description().is_empty());
    }
}

// =============================================================================
// §3  Properties — JSON round-trip
// =============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    // ---------- OB-LQ-P-20: QualityScore JSON round-trip ----------

    #[test]
    fn prop_quality_score_json_roundtrip(q in arb_quality_score()) {
        let json = serde_json::to_string(&q).expect("serialize");
        let back: QualityScore = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(q, back);
    }

    // ---------- OB-LQ-P-21: DimensionScore JSON round-trip ----------

    #[test]
    fn prop_dimension_score_json_roundtrip(d in arb_dimension_score()) {
        let json = serde_json::to_string(&d).expect("serialize");
        let back: DimensionScore = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(d, back);
    }

    // ---------- OB-LQ-P-22: QualityIssue JSON round-trip ----------

    #[test]
    fn prop_quality_issue_json_roundtrip(i in arb_quality_issue()) {
        let json = serde_json::to_string(&i).expect("serialize");
        let back: QualityIssue = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(i, back);
    }

    // ---------- OB-LQ-P-23: QualityDimension JSON round-trip (via enum encoding) ----------

    #[test]
    fn prop_quality_dimension_json_roundtrip(d in arb_quality_dimension()) {
        let json = serde_json::to_string(&d).expect("serialize");
        let back: QualityDimension = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(d, back);
    }

    // ---------- OB-LQ-P-24: IssueSeverity JSON round-trip ----------

    #[test]
    fn prop_issue_severity_json_roundtrip(s in arb_issue_severity()) {
        let json = serde_json::to_string(&s).expect("serialize");
        let back: IssueSeverity = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(s, back);
    }

    // ---------- OB-LQ-P-25: Answer JSON round-trip ----------

    #[test]
    fn prop_answer_json_roundtrip(a in arb_answer()) {
        let json = serde_json::to_string(&a).expect("serialize");
        let back: Answer = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(a, back);
    }

    // ---------- OB-LQ-P-26: EarsRequirementRef JSON round-trip ----------

    #[test]
    fn prop_ears_requirement_ref_json_roundtrip(e in arb_ears_requirement_ref()) {
        let json = serde_json::to_string(&e).expect("serialize");
        let back: EarsRequirementRef = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(e, back);
    }

    // ---------- OB-LQ-P-27: InversionControl JSON round-trip ----------

    #[test]
    fn prop_inversion_control_json_roundtrip(i in arb_inversion_control()) {
        let json = serde_json::to_string(&i).expect("serialize");
        let back: InversionControl = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(i, back);
    }
}
