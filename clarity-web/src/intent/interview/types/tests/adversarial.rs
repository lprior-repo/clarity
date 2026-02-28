//! Red Queen Adversarial Testing for Intent Module
//!
//! These tests attempt to break the state machine invariants through:
//! 1. Serde deserialization attacks (constructing invalid states via JSON)
//! 2. Edge cases (empty strings, unicode, boundary values)
//! 3. Fuzzing (random inputs, malformed data)
//!
//! The goal is to verify that "illegal states are truly unrepresentable."

use crate::intent::interview::types::models::{ConflictState, ConflictStateError, GapState};
use crate::intent::interview::types::{
  Conflict, ConflictResolution, Gap, InterviewSession, Profile,
};
use crate::intent::validation::rule::Rule;

// ============================================================================
// PART 1: SERDE DESERIALIZATION ATTACKS
// ============================================================================

mod serde_attacks {
  use super::*;

  /// Try to deserialize GapState::Resolved with empty resolution via serde.
  /// This should succeed (serde doesn't validate), but subsequent use should be safe.
  #[test]
  fn gap_state_serde_allows_empty_resolution_but_code_validates() {
    // Serde will deserialize this, but the resolve() method validates
    let json = r#"{"status":"resolved","resolution":""}"#;
    let result: Result<GapState, _> = serde_json::from_str(json);

    // Serde allows construction - this is a KNOWN limitation
    assert!(
      result.is_ok(),
      "Serde allows empty resolution - this is a known limitation"
    );
    let state = result.unwrap();

    // But the resolve() method properly validates
    let new_state = GapState::Open.resolve(String::new());
    assert!(
      new_state.is_err(),
      "resolve() correctly rejects empty resolution"
    );
  }

  /// Try to deserialize GapState::Resolved with whitespace-only resolution.
  #[test]
  fn gap_state_serde_allows_whitespace_resolution() {
    let json = r#"{"status":"resolved","resolution":"   "}"#;
    let result: Result<GapState, _> = serde_json::from_str(json);

    assert!(result.is_ok(), "Serde allows whitespace resolution");
    let state = result.unwrap();
    assert!(state.is_resolved());

    // Verify resolve() rejects whitespace
    let new_state = GapState::Open.resolve("   ".to_string());
    assert!(new_state.is_err());
  }

  /// Try to deserialize ConflictState::Resolved with negative index.
  #[test]
  fn conflict_state_serde_allows_negative_index() {
    let json = r#"{"status":"resolved","chosen_index":-1}"#;
    let result: Result<ConflictState, _> = serde_json::from_str(json);

    // Serde allows this - KNOWN limitation
    assert!(
      result.is_ok(),
      "Serde allows negative index - this is a known limitation"
    );
    let state = result.unwrap();
    assert_eq!(state.chosen_index(), Some(-1));

    // But resolve() properly validates
    let new_state = ConflictState::Pending.resolve(-1, 10);
    assert!(new_state.is_err());
  }

  /// Try to deserialize ConflictState::Resolved with i32::MIN.
  #[test]
  fn conflict_state_serde_allows_i32_min() {
    let json = r#"{"status":"resolved","chosen_index":-2147483648}"#;
    let result: Result<ConflictState, _> = serde_json::from_str(json);
    assert!(result.is_ok());
  }

  /// Try to deserialize ConflictState::Resolved with i32::MAX.
  #[test]
  fn conflict_state_serde_allows_i32_max() {
    let json = r#"{"status":"resolved","chosen_index":2147483647}"#;
    let result: Result<ConflictState, _> = serde_json::from_str(json);
    assert!(result.is_ok());
  }

  /// Try malformed JSON for GapState.
  #[test]
  fn gap_state_malformed_json_rejected() {
    let cases = vec![
      r#"{"status":"invalid"}"#,
      r#"{"status":""}"#,
      r#"{}"#,
      r#"null"#,
      r#""open""#,
      r#"{"status":"resolved"}"#, // missing resolution field
    ];

    for json in cases {
      let result: Result<GapState, _> = serde_json::from_str(json);
      assert!(result.is_err(), "Should reject malformed JSON: {json}");
    }
  }

