#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
// Additional clippy lints to allow
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::trivially_copy_pass_by_ref)]
#![allow(clippy::assigning_clones)]
#![allow(clippy::option_if_let_else)]
#![allow(clippy::unused_self)]
#![allow(clippy::unnecessary_wraps)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::manual_strip)]
#![allow(clippy::format_push_string)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::struct_field_names)]
#![allow(clippy::suspicious_else_formatting)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::ptr_arg)]
#![allow(clippy::missing_fields_in_debug)]
#![allow(clippy::must_use_unit)]
#![allow(clippy::collection_is_never_read)]
#![allow(clippy::needless_collect)]
#![allow(clippy::manual_checked_ops)]
#![allow(clippy::needless_pass_by_value)]

//! Design by Contract module for requirements validation.
//!
//! This module implements contract-based assertions following Bertrand Meyer's
//! Design by Contract methodology: preconditions, postconditions, and invariants.

use itertools::Itertools;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Domain errors for design by contract
#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub enum ContractError {
  #[error("contract name is empty")]
  EmptyName,

  #[error("contract expression is empty")]
  EmptyExpression,

  #[error("contract validation failed: {0}")]
  ValidationFailed(String),

  #[error("precondition violated: {0}")]
  PreconditionViolated(String),

  #[error("postcondition violated: {0}")]
  PostconditionViolated(String),

  #[error("invariant violated: {0}")]
  InvariantViolated(String),
}

/// Contract type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ContractType {
  /// Condition that must be true before operation
  Precondition,
  /// Condition that must be true after operation
  Postcondition,
  /// Condition that must always remain true
  Invariant,
}

impl ContractType {
  /// Get all contract types
  #[must_use]
  pub const fn all() -> [Self; 3] {
    [Self::Precondition, Self::Postcondition, Self::Invariant]
  }

  /// Get human-readable label
  #[must_use]
  pub const fn label(&self) -> &'static str {
    match self {
      Self::Precondition => "Precondition",
      Self::Postcondition => "Postcondition",
      Self::Invariant => "Invariant",
    }
  }

  /// Get description
  #[must_use]
  pub const fn description(&self) -> &'static str {
    match self {
      Self::Precondition => "Condition that must be true before operation",
      Self::Postcondition => "Condition that must be true after operation",
      Self::Invariant => "Condition that must always remain true",
    }
  }
}

/// Severity of contract violation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum ViolationSeverity {
  /// Minor issue, contract may be relaxed
  Low,
  /// Significant issue, requires attention
  Medium,
  /// Critical issue, must be addressed
  High,
  /// Blocking issue, contract breach
  Critical,
}

impl ViolationSeverity {
  /// Convert to numeric score (0-100)
  #[must_use]
  pub const fn score(&self) -> u8 {
    match self {
      Self::Low => 25,
      Self::Medium => 50,
      Self::High => 75,
      Self::Critical => 100,
    }
  }
}

/// A single contract assertion
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Contract {
  /// Unique identifier
  pub id: String,
  /// Contract type
  pub contract_type: ContractType,
  /// Human-readable name
  pub name: String,
  /// Contract expression/condition
  pub expression: String,
  /// Description of what this contract ensures
  pub description: String,
  /// Associated requirement or component
  pub scope: Option<String>,
}

impl Contract {
  /// Create a new contract
  ///
  /// # Errors
  ///
  /// Returns `ContractError` if name or expression is empty
  pub fn new(
    id: String,
    contract_type: ContractType,
    name: String,
    expression: String,
  ) -> Result<Self, ContractError> {
    if name.trim().is_empty() {
      return Err(ContractError::EmptyName);
    }
    if expression.trim().is_empty() {
      return Err(ContractError::EmptyExpression);
    }

    Ok(Self {
      id,
      contract_type,
      name,
      expression,
      description: String::new(),
      scope: None,
    })
  }

  /// Add description using builder pattern
  #[must_use]
  pub fn with_description(mut self, description: String) -> Self {
    self.description = description;
    self
  }

  /// Add scope using builder pattern
  #[must_use]
  pub fn with_scope(mut self, scope: String) -> Self {
    self.scope = Some(scope);
    self
  }
}

