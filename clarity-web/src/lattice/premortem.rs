#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![allow(clippy::suspicious_else_formatting)]
#![warn(clippy::nursery)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::redundant_clone)]
#![allow(clippy::needless_collect)]
#![forbid(unsafe_code)]

/// Likelihood of a failure scenario occurring
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Likelihood {
  /// Very likely to occur (>70% probability)
  VeryLikely,
  /// Possible to occur (30-70% probability)
  Possible,
  /// Unlikely to occur (<30% probability)
  Unlikely,
}

impl Likelihood {
  /// Returns the numeric probability threshold for this likelihood
  #[must_use]
  pub const fn probability_threshold(self) -> u8 {
    match self {
      Self::VeryLikely => 70,
      Self::Possible => 30,
      Self::Unlikely => 0,
    }
  }

  /// Returns true if this likelihood represents a high-risk scenario
  #[must_use]
  pub const fn is_high_risk(self) -> bool {
    matches!(self, Self::VeryLikely)
  }
}

/// Category of failure scenario
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FailureCategory {
  /// Technical failure (e.g., bugs, performance, scalability)
  Technical,
  /// User-related failure (e.g., adoption, usability, resistance)
  User,
  /// Business-related failure (e.g., cost, market fit, competition)
  Business,
  /// Security-related failure (e.g., data breaches, vulnerabilities)
  Security,
}

impl FailureCategory {
  /// Returns all possible failure categories
  #[must_use]
  pub const fn all() -> [Self; 4] {
    [Self::Technical, Self::User, Self::Business, Self::Security]
  }
}

/// A single failure scenario identified in the premortem
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FailureScenario {
  /// Category of this failure
  pub category: FailureCategory,
  /// Description of what could trigger this failure
  pub trigger: String,
  /// Description of the consequences if this failure occurs
  pub consequence: String,
  /// Estimated likelihood of this failure occurring
  pub likelihood: Likelihood,
  /// Suggested mitigation strategies
  pub mitigation: Vec<String>,
}

impl FailureScenario {
  /// Creates a new failure scenario
  #[must_use]
  pub fn new(
    category: FailureCategory,
    trigger: String,
    consequence: String,
    likelihood: Likelihood,
    mitigation: Vec<String>,
  ) -> Self {
    Self {
      category,
      trigger,
      consequence,
      likelihood,
      mitigation,
    }
  }

  /// Returns true if this scenario is high-risk
  #[must_use]
  pub const fn is_high_risk(&self) -> bool {
    self.likelihood.is_high_risk()
  }
}

/// Output from a premortem analysis
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PremortemOutput {
  /// The solution being analyzed
  pub solution: String,
  /// Constraints considered in the analysis
  pub constraints: Vec<String>,
  /// Identified failure scenarios
  pub scenarios: Vec<FailureScenario>,
  /// High-risk scenarios (likelihood >= VeryLikely)
  pub high_risk_scenarios: Vec<FailureScenario>,
}

impl PremortemOutput {
  /// Creates a new premortem output
  #[must_use]
  pub fn new(solution: String, constraints: Vec<String>, scenarios: Vec<FailureScenario>) -> Self {
    let high_risk_scenarios = scenarios
      .iter()
      .filter(|scenario| scenario.is_high_risk())
      .cloned()
      .collect();

    Self {
      solution,
      constraints,
      scenarios,
      high_risk_scenarios,
    }
  }

  /// Returns scenarios grouped by category
  #[must_use]
  pub fn scenarios_by_category(&self) -> Vec<(FailureCategory, Vec<FailureScenario>)> {
    FailureCategory::all()
      .iter()
      .map(|&category| {
        let category_scenarios: Vec<FailureScenario> = self
          .scenarios
          .iter()
          .filter(|s| s.category == category)
          .cloned()
          .collect();
        (category, category_scenarios)
      })
      .filter(|(_, scenarios)| !scenarios.is_empty())
      .collect()
  }

  /// Returns the total number of scenarios
  #[must_use]
  pub fn scenario_count(&self) -> usize {
    self.scenarios.len()
  }

  /// Returns the number of high-risk scenarios
  #[must_use]
  pub fn high_risk_count(&self) -> usize {
    self.high_risk_scenarios.len()
  }
}

