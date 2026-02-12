//! Persona Forge Module - First Principle: Prevent "Straw Man" Users
//!
//! Creates realistic user personas with demographics, means (resources),
//! and universal human limitations to prevent designing for mythical
//! "perfect" users who don't exist in reality.
//!
//! Key insight: Real users are lazy, distracted, risk-averse, impatient, and forgetful.
//! A persona without these limitations is a "Straw Man" - an unrealistic user model.

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::missing_const_for_fn)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;
use uuid::Uuid;

// ============================================================================
// EDUCATION LEVEL
// ============================================================================

/// Education level for demographics
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum EducationLevel {
  #[default]
  None,
  HighSchool,
  Bachelors,
  Masters,
  Doctorate,
}

impl fmt::Display for EducationLevel {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::None => write!(f, "No formal education"),
      Self::HighSchool => write!(f, "High School"),
      Self::Bachelors => write!(f, "Bachelor's Degree"),
      Self::Masters => write!(f, "Master's Degree"),
      Self::Doctorate => write!(f, "Doctorate"),
    }
  }
}

// ============================================================================
// SKILL LEVEL
// ============================================================================

/// Technical skill level
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SkillLevel {
  #[default]
  Novice,
  Intermediate,
  Advanced,
  Expert,
}

impl fmt::Display for SkillLevel {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Novice => write!(f, "Novice"),
      Self::Intermediate => write!(f, "Intermediate"),
      Self::Advanced => write!(f, "Advanced"),
      Self::Expert => write!(f, "Expert"),
    }
  }
}

// ============================================================================
// AUTHORITY LEVEL
// ============================================================================

/// Decision-making authority level
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AuthorityLevel {
  #[default]
  None,
  Some,
  Full,
}

impl fmt::Display for AuthorityLevel {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::None => write!(f, "No authority"),
      Self::Some => write!(f, "Some authority"),
      Self::Full => write!(f, "Full authority"),
    }
  }
}

// ============================================================================
// DEMOGRAPHICS
// ============================================================================

/// Demographic configuration for a persona
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Demographics {
  /// Age range (min, max)
  pub age_range: (u8, u8),
  /// Occupation or job title
  pub occupation: String,
  /// Annual income range (min, max) in USD
  pub income_range: (u32, u32),
  /// Education level
  pub education: EducationLevel,
  /// Geographic location (country/region)
  pub location: String,
}

impl Demographics {
  /// Create new demographics configuration
  ///
  /// # Errors
  /// Returns `PersonaError::InvalidAgeRange` if min > max or values are invalid
  /// Returns `PersonaError::EmptyField` if occupation or location is empty
  /// Returns `PersonaError::InvalidIncomeRange` if min > max
  #[allow(clippy::too_many_arguments)]
  pub fn new(
    age_min: u8,
    age_max: u8,
    occupation: String,
    income_min: u32,
    income_max: u32,
    education: EducationLevel,
    location: String,
  ) -> Result<Self, PersonaError> {
    if age_min > age_max {
      return Err(PersonaError::InvalidAgeRange {
        min: age_min,
        max: age_max,
      });
    }
    if age_min == 0 {
      return Err(PersonaError::InvalidAgeRange {
        min: age_min,
        max: age_max,
      });
    }
    if occupation.trim().is_empty() {
      return Err(PersonaError::EmptyField {
        field: "occupation".to_string(),
      });
    }
    if location.trim().is_empty() {
      return Err(PersonaError::EmptyField {
        field: "location".to_string(),
      });
    }
    if income_min > income_max {
      return Err(PersonaError::InvalidIncomeRange {
        min: income_min,
        max: income_max,
      });
    }

    Ok(Self {
      age_range: (age_min, age_max),
      occupation,
      income_range: (income_min, income_max),
      education,
      location,
    })
  }

  /// Check if age falls within range
  #[must_use]
  pub const fn age_in_range(&self, age: u8) -> bool {
    age >= self.age_range.0 && age <= self.age_range.1
  }

  /// Get the midpoint age
  #[must_use]
  #[allow(clippy::cast_possible_truncation)]
  pub fn midpoint_age(&self) -> u8 {
    let midpoint = u16::midpoint(u16::from(self.age_range.0), u16::from(self.age_range.1));
    midpoint as u8
  }
}

// ============================================================================
// MEANS (RESOURCES)
// ============================================================================

