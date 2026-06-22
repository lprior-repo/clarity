//! Property-based tests for `clarity-web::intent::parser`.
//!
//! Lane: **P** (proptest). Primary objective: round-trip + adversarial-input
//! properties on the public parser API. Companion lane **Z** (cargo-fuzz) lives
//! in `proofs/parser_fuzz/fuzz_targets/parse_dsl.rs` and is execution-gated on
//! `cl-u04` (cargo-fuzz install).
//!
//! These tests anchor to the documented public API of
//! `clarity-web/src/intent/parser.rs`:
//!
//! - `parse_spec(json: &str) -> Result<Spec, ParseError>`
//! - `parse_spec_from_value(value: &serde_json::Value) -> Result<Spec, ParseError>`
//! - `sanitize_string(s: &str) -> String`
//! - `validate_spec(spec: &Spec) -> Result<(), ParseError>`
//! - `ParseError` enum (4 variants)
//!
//! Output type `Spec` lives at `clarity-web/src/intent/types/spec.rs` and is
//! built from `Feature`, `Behavior`, `Invariant`, `AntiPattern`, and `AIHints`
//! in `clarity-web/src/intent/types/*`. This module relies on those types'
//! `Serialize`/`Deserialize`/`PartialEq` derives and never inspects
//! implementation internals.
//!
//! ## Obligation IDs (this artifact)
//!
//! - `PO-P1`  round-trip: `Spec → JSON → Spec` is identity
//! - `PO-P2`  arbitrary-AST round-trip: any well-formed arbitrary `Spec`
//!            serializes, parses back, and re-serializes to the same JSON
//! - `PO-P3`  `sanitize_string` is idempotent under composition
//! - `PO-P4`  `sanitize_string` removes every `'\0'`
//! - `PO-P5`  `sanitize_string` always returns a trimmed string
//! - `PO-P6`  `parse_spec` never panics on arbitrary bytes (panic-freedom)
//! - `PO-P7`  error classification: distinct malformed inputs map to the
//!            expected `ParseError` variant
//! - `PO-P8`  `parse_spec_from_value` agrees with `parse_spec` on
//!            string-form input when the JSON is well-formed
//! - `PO-P9`  validate_round_trip: a Spec that validates OK stays OK across
//!            a JSON round-trip; a Spec that fails validation round-trips
//!            to a Spec that fails validation identically
//! - `PO-P10` field preservation: round-trip preserves every optional field,
//!            including default-empty ones
//!
//! ## Execution
//!
//! Expected command (gated on the artifact being reachable from `clarity-web`'s
//! test target; see `proofs/parser-writeup.md` for the wiring story):
//!
//! ```text
//! cargo test -p clarity-web --test parser_proptest -- --nocapture
//! ```
//!
//! Until the file is wired into `clarity-web`'s `[[test]]` table (by infra /
//! holzman-rust) the artifact exists but cannot be executed; that is recorded
//! as `BLOCKED_TOOLING` in `proofs/parser-writeup.md` §4.
//!
//! ## Anti-Laundering
//!
//! Every property below calls the **production** parser entry point
//! (`parse_spec`, `parse_spec_from_value`, `sanitize_string`, `validate_spec`)
//! directly. No local "model" of the parser is built. The generators are
//! *consumers* of the public types, not re-implementations of them.

#![allow(
  clippy::unwrap_used,
  clippy::expect_used,
  clippy::panic,
  clippy::float_cmp,
  clippy::needless_collect,
  clippy::match_same_arms,
  clippy::option_if_let_else,
  clippy::suspicious_else_formatting,
  clippy::manual_let_else,
  clippy::match_wild_err_arm,
  clippy::match_like_matches_macro,
  clippy::needless_pass_by_value,
  clippy::redundant_closure_for_method_calls
)]

use proptest::prelude::*;
use serde_json::{json, Value};

// Public API of the parser under test.
use clarity_web::intent::parser::{
  parse_spec, parse_spec_from_value, sanitize_string, validate_spec, ParseError,
};
use clarity_web::intent::types::{
  AIHints, AntiPattern, Behavior, Feature, Invariant, Spec,
};

