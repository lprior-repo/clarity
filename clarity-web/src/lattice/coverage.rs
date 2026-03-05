#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![allow(clippy::suspicious_else_formatting)]
#![warn(clippy::nursery)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::struct_field_names)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::ptr_arg)]
#![allow(clippy::missing_fields_in_debug)]
#![allow(clippy::must_use_unit)]
#![allow(clippy::collection_is_never_read)]
#![allow(clippy::manual_checked_ops)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::derive_partial_eq_without_eq)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_wraps)]
#![allow(clippy::redundant_closure_for_method_calls)]
#![allow(clippy::non_std_lazy_statics)]
#![allow(clippy::manual_let_else)]
#![allow(clippy::match_wild_err_arm)]
#![allow(clippy::option_if_let_else)]
#![allow(clippy::needless_collect)]
#![allow(clippy::useless_vec)]
#![forbid(unsafe_code)]

use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use thiserror::Error;

/// Pattern for capitalized words with a capture group (e.g., "Service", "Controller")
const CAPITALIZED_WORD_PATTERN_STR: &str = r"(\b[A-Z][a-z]{2,}\b)";

/// Ultra-simple fallback that matches any non-empty content - guaranteed valid by regex syntax
const ULTIMATE_FALLBACK_PATTERN_STR: &str = r"(.+)";

/// Attempt to create a Regex, returning an Option to avoid panicking
fn try_create_regex(pattern: &str) -> Option<Regex> {
  Regex::new(pattern).ok()
}

/// Create a Regex with cascading fallbacks, never panicking
///
/// Tries patterns in order: primary -> secondary -> ultimate fallback
fn create_regex_safe(primary: &str, secondary: &str) -> Regex {
  try_create_regex(primary)
    .or_else(|| try_create_regex(secondary))
    .or_else(|| try_create_regex(ULTIMATE_FALLBACK_PATTERN_STR))
    .unwrap_or_else(|| {
      // SAFETY: The pattern "(.+)" is syntactically valid and will always compile.
      // This branch is unreachable in practice, but required by the type system.
      #[allow(clippy::expect_used)]
      Regex::new(ULTIMATE_FALLBACK_PATTERN_STR)
        .expect("ULTIMATE_FALLBACK_PATTERN_STR must be valid regex syntax")
    })
}

/// Lazy-initialized regex for capitalized word detection
/// Uses functional pattern with safe initialization
static CAPITALIZED_WORD_PATTERN: Lazy<Regex> =
  Lazy::new(|| create_regex_safe(CAPITALIZED_WORD_PATTERN_STR, ULTIMATE_FALLBACK_PATTERN_STR));

/// Lazy-initialized fallback regex that matches any non-empty content
static FALLBACK_PATTERN: Lazy<Regex> =
  Lazy::new(|| create_regex_safe(ULTIMATE_FALLBACK_PATTERN_STR, r".+"));

/// Domain errors for coverage analysis
#[derive(Debug, Error, PartialEq, Clone)]
pub enum CoverageError {
  #[error("empty use cases provided")]
  EmptyUseCases,

  #[error("empty tasks provided")]
  EmptyTasks,

  #[error("invalid percentage: {0}")]
  InvalidPercentage(String),

  #[error("component extraction failed: {0}")]
  ComponentExtractionFailed(String),
}

/// A use case representing a requirement or feature
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UseCase {
  pub id: String,
  pub name: String,
  pub description: String,
}

impl UseCase {
  pub fn new(id: String, name: String, description: String) -> Self {
    Self {
      id,
      name,
      description,
    }
  }
}

/// A task representing implementation work
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Task {
  pub id: String,
  pub title: String,
  pub description: String,
}

impl Task {
  pub fn new(id: String, title: String, description: String) -> Self {
    Self {
      id,
      title,
      description,
    }
  }

  /// Get combined text for component extraction
  fn combined_text(&self) -> String {
    format!("{} {}", self.title, self.description)
  }
}

/// A component identified in the system
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Component {
  pub name: String,
}

impl Component {
  pub fn new(name: String) -> Self {
    Self { name }
  }
}

/// A component with its covered use cases
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CoveredComponent {
  pub name: String,
  pub use_cases: Vec<String>,
  pub coverage_percent: u8,
}