/// Resources and constraints available to a persona
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Means {
  /// Hours available per week to use the product
  pub time_available_hours_per_week: u8,
  /// Monthly budget available (in USD)
  pub budget_monthly: u32,
  /// Technical skill level
  pub technical_skill: SkillLevel,
  /// Decision-making authority
  pub authority: AuthorityLevel,
}

impl Means {
  /// Create new means specification
  ///
  /// # Errors
  /// Returns `PersonaError::InvalidTimeBudget` if time exceeds 168 hours (1 week)
  pub fn new(
    time_available_hours_per_week: u8,
    budget_monthly: u32,
    technical_skill: SkillLevel,
    authority: AuthorityLevel,
  ) -> Result<Self, PersonaError> {
    if time_available_hours_per_week > 168 {
      return Err(PersonaError::InvalidTimeBudget {
        hours: time_available_hours_per_week,
        reason: "Cannot exceed 168 hours per week".to_string(),
      });
    }

    Ok(Self {
      time_available_hours_per_week,
      budget_monthly,
      technical_skill,
      authority,
    })
  }

  /// Check if persona has any time available
  #[must_use]
  pub const fn has_time(&self) -> bool {
    self.time_available_hours_per_week > 0
  }

  /// Check if persona has any budget
  #[must_use]
  pub const fn has_budget(&self) -> bool {
    self.budget_monthly > 0
  }

  /// Check if persona has decision-making authority
  #[must_use]
  pub const fn has_authority(&self) -> bool {
    !matches!(self.authority, AuthorityLevel::None)
  }
}

impl Default for Means {
  fn default() -> Self {
    Self {
      time_available_hours_per_week: 10,
      budget_monthly: 0,
      technical_skill: SkillLevel::Novice,
      authority: AuthorityLevel::None,
    }
  }
}

// ============================================================================
// HUMAN LIMITATIONS
// ============================================================================

/// Universal human limitations with intensity levels (0.0 = none, 1.0 = extreme)
///
/// These limitations make personas realistic. A "Straw Man" user is one
/// with all limitations at zero - such users don't exist in reality.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HumanLimitations {
  /// Tendency to avoid effort (0.0 = highly motivated, 1.0 = extremely lazy)
  pub laziness: f32,
  /// Susceptibility to distraction (0.0 = laser focused, 1.0 = easily distracted)
  pub distractibility: f32,
  /// Fear of negative outcomes (0.0 = risk seeker, 1.0 = extremely risk-averse)
  pub risk_aversion: f32,
  /// Intolerance for delays (0.0 = patient, 1.0 = extremely impatient)
  pub impatience: f32,
  /// Tendency to forget things (0.0 = perfect memory, 1.0 = very forgetful)
  pub forgetfulness: f32,
}

impl HumanLimitations {
  /// Minimum limitation value
  pub const MIN: f32 = 0.0;
  /// Maximum limitation value
  pub const MAX: f32 = 1.0;

  /// Create new human limitations with validated values
  ///
  /// # Errors
  /// Returns `PersonaError::InvalidLimitationValue` if any value is outside [0.0, 1.0]
  pub fn new(
    laziness: f32,
    distractibility: f32,
    risk_aversion: f32,
    impatience: f32,
    forgetfulness: f32,
  ) -> Result<Self, PersonaError> {
    let validate = |value: f32, name: &str| -> Result<f32, PersonaError> {
      if !(Self::MIN..=Self::MAX).contains(&value) {
        return Err(PersonaError::InvalidLimitationValue {
          field: name.to_string(),
          value,
        });
      }
      Ok(value)
    };

    Ok(Self {
      laziness: validate(laziness, "laziness")?,
      distractibility: validate(distractibility, "distractibility")?,
      risk_aversion: validate(risk_aversion, "risk_aversion")?,
      impatience: validate(impatience, "impatience")?,
      forgetfulness: validate(forgetfulness, "forgetfulness")?,
    })
  }

  /// Create limitations with typical values (moderate on all dimensions)
  #[must_use]
  pub fn typical() -> Self {
    Self {
      laziness: 0.5,
      distractibility: 0.5,
      risk_aversion: 0.5,
      impatience: 0.5,
      forgetfulness: 0.5,
    }
  }