// =========================================================================
// Generators
// =========================================================================
//
// These are bounded, deterministic, and **produce values that are guaranteed
// to parse cleanly when serialized back to JSON**. They are not general
// `Arbitrary` impls for `Spec` (which would risk producing unparseable
// strings, defeating the round-trip property); they are *round-trip-safe*
// generators that respect the documented contract:
//   - `Spec.name` is non-empty after `.trim()`
//   - `Feature.name` is non-empty
//   - `Behavior.name` is non-empty (the JSON layer does not enforce
//     snake_case; only the constructor `Behavior::new` does)
//
// The upper bounds are conservative to keep `cargo test` fast and avoid the
// `Spec::validate` cardinality ceilings (100 features / invariants /
// anti-patterns, 50 behaviors per feature, 20 dependencies per feature).
// These ceilings are runtime-checked by `Spec::validate`, not by the parser.
// Properties whose correctness depends on those ceilings are out of scope
// for the parser lane; they belong to the validation lane (`intent/validation/`).

const MAX_FEATURES: usize = 4;
const MAX_BEHAVIORS_PER_FEATURE: usize = 3;
const MAX_INVARIANTS: usize = 3;
const MAX_ANTI_PATTERNS: usize = 3;
const MAX_DEPS_PER_FEATURE: usize = 2;
const MAX_LIST_STRINGS: usize = 4;

/// A non-empty, non-whitespace, JSON-safe string. No control chars, no
/// nulls, no leading/trailing whitespace.
fn arb_spec_name() -> impl Strategy<Value = String> {
  "[A-Za-z][A-Za-z0-9_-]{0,15}".prop_map(String::from)
}

fn arb_opt_string() -> impl Strategy<Value = String> {
  "[A-Za-z0-9 ,._!?\\-]{0,40}".prop_map(String::from)
}

fn arb_short_string() -> impl Strategy<Value = String> {
  "[A-Za-z0-9_-]{0,20}".prop_map(String::from)
}

fn arb_behavior_name() -> impl Strategy<Value = String> {
  // Lowercase letters, digits, underscores — matches the `snake_case`
  // contract enforced by `Behavior::new`. We do NOT use the constructor
  // here because the JSON path does not require snake_case; we just want
  // round-trip safety.
  "[a-z][a-z0-9_]{0,15}".prop_map(String::from)
}

fn arb_feature_name() -> impl Strategy<Value = String> {
  "[A-Za-z][A-Za-z0-9_-]{0,15}".prop_map(String::from)
}

fn arb_anti_pattern_name() -> impl Strategy<Value = String> {
  "[A-Za-z][A-Za-z0-9_-]{0,15}".prop_map(String::from)
}

fn arb_invariant_name() -> impl Strategy<Value = String> {
  "[A-Za-z][A-Za-z0-9_-]{0,15}".prop_map(String::from)
}

/// An arbitrary `Behavior` that will round-trip safely through JSON.
fn arb_behavior() -> impl Strategy<Value = Behavior> {
  (
    arb_behavior_name(),
    arb_opt_string(),
    proptest::option::of(arb_short_string()),
    proptest::collection::vec(arb_short_string(), 0..MAX_LIST_STRINGS),
    proptest::collection::vec(arb_short_string(), 0..MAX_LIST_STRINGS),
  )
    .prop_map(|(name, description, verification_type, pre, post)| Behavior {
      name,
      description,
      verification: verification_type.map(|t| {
        use clarity_web::intent::types::Verification;
        Verification::new(t, String::new())
      }),
      preconditions: pre,
      postconditions: post,
    })
}

/// An arbitrary `Feature` that will round-trip safely through JSON.
fn arb_feature() -> impl Strategy<Value = Feature> {
  (
    arb_feature_name(),
    arb_opt_string(),
    proptest::collection::vec(arb_behavior(), 0..=MAX_BEHAVIORS_PER_FEATURE),
    proptest::collection::vec(arb_feature_name(), 0..=MAX_DEPS_PER_FEATURE),
  )
    .prop_map(|(name, description, behaviors, depends_on)| Feature {
      name,
      description,
      behaviors,
      depends_on,
    })
}

/// An arbitrary `Invariant` that will round-trip safely through JSON.
fn arb_invariant() -> impl Strategy<Value = Invariant> {
  (
    arb_invariant_name(),
    arb_opt_string(),
    arb_opt_string(),
  )
    .prop_map(|(name, description, constraint)| Invariant {
      name,
      description,
      constraint,
    })
}

/// An arbitrary `AntiPattern` that will round-trip safely through JSON.
fn arb_anti_pattern() -> impl Strategy<Value = AntiPattern> {
  (
    arb_anti_pattern_name(),
    arb_opt_string(),
    arb_opt_string(),
    arb_opt_string(),
  )
    .prop_map(|(name, description, why_avoid, alternative)| AntiPattern {
      name,
      description,
      why_avoid,
      alternative,
    })
}