impl CoveredComponent {
  pub fn new(
    name: String,
    use_cases: Vec<String>,
    coverage_percent: u8,
  ) -> Result<Self, CoverageError> {
    match coverage_percent {
      0..=100 => Ok(Self {
        name,
        use_cases,
        coverage_percent,
      }),
      invalid => Err(CoverageError::InvalidPercentage(invalid.to_string())),
    }
  }

  /// Create from component and matched use case IDs
  fn from_component_and_matches(
    component: Component,
    matched_use_cases: Vec<String>,
    total_use_cases: usize,
  ) -> Result<Self, CoverageError> {
    let coverage_percent = if total_use_cases > 0 {
      let percent = (matched_use_cases.len() * 100) / total_use_cases;
      u8::try_from(percent).map_err(|_| CoverageError::InvalidPercentage("overflow".to_string()))?
    } else {
      0
    };

    Self::new(component.name, matched_use_cases, coverage_percent)
  }
}

/// A coverage gap representing missing coverage
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CoverageGap {
  pub use_case: String,
  pub missing_components: Vec<String>,
  pub suggestion: String,
}

impl CoverageGap {
  pub fn new(use_case: String, missing_components: Vec<String>, suggestion: String) -> Self {
    Self {
      use_case,
      missing_components,
      suggestion,
    }
  }
}

/// Output of coverage analysis
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CoverageOutput {
  /// Components and their coverage
  pub covered_components: Vec<CoveredComponent>,
  /// Identified coverage gaps
  pub coverage_gaps: Vec<CoverageGap>,
  /// Overall coverage percentage
  pub overall_coverage_percent: u8,
  /// Total use cases analyzed
  pub total_use_cases: usize,
  /// Covered use cases count
  pub covered_use_cases_count: usize,
}

impl CoverageOutput {
  pub fn new(
    covered_components: Vec<CoveredComponent>,
    coverage_gaps: Vec<CoverageGap>,
    overall_coverage_percent: u8,
    total_use_cases: usize,
    covered_use_cases_count: usize,
  ) -> Result<Self, CoverageError> {
    match overall_coverage_percent {
      0..=100 => Ok(Self {
        covered_components,
        coverage_gaps,
        overall_coverage_percent,
        total_use_cases,
        covered_use_cases_count,
      }),
      invalid => Err(CoverageError::InvalidPercentage(invalid.to_string())),
    }
  }

  /// Check if overall coverage meets threshold
  pub fn meets_threshold(&self, threshold: u8) -> bool {
    self.overall_coverage_percent >= threshold
  }
}

/// Analyze coverage of use cases by tasks
///
/// # Arguments
/// * `use_cases` - List of use cases to check coverage for
/// * `tasks` - List of tasks that may implement use cases
///
/// # Returns
/// Coverage analysis showing which components cover which use cases
pub fn analyze_coverage(
  use_cases: &[UseCase],
  tasks: &[Task],
) -> Result<CoverageOutput, CoverageError> {
  if use_cases.is_empty() {
    return Err(CoverageError::EmptyUseCases);
  }

  if tasks.is_empty() {
    return Err(CoverageError::EmptyTasks);
  }

  // Extract all components from tasks
  let components = extract_components_from_tasks(tasks)?;

  // Match use cases to components
  let component_matches = match_use_cases_to_components(use_cases, &components, tasks)?;

  // Build covered components list
  let covered_components = component_matches
    .into_iter()
    .map(|(component, matched_ids)| {
      CoveredComponent::from_component_and_matches(component, matched_ids, use_cases.len())
    })
    .collect::<Result<Vec<_>, _>>()?;

  // Find covered use case IDs
  let covered_use_case_ids: HashSet<_> = covered_components
    .iter()
    .flat_map(|cc| cc.use_cases.iter().cloned())
    .collect();

  // Find uncovered use cases
  let uncovered_use_cases: Vec<_> = use_cases
    .iter()
    .filter(|uc| !covered_use_case_ids.contains(&uc.id))
    .collect();

  // Generate coverage gaps with suggestions
  let coverage_gaps = generate_coverage_gaps(&uncovered_use_cases, &covered_components);

  // Calculate overall coverage
  let covered_count = covered_use_case_ids.len();
  let total_count = use_cases.len();
  let overall_percent = if total_count > 0 {
    let percent = (covered_count * 100) / total_count;
    u8::try_from(percent).map_err(|_| CoverageError::InvalidPercentage("overflow".to_string()))?
  } else {
    0
  };

  CoverageOutput::new(
    covered_components,
    coverage_gaps,
    overall_percent,
    total_count,
    covered_count,
  )
}