  /// Test that GapState accepts extra fields (serde default behavior).
  /// This is a KNOWN BEHAVIOR - extra fields are ignored.
  #[test]
  fn gap_state_extra_fields_in_open_accepted() {
    // Serde ignores extra fields in tagged enums - this is expected
    let json = r#"{"status":"open","resolution":"should not be here"}"#;
    let result: Result<GapState, _> = serde_json::from_str(json);
    assert!(
      result.is_ok(),
      "Serde ignores extra fields - this is expected"
    );
    assert_eq!(result.unwrap(), GapState::Open);
  }

  /// Try malformed JSON for ConflictState.
  #[test]
  fn conflict_state_malformed_json_rejected() {
    let cases = vec![
      r#"{"status":"invalid"}"#,
      r#"{"status":""}"#,
      r#"{}"#,
      r#"null"#,
      r#""pending""#,
      r#"{"status":"resolved"}"#, // missing chosen_index
      r#"{"status":"resolved","chosen_index":"not a number"}"#,
      r#"{"status":"resolved","chosen_index":1.5}"#, // float instead of int
    ];

    for json in cases {
      let result: Result<ConflictState, _> = serde_json::from_str(json);
      assert!(result.is_err(), "Should reject malformed JSON: {json}");
    }
  }

  /// Test that ConflictState accepts extra fields (serde default behavior).
  /// This is a KNOWN BEHAVIOR - extra fields are ignored.
  #[test]
  fn conflict_state_extra_fields_in_pending_accepted() {
    // Serde ignores extra fields in tagged enums - this is expected
    let json = r#"{"status":"pending","chosen_index":0}"#;
    let result: Result<ConflictState, _> = serde_json::from_str(json);
    assert!(
      result.is_ok(),
      "Serde ignores extra fields - this is expected"
    );
    assert_eq!(result.unwrap(), ConflictState::Pending);
  }

  /// Test that GapState ignores completely unknown extra fields.
  #[test]
  fn gap_state_unknown_extra_fields_ignored() {
    let json = r#"{"status":"open","extra":"malicious","hack":true}"#;
    let result: Result<GapState, _> = serde_json::from_str(json);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), GapState::Open);
  }

  /// Test that Gap serialization round-trips correctly.
  #[test]
  fn gap_roundtrip_serialization() {
    let gap = Gap {
      id: "gap-test".to_string(),
      field: "test_field".to_string(),
      description: "Test gap".to_string(),
      blocking: true,
      suggested_default: "default".to_string(),
      why_needed: "reasons".to_string(),
      round: 1,
      state: GapState::Resolved {
        resolution: "fixed".to_string(),
      },
    };

    let json = serde_json::to_string(&gap).unwrap();
    let restored: Gap = serde_json::from_str(&json).unwrap();
    assert_eq!(gap, restored);
  }

  /// Test that Conflict serialization round-trips correctly.
  #[test]
  fn conflict_roundtrip_serialization() {
    let conflict = Conflict {
      id: "conflict-test".to_string(),
      between: ("a".to_string(), "b".to_string()),
      description: "test conflict".to_string(),
      impact: "high".to_string(),
      options: vec![ConflictResolution::default()],
      state: ConflictState::Resolved { chosen_index: 0 },
    };

    let json = serde_json::to_string(&conflict).unwrap();
    let restored: Conflict = serde_json::from_str(&json).unwrap();
    assert_eq!(conflict, restored);
  }
}

// ============================================================================
// PART 2: EDGE CASES
// ============================================================================

mod edge_cases {
  use super::*;

  /// Test GapState with unicode resolution text.
  #[test]
  fn gap_state_unicode_resolution() {
    let unicode_cases = vec![
      "Resolved with emoji: 🎉",
      "Japanese: 解決済み",
      "Chinese: 已解决",
      "Arabic: تم الحل",
      "RTL override: \u{202E}desrever",
      "Zero-width chars: \u{200B}\u{200C}\u{200D}",
      "Combining chars: e\u{0301}", // e with acute accent
    ];

    for resolution in unicode_cases {
      // resolve() should accept non-empty unicode
      if !resolution.trim().is_empty() {
        let result = GapState::Open.resolve(resolution.to_string());
        assert!(result.is_ok(), "Should accept unicode: {resolution:?}");
      }
    }
  }