/// A detected contract violation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractViolation {
  /// The violated contract
  pub contract: Contract,
  /// Severity of the violation
  pub severity: ViolationSeverity,
  /// Evidence/reason for the violation
  pub evidence: String,
  /// Suggested remediation
  pub remediation: Option<String>,
}

impl ContractViolation {
  /// Create a new contract violation
  #[must_use]
  pub const fn new(contract: Contract, severity: ViolationSeverity, evidence: String) -> Self {
    Self {
      contract,
      severity,
      evidence,
      remediation: None,
    }
  }

  /// Add remediation suggestion
  #[must_use]
  pub fn with_remediation(mut self, remediation: String) -> Self {
    self.remediation = Some(remediation);
    self
  }

  /// Calculate quality impact
  #[must_use]
  pub const fn quality_impact(&self) -> u8 {
    self.severity.score()
  }
}

/// Complete contract analysis output
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractAnalysis {
  /// All defined contracts
  pub contracts: Vec<Contract>,
  /// Detected violations
  pub violations: Vec<ContractViolation>,
  /// Contracts grouped by type
  pub by_type: Vec<(ContractType, usize)>,
  /// Overall contract health score (0-100)
  pub health_score: u8,
  /// Summary message
  pub summary: String,
}

impl ContractAnalysis {
  /// Create new contract analysis
  #[must_use]
  pub fn new(contracts: Vec<Contract>, violations: Vec<ContractViolation>) -> Self {
    let by_type = ContractType::all()
      .iter()
      .map(|ct| {
        let count = contracts.iter().filter(|c| c.contract_type == *ct).count();
        (*ct, count)
      })
      .collect();

    // Calculate health score: start at 100, subtract based on violations
    let total_impact: u32 = violations
      .iter()
      .map(|v| u32::from(v.quality_impact()))
      .sum();
    let health_score = u8::try_from(100_u32.saturating_sub(total_impact / 10)).unwrap_or(0);

    let summary = generate_summary(&contracts, &violations);

    Self {
      contracts,
      violations,
      by_type,
      health_score,
      summary,
    }
  }

  /// Get violations by severity
  #[must_use]
  pub fn violations_by_severity(&self, severity: ViolationSeverity) -> Vec<&ContractViolation> {
    self
      .violations
      .iter()
      .filter(|v| v.severity == severity)
      .collect()
  }

  /// Get contracts by type
  #[must_use]
  pub fn contracts_by_type(&self, contract_type: ContractType) -> Vec<&Contract> {
    self
      .contracts
      .iter()
      .filter(|c| c.contract_type == contract_type)
      .collect()
  }

  /// Check if any critical violations exist
  #[must_use]
  pub fn has_critical_violations(&self) -> bool {
    self
      .violations
      .iter()
      .any(|v| v.severity == ViolationSeverity::Critical)
  }
}

/// Generate summary message
fn generate_summary(contracts: &[Contract], violations: &[ContractViolation]) -> String {
  let pre_count = contracts
    .iter()
    .filter(|c| c.contract_type == ContractType::Precondition)
    .count();
  let post_count = contracts
    .iter()
    .filter(|c| c.contract_type == ContractType::Postcondition)
    .count();
  let inv_count = contracts
    .iter()
    .filter(|c| c.contract_type == ContractType::Invariant)
    .count();

  let critical = violations
    .iter()
    .filter(|v| v.severity == ViolationSeverity::Critical)
    .count();
  let high = violations
    .iter()
    .filter(|v| v.severity == ViolationSeverity::High)
    .count();

  format!(
    "Contracts: {pre_count} preconditions, {post_count} postconditions, {inv_count} invariants. \
         Violations: {critical} critical, {high} high severity."
  )
}