/// Extract component names from task descriptions
///
/// Looks for capitalized words that might be component names,
/// as well as common architectural patterns.
fn extract_components_from_tasks(tasks: &[Task]) -> Result<Vec<Component>, CoverageError> {
  let mut component_names = HashSet::new();

  // Patterns for component extraction:
  // 1. Capitalized words (likely proper nouns/component names)
  // 2. Words ending in common component suffixes
  // 3. Tech stack terms
  let component_pattern = Regex::new(
        r"\b([A-Z][a-z]+(?:Service|Controller|Repository|Manager|Handler|Component|Module|System|Interface|API|Client|Database|Cache|Queue|Logger|Validator|Parser|Generator|Engine|Processor|Builder|Factory|Adapter|Wrapper|Proxy|Gateway|Router|Middleware))\b"
    ).map_err(|e| CoverageError::ComponentExtractionFailed(e.to_string()))?;

  // Also catch standalone capitalized words (at least 2 letters)
  let capitalized_pattern = Regex::new(r"\b[A-Z][a-z]{2,}\b")
    .map_err(|e| CoverageError::ComponentExtractionFailed(e.to_string()))?;

  for task in tasks {
    let text = task.combined_text();

    // Extract full component names with suffixes
    for cap in component_pattern.captures_iter(&text) {
      if let Some(name) = cap.get(1) {
        component_names.insert(name.as_str().to_string());
      }
    }

    // Extract capitalized words (potential components)
    for cap in capitalized_pattern.captures_iter(&text) {
      if let Some(name) = cap.get(1) {
        let name_str = name.as_str().to_string();
        // Filter out common non-component words
        if !is_common_word(&name_str) {
          component_names.insert(name_str);
        }
      }
    }
  }

  // Convert to sorted vector
  let mut components: Vec<_> = component_names.into_iter().map(Component::new).collect();

  components.sort_by(|a, b| a.name.cmp(&b.name));

  Ok(components)
}

/// Check if a word is a common non-component word
fn is_common_word(word: &str) -> bool {
  const COMMON_WORDS: &[&str] = &[
    "The",
    "This",
    "That",
    "These",
    "Those",
    "When",
    "Then",
    "With",
    "From",
    "Will",
    "Must",
    "Should",
    "Can",
    "Need",
    "Make",
    "Create",
    "Update",
    "Delete",
    "Add",
    "Remove",
    "Get",
    "Set",
    "List",
    "Find",
    "Search",
    "Check",
    "Validate",
    "Process",
    "Handle",
    "Manage",
    "Ensure",
    "Allow",
    "Require",
    "Implement",
    "Build",
    "Test",
    "Deploy",
    "Config",
    "User",
    "Admin",
    "System",
    "Data",
    "Code",
    "File",
    "Each",
    "Every",
    "Some",
    "Which",
    "Where",
    "What",
    "After",
    "Before",
    "Between",
    "Without",
    "During",
    "About",
    "Above",
    "Below",
    "Under",
    "Over",
  ];

  COMMON_WORDS.contains(&word)
}

/// Match use cases to components based on text similarity
fn match_use_cases_to_components(
  use_cases: &[UseCase],
  components: &[Component],
  tasks: &[Task],
) -> Result<Vec<(Component, Vec<String>)>, CoverageError> {
  let mut matches = Vec::new();

  for component in components {
    let mut matched_use_case_ids = Vec::new();

    for use_case in use_cases {
      if component_covers_use_case(component, use_case, tasks) {
        matched_use_case_ids.push(use_case.id.clone());
      }
    }

    if !matched_use_case_ids.is_empty() {
      matches.push((component.clone(), matched_use_case_ids));
    }
  }

  Ok(matches)
}

/// Check if a component covers a use case
fn component_covers_use_case(component: &Component, use_case: &UseCase, tasks: &[Task]) -> bool {
  let component_lower = component.name.to_lowercase();

  // Check if component name appears in any task related to this use case
  for task in tasks {
    let task_text = task.combined_text().to_lowercase();

    // If component appears in task and use case keywords appear in task
    if task_text.contains(&component_lower) {
      // Check if use case is related to this task
      if use_case_related_to_task(use_case, task) {
        return true;
      }
    }
  }

  false
}