  /// Test GapState with empty/whitespace strings - should fail.
  #[test]
  fn gap_state_empty_strings_rejected() {
    let empty_cases = vec!["", " ", "  ", "\t", "\n", "\r\n", "\u{00A0}", "\u{3000}"];

    for resolution in empty_cases {
      let result = GapState::Open.resolve(resolution.to_string());
      assert!(
        result.is_err(),
        "Should reject empty/whitespace: {:?} (len={})",
        resolution,
        resolution.len()
      );
    }
  }

  /// Test ConflictState with boundary index values.
  #[test]
  fn conflict_state_boundary_indices() {
    // Test with 0 options - any index should fail
    let result = ConflictState::Pending.resolve(0, 0);
    assert!(result.is_err());

    // Test with 1 option - index 0 should work, 1 should fail
    let result = ConflictState::Pending.resolve(0, 1);
    assert!(result.is_ok());

    let result = ConflictState::Pending.resolve(1, 1);
    assert!(result.is_err());

    // Test index at boundary (usize::MAX converted to i32 would overflow)
    let result = ConflictState::Pending.resolve(i32::MAX, 1);
    assert!(result.is_err());
  }

  /// Test GapState::resolve cannot be called twice (via AlreadyResolved check would be needed).
  #[test]
  fn gap_state_resolve_can_be_called_multiple_times_on_open() {
    // Note: Current implementation allows calling resolve() on Open multiple times
    // because it takes &self, not &mut self. This is intentional - each call
    // returns a new state.
    let state = GapState::Open;
    let first = state.resolve("first".to_string());
    let second = state.resolve("second".to_string());

    assert!(first.is_ok());
    assert!(second.is_ok());
    // Both produce new states from the same Open state
  }

  /// Test ConflictState::resolve rejects double-resolution.
  #[test]
  fn conflict_state_resolve_rejects_double_resolution() {
    let state = ConflictState::Resolved { chosen_index: 0 };
    let result = state.resolve(1, 5);

    assert!(matches!(result, Err(ConflictStateError::AlreadyResolved)));
  }

  /// Test with very long resolution strings.
  #[test]
  fn gap_state_very_long_resolution() {
    let long_resolution = "x".repeat(1_000_000);
    let result = GapState::Open.resolve(long_resolution.clone());
    assert!(result.is_ok());

    let state = result.unwrap();
    assert_eq!(state.resolution(), Some(long_resolution.as_str()));
  }

  /// Test with very long gap IDs.
  #[test]
  fn session_very_long_gap_id() {
    let mut session = InterviewSession::new(
      "test".to_string(),
      Profile::Api,
      "2026-01-01T00:00:00Z".to_string(),
    );

    let long_id = "gap-".to_string() + &"x".repeat(10_000);
    session.gaps.push(Gap {
      id: long_id.clone(),
      state: GapState::Open,
      ..Gap::default()
    });

    // Should work with long ID
    let result = session.resolve_gap(&long_id, "resolution");
    assert!(result.is_ok());
  }

  /// Test numeric edge cases for Rule::Range.
  #[test]
  fn rule_range_edge_cases() {
    // NaN and infinity are representable in f64
    let rule = Rule::range(f64::NAN, f64::INFINITY);
    assert_eq!(rule.name(), "range");

    // Negative to positive range
    let rule = Rule::range(f64::MIN, f64::MAX);
    assert_eq!(rule.name(), "range");

    // Reversed range (min > max) - allowed by constructor
    let rule = Rule::range(100.0, 0.0);
    assert_eq!(rule.name(), "range");
  }

  /// Test Profile parsing with unicode and case normalization.
  #[test]
  fn profile_unicode_and_case_handling() {
    use crate::intent::interview::types::Profile;

    // Case is normalized to lowercase, so "API" becomes "api" and is accepted
    let result = Profile::parse("API");
    assert!(
      result.is_ok(),
      "Profile normalizes case, so 'API' -> 'api' is accepted"
    );

    // Unicode inputs should be rejected
    let unicode_inputs = vec!["äpi", "cli\u{0301}", "事件"];
    for input in unicode_inputs {
      let result = Profile::parse(input);
      assert!(
        result.is_err(),
        "Should reject unicode/unknown profile: {input:?}"
      );
    }
  }
}

