#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![allow(clippy::suspicious_else_formatting)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Persona Forge - Realistic User Persona Generator
//!
//! Creates user personas with realistic human limitations to prevent "Straw Man" users.
//! Validates personas against irrational actor detection.
//!
//! # Human Limitations
//!
//! Every persona includes universal human limitations:
//! - Lazy (avoids unnecessary effort)
//! - Distracted (limited attention span)
//! - Risk-averse (fears loss more than values gain)
//! - Impatient (wants immediate results)
//! - Forgetful (imperfect memory)
//!
//! # Example
//!
//! ```
//! use clarity_web::pme::discover::persona_forge::{PersonaForge, Persona, HumanLimitation};
//!
//! let persona = Persona::new("Enterprise Analyst".to_string())
//!     .with_demographic("Age 30-45, Mid-career".to_string())
//!     .with_limitation(HumanLimitation::Lazy, 0.7);
//!
//! let validation = PersonaForge::validate(&persona);
//! ```

use itertools::Itertools;
use serde::{Deserialize, Serialize};
use thiserror::Error;

// ============================================================================
// Human Limitations
// ============================================================================

/// Universal human limitations that affect product adoption and usage.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HumanLimitation {
  /// Avoids unnecessary effort - will seek easier alternatives
  Lazy,
  /// Limited attention span - easily interrupted
  Distracted,
  /// Fears loss more than values gain - resistant to change
  RiskAverse,
  /// Wants immediate results - low tolerance for delay
  Impatient,
  /// Imperfect memory - needs reminders and guidance
  Forgetful,
}

impl HumanLimitation {
  /// Get the name of this limitation.
  #[must_use]
  pub const fn name(&self) -> &'static str {
    match self {
      Self::Lazy => "Lazy",
      Self::Distracted => "Distracted",
      Self::RiskAverse => "Risk-Averse",
      Self::Impatient => "Impatient",
      Self::Forgetful => "Forgetful",
    }
  }

  /// Get a description of how this limitation affects behavior.
  #[must_use]
  pub const fn description(&self) -> &'static str {
    match self {
      Self::Lazy => "Avoids unnecessary effort; will seek easier alternatives even if suboptimal",
      Self::Distracted => "Limited attention span; easily interrupted and may abandon tasks",
      Self::RiskAverse => "Fears loss more than values gain; resistant to change and new workflows",
      Self::Impatient => "Wants immediate results; low tolerance for delay or learning curves",
      Self::Forgetful => "Imperfect memory; needs reminders, clear guidance, and recovery paths",
    }
  }

  /// Get all human limitations.
  #[must_use]
  pub const fn all() -> [Self; 5] {
    [
      Self::Lazy,
      Self::Distracted,
      Self::RiskAverse,
      Self::Impatient,
      Self::Forgetful,
    ]
  }

  /// Get design implications for this limitation.
  #[must_use]
  pub fn design_implications(&self) -> Vec<&'static str> {
    match self {
      Self::Lazy => vec![
        "Minimize steps required to complete tasks",
        "Provide sensible defaults",
        "Automate repetitive actions",
        "Make the easy path the right path",
      ],
      Self::Distracted => vec![
        "Support interruption and resumption",
        "Provide clear progress indicators",
        "Avoid requiring sustained attention",
        "Enable quick context recovery",
      ],
      Self::RiskAverse => vec![
        "Provide clear rollback options",
        "Show social proof and testimonials",
        "Offer trial periods and guarantees",
        "Minimize commitment required upfront",
      ],
      Self::Impatient => vec![
        "Optimize for speed and responsiveness",
        "Show immediate value/progress",
        "Minimize onboarding time",
        "Provide quick wins early",
      ],
      Self::Forgetful => vec![
        "Provide clear navigation and wayfinding",
        "Offer reminders and notifications",
        "Enable easy recovery from mistakes",
        "Document workflows clearly",
      ],
    }
  }
}