/// Generates a premortem analysis for a given solution and constraints
///
/// # Arguments
/// * `solution` - The proposed solution to analyze
/// * `constraints` - Constraints and context for the solution
///
/// # Returns
/// A comprehensive premortem analysis with failure scenarios across all categories
#[must_use]
pub fn generate_premortem(solution: &str, constraints: &[&str]) -> PremortemOutput {
  let solution_lower = solution.to_lowercase();
  let constraints_lower: Vec<String> = constraints.iter().map(|c| c.to_lowercase()).collect();

  let scenarios = analyze_failure_scenarios(&solution_lower, &constraints_lower);

  PremortemOutput::new(
    solution.to_string(),
    constraints.iter().map(|&s| s.to_string()).collect(),
    scenarios,
  )
}

/// Analyzes potential failure scenarios based on solution and constraints
fn analyze_failure_scenarios(solution: &str, constraints: &[String]) -> Vec<FailureScenario> {
  let mut scenarios = Vec::new();

  // Technical scenarios
  scenarios.extend(analyze_technical_failures(solution, constraints));

  // User scenarios
  scenarios.extend(analyze_user_failures(solution, constraints));

  // Business scenarios
  scenarios.extend(analyze_business_failures(solution, constraints));

  // Security scenarios
  scenarios.extend(analyze_security_failures(solution, constraints));

  scenarios
}

/// Analyzes technical failure scenarios
fn analyze_technical_failures(solution: &str, _constraints: &[String]) -> Vec<FailureScenario> {
  let mut scenarios = Vec::new();

  // Performance bottlenecks
  let likelihood = if contains_any(solution, &["scale", "performance", "concurrent", "load"]) {
    Likelihood::VeryLikely
  } else if contains_any(solution, &["system", "application", "software"]) {
    Likelihood::Possible
  } else {
    Likelihood::Unlikely
  };

  scenarios.push(FailureScenario::new(
        FailureCategory::Technical,
        "Performance degradation under load".to_string(),
        "System becomes unresponsive or significantly slower when handling concurrent users or large datasets".to_string(),
        likelihood,
        vec![
            "Implement caching strategies for frequently accessed data".to_string(),
            "Add load testing in CI/CD pipeline to identify bottlenecks early".to_string(),
            "Design horizontal scaling capabilities from the start".to_string(),
            "Monitor performance metrics and set up alerting".to_string(),
        ],
    ));

  // Integration failures
  let integration_likelihood =
    if contains_any(solution, &["api", "integration", "external", "service"]) {
      Likelihood::Possible
    } else {
      Likelihood::Unlikely
    };

  scenarios.push(FailureScenario::new(
    FailureCategory::Technical,
    "Third-party API changes or failures".to_string(),
    "External dependencies change their API, become unavailable, or have unexpected downtime"
      .to_string(),
    integration_likelihood,
    vec![
      "Implement circuit breakers for external service calls".to_string(),
      "Use versioned API contracts and document assumptions".to_string(),
      "Design fallback mechanisms for critical dependencies".to_string(),
      "Monitor external service health and implement graceful degradation".to_string(),
    ],
  ));

  scenarios
}

/// Analyzes user-related failure scenarios
fn analyze_user_failures(solution: &str, _constraints: &[String]) -> Vec<FailureScenario> {
  let mut scenarios = Vec::new();

  // Poor adoption
  let adoption_likelihood =
    if contains_any(solution, &["new", "tool", "platform", "system", "workflow"]) {
      Likelihood::Possible
    } else {
      Likelihood::Unlikely
    };

  scenarios.push(FailureScenario::new(
        FailureCategory::User,
        "Low user adoption due to complexity".to_string(),
        "Users find the solution too complex or unfamiliar, leading to resistance and low adoption rates".to_string(),
        adoption_likelihood,
        vec![
            "Conduct user research and testing before full deployment".to_string(),
            "Provide comprehensive onboarding and training materials".to_string(),
            "Design intuitive UI/UX with user feedback loops".to_string(),
            "Identify and empower internal champions to drive adoption".to_string(),
            "Start with a pilot program to identify and address pain points".to_string(),
        ],
    ));

  // Usability issues
  scenarios.push(FailureScenario::new(
        FailureCategory::User,
        "Critical usability barriers block key workflows".to_string(),
        "Users encounter friction or blocking issues in common tasks, leading to workarounds or abandonment".to_string(),
        Likelihood::Possible,
        vec![
            "Map out key user journeys and optimize for most common tasks".to_string(),
            "Conduct usability testing with actual users regularly".to_string(),
            "Implement progressive disclosure to avoid overwhelming users".to_string(),
            "Track user interactions to identify friction points".to_string(),
        ],
    ));

  scenarios
}