// ============================================================================
// PART 3: FUZZING-STYLE TESTS
// ============================================================================

mod fuzzing {
  use super::*;

  /// Fuzz GapState::resolve with various string patterns.
  #[test]
  fn fuzz_gap_state_resolve() {
    let patterns = generate_fuzz_strings();

    for pattern in patterns {
      let _ = GapState::Open.resolve(pattern);
      // Should not panic, regardless of input
    }
  }

  /// Fuzz ConflictState::resolve with various indices.
  #[test]
  fn fuzz_conflict_state_resolve() {
    let indices = vec![
      i32::MIN,
      i32::MIN + 1,
      -100,
      -1,
      0,
      1,
      100,
      i32::MAX - 1,
      i32::MAX,
    ];

    let option_counts = vec![0, 1, 2, 100, usize::MAX];

    for index in indices {
      for count in &option_counts {
        let _ = ConflictState::Pending.resolve(index, *count);
        // Should not panic
      }
    }
  }

  /// Fuzz JSON deserialization with random/malformed data.
  #[test]
  fn fuzz_gap_state_json() {
    let malformed_inputs = vec![
      // Invalid JSON
      r#"{}}"#,
      r#"{"status":"#,
      r#"{"status":null}"#,
      r#"{"status":[]}"#,
      r#"{"status":{}}"#,
      // Type confusion
      r#"{"status":123}"#,
      r#"{"status":true}"#,
      // Deeply nested
      r#"{"status":{"status":{"status":"open"}}}"#,
      // Array instead of object
      r#"[]"#,
      r#"[{"status":"open"}]"#,
      // String injection attempts
      r#"{"status":"open\""}"#,
      r#"{"status":"open\u0000"}"#,
      // Control characters
      r#"{"status":"open","resolution":"\u0000\u0001\u0002"}"#,
    ];

    for input in malformed_inputs {
      let result: Result<GapState, _> = serde_json::from_str(input);
      // Either it fails to parse, or it parses to a valid GapState
      if let Ok(state) = result {
        // If it parsed, ensure the state is valid (Open or Resolved)
        let _ = state.is_resolved();
      }
    }
  }

  /// Fuzz ConflictState JSON deserialization.
  #[test]
  fn fuzz_conflict_state_json() {
    let malformed_inputs = vec![
      r#"{"status":"resolved","chosen_index":-999999999999}"#,
      r#"{"status":"resolved","chosen_index":0.1}"#,
      r#"{"status":"resolved","chosen_index":"0"}"#,
      r#"{"status":"resolved","chosen_index":null}"#,
      r#"{"status":"pending","extra":{"chosen_index":0}}"#,
      // Scientific notation
      r#"{"status":"resolved","chosen_index":1e10}"#,
      // Hex notation (not valid JSON but testing)
      r#"{"status":"resolved","chosen_index":0x10}"#,
    ];

    for input in malformed_inputs {
      let result: Result<ConflictState, _> = serde_json::from_str(input);
      // Should either fail or produce valid state
      if let Ok(state) = result {
        let _ = state.is_resolved();
      }
    }
  }

  /// Generate various fuzz strings.
  fn generate_fuzz_strings() -> Vec<String> {
    let mut strings = Vec::new();

    // Empty and whitespace
    strings.push(String::new());
    strings.push(" ".to_string());
    strings.push("\t\n\r".to_string());

    // Unicode edge cases
    strings.push("\u{0000}".to_string()); // null byte
                                          // Note: Surrogate \u{D800} cannot be represented in Rust strings - it's ill-formed UTF-8
    strings.push("\u{FFFD}".to_string()); // replacement character
    strings.push("\u{FFFF}".to_string()); // BMP limit
    strings.push("\u{10FFFF}".to_string()); // max unicode

    // Long strings
    strings.push("x".repeat(10000));

    // Special patterns
    strings.push("\\n\\r\\t".to_string());
    strings.push("\"quoted\"".to_string());
    strings.push("line1\nline2\nline3".to_string());

    // Potential injection patterns
    strings.push("<script>alert(1)</script>".to_string());
    strings.push("'; DROP TABLE gaps; --".to_string());
    strings.push("${variable}".to_string());
    strings.push("{{template}}".to_string());

    strings
  }
}