/// Check if a use case is related to a task
fn use_case_related_to_task(use_case: &UseCase, task: &Task) -> bool {
  let use_case_text = format!("{} {}", use_case.name, use_case.description).to_lowercase();
  let task_text = task.combined_text().to_lowercase();

  // Extract keywords from use case (words longer than 3 chars)
  let use_case_keywords: HashSet<_> = use_case_text
    .split_whitespace()
    .filter(|w| w.len() > 3)
    .collect();

  // Check if any significant keyword appears in task
  use_case_keywords
    .iter()
    .any(|keyword| task_text.contains(*keyword))
}

/// Generate coverage gaps with suggestions
fn generate_coverage_gaps(
  uncovered_use_cases: &[&UseCase],
  covered_components: &[CoveredComponent],
) -> Vec<CoverageGap> {
  uncovered_use_cases
    .iter()
    .map(|use_case| {
      // Extract potential components from use case description
      let missing_components = extract_missing_components(use_case, covered_components);

      // Generate suggestion
      let suggestion = generate_suggestion(use_case, &missing_components);

      CoverageGap::new(use_case.id.clone(), missing_components, suggestion)
    })
    .collect()
}

/// Extract missing components from use case
fn extract_missing_components(
  use_case: &UseCase,
  covered_components: &[CoveredComponent],
) -> Vec<String> {
  let covered_names: HashSet<_> = covered_components
    .iter()
    .map(|cc| cc.name.as_str())
    .collect();

  // Try to extract component names from use case
  let text = format!("{} {}", use_case.name, use_case.description);

  // Use functional iterator pipeline to extract components
  let components: Vec<String> = CAPITALIZED_WORD_PATTERN
    .captures_iter(&text)
    .filter_map(|cap| cap.get(1))
    .map(|m| m.as_str())
    .filter(|name| !is_common_word(name) && !covered_names.contains(*name))
    .map(|s| s.to_string())
    .collect();

  // If no components found, suggest based on use case
  if components.is_empty() {
    vec![
      format!("{}Handler", use_case.name),
      format!("{}Service", use_case.name),
    ]
  } else {
    components
  }
}

/// Generate suggestion for covering a use case
fn generate_suggestion(use_case: &UseCase, missing_components: &[String]) -> String {
  if missing_components.is_empty() {
    format!("Consider creating a component to handle: {}", use_case.name)
  } else {
    format!(
      "Implement {} to cover use case: {}",
      missing_components.join(", "),
      use_case.name
    )
  }
}

#[cfg(test)]
mod tests {
  #![allow(clippy::unwrap_used)]
  #![allow(clippy::expect_used)]
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
  #![allow(clippy::return_self_not_must_use)]
  #![allow(clippy::items_after_statements)]
  #![allow(clippy::ptr_arg)]
  #![allow(clippy::missing_fields_in_debug)]
  #![allow(clippy::must_use_unit)]
  #![allow(clippy::collection_is_never_read)]
  #![allow(clippy::manual_checked_ops)]
  #![allow(clippy::needless_pass_by_value)]

  use super::*;

  fn create_use_case(id: &str, name: &str, description: &str) -> UseCase {
    UseCase::new(id.to_string(), name.to_string(), description.to_string())
  }

  fn create_task(id: &str, title: &str, description: &str) -> Task {
    Task::new(id.to_string(), title.to_string(), description.to_string())
  }

  #[test]
  fn test_covered_component_valid_percentage() {
    let component = CoveredComponent::new("TestService".to_string(), vec!["uc1".to_string()], 75);
    assert!(component.is_ok());
  }

  #[test]
  fn test_covered_component_invalid_percentage() {
    let component = CoveredComponent::new("TestService".to_string(), vec!["uc1".to_string()], 101);
    assert!(matches!(
      component,
      Err(CoverageError::InvalidPercentage(_))
    ));
  }

  #[test]
  fn test_covered_component_percentage_boundary() {
    // Test 0%
    let comp0 = CoveredComponent::new("TestService".to_string(), vec![], 0);
    assert!(comp0.is_ok());

    // Test 100%
    let comp100 = CoveredComponent::new("TestService".to_string(), vec!["uc1".to_string()], 100);
    assert!(comp100.is_ok());
  }