/// Extract contracts from requirement text
///
/// # Arguments
/// * `text` - Requirement text to analyze
///
/// # Returns
/// Vector of detected contracts
#[must_use]
pub fn extract_contracts(text: &str) -> Vec<Contract> {
  let mut contracts = Vec::new();
  let mut id_counter = 0;

  // Precondition patterns
  let pre_patterns = [
    ("before", "must be"),
    ("requires", ""),
    ("prerequisite", ""),
    ("assumes", ""),
    ("given that", ""),
    ("provided that", ""),
    ("if", "then"),
  ];

  // Postcondition patterns
  let post_patterns = [
    ("after", "must"),
    ("ensures", ""),
    ("guarantees", ""),
    ("will result in", ""),
    ("returns", ""),
    ("produces", ""),
  ];

  // Invariant patterns
  let inv_patterns = [
    ("always", ""),
    ("never", ""),
    ("must always", ""),
    ("must never", ""),
    ("invariant", ""),
    ("maintains", ""),
  ];

  let sentences: Vec<&str> = text
    .split(['.', '!', '?'])
    .map(str::trim)
    .filter(|s| !s.is_empty())
    .collect();

  for sentence in &sentences {
    let lower = sentence.to_lowercase();

    // Check precondition patterns
    for (pattern, _secondary) in &pre_patterns {
      if lower.contains(pattern) {
        id_counter += 1;
        if let Ok(contract) = Contract::new(
          format!("PRE-{id_counter:03}"),
          ContractType::Precondition,
          format!("Precondition {id_counter}"),
          sentence.to_string(),
        ) {
          contracts.push(contract);
        }
        break;
      }
    }

    // Check postcondition patterns
    for (pattern, _) in &post_patterns {
      if lower.contains(pattern) {
        id_counter += 1;
        if let Ok(contract) = Contract::new(
          format!("POST-{id_counter:03}"),
          ContractType::Postcondition,
          format!("Postcondition {id_counter}"),
          sentence.to_string(),
        ) {
          contracts.push(contract);
        }
        break;
      }
    }

    // Check invariant patterns
    for (pattern, _) in &inv_patterns {
      if lower.contains(pattern) {
        id_counter += 1;
        if let Ok(contract) = Contract::new(
          format!("INV-{id_counter:03}"),
          ContractType::Invariant,
          format!("Invariant {id_counter}"),
          sentence.to_string(),
        ) {
          contracts.push(contract);
        }
        break;
      }
    }
  }

  contracts.into_iter().unique_by(|c| c.id.clone()).collect()
}

/// Analyze requirements for contract violations
///
/// # Arguments
/// * `requirements` - List of requirement texts
///
/// # Returns
/// Contract analysis with detected contracts and violations
#[must_use]
pub fn analyze_contracts(requirements: &[&str]) -> ContractAnalysis {
  // Extract all contracts
  let contracts: Vec<Contract> = requirements
    .iter()
    .flat_map(|req| extract_contracts(req))
    .collect();

  // Detect violations
  let violations = detect_violations(&contracts);

  ContractAnalysis::new(contracts, violations)
}

/// Detect contract violations by analyzing contract consistency
fn detect_violations(contracts: &[Contract]) -> Vec<ContractViolation> {
  let mut violations = Vec::new();

  // Check for conflicting preconditions
  let preconditions: Vec<&Contract> = contracts
    .iter()
    .filter(|c| c.contract_type == ContractType::Precondition)
    .collect();

  for (i, pre1) in preconditions.iter().enumerate() {
    for pre2 in preconditions.iter().skip(i + 1) {
      if let Some(violation) = check_precondition_conflict(pre1, pre2) {
        violations.push(violation);
      }
    }
  }

  // Check for invariant contradictions
  let invariants: Vec<&Contract> = contracts
    .iter()
    .filter(|c| c.contract_type == ContractType::Invariant)
    .collect();

  for (i, inv1) in invariants.iter().enumerate() {
    for inv2 in invariants.iter().skip(i + 1) {
      if let Some(violation) = check_invariant_contradiction(inv1, inv2) {
        violations.push(violation);
      }
    }
  }

  // Check for missing postconditions when preconditions exist
  let has_postconditions = contracts
    .iter()
    .any(|c| c.contract_type == ContractType::Postcondition);

  if !preconditions.is_empty() && !has_postconditions {
    if let Some(first_pre) = preconditions.first() {
      violations.push(
        ContractViolation::new(
          (*first_pre).clone(),
          ViolationSeverity::Medium,
          "Preconditions exist but no postconditions defined".to_string(),
        )
        .with_remediation("Add postconditions to specify expected outcomes".to_string()),
      );
    }
  }

  violations
}