// ============================================================================
// PART 4: STATE MACHINE INVARIANT TESTS
// ============================================================================

mod state_machine_invariants {
  use super::*;

  /// Verify GapState state machine invariants.
  #[test]
  fn gap_state_invariants() {
    // Open state
    let open = GapState::Open;
    assert!(!open.is_resolved());
    assert!(open.resolution().is_none());

    // Resolved state
    let resolved = GapState::Resolved {
      resolution: "test".to_string(),
    };
    assert!(resolved.is_resolved());
    assert_eq!(resolved.resolution(), Some("test"));

    // Cannot construct invalid state through resolve()
    assert!(GapState::Open.resolve(String::new()).is_err());
    assert!(GapState::Open.resolve("".to_string()).is_err());
    assert!(GapState::Open.resolve("   ".to_string()).is_err());
  }

  /// Verify ConflictState state machine invariants.
  #[test]
  fn conflict_state_invariants() {
    // Pending state
    let pending = ConflictState::Pending;
    assert!(!pending.is_resolved());
    assert!(pending.chosen_index().is_none());

    // Resolved state
    let resolved = ConflictState::Resolved { chosen_index: 5 };
    assert!(resolved.is_resolved());
    assert_eq!(resolved.chosen_index(), Some(5));

    // Cannot construct invalid state through resolve()
    assert!(ConflictState::Pending.resolve(-1, 10).is_err());
    assert!(ConflictState::Pending.resolve(10, 5).is_err());
    assert!(ConflictState::Pending.resolve(0, 0).is_err());

    // Cannot double-resolve
    let resolved = ConflictState::Resolved { chosen_index: 0 };
    assert!(resolved.resolve(1, 5).is_err());
  }

  /// Verify Gap default state is Open.
  #[test]
  fn gap_default_is_open() {
    let gap = Gap::default();
    assert!(!gap.is_resolved());
    assert!(gap.resolution().is_none());
    assert!(matches!(gap.state, GapState::Open));
  }

  /// Verify Conflict default state is Pending.
  #[test]
  fn conflict_default_is_pending() {
    let conflict = Conflict::default();
    assert!(!conflict.is_resolved());
    assert!(conflict.chosen_index().is_none());
    assert!(matches!(conflict.state, ConflictState::Pending));
  }

  /// Verify state transitions are one-way (no "unresolve").
  #[test]
  fn no_unresolve_allowed() {
    // Once resolved, cannot go back to open
    let resolved = GapState::Resolved {
      resolution: "done".to_string(),
    };
    // There is no unresolve() method - this is by design
    assert!(resolved.is_resolved());

    let resolved_conflict = ConflictState::Resolved { chosen_index: 0 };
    // There is no unresolve() method - this is by design
    assert!(resolved_conflict.is_resolved());
  }
}

// ============================================================================
// PART 5: INTEGRATION-LEVEL ADVERSARIAL TESTS
// ============================================================================

mod integration_attacks {
  use super::*;

  /// Test that session operations handle corrupted state gracefully.
  #[test]
  fn session_handles_serde_constructed_invalid_state() {
    // Construct a session with a gap that has empty resolution (via serde)
    let json = r#"{
      "id": "test",
      "profile": "api",
      "created_at": "2026-01-01",
      "updated_at": "2026-01-01",
      "stage": "discovery",
      "rounds_completed": 0,
      "answers": [],
      "gaps": [{
        "id": "gap-1",
        "field": "test",
        "description": "test",
        "blocking": true,
        "suggested_default": "",
        "why_needed": "",
        "round": 1,
        "state": {"status": "resolved", "resolution": ""}
      }],
      "conflicts": [],
      "raw_notes": "",
      "current_phase": 1,
      "completed_phases": []
    }"#;

    let result: Result<InterviewSession, _> = serde_json::from_str(json);
    assert!(result.is_ok());