  #[test]
  fn test_coverage_output_valid_percentage() {
    let output = CoverageOutput::new(vec![], vec![], 50, 10, 5);
    assert!(output.is_ok());
  }

  #[test]
  fn test_coverage_output_invalid_percentage() {
    let output = CoverageOutput::new(vec![], vec![], 150, 10, 5);
    assert!(matches!(output, Err(CoverageError::InvalidPercentage(_))));
  }

  #[test]
  fn test_coverage_output_meets_threshold() {
    let output = match CoverageOutput::new(vec![], vec![], 75, 10, 5) {
      Ok(o) => o,
      Err(_) => panic!("Expected valid output"),
    };

    assert!(output.meets_threshold(70));
    assert!(output.meets_threshold(75));
    assert!(!output.meets_threshold(80));
  }

  #[test]
  fn test_analyze_coverage_empty_use_cases() {
    let tasks = vec![create_task("t1", "Task", "Description")];
    let result = analyze_coverage(&[], &tasks);
    assert!(matches!(result, Err(CoverageError::EmptyUseCases)));
  }

  #[test]
  fn test_analyze_coverage_empty_tasks() {
    let use_cases = vec![create_use_case("uc1", "Use Case", "Description")];
    let result = analyze_coverage(&use_cases, &[]);
    assert!(matches!(result, Err(CoverageError::EmptyTasks)));
  }

  #[test]
  fn test_analyze_coverage_full_coverage() {
    let use_cases = vec![
      create_use_case("uc1", "Authentication", "User login functionality"),
      create_use_case("uc2", "DataStorage", "Persist user data"),
    ];

    let tasks = vec![
      create_task(
        "t1",
        "Implement AuthService",
        "Create AuthService component to handle user login and authentication",
      ),
      create_task(
        "t2",
        "Implement DatabaseService",
        "Create DatabaseService component to persist user data",
      ),
    ];

    let result = analyze_coverage(&use_cases, &tasks);
    assert!(result.is_ok());

    let output = match result {
      Ok(o) => o,
      Err(_) => panic!("Expected Ok result"),
    };

    // Should have 100% coverage
    assert_eq!(output.overall_coverage_percent, 100);
    assert_eq!(output.covered_use_cases_count, 2);
    assert_eq!(output.total_use_cases, 2);
  }

  #[test]
  fn test_analyze_coverage_partial_coverage() {
    let use_cases = vec![
      create_use_case("uc1", "Authentication", "User login functionality"),
      create_use_case("uc2", "Reporting", "Generate reports"),
    ];

    let tasks = vec![create_task(
      "t1",
      "Implement AuthService",
      "Create AuthService component to handle user login",
    )];

    let result = analyze_coverage(&use_cases, &tasks);
    assert!(result.is_ok());

    let output = match result {
      Ok(o) => o,
      Err(_) => panic!("Expected Ok result"),
    };

    // Should have 50% coverage (1 out of 2)
    assert_eq!(output.overall_coverage_percent, 50);
    assert_eq!(output.covered_use_cases_count, 1);
    assert_eq!(output.total_use_cases, 2);

    // Should have 1 coverage gap
    assert_eq!(output.coverage_gaps.len(), 1);
    assert_eq!(output.coverage_gaps[0].use_case, "uc2");
  }

  #[test]
  fn test_analyze_coverage_no_coverage() {
    let use_cases = vec![create_use_case(
      "uc1",
      "Authentication",
      "User login functionality",
    )];

    let tasks = vec![create_task(
      "t1",
      "Setup build process",
      "Configure webpack and build pipeline",
    )];

    let result = analyze_coverage(&use_cases, &tasks);
    assert!(result.is_ok());

    let output = match result {
      Ok(o) => o,
      Err(_) => panic!("Expected Ok result"),
    };

    // Should have 0% coverage
    assert_eq!(output.overall_coverage_percent, 0);
    assert_eq!(output.covered_use_cases_count, 0);

    // Should have 1 coverage gap
    assert_eq!(output.coverage_gaps.len(), 1);
  }

  #[test]
  fn test_extract_components_from_tasks() {
    let tasks = vec![
      create_task(
        "t1",
        "Implement AuthService",
        "Create AuthService component",
      ),
      create_task(
        "t2",
        "Create DatabaseRepository",
        "Implement DatabaseRepository for data access",
      ),
      create_task(
        "t3",
        "Build UserController",
        "Add UserController for REST API",
      ),
    ];

    let components = extract_components_from_tasks(&tasks).unwrap();

    // Should extract component names
    let names: Vec<_> = components.iter().map(|c| c.name.as_str()).collect();
    assert!(names.contains(&"AuthService"));
    assert!(names.contains(&"DatabaseRepository"));
    assert!(names.contains(&"UserController"));
  }