// ============================================================================
// Persona Types
// ============================================================================

/// A user persona with demographics, resources, and human limitations.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Persona {
  /// Persona name/identifier
  pub name: String,
  /// Demographic information
  pub demographics: Vec<String>,
  /// Available resources (time, money, skills)
  pub resources: Resources,
  /// Human limitations with severity (0.0-1.0)
  pub limitations: Vec<(HumanLimitation, f64)>,
  /// Goals and motivations
  pub goals: Vec<String>,
  /// Pain points and frustrations
  pub pain_points: Vec<String>,
  /// Current behaviors and workarounds
  pub current_behaviors: Vec<String>,
  /// Technology comfort level (0.0-1.0)
  pub tech_comfort: f64,
  /// Decision-making authority (0.0-1.0)
  pub decision_authority: f64,
}

/// Resources available to a persona.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Resources {
  /// Time availability (0.0 = none, 1.0 = abundant)
  pub time: f64,
  /// Budget/money (0.0 = none, 1.0 = abundant)
  pub budget: f64,
  /// Skill level for relevant domain (0.0 = novice, 1.0 = expert)
  pub skill: f64,
  /// Social capital (network, influence)
  pub social_capital: f64,
}

impl Default for Resources {
  fn default() -> Self {
    Self {
      time: 0.5,
      budget: 0.5,
      skill: 0.5,
      social_capital: 0.5,
    }
  }
}

impl Persona {
  /// Create a new persona with the given name.
  #[must_use]
  pub fn new(name: String) -> Self {
    Self {
      name,
      demographics: Vec::new(),
      resources: Resources::default(),
      limitations: HumanLimitation::all().iter().map(|&l| (l, 0.5)).collect(),
      goals: Vec::new(),
      pain_points: Vec::new(),
      current_behaviors: Vec::new(),
      tech_comfort: 0.5,
      decision_authority: 0.5,
    }
  }

  /// Add demographic information.
  #[must_use]
  pub fn with_demographic(mut self, demographic: String) -> Self {
    self.demographics.push(demographic);
    self
  }

  /// Set resources.
  #[must_use]
  pub fn with_resources(mut self, resources: Resources) -> Self {
    self.resources = resources;
    self
  }

  /// Set a specific human limitation severity.
  #[must_use]
  pub fn with_limitation(mut self, limitation: HumanLimitation, severity: f64) -> Self {
    let severity = severity.clamp(0.0, 1.0);
    for (l, s) in &mut self.limitations {
      if *l == limitation {
        *s = severity;
        return self;
      }
    }
    self.limitations.push((limitation, severity));
    self
  }

  /// Add a goal.
  #[must_use]
  pub fn with_goal(mut self, goal: String) -> Self {
    self.goals.push(goal);
    self
  }

  /// Add a pain point.
  #[must_use]
  pub fn with_pain_point(mut self, pain_point: String) -> Self {
    self.pain_points.push(pain_point);
    self
  }

  /// Add a current behavior.
  #[must_use]
  pub fn with_behavior(mut self, behavior: String) -> Self {
    self.current_behaviors.push(behavior);
    self
  }

  /// Set technology comfort level.
  #[must_use]
  pub fn with_tech_comfort(mut self, level: f64) -> Self {
    self.tech_comfort = level.clamp(0.0, 1.0);
    self
  }

  /// Set decision authority level.
  #[must_use]
  pub fn with_decision_authority(mut self, level: f64) -> Self {
    self.decision_authority = level.clamp(0.0, 1.0);
    self
  }

  /// Get the severity of a specific limitation.
  #[must_use]
  pub fn get_limitation(&self, limitation: HumanLimitation) -> f64 {
    self
      .limitations
      .iter()
      .find(|(l, _)| *l == limitation)
      .map(|(_, s)| *s)
      .unwrap_or(0.5)
  }

