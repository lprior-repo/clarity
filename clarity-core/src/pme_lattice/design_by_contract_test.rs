//! Tests for Design by Contract module (Meyer's `DbC`)
//!
//! Tests verify:
//! - Precondition creation and validation
//! - Postcondition creation and validation
//! - Invariant creation and validation
//! - Contract composition and verification
//! - Contract violation error handling

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::manual_string_new)]
#![allow(clippy::redundant_clone)]
#![allow(clippy::no_effect_underscore_binding)]
#![allow(clippy::doc_markdown)]

use super::design_by_contract::*;

// ============================================================================
// PRECONDITION TESTS
// ============================================================================

#[test]
fn precondition_new_creates_valid_precondition() {
  let predicate = Box::new(|x: &i32| *x > 0);
  let precondition = Precondition::new("must be positive".to_string(), predicate);

  assert_eq!(precondition.error_message(), "must be positive");
  assert!(precondition.check(&5));
  assert!(!precondition.check(&-1));
  assert!(!precondition.check(&0));
}

#[test]
fn precondition_builder_pattern_works() {
  let predicate = Box::new(|s: &String| !s.is_empty());
  let precondition = Precondition::new("string not empty".to_string(), predicate)
    .with_description("The input string must not be empty".to_string());

  assert!(precondition.check(&"hello".to_string()));
  assert!(!precondition.check(&"".to_string()));
  assert_eq!(
    precondition.description(),
    Some(&"The input string must not be empty".to_string())
  );
}

#[test]
fn precondition_can_be_cloned() {
  let predicate = Box::new(|x: &i32| *x >= 0);
  let precondition = Precondition::new("non-negative".to_string(), predicate);

  let cloned = precondition.clone();
  assert!(cloned.check(&10));
  assert!(cloned.check(&0));
  assert!(!cloned.check(&-5));
}

// ============================================================================
// POSTCONDITION TESTS
// ============================================================================

#[test]
fn postcondition_new_creates_valid_postcondition() {
  let predicate = Box::new(|result: &Result<i32, String>| result.is_ok());
  let postcondition = Postcondition::new("must succeed".to_string(), predicate);

  assert!(postcondition.check(&Ok(42)));
  assert!(!postcondition.check(&Err("failed".to_string())));
}

#[test]
fn postcondition_builder_pattern_works() {
  let predicate = Box::new(|s: &String| s.len() <= 100);
  let postcondition = Postcondition::new("max length 100".to_string(), predicate)
    .with_description("Output must not exceed 100 characters".to_string());

  assert!(postcondition.check(&"short".to_string()));
  assert!(!postcondition.check(&"x".repeat(101)));
  assert_eq!(
    postcondition.description(),
    Some(&"Output must not exceed 100 characters".to_string())
  );
}

#[test]
fn postcondition_with_tag_works() {
  let predicate = Box::new(|_: &i32| true);
  let postcondition =
    Postcondition::new("always true".to_string(), predicate).with_tag("unit-test");

  assert_eq!(postcondition.tag(), Some(&"unit-test".to_string()));
}

// ============================================================================
// INVARIANT TESTS
// ============================================================================

#[test]
fn invariant_new_creates_valid_invariant() {
  let predicate = Box::new(|state: &TestState| state.counter >= 0);
  let invariant = Invariant::new("counter non-negative".to_string(), predicate);

  assert_eq!(invariant.description(), "counter non-negative");
  assert!(invariant.check(&TestState { counter: 0 }));
  assert!(invariant.check(&TestState { counter: 100 }));
  assert!(!invariant.check(&TestState { counter: -1 }));
}

#[test]
fn invariant_with_severity_works() {
  let predicate = Box::new(|_: &TestState| true);
  let invariant = Invariant::new("test invariant".to_string(), predicate)
    .with_severity(InvariantSeverity::Critical);

  assert_eq!(invariant.severity(), InvariantSeverity::Critical);
}

#[test]
fn invariant_severity_default_is_warning() {
  let predicate = Box::new(|_: &TestState| true);
  let invariant = Invariant::new("test".to_string(), predicate);

  assert_eq!(invariant.severity(), InvariantSeverity::Warning);
}