  /// Check if all limitations are at zero (indicates "Straw Man")
  #[must_use]
  pub fn is_straw_man(&self) -> bool {
    self.laziness == 0.0
      && self.distractibility == 0.0
      && self.risk_aversion == 0.0
      && self.impatience == 0.0
      && self.forgetfulness == 0.0
  }

  /// Check if all limitations are at maximum (unrealistic)
  #[must_use]
  pub fn is_completely_dysfunctional(&self) -> bool {
    self.laziness >= Self::MAX
      && self.distractibility >= Self::MAX
      && self.risk_aversion >= Self::MAX
      && self.impatience >= Self::MAX
      && self.forgetfulness >= Self::MAX
  }

  /// Calculate average limitation level
  #[must_use]
  pub fn average_limitation(&self) -> f32 {
    (self.laziness
      + self.distractibility
      + self.risk_aversion
      + self.impatience
      + self.forgetfulness)
      / 5.0
  }
}

impl Default for HumanLimitations {
  fn default() -> Self {
    Self::typical()
  }
}

// ============================================================================
// PERSONA
// ============================================================================

/// A realistic user persona with demographics, resources, and limitations
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Persona {
  /// Unique identifier
  pub id: Uuid,
  /// Persona name
  pub name: String,
  /// Demographic information
  pub demographics: Demographics,
  /// Available resources
  pub means: Means,
  /// Human limitations (makes the persona realistic)
  pub limitations: HumanLimitations,
  /// Goals the persona wants to achieve
  pub goals: Vec<String>,
  /// Pain points and frustrations
  pub pain_points: Vec<String>,
  /// Creation timestamp
  pub created_at: DateTime<Utc>,
  /// Last update timestamp
  pub updated_at: DateTime<Utc>,
}

impl Persona {
  /// Create a new persona
  ///
  /// # Errors
  /// Returns `PersonaError::EmptyField` if name is empty
  /// Returns `PersonaError::StrawManDetected` if the persona is unrealistic
  #[allow(clippy::too_many_arguments)]
  pub fn new(
    name: String,
    demographics: Demographics,
    means: Means,
    limitations: HumanLimitations,
    goals: Vec<String>,
    pain_points: Vec<String>,
  ) -> Result<Self, PersonaError> {
    if name.trim().is_empty() {
      return Err(PersonaError::EmptyField {
        field: "name".to_string(),
      });
    }

    let now = Utc::now();
    let persona = Self {
      id: Uuid::new_v4(),
      name,
      demographics,
      means,
      limitations,
      goals,
      pain_points,
      created_at: now,
      updated_at: now,
    };

    persona.validate_rationality()?;

    Ok(persona)
  }

  /// Validate that the persona is not an "irrational actor"
  fn validate_rationality(&self) -> Result<(), PersonaError> {
    if self.limitations.is_straw_man() {
      return Err(PersonaError::StrawManDetected {
        reason: "All human limitations are zero - such users don't exist".to_string(),
      });
    }

    if self.limitations.is_completely_dysfunctional() {
      return Err(PersonaError::IrrationalActor {
        reason: "All limitations at maximum - completely dysfunctional user".to_string(),
      });
    }

    if !self.means.has_time() && !self.goals.is_empty() {
      return Err(PersonaError::IrrationalActor {
        reason: "No time available but has goals".to_string(),
      });
    }

    Ok(())
  }

  /// Check if the persona is realistic
  #[must_use]
  pub fn is_realistic(&self) -> bool {
    self.validate_rationality().is_ok()
  }

  /// Add a goal
  #[must_use]
  pub fn with_goal(mut self, goal: String) -> Self {
    if !goal.trim().is_empty() && !self.goals.contains(&goal) {
      self.goals.push(goal);
      self.updated_at = Utc::now();
    }
    self
  }

  /// Add a pain point
  #[must_use]
  pub fn with_pain_point(mut self, pain_point: String) -> Self {
    if !pain_point.trim().is_empty() && !self.pain_points.contains(&pain_point) {
      self.pain_points.push(pain_point);
      self.updated_at = Utc::now();
    }
    self
  }

  /// Update demographics
  #[must_use]
  pub fn with_demographics(mut self, demographics: Demographics) -> Self {
    self.demographics = demographics;
    self.updated_at = Utc::now();
    self
  }

  /// Update means
  #[must_use]
  pub fn with_means(mut self, means: Means) -> Self {
    self.means = means;
    self.updated_at = Utc::now();
    self
  }