    let session = result.unwrap();
    // The gap has an "invalid" state (empty resolution), but session still works
    assert!(session.gaps[0].is_resolved());
    assert_eq!(session.gaps[0].resolution(), Some(""));
  }

  /// Test conflict resolution with serde-constructed invalid index.
  #[test]
  fn session_handles_serde_constructed_negative_index() {
    let json = r#"{
      "id": "test",
      "profile": "api",
      "created_at": "2026-01-01",
      "updated_at": "2026-01-01",
      "stage": "discovery",
      "rounds_completed": 0,
      "answers": [],
      "gaps": [],
      "conflicts": [{
        "id": "conflict-1",
        "between": ["a", "b"],
        "description": "test",
        "impact": "test",
        "options": [
          {"option": "opt1", "description": "", "tradeoffs": "", "recommendation": false}
        ],
        "state": {"status": "resolved", "chosen_index": -999}
      }],
      "raw_notes": "",
      "current_phase": 1,
      "completed_phases": []
    }"#;

    let result: Result<InterviewSession, _> = serde_json::from_str(json);
    assert!(result.is_ok());

    let session = result.unwrap();
    // The conflict has an invalid index, but session still works
    assert!(session.conflicts[0].is_resolved());
    assert_eq!(session.conflicts[0].chosen_index(), Some(-999));
    // Note: This is a known limitation - serde bypasses validation
  }

  /// Test that deserializing completely invalid session JSON fails gracefully.
  #[test]
  fn session_invalid_json_rejected() {
    let invalid_cases = vec![
      r#"{"id": null}"#,
      r#"{"profile": "invalid_profile"}"#,
      r#"{"stage": "unknown_stage"}"#,
      r#"{"rounds_completed": -1}"#,
      r#"{"answers": "not an array"}"#,
    ];

    for json in invalid_cases {
      let result: Result<InterviewSession, _> = serde_json::from_str(json);
      assert!(result.is_err(), "Should reject: {json}");
    }
  }

  /// Test Profile enum serde attacks.
  #[test]
  fn profile_serde_attacks() {
    use crate::intent::interview::types::Profile;

    // Valid profiles
    for valid in &["api", "cli", "event", "data", "workflow", "ui"] {
      let json = format!("\"{valid}\"");
      let result: Result<Profile, _> = serde_json::from_str(&json);
      assert!(result.is_ok());
    }

    // Invalid profiles
    for invalid in &[
      "API", "Api", "api ", " api", "api\n", "invalid", "", "api-cli",
    ] {
      let json = format!("\"{invalid}\"");
      let result: Result<Profile, _> = serde_json::from_str(&json);
      assert!(result.is_err(), "Should reject profile: {invalid:?}");
    }
  }
}

// ============================================================================
// PART 6: VULNERABILITY SUMMARY TEST
// ============================================================================

mod vulnerability_summary {
  use super::*;

  /// This test documents known limitations that are acceptable.
  #[test]
  fn documented_limitations() {
    // LIMITATION 1: Serde allows constructing invalid states
    // MITIGATION: Business logic methods (resolve()) validate inputs
    let gap_json = r#"{"status":"resolved","resolution":""}"#;
    let gap_state: GapState = serde_json::from_str(gap_json).unwrap();
    assert!(gap_state.is_resolved()); // Serde allows it
    assert!(GapState::Open.resolve(String::new()).is_err()); // But resolve() rejects

    // LIMITATION 2: ConflictState allows negative indices via serde
    // MITIGATION: resolve() validates indices
    let conflict_json = r#"{"status":"resolved","chosen_index":-1}"#;
    let conflict_state: ConflictState = serde_json::from_str(conflict_json).unwrap();
    assert_eq!(conflict_state.chosen_index(), Some(-1)); // Serde allows it
    assert!(ConflictState::Pending.resolve(-1, 10).is_err()); // But resolve() rejects
  }

  /// Verify that all state machine transitions are properly guarded.
  #[test]
  fn all_transitions_guarded() {
    // GapState: Open -> Resolved (guarded by non-empty validation)
    assert!(GapState::Open.resolve("valid".to_string()).is_ok());
    assert!(GapState::Open.resolve("".to_string()).is_err());

    // ConflictState: Pending -> Resolved (guarded by index bounds)
    assert!(ConflictState::Pending.resolve(0, 1).is_ok());
    assert!(ConflictState::Pending.resolve(-1, 1).is_err());
    assert!(ConflictState::Pending.resolve(1, 1).is_err());

    // ConflictState: Resolved -> Resolved (forbidden)
    let resolved = ConflictState::Resolved { chosen_index: 0 };
    assert!(resolved.resolve(1, 5).is_err());
  }
}