/// An arbitrary `AIHints` (preferred_libraries + style_hints varied;
/// remaining fields use the public `Default` impl). No private state is
/// touched.
fn arb_ai_hints() -> impl Strategy<Value = AIHints> {
  (
    proptest::collection::vec(arb_short_string(), 0..=MAX_LIST_STRINGS),
    proptest::collection::vec(arb_short_string(), 0..=MAX_LIST_STRINGS),
  )
    .prop_map(|(preferred_libraries, style_hints)| AIHints {
      preferred_libraries,
      style_hints,
      ..AIHints::default()
    })
}

/// An arbitrary `Spec` that will round-trip safely through the parser.
fn arb_spec() -> impl Strategy<Value = Spec> {
  (
    arb_spec_name(),
    arb_opt_string(),
    proptest::collection::vec(arb_feature(), 0..=MAX_FEATURES),
    proptest::collection::vec(arb_invariant(), 0..=MAX_INVARIANTS),
    proptest::collection::vec(arb_anti_pattern(), 0..=MAX_ANTI_PATTERNS),
    arb_ai_hints(),
  )
    .prop_map(|(name, description, features, invariants, anti_patterns, ai_hints)| Spec {
      name,
      description,
      features,
      invariants,
      anti_patterns,
      ai_hints,
    })
}

/// A JSON `Value` that is *not* an object — for the
/// `ParseError::InvalidType { field: "root" }` property.
fn arb_non_object_value() -> impl Strategy<Value = Value> {
  prop_oneof![
    Just(Value::Null),
    Just(Value::Bool(true)),
    Just(Value::Bool(false)),
    any::<i64>().prop_map(|n| Value::Number(n.into())),
    arb_opt_string().prop_map(Value::String),
    proptest::collection::vec(arb_opt_string(), 0..4).prop_map(Value::Array),
  ]
}

// =========================================================================
// PO-P1 — Spec → JSON → Spec round-trip is identity
// =========================================================================

proptest! {
  /// **PO-P1** Round-trip on a generated valid `Spec` is identity.
  ///
  /// Production API used: `serde_json::to_string(&spec)` →
  /// `clarity_web::intent::parser::parse_spec(&json)` →
  /// `Spec::eq(&original)`.
  ///
  /// This is the canonical "round-trip property" of the parser lane.
  #[test]
  fn proptest_p1_roundtrip(spec in arb_spec()) {
    let json = serde_json::to_string(&spec)
      .expect("serialize_spec");
    let parsed = parse_spec(&json)
      .expect("parse_spec must succeed for generated valid input");

    // Parser rebuilds the exact same Spec value. Because `Spec`, `Feature`,
    // `Behavior`, `Invariant`, `AntiPattern`, `AIHints` all derive
    // `PartialEq`, `Eq`, and `Serialize`/`Deserialize`, identity is the
    // strongest round-trip assertion we can make without inventing
    // language semantics.
    prop_assert_eq!(parsed, spec);

    // `name` in particular must round-trip exactly (it is the only
    // required field the parser enforces independently of serde).
    prop_assert_eq!(parsed.name, spec.name);
  }
}

// =========================================================================
// PO-P2 — Re-serialization is stable
// =========================================================================

proptest! {
  /// **PO-P2** Two consecutive parses of the same JSON produce the same
  /// Spec, and re-serializing yields the same JSON.
  ///
  /// Production API used: `parse_spec` and `serde_json::to_string`.
  /// Catches: non-deterministic map ordering, hidden mutable state in the
  /// parser, accidental cloning with side effects.
  #[test]
  fn proptest_p2_reserialize_stable(spec in arb_spec()) {
    let json1 = serde_json::to_string(&spec).expect("serialize once");
    let parsed1 = parse_spec(&json1).expect("parse once");

    let json2 = serde_json::to_string(&parsed1).expect("serialize twice");
    let parsed2 = parse_spec(&json2).expect("parse twice");

    prop_assert_eq!(&json1, &json2);
    prop_assert_eq!(&parsed1, &parsed2);
  }
}

// =========================================================================
// PO-P3, PO-P4, PO-P5 — sanitize_string properties
// =========================================================================