  /// Update limitations
  #[must_use]
  pub fn with_limitations(mut self, limitations: HumanLimitations) -> Self {
    self.limitations = limitations;
    self.updated_at = Utc::now();
    self
  }
}

// ============================================================================
// ERRORS
// ============================================================================

/// Errors for the persona forge module
#[derive(Debug, Error, PartialEq)]
pub enum PersonaError {
  /// A required field was empty
  #[error("required field is empty: {field}")]
  EmptyField { field: String },

  /// Invalid age range
  #[error("invalid age range: min={min}, max={max}")]
  InvalidAgeRange { min: u8, max: u8 },

  /// Invalid income range
  #[error("invalid income range: min={min}, max={max}")]
  InvalidIncomeRange { min: u32, max: u32 },

  /// Invalid time budget
  #[error("invalid time budget: {hours} hours - {reason}")]
  InvalidTimeBudget { hours: u8, reason: String },

  /// Invalid limitation value
  #[error("invalid limitation value for {field}: {value} (must be 0.0-1.0)")]
  InvalidLimitationValue { field: String, value: f32 },

  /// Straw man user detected (unrealistic persona)
  #[error("straw man detected: {reason}")]
  StrawManDetected { reason: String },

  /// Irrational actor detected
  #[error("irrational actor: {reason}")]
  IrrationalActor { reason: String },
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
  #![allow(clippy::expect_used)]
  #![allow(clippy::manual_string_new)]

  use super::*;

  fn valid_demographics() -> Demographics {
    Demographics::new(
      25,
      35,
      "Software Engineer".to_string(),
      80_000,
      120_000,
      EducationLevel::Bachelors,
      "United States".to_string(),
    )
    .expect("valid demographics")
  }

  fn valid_means() -> Means {
    Means::new(20, 100, SkillLevel::Intermediate, AuthorityLevel::Some).expect("valid means")
  }

  fn valid_limitations() -> HumanLimitations {
    HumanLimitations::new(0.3, 0.4, 0.5, 0.6, 0.4).expect("valid limitations")
  }

  // ==========================================================================
  // DEMOGRAPHICS TESTS
  // ==========================================================================

  #[test]
  fn demographics_valid_creation() {
    let demo = Demographics::new(
      25,
      35,
      "Software Engineer".to_string(),
      80_000,
      120_000,
      EducationLevel::Bachelors,
      "United States".to_string(),
    );

    assert!(demo.is_ok());
    let d = demo.expect("valid");
    assert_eq!(d.age_range, (25, 35));
    assert_eq!(d.occupation, "Software Engineer");
    assert_eq!(d.income_range, (80_000, 120_000));
  }

  #[test]
  fn demographics_rejects_invalid_age_range() {
    let result = Demographics::new(
      35,
      25,
      "Test".to_string(),
      0,
      100,
      EducationLevel::None,
      "USA".to_string(),
    );
    assert!(matches!(result, Err(PersonaError::InvalidAgeRange { .. })));
  }

  #[test]
  fn demographics_rejects_zero_age() {
    let result = Demographics::new(
      0,
      25,
      "Test".to_string(),
      0,
      100,
      EducationLevel::None,
      "USA".to_string(),
    );
    assert!(matches!(result, Err(PersonaError::InvalidAgeRange { .. })));
  }

  #[test]
  fn demographics_rejects_empty_occupation() {
    let result = Demographics::new(
      25,
      35,
      "".to_string(),
      0,
      100,
      EducationLevel::None,
      "USA".to_string(),
    );
    let is_correct_error = match result {
      Err(PersonaError::EmptyField { field }) => field == "occupation",
      _ => false,
    };
    assert!(is_correct_error);
  }

  #[test]
  fn demographics_rejects_empty_location() {
    let result = Demographics::new(
      25,
      35,
      "Test".to_string(),
      0,
      100,
      EducationLevel::None,
      "".to_string(),
    );
    let is_correct_error = match result {
      Err(PersonaError::EmptyField { field }) => field == "location",
      _ => false,
    };
    assert!(is_correct_error);
  }

  #[test]
  fn demographics_rejects_invalid_income_range() {
    let result = Demographics::new(
      25,
      35,
      "Test".to_string(),
      100_000,
      50_000,
      EducationLevel::None,
      "USA".to_string(),
    );
    assert!(matches!(
      result,
      Err(PersonaError::InvalidIncomeRange { .. })
    ));
  }