/// Analyzes business-related failure scenarios
fn analyze_business_failures(solution: &str, _constraints: &[String]) -> Vec<FailureScenario> {
  let mut scenarios = Vec::new();

  // Cost overruns
  let cost_likelihood = if contains_any(
    solution,
    &["scale", "infrastructure", "cloud", "storage", "compute"],
  ) {
    Likelihood::Possible
  } else {
    Likelihood::Unlikely
  };

  scenarios.push(FailureScenario::new(
        FailureCategory::Business,
        "Operating costs exceed budget or ROI projections".to_string(),
        "Infrastructure, scaling, or operational costs grow faster than anticipated, making the solution economically unviable".to_string(),
        cost_likelihood,
        vec![
            "Create detailed cost projections with scaling scenarios".to_string(),
            "Implement cost monitoring and alerting from day one".to_string(),
            "Design cost optimization strategies (e.g., auto-scaling, spot instances)".to_string(),
            "Define clear success metrics and ROI thresholds".to_string(),
            "Review costs regularly and optimize resource usage".to_string(),
        ],
    ));

  // Competitive pressure
  scenarios.push(FailureScenario::new(
        FailureCategory::Business,
        "Competitors launch superior alternatives".to_string(),
        "Market dynamics shift and competitors introduce solutions that render this approach obsolete or less attractive".to_string(),
        Likelihood::Possible,
        vec![
            "Conduct competitive analysis before and during development".to_string(),
            "Focus on unique value propositions that are difficult to replicate".to_string(),
            "Build flexibility to adapt to market changes".to_string(),
            "Maintain a roadmap for continuous improvement".to_string(),
        ],
    ));

  scenarios
}

/// Analyzes security-related failure scenarios
fn analyze_security_failures(solution: &str, _constraints: &[String]) -> Vec<FailureScenario> {
  let mut scenarios = Vec::new();

  // Data breach
  let breach_likelihood = if contains_any(
    solution,
    &["data", "user", "authentication", "api", "network"],
  ) {
    Likelihood::Possible
  } else {
    Likelihood::Unlikely
  };

  scenarios.push(FailureScenario::new(
        FailureCategory::Security,
        "Data breach exposes sensitive information".to_string(),
        "Security vulnerabilities are exploited, leading to unauthorized access to sensitive data or systems".to_string(),
        breach_likelihood,
        vec![
            "Implement defense-in-depth with multiple security layers".to_string(),
            "Conduct regular security audits and penetration testing".to_string(),
            "Follow principle of least privilege for all access".to_string(),
            "Encrypt sensitive data at rest and in transit".to_string(),
            "Implement comprehensive logging and monitoring for security events".to_string(),
        ],
    ));

  // Authentication/authorization issues
  let auth_likelihood = if contains_any(
    solution,
    &["authentication", "authorization", "user", "access", "login"],
  ) {
    Likelihood::VeryLikely
  } else {
    Likelihood::Possible
  };

  scenarios.push(FailureScenario::new(
        FailureCategory::Security,
        "Authentication or authorization vulnerabilities".to_string(),
        "Attackers gain unauthorized access through weak authentication, session hijacking, or privilege escalation".to_string(),
        auth_likelihood,
        vec![
            "Implement multi-factor authentication for sensitive operations".to_string(),
            "Use established authentication libraries (avoid rolling your own)".to_string(),
            "Implement proper session management with secure cookies".to_string(),
            "Regularly audit and update dependencies for security patches".to_string(),
            "Implement rate limiting to prevent brute force attacks".to_string(),
        ],
    ));

  scenarios
}

/// Checks if the text contains any of the given keywords
fn contains_any(text: &str, keywords: &[&str]) -> bool {
  keywords.iter().any(|&keyword| text.contains(keyword))
}

#[cfg(test)]
mod tests {
  #![allow(clippy::unwrap_used)]
  #![allow(clippy::expect_used)]

  use super::*;