#[test]
fn invariant_all_severities_work() {
  let predicate = Box::new(|_: &TestState| true);

  let info =
    Invariant::new("info".to_string(), predicate.clone()).with_severity(InvariantSeverity::Info);
  let warning = Invariant::new("warning".to_string(), predicate.clone())
    .with_severity(InvariantSeverity::Warning);
  let error =
    Invariant::new("error".to_string(), predicate.clone()).with_severity(InvariantSeverity::Error);
  let critical =
    Invariant::new("critical".to_string(), predicate).with_severity(InvariantSeverity::Critical);

  assert_eq!(info.severity(), InvariantSeverity::Info);
  assert_eq!(warning.severity(), InvariantSeverity::Warning);
  assert_eq!(error.severity(), InvariantSeverity::Error);
  assert_eq!(critical.severity(), InvariantSeverity::Critical);
}

// Helper struct for invariant tests
#[derive(Clone, Debug)]
struct TestState {
  counter: i32,
}

// ============================================================================
// CONTRACT TESTS
// ============================================================================

#[test]
fn contract_builder_creates_empty_contract() {
  let contract = Contract::<i32, String, TestState>::new("test contract");

  assert!(contract.preconditions().is_empty());
  assert!(contract.postconditions().is_empty());
  assert!(contract.invariants().is_empty());
  assert_eq!(contract.name(), "test contract");
}

#[test]
fn contract_with_preconditions_works() {
  let pre1 = Precondition::new("positive".to_string(), Box::new(|x: &i32| *x > 0));
  let pre2 = Precondition::new("less than 100".to_string(), Box::new(|x: &i32| *x < 100));

  let contract = Contract::<i32, String, TestState>::new("number contract")
    .with_precondition(pre1)
    .with_precondition(pre2);

  assert_eq!(contract.preconditions().len(), 2);
}

#[test]
fn contract_with_postconditions_works() {
  let post1 = Postcondition::new(
    "non-empty".to_string(),
    Box::new(|s: &String| !s.is_empty()),
  );
  let post2 = Postcondition::new(
    "has prefix".to_string(),
    Box::new(|s: &String| s.starts_with("result:")),
  );

  let contract = Contract::<i32, String, TestState>::new("string contract")
    .with_postcondition(post1)
    .with_postcondition(post2);

  assert_eq!(contract.postconditions().len(), 2);
}

#[test]
fn contract_with_invariants_works() {
  let inv1 = Invariant::new(
    "counter valid".to_string(),
    Box::new(|s: &TestState| s.counter >= 0),
  );
  let inv2 = Invariant::new(
    "counter bounded".to_string(),
    Box::new(|s: &TestState| s.counter < 1000),
  );

  let contract = Contract::<i32, String, TestState>::new("state contract")
    .with_invariant(inv1)
    .with_invariant(inv2);

  assert_eq!(contract.invariants().len(), 2);
}

#[test]
fn contract_verify_preconditions_passes_when_all_satisfied() {
  let contract = Contract::<i32, String, TestState>::new("test")
    .with_precondition(Precondition::new(
      "positive".to_string(),
      Box::new(|x: &i32| *x > 0),
    ))
    .with_precondition(Precondition::new(
      "less than 100".to_string(),
      Box::new(|x: &i32| *x < 100),
    ));

  let result = contract.verify_preconditions(&50);

  assert!(result.is_ok());
}

#[test]
fn contract_verify_preconditions_fails_when_any_unsatisfied() {
  let contract = Contract::<i32, String, TestState>::new("test")
    .with_precondition(Precondition::new(
      "positive".to_string(),
      Box::new(|x: &i32| *x > 0),
    ))
    .with_precondition(Precondition::new(
      "less than 100".to_string(),
      Box::new(|x: &i32| *x < 100),
    ));

  let result = contract.verify_preconditions(&150);

  assert!(result.is_err());
  match result {
    Err(ContractViolation::PreconditionFailed { message, .. }) => {
      assert!(message.contains("less than 100"));
    }
    _ => panic!("Expected PreconditionFailed error"),
  }
}

#[test]
fn contract_verify_preconditions_fails_on_first_violation() {
  let contract = Contract::<i32, String, TestState>::new("test")
    .with_precondition(Precondition::new(
      "positive".to_string(),
      Box::new(|x: &i32| *x > 0),
    ))
    .with_precondition(Precondition::new(
      "even".to_string(),
      Box::new(|x: &i32| *x % 2 == 0),
    ));

  // -5 fails "positive" precondition first
  let result = contract.verify_preconditions(&-5);

  assert!(result.is_err());
  match result {
    Err(ContractViolation::PreconditionFailed { message, .. }) => {
      assert!(message.contains("positive"));
    }
    _ => panic!("Expected PreconditionFailed for 'positive'"),
  }
}