  #[test]
  fn demographics_age_in_range() {
    let demo = valid_demographics();
    assert!(demo.age_in_range(30));
    assert!(demo.age_in_range(25));
    assert!(demo.age_in_range(35));
    assert!(!demo.age_in_range(20));
    assert!(!demo.age_in_range(40));
  }

  // ==========================================================================
  // MEANS TESTS
  // ==========================================================================

  #[test]
  fn means_valid_creation() {
    let means = Means::new(20, 100, SkillLevel::Intermediate, AuthorityLevel::Some);
    assert!(means.is_ok());
    let m = means.expect("valid");
    assert_eq!(m.time_available_hours_per_week, 20);
    assert_eq!(m.budget_monthly, 100);
    assert!(m.has_time());
    assert!(m.has_budget());
    assert!(m.has_authority());
  }

  #[test]
  fn means_rejects_excessive_time() {
    let result = Means::new(200, 0, SkillLevel::Novice, AuthorityLevel::None);
    assert!(matches!(
      result,
      Err(PersonaError::InvalidTimeBudget { .. })
    ));
  }

  #[test]
  fn means_zero_time_budget_allowed() {
    let means = Means::new(0, 0, SkillLevel::Novice, AuthorityLevel::None);
    assert!(means.is_ok());
    let m = means.expect("valid");
    assert!(!m.has_time());
    assert!(!m.has_budget());
    assert!(!m.has_authority());
  }

  // ==========================================================================
  // HUMAN LIMITATIONS TESTS
  // ==========================================================================

  #[test]
  fn limitations_valid_creation() {
    let limits = HumanLimitations::new(0.3, 0.4, 0.5, 0.6, 0.7);
    assert!(limits.is_ok());
  }

  #[test]
  fn limitations_rejects_negative_value() {
    let result = HumanLimitations::new(-0.1, 0.5, 0.5, 0.5, 0.5);
    assert!(matches!(
      result,
      Err(PersonaError::InvalidLimitationValue { .. })
    ));
  }

  #[test]
  fn limitations_rejects_value_above_one() {
    let result = HumanLimitations::new(1.1, 0.5, 0.5, 0.5, 0.5);
    assert!(matches!(
      result,
      Err(PersonaError::InvalidLimitationValue { .. })
    ));
  }

  #[test]
  fn limitations_accepts_boundary_values() {
    let limits = HumanLimitations::new(0.0, 1.0, 0.0, 1.0, 0.5);
    assert!(limits.is_ok());
  }

  #[test]
  fn limitations_detects_straw_man() {
    let straw_man = HumanLimitations::new(0.0, 0.0, 0.0, 0.0, 0.0).expect("valid values");
    assert!(straw_man.is_straw_man());
  }

  #[test]
  fn limitations_detects_completely_dysfunctional() {
    let dysfunctional = HumanLimitations::new(1.0, 1.0, 1.0, 1.0, 1.0).expect("valid values");
    assert!(dysfunctional.is_completely_dysfunctional());
  }

  #[test]
  fn limitations_typical_has_moderate_values() {
    let typical = HumanLimitations::typical();
    assert!(!typical.is_straw_man());
    assert!(!typical.is_completely_dysfunctional());
    assert!((typical.average_limitation() - 0.5).abs() < 0.01);
  }

  // ==========================================================================
  // PERSONA TESTS
  // ==========================================================================

  #[test]
  fn persona_valid_creation() {
    let persona = Persona::new(
      "Alice".to_string(),
      valid_demographics(),
      valid_means(),
      valid_limitations(),
      vec!["Complete tasks efficiently".to_string()],
      vec!["Too many clicks".to_string()],
    );

    assert!(persona.is_ok());
    let p = persona.expect("valid");
    assert_eq!(p.name, "Alice");
    assert!(p.is_realistic());
  }

  #[test]
  fn persona_rejects_empty_name() {
    let result = Persona::new(
      "".to_string(),
      valid_demographics(),
      valid_means(),
      valid_limitations(),
      vec![],
      vec![],
    );
    let is_correct_error = match result {
      Err(PersonaError::EmptyField { field }) => field == "name",
      _ => false,
    };
    assert!(is_correct_error);
  }