  /// Check if persona has high severity for any limitation.
  #[must_use]
  pub fn has_severe_limitation(&self, threshold: f64) -> bool {
    self.limitations.iter().any(|(_, s)| *s >= threshold)
  }

  /// Calculate overall friction score (higher = more friction to adopt).
  #[must_use]
  pub fn friction_score(&self) -> f64 {
    let limitation_avg = self.limitations.iter().map(|(_, s)| s).sum::<f64>()
      / f64::from(u8::try_from(self.limitations.len()).unwrap_or(1));

    let resource_friction = 1.0
      - (self.resources.time * 0.3
        + self.resources.budget * 0.3
        + self.resources.skill * 0.2
        + self.resources.social_capital * 0.2);

    let authority_friction = 1.0 - self.decision_authority;

    (limitation_avg * 0.4 + resource_friction * 0.4 + authority_friction * 0.2).clamp(0.0, 1.0)
  }
}

// ============================================================================
// Validation Types
// ============================================================================

/// Result of persona validation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ValidationResult {
  /// Whether the persona is valid
  pub is_valid: bool,
  /// Issues detected
  pub issues: Vec<ValidationIssue>,
  /// Irrational actor warnings
  pub irrational_warnings: Vec<String>,
  /// Suggestions for improvement
  pub suggestions: Vec<String>,
  /// Realism score (0.0-1.0)
  pub realism_score: f64,
}

/// An issue detected during validation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ValidationIssue {
  /// Issue category
  pub category: IssueCategory,
  /// Description of the issue
  pub description: String,
  /// Severity (0.0-1.0)
  pub severity: f64,
}

/// Categories of validation issues.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IssueCategory {
  /// Missing required information
  MissingInformation,
  /// Unrealistic characteristics
  Unrealistic,
  /// Contradictory attributes
  Contradictory,
  /// Irrational actor detected
  IrrationalActor,
  /// Insufficient limitations
  InsufficientLimitations,
}

// ============================================================================
// Output Types
// ============================================================================

/// Output from the Persona Forge.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PersonaOutput {
  /// The created personas
  pub personas: Vec<Persona>,
  /// Validation results for each persona
  pub validations: Vec<ValidationResult>,
  /// Aggregate statistics
  pub stats: PersonaStats,
  /// Design recommendations based on personas
  pub design_recommendations: Vec<String>,
}

/// Statistics about created personas.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PersonaStats {
  /// Total personas created
  pub total_personas: usize,
  /// Average friction score
  pub avg_friction: f64,
  /// Average realism score
  pub avg_realism: f64,
  /// Personas with severe limitations
  pub severe_limitation_count: usize,
  /// High authority personas
  pub high_authority_count: usize,
}

// ============================================================================
// Error Types
// ============================================================================

/// Errors from the Persona Forge.
#[derive(Debug, Error)]
pub enum PersonaError {
  /// Persona name is empty
  #[error("Persona name cannot be empty")]
  EmptyName,

  /// No limitations defined
  #[error("At least one human limitation must be defined")]
  NoLimitations,

  /// Invalid resource value
  #[error("Resource values must be between 0.0 and 1.0")]
  InvalidResourceValue,

  /// Irrational actor detected
  #[error("Irrational actor detected: {0}")]
  IrrationalActor(String),
}

// ============================================================================
// Persona Forge Implementation
// ============================================================================

/// Persona Forge - Creates and validates realistic user personas.
pub struct PersonaForge;