  #[test]
  fn test_extract_components_capitalized_words() {
    let tasks = vec![create_task(
      "t1",
      "Integrate PaymentGateway",
      "Connect to PaymentGateway for processing",
    )];

    let components = extract_components_from_tasks(&tasks).unwrap();

    let names: Vec<_> = components.iter().map(|c| c.name.as_str()).collect();
    assert!(names.contains(&"PaymentGateway"));
  }

  #[test]
  fn test_is_common_word() {
    assert!(is_common_word("The"));
    assert!(is_common_word("Create"));
    assert!(is_common_word("User"));

    assert!(!is_common_word("AuthService"));
    assert!(!is_common_word("PaymentGateway"));
  }

  #[test]
  fn test_component_covers_use_case() {
    let use_cases = vec![create_use_case("uc1", "Authentication", "User login")];
    let tasks = vec![create_task(
      "t1",
      "Implement AuthService",
      "Create AuthService to handle user login and authentication",
    )];

    let component = Component::new("AuthService".to_string());

    // Component should cover the use case
    assert!(component_covers_use_case(&component, &use_cases[0], &tasks));
  }

  #[test]
  fn test_use_case_related_to_task() {
    let use_case = create_use_case("uc1", "Authentication", "User login functionality");
    let task = create_task(
      "t1",
      "Implement AuthService",
      "Create AuthService to handle user login and authentication",
    );

    assert!(use_case_related_to_task(&use_case, &task));
  }

  #[test]
  fn test_generate_coverage_gaps() {
    let uc1 = create_use_case("uc1", "Reporting", "Generate sales reports");
    let uncovered = vec![&uc1];
    let covered_components =
      vec![CoveredComponent::new("AuthService".to_string(), vec!["uc2".to_string()], 100).unwrap()];

    let gaps = generate_coverage_gaps(&uncovered, &covered_components);

    assert_eq!(gaps.len(), 1);
    assert_eq!(gaps[0].use_case, "uc1");
    assert!(!gaps[0].suggestion.is_empty());
  }

  #[test]
  fn test_coverage_percentage_calculation() {
    let use_cases = vec![
      create_use_case("uc1", "Auth", "Authentication"),
      create_use_case("uc2", "Data", "Storage"),
      create_use_case("uc3", "UI", "Interface"),
      create_use_case("uc4", "API", "Integration"),
    ];

    let tasks = vec![
      create_task("t1", "AuthService", "Handle authentication"),
      create_task("t2", "DataService", "Store data"),
    ];

    let result = analyze_coverage(&use_cases, &tasks).unwrap();

    // 2 out of 4 = 50%
    assert_eq!(result.overall_coverage_percent, 50);
    assert_eq!(result.covered_use_cases_count, 2);
  }

  #[test]
  fn test_multiple_components_same_use_case() {
    let use_cases = vec![create_use_case("uc1", "Authentication", "User login")];

    let tasks = vec![
      create_task("t1", "AuthService", "Authentication service"),
      create_task("t2", "LoginController", "Handle login requests"),
    ];

    let result = analyze_coverage(&use_cases, &tasks).unwrap();

    // Should be covered (100%)
    assert_eq!(result.overall_coverage_percent, 100);
  }

  #[test]
  fn test_covered_component_from_matches() {
    let component = Component::new("TestService".to_string());
    let matches = vec!["uc1".to_string(), "uc2".to_string()];

    let covered = CoveredComponent::from_component_and_matches(component, matches, 4).unwrap();

    assert_eq!(covered.name, "TestService");
    assert_eq!(covered.coverage_percent, 50); // 2 out of 4
  }

  #[test]
  fn test_coverage_gap_with_suggestion() {
    let gap = CoverageGap::new(
      "uc1".to_string(),
      vec!["ReportingService".to_string()],
      "Implement ReportingService".to_string(),
    );

    assert_eq!(gap.use_case, "uc1");
    assert_eq!(gap.missing_components.len(), 1);
    assert_eq!(gap.suggestion, "Implement ReportingService");
  }
}