#[test]
fn contract_verify_postconditions_passes_when_all_satisfied() {
  let contract = Contract::<i32, String, TestState>::new("test")
    .with_postcondition(Postcondition::new(
      "non-empty".to_string(),
      Box::new(|s: &String| !s.is_empty()),
    ))
    .with_postcondition(Postcondition::new(
      "has value".to_string(),
      Box::new(|s: &String| s.contains(':')),
    ));

  let result = contract.verify_postconditions(&"result: 42".to_string());

  assert!(result.is_ok());
}

#[test]
fn contract_verify_postconditions_fails_when_any_unsatisfied() {
  let contract = Contract::<i32, String, TestState>::new("test")
    .with_postcondition(Postcondition::new(
      "non-empty".to_string(),
      Box::new(|s: &String| !s.is_empty()),
    ))
    .with_postcondition(Postcondition::new(
      "has value".to_string(),
      Box::new(|s: &String| s.contains(':')),
    ));

  let result = contract.verify_postconditions(&"no colon here".to_string());

  assert!(result.is_err());
  match result {
    Err(ContractViolation::PostconditionFailed { message, .. }) => {
      assert!(message.contains("has value"));
    }
    _ => panic!("Expected PostconditionFailed error"),
  }
}

#[test]
fn contract_verify_invariants_passes_when_all_satisfied() {
  let contract = Contract::<i32, String, TestState>::new("test")
    .with_invariant(Invariant::new(
      "counter >= 0".to_string(),
      Box::new(|s: &TestState| s.counter >= 0),
    ))
    .with_invariant(Invariant::new(
      "counter < 1000".to_string(),
      Box::new(|s: &TestState| s.counter < 1000),
    ));

  let state = TestState { counter: 500 };
  let result = contract.verify_invariants(&state);

  assert!(result.is_ok());
}

#[test]
fn contract_verify_invariants_fails_when_any_unsatisfied() {
  let contract = Contract::<i32, String, TestState>::new("test")
    .with_invariant(Invariant::new(
      "counter >= 0".to_string(),
      Box::new(|s: &TestState| s.counter >= 0),
    ))
    .with_invariant(Invariant::new(
      "counter < 1000".to_string(),
      Box::new(|s: &TestState| s.counter < 1000),
    ));

  let state = TestState { counter: -5 };
  let result = contract.verify_invariants(&state);

  assert!(result.is_err());
  match result {
    Err(ContractViolation::InvariantViolated { description, .. }) => {
      assert!(description.contains("counter >= 0"));
    }
    _ => panic!("Expected InvariantViolated error"),
  }
}

#[test]
fn contract_verify_all_checks_everything() {
  let contract = Contract::<i32, String, TestState>::new("full contract")
    .with_precondition(Precondition::new(
      "positive".to_string(),
      Box::new(|x: &i32| *x > 0),
    ))
    .with_postcondition(Postcondition::new(
      "non-empty".to_string(),
      Box::new(|s: &String| !s.is_empty()),
    ))
    .with_invariant(Invariant::new(
      "valid state".to_string(),
      Box::new(|s: &TestState| s.counter >= 0),
    ));

  let input = 42;
  let output = "result: 42".to_string();
  let state = TestState { counter: 10 };

  // All pass
  let result = contract.verify_all(&input, &output, &state);
  assert!(result.is_ok());
}

#[test]
fn contract_verify_all_fails_on_precondition() {
  let contract = Contract::<i32, String, TestState>::new("full contract")
    .with_precondition(Precondition::new(
      "positive".to_string(),
      Box::new(|x: &i32| *x > 0),
    ))
    .with_postcondition(Postcondition::new(
      "non-empty".to_string(),
      Box::new(|s: &String| !s.is_empty()),
    ));

  let input = -1; // fails precondition
  let output = "result".to_string();
  let state = TestState { counter: 10 };

  let result = contract.verify_all(&input, &output, &state);
  assert!(result.is_err());
}