impl PersonaForge {
  /// Validate a persona for realism and rationality.
  #[must_use]
  pub fn validate(persona: &Persona) -> ValidationResult {
    let mut issues = Vec::new();
    let mut irrational_warnings = Vec::new();
    let mut suggestions = Vec::new();

    // Check for missing information
    if persona.demographics.is_empty() {
      issues.push(ValidationIssue {
        category: IssueCategory::MissingInformation,
        description: "No demographic information provided".to_string(),
        severity: 0.3,
      });
      suggestions.push("Add demographic details (age, role, industry, etc.)".to_string());
    }

    if persona.goals.is_empty() {
      issues.push(ValidationIssue {
        category: IssueCategory::MissingInformation,
        description: "No goals defined".to_string(),
        severity: 0.4,
      });
      suggestions.push("Define at least 2-3 goals this persona wants to achieve".to_string());
    }

    if persona.pain_points.is_empty() {
      issues.push(ValidationIssue {
        category: IssueCategory::MissingInformation,
        description: "No pain points defined".to_string(),
        severity: 0.4,
      });
      suggestions.push("Document current frustrations and challenges".to_string());
    }

    // Check for insufficient limitations
    let avg_limitation = persona.limitations.iter().map(|(_, s)| s).sum::<f64>()
      / f64::from(u8::try_from(persona.limitations.len()).unwrap_or(1));

    if avg_limitation < 0.3 {
      issues.push(ValidationIssue {
        category: IssueCategory::InsufficientLimitations,
        description: "Human limitations are too low - this may be a 'Straw Man' persona"
          .to_string(),
        severity: 0.5,
      });
      suggestions
        .push("Increase limitation severities to reflect realistic human behavior".to_string());
    }

    // Check for irrational actors
    let irrational_checks = Self::check_irrational_actor(persona);
    for warning in irrational_checks {
      issues.push(ValidationIssue {
        category: IssueCategory::IrrationalActor,
        description: warning.clone(),
        severity: 0.6,
      });
      irrational_warnings.push(warning);
    }

    // Check for contradictions
    if persona.resources.budget < 0.2 && persona.decision_authority > 0.8 {
      issues.push(ValidationIssue {
        category: IssueCategory::Contradictory,
        description: "Low budget but high decision authority may indicate an unrealistic persona"
          .to_string(),
        severity: 0.4,
      });
    }

    if persona.tech_comfort < 0.3 && persona.resources.skill > 0.8 {
      issues.push(ValidationIssue {
        category: IssueCategory::Contradictory,
        description: "Low tech comfort but high skill level may be contradictory".to_string(),
        severity: 0.3,
      });
    }

    // Calculate realism score
    let realism_score = Self::calculate_realism_score(&issues, persona);

    // Add general suggestions
    if realism_score < 0.7 {
      suggestions
        .push("Consider adding more context about daily workflow and environment".to_string());
    }

    let is_valid = !issues.iter().any(|i| i.severity > 0.7);

    ValidationResult {
      is_valid,
      issues,
      irrational_warnings,
      suggestions,
      realism_score,
    }
  }

  /// Check for irrational actor patterns.
  fn check_irrational_actor(persona: &Persona) -> Vec<String> {
    let mut warnings = Vec::new();

    // Check: Perfect patience with no resources
    if persona.get_limitation(HumanLimitation::Impatient) < 0.3 && persona.resources.time < 0.3 {
      warnings.push("Unrealistically patient persona with very limited time".to_string());
    }

    // Check: Risk-loving with high decision authority but no budget
    if persona.get_limitation(HumanLimitation::RiskAverse) < 0.2
      && persona.decision_authority > 0.7
      && persona.resources.budget < 0.3
    {
      warnings.push("Risk-seeking decision maker with limited budget is unusual".to_string());
    }

    // Check: No forgetfulness but low tech comfort
    if persona.get_limitation(HumanLimitation::Forgetful) < 0.2 && persona.tech_comfort < 0.3 {
      warnings
        .push("Perfect memory with low tech comfort may indicate unrealistic persona".to_string());
    }

    // Check: Super-human across all limitations
    let low_limitation_count = persona.limitations.iter().filter(|(_, s)| *s < 0.2).count();
    if low_limitation_count >= 4 {
      warnings.push(
        "Persona appears super-human (very low limitations across most dimensions)".to_string(),
      );
    }

    // Check: No current behaviors defined despite pain points
    if !persona.pain_points.is_empty() && persona.current_behaviors.is_empty() {
      warnings.push(
        "Persona has pain points but no current behaviors - how are they coping now?".to_string(),
      );
    }

    warnings
  }