proptest! {
  /// **PO-P3** `sanitize_string` is idempotent under composition.
  ///
  /// Production API used: `clarity_web::intent::parser::sanitize_string`.
  /// The implementation is a one-pass filter + trim, so this property
  /// must hold by construction; the property pins the contract.
  #[test]
  fn proptest_p3_sanitize_idempotent(input in ".*") {
    let once = sanitize_string(&input);
    let twice = sanitize_string(&once);
    prop_assert_eq!(&once, &twice);
  }

  /// **PO-P4** `sanitize_string` removes every null byte from its input.
  ///
  /// Production API used: `sanitize_string`. The implementation strips
  /// `'\0'` by `.chars().filter(|&c| c != '\0')`, so this property must
  /// hold by construction.
  #[test]
  fn proptest_p4_sanitize_strips_nulls(input in ".*") {
    let cleaned = sanitize_string(&input);
    prop_assert!(!cleaned.contains('\0'),
      "sanitize_string output contains a null byte: {:?}", cleaned);
  }

  /// **PO-P5** `sanitize_string` always returns a trimmed string.
  ///
  /// Production API used: `sanitize_string`. The implementation applies
  /// `.trim()` to the filtered result, so this property must hold.
  #[test]
  fn proptest_p5_sanitize_trims(input in ".*") {
    let cleaned = sanitize_string(&input);
    prop_assert_eq!(&cleaned, cleaned.trim());
  }
}

// =========================================================================
// PO-P6 — parse_spec never panics on arbitrary bytes
// =========================================================================

proptest! {
  /// **PO-P6** `parse_spec` is panic-free on arbitrary input.
  ///
  /// Production API used: `parse_spec`. We rely on the documented
  /// design principle that "All fallible operations return `Result<T, E>`"
  /// (parser.rs lines 6–9). The property does not assert any particular
  /// outcome — only that we always get back a `Result`, never a panic.
  #[test]
  fn proptest_p6_parse_spec_no_panic(input in ".*") {
    let _ = parse_spec(&input);
    // Reaching here is success. No assertions on the Result are made
    // because they belong to PO-P7 (error classification) below.
  }
}

// =========================================================================
// PO-P7 — error classification
// =========================================================================