#[test]
fn contract_verify_all_fails_on_postcondition() {
  let contract = Contract::<i32, String, TestState>::new("full contract")
    .with_precondition(Precondition::new(
      "positive".to_string(),
      Box::new(|x: &i32| *x > 0),
    ))
    .with_postcondition(Postcondition::new(
      "non-empty".to_string(),
      Box::new(|s: &String| !s.is_empty()),
    ));

  let input = 42;
  let output = "".to_string(); // fails postcondition
  let state = TestState { counter: 10 };

  let result = contract.verify_all(&input, &output, &state);
  assert!(result.is_err());
}

#[test]
fn contract_verify_all_fails_on_invariant() {
  let contract = Contract::<i32, String, TestState>::new("full contract")
    .with_precondition(Precondition::new(
      "positive".to_string(),
      Box::new(|x: &i32| *x > 0),
    ))
    .with_postcondition(Postcondition::new(
      "non-empty".to_string(),
      Box::new(|s: &String| !s.is_empty()),
    ))
    .with_invariant(Invariant::new(
      "valid state".to_string(),
      Box::new(|s: &TestState| s.counter >= 0),
    ));

  let input = 42;
  let output = "result".to_string();
  let state = TestState { counter: -5 }; // fails invariant

  let result = contract.verify_all(&input, &output, &state);
  assert!(result.is_err());
}

// ============================================================================
// CONTRACT VIOLATION TESTS
// ============================================================================

#[test]
fn contract_violation_precondition_failed_display() {
  let violation = ContractViolation::PreconditionFailed {
    contract_name: "test contract".to_string(),
    message: "value must be positive".to_string(),
  };

  let display = format!("{violation}");
  assert!(display.contains("Precondition failed"));
  assert!(display.contains("test contract"));
  assert!(display.contains("value must be positive"));
}

#[test]
fn contract_violation_postcondition_failed_display() {
  let violation = ContractViolation::PostconditionFailed {
    contract_name: "output contract".to_string(),
    message: "output must not be empty".to_string(),
  };

  let display = format!("{violation}");
  assert!(display.contains("Postcondition failed"));
  assert!(display.contains("output contract"));
  assert!(display.contains("output must not be empty"));
}

#[test]
fn contract_violation_invariant_violated_display() {
  let violation = ContractViolation::InvariantViolated {
    contract_name: "state contract".to_string(),
    description: "counter must be non-negative".to_string(),
    severity: InvariantSeverity::Error,
  };

  let display = format!("{violation}");
  assert!(display.contains("Invariant violated"));
  assert!(display.contains("state contract"));
  assert!(display.contains("counter must be non-negative"));
  assert!(display.contains("Error"));
}

#[test]
fn contract_violation_critical_severity_display() {
  let violation = ContractViolation::InvariantViolated {
    contract_name: "critical contract".to_string(),
    description: "critical constraint".to_string(),
    severity: InvariantSeverity::Critical,
  };

  let display = format!("{violation}");
  assert!(display.contains("Critical"));
}

#[test]
fn contract_violation_implements_error() {
  let violation = ContractViolation::PreconditionFailed {
    contract_name: "test".to_string(),
    message: "failed".to_string(),
  };

  let _error: &dyn std::error::Error = &violation;
}

#[test]
fn contract_violation_can_be_cloned() {
  let violation = ContractViolation::PreconditionFailed {
    contract_name: "test".to_string(),
    message: "failed".to_string(),
  };

  let cloned = violation.clone();
  assert_eq!(violation, cloned);
}

// ============================================================================
// CONTRACT WITH UUID AND TIMESTAMP TESTS
// ============================================================================

#[test]
fn contract_has_id_and_timestamp() {
  let contract = Contract::<i32, String, TestState>::new("test contract");

  // Contract should have an ID
  assert!(!contract.id().is_nil());

  // Contract should have a created_at timestamp
  let created = contract.created_at();
  assert!(created.timestamp() > 0);
}

#[test]
fn contract_with_description_works() {
  let contract = Contract::<i32, String, TestState>::new("test")
    .with_description("A contract for testing purposes".to_string());

  assert_eq!(
    contract.description(),
    Some(&"A contract for testing purposes".to_string())
  );
}

#[test]
fn contract_with_tag_works() {
  let contract = Contract::<i32, String, TestState>::new("test").with_tag("api-v1");

  assert_eq!(contract.tag(), Some(&"api-v1".to_string()));
}