  /// Calculate realism score based on issues and persona completeness.
  fn calculate_realism_score(issues: &[ValidationIssue], persona: &Persona) -> f64 {
    let base_score = 1.0;

    // Penalty for issues
    let issue_penalty: f64 = issues
      .iter()
      .map(|i| i.severity * 0.15)
      .sum::<f64>()
      .min(0.5);

    // Bonus for completeness
    let completeness_bonus = {
      let mut bonus = 0.0;
      if !persona.demographics.is_empty() {
        bonus += 0.05;
      }
      if persona.goals.len() >= 2 {
        bonus += 0.05;
      }
      if persona.pain_points.len() >= 2 {
        bonus += 0.05;
      }
      if !persona.current_behaviors.is_empty() {
        bonus += 0.05;
      }
      bonus
    };

    // Penalty for unrealistic limitation profile
    let avg_limitation = persona.limitations.iter().map(|(_, s)| s).sum::<f64>()
      / f64::from(u8::try_from(persona.limitations.len()).unwrap_or(1));
    let limitation_penalty = if avg_limitation < 0.3 { 0.15 } else { 0.0 };

    (base_score - issue_penalty - limitation_penalty + completeness_bonus).clamp(0.0, 1.0)
  }

  /// Generate design recommendations based on persona limitations.
  #[must_use]
  pub fn generate_design_recommendations(persona: &Persona) -> Vec<String> {
    let mut recommendations = Vec::new();

    // Sort limitations by severity
    let sorted_limitations: Vec<_> = persona
      .limitations
      .iter()
      .sorted_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal))
      .take(3)
      .collect();

    for (limitation, severity) in sorted_limitations {
      if *severity > 0.5 {
        let implication = limitation.design_implications();
        recommendations.extend(
          implication
            .into_iter()
            .take(2)
            .map(|i| format!("[{:.0}% {}] {}", severity * 100.0, limitation.name(), i)),
        );
      }
    }

    // Resource-based recommendations
    if persona.resources.time < 0.3 {
      recommendations
        .push("Persona has limited time - prioritize quick wins and efficiency".to_string());
    }
    if persona.resources.budget < 0.3 {
      recommendations.push(
        "Persona has limited budget - consider freemium or low-cost entry points".to_string(),
      );
    }

    recommendations
  }

  /// Create a persona set with validation.
  ///
  /// # Errors
  ///
  /// Returns an error if any persona has an empty name.
  pub fn create_persona_set(personas: Vec<Persona>) -> Result<PersonaOutput, PersonaError> {
    if personas.iter().any(|p| p.name.is_empty()) {
      return Err(PersonaError::EmptyName);
    }

    let validations: Vec<ValidationResult> = personas.iter().map(|p| Self::validate(p)).collect();

    let stats = PersonaStats {
      total_personas: personas.len(),
      avg_friction: personas.iter().map(|p| p.friction_score()).sum::<f64>()
        / f64::from(u8::try_from(personas.len()).unwrap_or(1)),
      avg_realism: validations.iter().map(|v| v.realism_score).sum::<f64>()
        / f64::from(u8::try_from(validations.len()).unwrap_or(1)),
      severe_limitation_count: personas
        .iter()
        .filter(|p| p.has_severe_limitation(0.7))
        .count(),
      high_authority_count: personas
        .iter()
        .filter(|p| p.decision_authority > 0.7)
        .count(),
    };

    // Aggregate design recommendations
    let design_recommendations = personas
      .iter()
      .flat_map(|p| Self::generate_design_recommendations(p))
      .unique()
      .take(10)
      .collect();

    Ok(PersonaOutput {
      personas,
      validations,
      stats,
      design_recommendations,
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn create_test_persona() -> Persona {
    Persona::new("Enterprise Analyst".to_string())
      .with_demographic("Age 30-45, Finance industry".to_string())
      .with_limitation(HumanLimitation::Lazy, 0.7)
      .with_limitation(HumanLimitation::Impatient, 0.8)
      .with_goal("Generate monthly reports faster".to_string())
      .with_pain_point("Current process takes 10 hours/week".to_string())
      .with_behavior("Uses Excel and manual data collection".to_string())
      .with_tech_comfort(0.6)
      .with_decision_authority(0.4)
  }

  #[test]
  fn test_persona_creation() {
    let persona = create_test_persona();

    assert_eq!(persona.name, "Enterprise Analyst");
    assert!(!persona.demographics.is_empty());
    assert!(!persona.goals.is_empty());
    assert!(!persona.pain_points.is_empty());
  }

  #[test]
  fn test_human_limitation_all() {
    let all = HumanLimitation::all();
    assert_eq!(all.len(), 5);
  }

  #[test]
  fn test_limitation_design_implications() {
    let implications = HumanLimitation::Lazy.design_implications();
    assert!(!implications.is_empty());
    assert!(implications.iter().all(|i| !i.is_empty()));
  }

  #[test]
  fn test_persona_get_limitation() {
    let persona = create_test_persona();

    let lazy_severity = persona.get_limitation(HumanLimitation::Lazy);
    assert!((lazy_severity - 0.7).abs() < 0.01);

    let impatient_severity = persona.get_limitation(HumanLimitation::Impatient);
    assert!((impatient_severity - 0.8).abs() < 0.01);
  }

  #[test]
  fn test_persona_has_severe_limitation() {
    let persona = create_test_persona();

    assert!(persona.has_severe_limitation(0.7));
    assert!(!persona.has_severe_limitation(0.9));
  }

  #[test]
  fn test_persona_friction_score() {
    let low_friction = Persona::new("Low Friction".to_string())
      .with_limitation(HumanLimitation::Lazy, 0.2)
      .with_limitation(HumanLimitation::Impatient, 0.2)
      .with_decision_authority(0.9);

    let high_friction = Persona::new("High Friction".to_string())
      .with_limitation(HumanLimitation::Lazy, 0.9)
      .with_limitation(HumanLimitation::Impatient, 0.9)
      .with_decision_authority(0.1);

    assert!(low_friction.friction_score() < high_friction.friction_score());
  }

  #[test]
  fn test_persona_forge_validate_valid() {
    let persona = create_test_persona();
    let result = PersonaForge::validate(&persona);

    assert!(result.is_valid);
    assert!(result.realism_score > 0.5);
  }

  #[test]
  fn test_persona_forge_validate_missing_info() {
    let persona = Persona::new("Incomplete".to_string());
    let result = PersonaForge::validate(&persona);

    assert!(!result.issues.is_empty());
    assert!(!result.suggestions.is_empty());
  }

  #[test]
  fn test_persona_forge_validate_straw_man() {
    // Create a "Straw Man" persona with very low limitations
    let persona = Persona::new("Super Human".to_string())
      .with_limitation(HumanLimitation::Lazy, 0.1)
      .with_limitation(HumanLimitation::Distracted, 0.1)
      .with_limitation(HumanLimitation::RiskAverse, 0.1)
      .with_limitation(HumanLimitation::Impatient, 0.1)
      .with_limitation(HumanLimitation::Forgetful, 0.1)
      .with_goal("Test goal".to_string())
      .with_pain_point("Test pain".to_string());

    let result = PersonaForge::validate(&persona);

    assert!(result
      .issues
      .iter()
      .any(|i| i.category == IssueCategory::InsufficientLimitations));
  }

  #[test]
  fn test_persona_forge_irrational_actor_detection() {
    // Perfectly patient with no time
    let persona = Persona::new("Irrational".to_string())
      .with_limitation(HumanLimitation::Impatient, 0.1)
      .with_resources(Resources {
        time: 0.1,
        budget: 0.5,
        skill: 0.5,
        social_capital: 0.5,
      })
      .with_goal("Test".to_string())
      .with_pain_point("Test".to_string());

    let result = PersonaForge::validate(&persona);

    assert!(!result.irrational_warnings.is_empty());
  }

  #[test]
  fn test_persona_forge_contradiction_detection() {
    let persona = Persona::new("Contradictory".to_string())
      .with_resources(Resources {
        budget: 0.1,
        ..Resources::default()
      })
      .with_decision_authority(0.9)
      .with_goal("Test".to_string())
      .with_pain_point("Test".to_string());

    let result = PersonaForge::validate(&persona);

    assert!(result
      .issues
      .iter()
      .any(|i| i.category == IssueCategory::Contradictory));
  }

  #[test]
  fn test_design_recommendations() {
    let persona = Persona::new("Test".to_string())
      .with_limitation(HumanLimitation::Lazy, 0.8)
      .with_limitation(HumanLimitation::Impatient, 0.7)
      .with_resources(Resources {
        time: 0.2,
        ..Resources::default()
      });

    let recommendations = PersonaForge::generate_design_recommendations(&persona);

    assert!(!recommendations.is_empty());
  }

  #[test]
  fn test_create_persona_set_empty_name() {
    let invalid_persona = Persona::new("".to_string());
    let result = PersonaForge::create_persona_set(vec![invalid_persona]);

    assert!(result.is_err());
    assert!(matches!(result, Err(PersonaError::EmptyName)));
  }

  #[test]
  fn test_create_persona_set_success() {
    let persona1 = create_test_persona();
    let persona2 = Persona::new("Manager".to_string())
      .with_goal("Approve reports".to_string())
      .with_pain_point("Review process is slow".to_string());

    let result = PersonaForge::create_persona_set(vec![persona1, persona2]);

    assert!(result.is_ok());
    let output = result.expect("Should succeed");

    assert_eq!(output.personas.len(), 2);
    assert_eq!(output.validations.len(), 2);
    assert_eq!(output.stats.total_personas, 2);
  }

  #[test]
  fn test_resources_default() {
    let resources = Resources::default();

    assert!((resources.time - 0.5).abs() < 0.01);
    assert!((resources.budget - 0.5).abs() < 0.01);
    assert!((resources.skill - 0.5).abs() < 0.01);
  }

  #[test]
  fn test_persona_with_resources() {
    let resources = Resources {
      time: 0.8,
      budget: 0.3,
      skill: 0.9,
      social_capital: 0.6,
    };

    let persona = Persona::new("Test".to_string()).with_resources(resources.clone());

    assert_eq!(persona.resources, resources);
  }

  #[test]
  fn test_validation_result_realism_score() {
    let valid_persona = create_test_persona()
      .with_goal("Second goal".to_string())
      .with_pain_point("Second pain".to_string());

    let result = PersonaForge::validate(&valid_persona);

    assert!(result.realism_score > 0.7);
  }

  #[test]
  fn test_limitation_severity_clamping() {
    let persona = Persona::new("Test".to_string()).with_limitation(HumanLimitation::Lazy, 1.5); // Invalid

    let severity = persona.get_limitation(HumanLimitation::Lazy);
    assert!((severity - 1.0).abs() < 0.01);
  }

  #[test]
  fn test_tech_comfort_clamping() {
    let persona = Persona::new("Test".to_string()).with_tech_comfort(1.5);

    assert!((persona.tech_comfort - 1.0).abs() < 0.01);
  }

  #[test]
  fn test_decision_authority_clamping() {
    let persona = Persona::new("Test".to_string()).with_decision_authority(-0.5);

    assert!((persona.decision_authority - 0.0).abs() < 0.01);
  }
}