  #[test]
  fn test_generate_premortem_creates_scenarios() {
    let solution = "Build a scalable web application for user management";
    let constraints = vec!["Must handle 10k concurrent users", "Data must be encrypted"];

    let output = generate_premortem(solution, &constraints);

    assert!(
      !output.scenarios.is_empty(),
      "Should generate at least one scenario"
    );
    assert_eq!(output.solution, solution);
    assert_eq!(output.constraints.len(), constraints.len());
  }

  #[test]
  fn test_generates_reasonable_scenarios() {
    let solution = "Build a scalable API service";
    let constraints = vec!["Must be secure", "Low latency required"];

    let output = generate_premortem(solution, &constraints);

    // Should generate at least 3 scenarios (one per most categories)
    // Upper bound is flexible as we provide comprehensive coverage
    assert!(
      output.scenario_count() >= 3,
      "Should generate at least 3 scenarios, got {}",
      output.scenario_count()
    );

    // Should not be excessive
    assert!(
      output.scenario_count() <= 12,
      "Should not generate excessive scenarios, got {}",
      output.scenario_count()
    );
  }

  #[test]
  fn test_covers_all_categories() {
    let solution = "Build a comprehensive user platform with authentication";
    let constraints = vec!["Must scale globally", "GDPR compliance required"];

    let output = generate_premortem(solution, &constraints);
    let categories = output
      .scenarios
      .iter()
      .map(|s| s.category)
      .collect::<std::collections::HashSet<_>>();

    assert_eq!(
      categories.len(),
      4,
      "Should cover all 4 categories: {:?}",
      categories
    );
    assert!(categories.contains(&FailureCategory::Technical));
    assert!(categories.contains(&FailureCategory::User));
    assert!(categories.contains(&FailureCategory::Business));
    assert!(categories.contains(&FailureCategory::Security));
  }

  #[test]
  fn test_flags_high_risk_items() {
    let solution = "Build an authentication system with session management";
    let constraints = vec!["Must handle millions of users"];

    let output = generate_premortem(solution, &constraints);

    assert!(
      output.high_risk_count() > 0,
      "Should flag at least one high-risk scenario"
    );

    for scenario in &output.high_risk_scenarios {
      assert!(
        scenario.is_high_risk(),
        "High-risk scenario should have VeryLikely likelihood"
      );
    }
  }

  #[test]
  fn test_mitigations_are_actionable() {
    let solution = "Build a data processing pipeline";
    let constraints = vec!["Must process 1TB/day"];

    let output = generate_premortem(solution, &constraints);

    for scenario in &output.scenarios {
      assert!(
        !scenario.mitigation.is_empty(),
        "Each scenario should have mitigations: {:?}",
        scenario.trigger
      );

      for mitigation in &scenario.mitigation {
        assert!(!mitigation.is_empty(), "Mitigation should not be empty");
        assert!(
          mitigation.len() > 10,
          "Mitigation should be detailed: {mitigation}"
        );
      }
    }
  }

  #[test]
  fn test_scenarios_by_category() {
    let solution = "Build a comprehensive platform";
    let constraints = vec!["Must be scalable and secure"];

    let output = generate_premortem(solution, &constraints);
    let by_category = output.scenarios_by_category();

    assert!(
      !by_category.is_empty(),
      "Should have scenarios grouped by category"
    );

    for (category, scenarios) in by_category {
      assert!(
        !scenarios.is_empty(),
        "Category {category:?} should have scenarios"
      );
      for scenario in scenarios {
        assert_eq!(scenario.category, category);
      }
    }
  }

  #[test]
  fn test_likelihood_high_risk_detection() {
    assert!(Likelihood::VeryLikely.is_high_risk());
    assert!(!Likelihood::Possible.is_high_risk());
    assert!(!Likelihood::Unlikely.is_high_risk());
  }

  #[test]
  fn test_likelihood_thresholds() {
    assert_eq!(Likelihood::VeryLikely.probability_threshold(), 70);
    assert_eq!(Likelihood::Possible.probability_threshold(), 30);
    assert_eq!(Likelihood::Unlikely.probability_threshold(), 0);
  }