// ============================================================================
// SERIALIZATION TESTS
// ============================================================================

#[test]
fn precondition_is_serializable() {
  // Precondition metadata should be serializable (without the predicate)
  let meta = PreconditionMeta {
    error_message: "must be positive".to_string(),
    description: Some("The value must be greater than zero".to_string()),
  };

  let json = serde_json::to_string(&meta);
  assert!(json.is_ok());

  let parsed: Result<PreconditionMeta, _> = serde_json::from_str(&json.unwrap());
  assert!(parsed.is_ok());
}

#[test]
fn postcondition_is_serializable() {
  let meta = PostconditionMeta {
    error_message: "must succeed".to_string(),
    description: Some("The operation must complete successfully".to_string()),
    tag: Some("api".to_string()),
  };

  let json = serde_json::to_string(&meta);
  assert!(json.is_ok());

  let parsed: Result<PostconditionMeta, _> = serde_json::from_str(&json.unwrap());
  assert!(parsed.is_ok());
}

#[test]
fn invariant_is_serializable() {
  let meta = InvariantMeta {
    description: "counter must be non-negative".to_string(),
    severity: InvariantSeverity::Error,
  };

  let json = serde_json::to_string(&meta);
  assert!(json.is_ok());

  let parsed: Result<InvariantMeta, _> = serde_json::from_str(&json.unwrap());
  assert!(parsed.is_ok());
}

#[test]
fn invariant_severity_is_serializable() {
  let severities = [
    InvariantSeverity::Info,
    InvariantSeverity::Warning,
    InvariantSeverity::Error,
    InvariantSeverity::Critical,
  ];

  for severity in severities {
    let json = serde_json::to_string(&severity);
    assert!(json.is_ok());

    let parsed: Result<InvariantSeverity, _> = serde_json::from_str(&json.unwrap());
    assert!(parsed.is_ok());
    assert_eq!(parsed.unwrap(), severity);
  }
}

#[test]
fn contract_meta_is_serializable() {
  let meta = ContractMeta {
    id: uuid::Uuid::new_v4(),
    name: "test contract".to_string(),
    description: Some("A contract for testing".to_string()),
    tag: Some("unit-test".to_string()),
    precondition_count: 2,
    postcondition_count: 1,
    invariant_count: 3,
  };

  let json = serde_json::to_string(&meta);
  assert!(json.is_ok());

  let parsed: Result<ContractMeta, _> = serde_json::from_str(&json.unwrap());
  assert!(parsed.is_ok());
}

// ============================================================================
// EDGE CASES
// ============================================================================

#[test]
fn empty_contract_verifies_anything() {
  let contract = Contract::<i32, String, TestState>::new("empty");

  // No conditions means everything passes
  assert!(contract.verify_preconditions(&42).is_ok());
  assert!(contract
    .verify_postconditions(&"anything".to_string())
    .is_ok());
  assert!(contract
    .verify_invariants(&TestState { counter: -999 })
    .is_ok());
  assert!(contract
    .verify_all(&42, &"result".to_string(), &TestState { counter: 0 })
    .is_ok());
}

#[test]
fn contract_with_multiple_violations_reports_first() {
  let contract = Contract::<i32, String, TestState>::new("multi")
    .with_precondition(Precondition::new(
      "positive".to_string(),
      Box::new(|x: &i32| *x > 0),
    ))
    .with_precondition(Precondition::new(
      "less than 10".to_string(),
      Box::new(|x: &i32| *x < 10),
    ));

  // -5 violates "positive" first
  let result = contract.verify_preconditions(&-5);
  match result {
    Err(ContractViolation::PreconditionFailed { message, .. }) => {
      assert!(message.contains("positive"));
    }
    _ => panic!("Expected precondition failure"),
  }
}

#[test]
fn contract_cloning_preserves_structure() {
  let contract = Contract::<i32, String, TestState>::new("original")
    .with_description("original description".to_string())
    .with_tag("original-tag");

  let cloned = contract.clone();

  assert_eq!(contract.name(), cloned.name());
  assert_eq!(contract.description(), cloned.description());
  assert_eq!(contract.tag(), cloned.tag());
  // IDs should be the same for cloned contracts
  assert_eq!(contract.id(), cloned.id());
}