proptest! {
  /// **PO-P7a** A structurally invalid JSON literal (object opened but
  /// never closed) yields `ParseError::JsonError`. We parameterize on a
  /// short prefix so the assertion is exercised across many shape
  /// variants rather than a single hand-written payload.
  #[test]
  fn proptest_p7a_malformed_json_is_json_error(prefix in "[A-Za-z0-9 _]{0,8}") {
    let bad = format!("{prefix}{{no_close_brace");
    let result = parse_spec(&bad);
    prop_assert!(result.is_err(), "expected error, got {:?}", result);
    prop_assert!(
      matches!(result, Err(ParseError::JsonError(_))),
      "expected ParseError::JsonError, got {:?}", result
    );
  }

  /// **PO-P7b** A JSON value that is not an object yields
  /// `ParseError::InvalidType { field: "root", .. }`.
  #[test]
  fn proptest_p7b_non_object_root_is_invalid_type(value in arb_non_object_value()) {
    let result = parse_spec_from_value(&value);
    prop_assert!(result.is_err(), "expected error, got {:?}", result);
    let err = result.expect_err("err");
    match err {
      ParseError::InvalidType { ref field, .. } => {
        prop_assert_eq!(field.as_str(), "root");
      }
      other => prop_assert!(false, "expected InvalidType{{root, ..}}, got {:?}", other),
    }
  }

  /// **PO-P7c** A JSON object missing the `name` field yields
  /// `ParseError::MissingField("name")`.
  #[test]
  fn proptest_p7c_missing_name_is_missing_field(
    description in arb_opt_string()
  ) {
    let v = json!({ "description": description });
    let result = parse_spec_from_value(&v);
    let err = result.expect_err("missing name must fail");
    prop_assert!(
      matches!(err, ParseError::MissingField(ref f) if f == "name"),
      "expected MissingField(name), got {:?}", err
    );
  }

  /// **PO-P7d** Whitespace-only `name` yields `ParseError::EmptyField("name")`.
  #[test]
  fn proptest_p7d_whitespace_name_is_empty_field(
    ws in "[ \\t]{1,5}"
  ) {
    let json = format!(r#"{{"name": {:?}}}"#, ws);
    let result = parse_spec(&json);
    let err = result.expect_err("whitespace name must fail");
    prop_assert!(
      matches!(err, ParseError::EmptyField(ref f) if f == "name"),
      "expected EmptyField(name), got {:?}", err
    );
  }

  /// **PO-P7e** A `name` of the wrong JSON type (number) yields
  /// `ParseError::InvalidType { field: "name", expected: "string", .. }`.
  #[test]
  fn proptest_p7e_wrong_name_type_is_invalid_type(n in any::<i64>()) {
    let v = json!({ "name": n });
    let result = parse_spec_from_value(&v);
    let err = result.expect_err("number name must fail");
    match err {
      ParseError::InvalidType { ref field, ref expected, .. } => {
        prop_assert_eq!(field.as_str(), "name");
        prop_assert_eq!(expected.as_str(), "string");
      }
      other => prop_assert!(false, "expected InvalidType{{name, string, ..}}, got {:?}", other),
    }
  }
}

// =========================================================================
// PO-P8 — parse_spec_from_value agrees with parse_spec on string input
// =========================================================================

proptest! {
  /// **PO-P8** When the input JSON is a single object with `name` (and
  /// possibly other fields), `parse_spec_from_value` and `parse_spec`
  /// produce the same `Spec`.
  ///
  /// Production API used: both public entry points.
  /// Catches: divergence between the two parsing paths.
  #[test]
  fn proptest_p8_from_value_matches_parse_spec(spec in arb_spec()) {
    let json = serde_json::to_string(&spec).expect("serialize");
    let from_string = parse_spec(&json).expect("parse from string");

    let value: Value = serde_json::from_str(&json).expect("re-parse to Value");
    let from_value = parse_spec_from_value(&value).expect("parse from Value");

    prop_assert_eq!(from_value, from_string);
    prop_assert_eq!(from_value, spec);
  }
}

// =========================================================================
// PO-P9 — validate_round_trip: validation outcome is preserved
// =========================================================================

proptest! {
  /// **PO-P9** If a `Spec` validates OK, its round-tripped copy also
  /// validates OK.
  ///
  /// Production API used: `validate_spec` after `parse_spec`.
  /// The property uses a fixed-shape spec (one feature, default fields)
  /// so that `validate_spec` returns `Ok`; we then check the
  /// round-tripped copy is also `Ok`.
  #[test]
  fn proptest_p9_validate_round_trip(_unused: ()) {
    let forced = Spec {
      name: "rt-valid".to_string(),
      description: String::new(),
      features: vec![Feature {
        name: "auth".to_string(),
        description: String::new(),
        behaviors: vec![],
        depends_on: vec![],
      }],
      invariants: vec![],
      anti_patterns: vec![],
      ai_hints: AIHints::default(),
    };

    let json = serde_json::to_string(&forced).expect("serialize");
    let parsed = parse_spec(&json).expect("parse");
    prop_assert!(validate_spec(&parsed).is_ok(),
      "validate_spec must succeed for a round-tripped valid spec");
  }

  /// **PO-P9b** A Spec whose `features` is empty fails `validate_spec`
  /// before and after a JSON round-trip, with the **same** variant —
  /// not just the same constructor, but the same value (because
  /// `ParseError` derives `PartialEq`).
  #[test]
  fn proptest_p9b_validate_empty_features_preserved(name in arb_spec_name()) {
    let empty = Spec {
      name,
      description: String::new(),
      features: vec![],
      invariants: vec![],
      anti_patterns: vec![],
      ai_hints: AIHints::default(),
    };
    let before_err = validate_spec(&empty).expect_err("empty features must fail");

    let json = serde_json::to_string(&empty).expect("serialize");
    let parsed = parse_spec(&json).expect("parse");
    let after_err = validate_spec(&parsed).expect_err("still must fail");

    // `ParseError: PartialEq + Eq`, so we can compare directly.
    prop_assert_eq!(
      after_err, before_err,
      "validate_spec error must round-trip identically"
    );
  }
}

// =========================================================================
// PO-P10 — field preservation
// =========================================================================

proptest! {
  /// **PO-P10** Every optional field with a default value is preserved
  /// through a round-trip (description, features, invariants,
  /// anti_patterns, ai_hints).
  ///
  /// Production API used: `parse_spec` and `Spec`'s `PartialEq` derive.
  /// Catches: silent dropping of defaulted fields, missed `#[serde(default)]`
  /// attributes in the parser's serde path.
  #[test]
  fn proptest_p10_field_preservation(name in arb_spec_name()) {
    let original = Spec {
      name,
      description: String::new(),
      features: vec![],
      invariants: vec![],
      anti_patterns: vec![],
      ai_hints: AIHints::default(),
    };
    let json = serde_json::to_string(&original).expect("serialize");
    let parsed = parse_spec(&json).expect("parse");

    prop_assert_eq!(parsed.description, original.description);
    prop_assert_eq!(parsed.features.len(), original.features.len());
    prop_assert_eq!(parsed.invariants.len(), original.invariants.len());
    prop_assert_eq!(parsed.anti_patterns.len(), original.anti_patterns.len());
    prop_assert_eq!(parsed.ai_hints, original.ai_hints);
  }
}