/// Check if two preconditions conflict
fn check_precondition_conflict(pre1: &Contract, pre2: &Contract) -> Option<ContractViolation> {
  let lower1 = pre1.expression.to_lowercase();
  let lower2 = pre2.expression.to_lowercase();

  // Check for direct contradictions
  let contradictions = [
    ("must be", "must not be"),
    ("required", "optional"),
    ("enabled", "disabled"),
    ("active", "inactive"),
    ("present", "absent"),
  ];

  for (pos, neg) in &contradictions {
    if (lower1.contains(pos) && lower2.contains(neg))
      || (lower1.contains(neg) && lower2.contains(pos))
    {
      return Some(
        ContractViolation::new(
          pre1.clone(),
          ViolationSeverity::High,
          format!(
            "Conflicting preconditions: '{}' vs '{}'",
            pre1.name, pre2.name
          ),
        )
        .with_remediation("Resolve conflicting preconditions".to_string()),
      );
    }
  }

  None
}

/// Check if two invariants contradict
fn check_invariant_contradiction(inv1: &Contract, inv2: &Contract) -> Option<ContractViolation> {
  let lower1 = inv1.expression.to_lowercase();
  let lower2 = inv2.expression.to_lowercase();

  // Check for always/never contradictions
  if (lower1.contains("always") && lower2.contains("never"))
    || (lower1.contains("never") && lower2.contains("always"))
  {
    // Check if they refer to the same concept
    let words1: std::collections::HashSet<&str> =
      lower1.split_whitespace().filter(|w| w.len() > 3).collect();
    let words2: std::collections::HashSet<&str> =
      lower2.split_whitespace().filter(|w| w.len() > 3).collect();

    let common_words: Vec<&&str> = words1.intersection(&words2).collect();
    if common_words.len() >= 2 {
      return Some(
        ContractViolation::new(
          inv1.clone(),
          ViolationSeverity::Critical,
          format!(
            "Invariant contradiction: '{}' vs '{}'",
            inv1.name, inv2.name
          ),
        )
        .with_remediation("Remove or clarify contradictory invariants".to_string()),
      );
    }
  }

  None
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_contract_type_labels() {
    assert_eq!(ContractType::Precondition.label(), "Precondition");
    assert_eq!(ContractType::Postcondition.label(), "Postcondition");
    assert_eq!(ContractType::Invariant.label(), "Invariant");
  }

  #[test]
  fn test_contract_type_descriptions() {
    for ct in ContractType::all() {
      assert!(!ct.description().is_empty());
    }
  }

  #[test]
  fn test_contract_new_valid() {
    let contract = Contract::new(
      "TEST-001".to_string(),
      ContractType::Precondition,
      "Test Contract".to_string(),
      "user must be authenticated".to_string(),
    );

    assert!(contract.is_ok());
    let c = contract.unwrap();
    assert_eq!(c.id, "TEST-001");
    assert_eq!(c.contract_type, ContractType::Precondition);
  }

  #[test]
  fn test_contract_new_empty_name() {
    let result = Contract::new(
      "TEST-001".to_string(),
      ContractType::Precondition,
      "".to_string(),
      "expression".to_string(),
    );

    assert!(matches!(result, Err(ContractError::EmptyName)));
  }

  #[test]
  fn test_contract_new_empty_expression() {
    let result = Contract::new(
      "TEST-001".to_string(),
      ContractType::Precondition,
      "name".to_string(),
      "   ".to_string(),
    );

    assert!(matches!(result, Err(ContractError::EmptyExpression)));
  }

  #[test]
  fn test_contract_builder_pattern() {
    let contract = Contract::new(
      "TEST-001".to_string(),
      ContractType::Invariant,
      "Test".to_string(),
      "always valid".to_string(),
    )
    .unwrap()
    .with_description("Test description".to_string())
    .with_scope("UserService".to_string());

    assert_eq!(contract.description, "Test description");
    assert_eq!(contract.scope, Some("UserService".to_string()));
  }

  #[test]
  fn test_violation_severity_ordering() {
    assert!(ViolationSeverity::Critical > ViolationSeverity::High);
    assert!(ViolationSeverity::High > ViolationSeverity::Medium);
    assert!(ViolationSeverity::Medium > ViolationSeverity::Low);
  }

  #[test]
  fn test_violation_severity_scores() {
    assert_eq!(ViolationSeverity::Low.score(), 25);
    assert_eq!(ViolationSeverity::Medium.score(), 50);
    assert_eq!(ViolationSeverity::High.score(), 75);
    assert_eq!(ViolationSeverity::Critical.score(), 100);
  }

  #[test]
  fn test_contract_violation_builder() {
    let contract = Contract::new(
      "TEST-001".to_string(),
      ContractType::Precondition,
      "Test".to_string(),
      "must be true".to_string(),
    )
    .unwrap();

    let violation = ContractViolation::new(
      contract.clone(),
      ViolationSeverity::High,
      "Evidence".to_string(),
    )
    .with_remediation("Fix it".to_string());

    assert_eq!(violation.contract, contract);
    assert_eq!(violation.severity, ViolationSeverity::High);
    assert_eq!(violation.remediation, Some("Fix it".to_string()));
  }

  #[test]
  fn test_extract_contracts_preconditions() {
    let text =
      "Before processing, the user must be authenticated. The system requires valid input.";

    let contracts = extract_contracts(text);

    assert!(!contracts.is_empty());
    let preconditions: Vec<_> = contracts
      .iter()
      .filter(|c| c.contract_type == ContractType::Precondition)
      .collect();
    assert!(!preconditions.is_empty());
  }

  #[test]
  fn test_extract_contracts_postconditions() {
    let text =
      "After processing, the system ensures data is saved. The function returns a valid result.";

    let contracts = extract_contracts(text);

    let postconditions: Vec<_> = contracts
      .iter()
      .filter(|c| c.contract_type == ContractType::Postcondition)
      .collect();
    assert!(!postconditions.is_empty());
  }

  #[test]
  fn test_extract_contracts_invariants() {
    let text =
      "The system must always maintain data integrity. Users can never access restricted data.";

    let contracts = extract_contracts(text);

    let invariants: Vec<_> = contracts
      .iter()
      .filter(|c| c.contract_type == ContractType::Invariant)
      .collect();
    assert!(!invariants.is_empty());
  }

  #[test]
  fn test_extract_contracts_empty_text() {
    let contracts = extract_contracts("");
    assert!(contracts.is_empty());
  }

  #[test]
  fn test_analyze_contracts_basic() {
    let requirements = vec![
      "The user must be authenticated before accessing data.",
      "After saving, the system guarantees data persistence.",
      "The system always validates input.",
    ];

    let analysis = analyze_contracts(&requirements);

    assert!(!analysis.contracts.is_empty());
    assert!(!analysis.summary.is_empty());
  }

  #[test]
  fn test_contract_analysis_violations_by_severity() {
    let contracts = vec![Contract::new(
      "TEST-001".to_string(),
      ContractType::Precondition,
      "Test".to_string(),
      "must be true".to_string(),
    )
    .unwrap()];

    let violations = vec![
      ContractViolation::new(
        contracts[0].clone(),
        ViolationSeverity::High,
        "Issue 1".to_string(),
      ),
      ContractViolation::new(
        contracts[0].clone(),
        ViolationSeverity::Low,
        "Issue 2".to_string(),
      ),
    ];

    let analysis = ContractAnalysis::new(contracts, violations);

    let high_severity = analysis.violations_by_severity(ViolationSeverity::High);
    assert_eq!(high_severity.len(), 1);

    let low_severity = analysis.violations_by_severity(ViolationSeverity::Low);
    assert_eq!(low_severity.len(), 1);
  }

  #[test]
  fn test_contract_analysis_contracts_by_type() {
    let contracts = vec![
      Contract::new(
        "PRE-001".to_string(),
        ContractType::Precondition,
        "Pre1".to_string(),
        "before".to_string(),
      )
      .unwrap(),
      Contract::new(
        "POST-001".to_string(),
        ContractType::Postcondition,
        "Post1".to_string(),
        "after".to_string(),
      )
      .unwrap(),
    ];

    let analysis = ContractAnalysis::new(contracts, vec![]);

    let preconditions = analysis.contracts_by_type(ContractType::Precondition);
    assert_eq!(preconditions.len(), 1);

    let postconditions = analysis.contracts_by_type(ContractType::Postcondition);
    assert_eq!(postconditions.len(), 1);
  }

  #[test]
  fn test_check_precondition_conflict_detected() {
    let pre1 = Contract::new(
      "PRE-001".to_string(),
      ContractType::Precondition,
      "P1".to_string(),
      "User must be enabled".to_string(),
    )
    .unwrap();

    let pre2 = Contract::new(
      "PRE-002".to_string(),
      ContractType::Precondition,
      "P2".to_string(),
      "User must be disabled".to_string(),
    )
    .unwrap();

    let violation = check_precondition_conflict(&pre1, &pre2);

    assert!(violation.is_some());
    let v = violation.unwrap();
    assert_eq!(v.severity, ViolationSeverity::High);
  }

  #[test]
  fn test_check_precondition_no_conflict() {
    let pre1 = Contract::new(
      "PRE-001".to_string(),
      ContractType::Precondition,
      "P1".to_string(),
      "User must be authenticated".to_string(),
    )
    .unwrap();

    let pre2 = Contract::new(
      "PRE-002".to_string(),
      ContractType::Precondition,
      "P2".to_string(),
      "Input must be valid".to_string(),
    )
    .unwrap();

    let violation = check_precondition_conflict(&pre1, &pre2);

    assert!(violation.is_none());
  }

  #[test]
  fn test_check_invariant_contradiction_detected() {
    let inv1 = Contract::new(
      "INV-001".to_string(),
      ContractType::Invariant,
      "I1".to_string(),
      "The system always allows user access".to_string(),
    )
    .unwrap();

    let inv2 = Contract::new(
      "INV-002".to_string(),
      ContractType::Invariant,
      "I2".to_string(),
      "The system never allows user access".to_string(),
    )
    .unwrap();

    let violation = check_invariant_contradiction(&inv1, &inv2);

    assert!(violation.is_some());
    let v = violation.unwrap();
    assert_eq!(v.severity, ViolationSeverity::Critical);
  }

  #[test]
  fn test_health_score_calculation() {
    let contracts = vec![Contract::new(
      "TEST-001".to_string(),
      ContractType::Invariant,
      "Test".to_string(),
      "always valid".to_string(),
    )
    .unwrap()];

    let violations = vec![ContractViolation::new(
      contracts[0].clone(),
      ViolationSeverity::Critical,
      "Critical issue".to_string(),
    )];

    let analysis = ContractAnalysis::new(contracts, violations);

    // Critical violation = 100 impact, divided by 10 = 10, so 100 - 10 = 90
    assert_eq!(analysis.health_score, 90);
  }

  #[test]
  fn test_has_critical_violations() {
    let contracts = vec![Contract::new(
      "TEST-001".to_string(),
      ContractType::Invariant,
      "Test".to_string(),
      "always valid".to_string(),
    )
    .unwrap()];

    let violations = vec![ContractViolation::new(
      contracts[0].clone(),
      ViolationSeverity::Critical,
      "Critical issue".to_string(),
    )];

    let analysis = ContractAnalysis::new(contracts.clone(), violations);
    assert!(analysis.has_critical_violations());

    let no_violations = ContractAnalysis::new(contracts, vec![]);
    assert!(!no_violations.has_critical_violations());
  }

  #[test]
  fn test_missing_postconditions_detection() {
    let requirements = vec!["Before processing, user must be authenticated."];

    let analysis = analyze_contracts(&requirements);

    // Should have a violation about missing postconditions
    let missing_post = analysis
      .violations
      .iter()
      .any(|v| v.evidence.contains("no postconditions"));
    assert!(missing_post);
  }
}