  #[test]
  fn test_scenario_is_high_risk() {
    let high_risk = FailureScenario::new(
      FailureCategory::Technical,
      "Test".to_string(),
      "Test".to_string(),
      Likelihood::VeryLikely,
      vec!["mitigation".to_string()],
    );
    assert!(high_risk.is_high_risk());

    let low_risk = FailureScenario::new(
      FailureCategory::Technical,
      "Test".to_string(),
      "Test".to_string(),
      Likelihood::Possible,
      vec!["mitigation".to_string()],
    );
    assert!(!low_risk.is_high_risk());
  }

  #[test]
  fn test_premortem_output_counts() {
    let scenarios = vec![
      FailureScenario::new(
        FailureCategory::Technical,
        "T1".to_string(),
        "C1".to_string(),
        Likelihood::VeryLikely,
        vec!["m1".to_string()],
      ),
      FailureScenario::new(
        FailureCategory::User,
        "T2".to_string(),
        "C2".to_string(),
        Likelihood::Possible,
        vec!["m2".to_string()],
      ),
    ];

    let output = PremortemOutput::new(
      "Test solution".to_string(),
      vec!["constraint1".to_string()],
      scenarios.clone(),
    );

    assert_eq!(output.scenario_count(), 2);
    assert_eq!(output.high_risk_count(), 1);
  }

  #[test]
  fn test_technical_failures_detected() {
    let solution = "Build a scalable API service";
    let constraints = vec!["Must handle load"];

    let output = generate_premortem(solution, &constraints);
    let tech_scenarios: Vec<_> = output
      .scenarios
      .iter()
      .filter(|s| s.category == FailureCategory::Technical)
      .collect();

    assert!(
      !tech_scenarios.is_empty(),
      "Should detect technical failures"
    );
  }

  #[test]
  fn test_security_failures_for_auth_systems() {
    let solution = "Build an authentication system with login and session management";
    let constraints = vec!["Must be secure"];

    let output = generate_premortem(solution, &constraints);
    let security_scenarios: Vec<_> = output
      .scenarios
      .iter()
      .filter(|s| s.category == FailureCategory::Security)
      .collect();

    assert!(
      !security_scenarios.is_empty(),
      "Should detect security failures"
    );

    // Authentication systems should have at least one high-risk scenario
    assert!(
      output.high_risk_count() > 0,
      "Auth systems should have high-risk security scenarios"
    );
  }

  #[test]
  fn test_user_failures_for_new_tools() {
    let solution = "Build a new collaboration tool for teams";
    let constraints = vec!["Must be easy to use"];

    let output = generate_premortem(solution, &constraints);
    let user_scenarios: Vec<_> = output
      .scenarios
      .iter()
      .filter(|s| s.category == FailureCategory::User)
      .collect();

    assert!(!user_scenarios.is_empty(), "Should detect user failures");
  }

  #[test]
  fn test_business_failures_for_cloud_solutions() {
    let solution = "Build a cloud infrastructure platform";
    let constraints = vec!["Must scale automatically"];

    let output = generate_premortem(solution, &constraints);
    let business_scenarios: Vec<_> = output
      .scenarios
      .iter()
      .filter(|s| s.category == FailureCategory::Business)
      .collect();

    assert!(
      !business_scenarios.is_empty(),
      "Should detect business failures"
    );
  }

  #[test]
  fn test_contains_any_helper() {
    let text = "This is a scalable API service";
    assert!(contains_any(text, &["scalable", "api"]));
    assert!(!contains_any(text, &["database", "storage"]));
    assert!(contains_any(text, &["notfound", "service"]));
  }

  #[test]
  fn test_failure_category_all() {
    let all = FailureCategory::all();
    assert_eq!(all.len(), 4);
    assert!(all.contains(&FailureCategory::Technical));
    assert!(all.contains(&FailureCategory::User));
    assert!(all.contains(&FailureCategory::Business));
    assert!(all.contains(&FailureCategory::Security));
  }

  #[test]
  fn test_mitigation_strategies_are_unique() {
    let solution = "Build a web application";
    let constraints = vec!["Must be secure"];

    let output = generate_premortem(solution, &constraints);

    for scenario in &output.scenarios {
      let unique_mitigations: std::collections::HashSet<_> = scenario.mitigation.iter().collect();
      assert_eq!(
        unique_mitigations.len(),
        scenario.mitigation.len(),
        "Mitigations should be unique for scenario: {}",
        scenario.trigger
      );
    }
  }
}