  #[test]
  fn persona_rejects_straw_man() {
    let straw_man_limits = HumanLimitations::new(0.0, 0.0, 0.0, 0.0, 0.0).expect("valid values");
    let result = Persona::new(
      "Perfect User".to_string(),
      valid_demographics(),
      valid_means(),
      straw_man_limits,
      vec![],
      vec![],
    );
    assert!(matches!(result, Err(PersonaError::StrawManDetected { .. })));
  }

  #[test]
  fn persona_rejects_completely_dysfunctional() {
    let dysfunctional = HumanLimitations::new(1.0, 1.0, 1.0, 1.0, 1.0).expect("valid values");
    let result = Persona::new(
      "Dysfunctional User".to_string(),
      valid_demographics(),
      valid_means(),
      dysfunctional,
      vec![],
      vec![],
    );
    assert!(matches!(result, Err(PersonaError::IrrationalActor { .. })));
  }

  #[test]
  fn persona_rejects_no_time_but_has_goals() {
    let no_time_means = Means::new(0, 0, SkillLevel::Novice, AuthorityLevel::None).expect("valid");
    let result = Persona::new(
      "Busy User".to_string(),
      valid_demographics(),
      no_time_means,
      valid_limitations(),
      vec!["Learn new skills".to_string()],
      vec![],
    );
    assert!(matches!(result, Err(PersonaError::IrrationalActor { .. })));
  }

  #[test]
  fn persona_allows_no_time_no_goals() {
    let no_time_means = Means::new(0, 0, SkillLevel::Novice, AuthorityLevel::None).expect("valid");
    let result = Persona::new(
      "Passive User".to_string(),
      valid_demographics(),
      no_time_means,
      valid_limitations(),
      vec![],
      vec!["Bored".to_string()],
    );
    assert!(result.is_ok());
  }

  #[test]
  fn persona_with_goal_adds_goal() {
    let persona = Persona::new(
      "Bob".to_string(),
      valid_demographics(),
      valid_means(),
      valid_limitations(),
      vec![],
      vec![],
    )
    .expect("valid");

    let updated = persona.with_goal("Save time".to_string());
    assert_eq!(updated.goals.len(), 1);
    assert!(updated.goals.contains(&"Save time".to_string()));
  }

  #[test]
  fn persona_with_pain_point_adds_pain_point() {
    let persona = Persona::new(
      "Carol".to_string(),
      valid_demographics(),
      valid_means(),
      valid_limitations(),
      vec![],
      vec![],
    )
    .expect("valid");

    let updated = persona.with_pain_point("Confusing UI".to_string());
    assert_eq!(updated.pain_points.len(), 1);
    assert!(updated.pain_points.contains(&"Confusing UI".to_string()));
  }

  #[test]
  fn persona_prevents_duplicate_goals() {
    let persona = Persona::new(
      "Dave".to_string(),
      valid_demographics(),
      valid_means(),
      valid_limitations(),
      vec!["Goal A".to_string()],
      vec![],
    )
    .expect("valid");

    let updated = persona.with_goal("Goal A".to_string());
    assert_eq!(updated.goals.len(), 1);
  }

  // ==========================================================================
  // DISPLAY TESTS
  // ==========================================================================

  #[test]
  fn education_level_display() {
    assert_eq!(EducationLevel::None.to_string(), "No formal education");
    assert_eq!(EducationLevel::HighSchool.to_string(), "High School");
    assert_eq!(EducationLevel::Bachelors.to_string(), "Bachelor's Degree");
    assert_eq!(EducationLevel::Masters.to_string(), "Master's Degree");
    assert_eq!(EducationLevel::Doctorate.to_string(), "Doctorate");
  }

  #[test]
  fn skill_level_display() {
    assert_eq!(SkillLevel::Novice.to_string(), "Novice");
    assert_eq!(SkillLevel::Intermediate.to_string(), "Intermediate");
    assert_eq!(SkillLevel::Advanced.to_string(), "Advanced");
    assert_eq!(SkillLevel::Expert.to_string(), "Expert");
  }

  #[test]
  fn authority_level_display() {
    assert_eq!(AuthorityLevel::None.to_string(), "No authority");
    assert_eq!(AuthorityLevel::Some.to_string(), "Some authority");
    assert_eq!(AuthorityLevel::Full.to_string(), "Full authority");
  }
}
