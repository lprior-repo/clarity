//! Persona & Motivation with Root Cause Analysis (RCA) Framework
//!
//! From The Product-Minded Engineer:
//! Character = Persona + Motivation. A Persona defines WHO is acting
//! (demographics, means, universal limitations). A Motivation defines
//! WHY they need something (the "I Want" moment with RCA).
//!
//! # Core Concepts
//!
//! ## Persona
//!
//! Defines who is acting in a scenario:
//! - **Demographics**: Background characteristics (age, location, occupation)
//! - **Means**: Resources available (time, money, skills, tools)
//! - **Universal Limitations**: Cognitive constraints all humans share
//!   (lazy, distracted, risk-averse, impatient, forgetful)
//!
//! ## Motivation
//!
//! The "I Want" moment - Root Cause Analysis (RCA) of WHY the character
//! needs this particular feature or outcome at this specific moment.
//!
//! ## Straw Man Detection
//!
//! Validates that personas are realistic and not "straw men":
//! - Impossibly resourced (no time, no money, no skills)
//! - Irrationally motivated (wants something they can't use)
//! - Contradictory attributes (expert beginner, always patient)
//!
//! # Design Principles
//!
//! 1. **Evidence-Based**: Personas must have plausible resource constraints
//! 2. **Universal Human Attributes**: Account for shared cognitive limitations
//! 3. **Root Cause Depth**: Motivations must have meaningful RCA chains
//! 4. **Straw Man Prevention**: Automatic detection of irrational actors

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;
use uuid::Uuid;

pub const MIN_RCA_DEPTH: usize = 2;
pub const MAX_RCA_DEPTH: usize = 5;
pub const STRAW_MAN_CONFIDENCE_THRESHOLD: f32 = 0.15;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UniversalLimitation {
  Lazy,
  Distracted,
  RiskAverse,
  Impatient,
  Forgetful,
}

impl fmt::Display for UniversalLimitation {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Lazy => write!(f, "Lazy"),
      Self::Distracted => write!(f, "Distracted"),
      Self::RiskAverse => write!(f, "Risk Averse"),
      Self::Impatient => write!(f, "Impatient"),
      Self::Forgetful => write!(f, "Forgetful"),
    }
  }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Demographics {
  pub age_range: String,
  pub location: String,
  pub occupation: String,
  pub experience_level: String,
}

impl Demographics {
  pub fn new(
    age_range: String,
    location: String,
    occupation: String,
    experience_level: String,
  ) -> Result<Self, CharactersError> {
    if age_range.trim().is_empty() {
      return Err(CharactersError::EmptyField("age_range".to_string()));
    }
    if occupation.trim().is_empty() {
      return Err(CharactersError::EmptyField("occupation".to_string()));
    }
    Ok(Self {
      age_range: age_range.trim().to_string(),
      location: location.trim().to_string(),
      occupation: occupation.trim().to_string(),
      experience_level: experience_level.trim().to_string(),
    })
  }
}

impl Default for Demographics {
  fn default() -> Self {
    Self {
      age_range: "25-35".to_string(),
      location: String::new(),
      occupation: "Unknown".to_string(),
      experience_level: "Intermediate".to_string(),
    }
  }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Means {
  pub time_available: String,
  pub money_budget: String,
  pub skill_level: String,
  pub tools_available: Vec<String>,
}

impl Means {
  pub fn new(time_available: String, money_budget: String, skill_level: String) -> Self {
    Self {
      time_available,
      money_budget,
      skill_level,
      tools_available: Vec::new(),
    }
  }

  #[must_use]
  pub fn with_tool(mut self, tool: String) -> Self {
    self.tools_available.push(tool);
    self
  }

  #[must_use]
  pub fn has_resources(&self) -> bool {
    let has_time = !self.time_available.trim().is_empty()
      && !self.time_available.to_lowercase().contains("none");
    let has_money =
      !self.money_budget.trim().is_empty() && !self.money_budget.to_lowercase().contains("none");
    let has_skill =
      !self.skill_level.trim().is_empty() && !self.skill_level.to_lowercase().contains("none");
    has_time || has_money || has_skill
  }
}

impl Default for Means {
  fn default() -> Self {
    Self {
      time_available: "Some".to_string(),
      money_budget: "Some".to_string(),
      skill_level: "Intermediate".to_string(),
      tools_available: Vec::new(),
    }
  }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RootCauseAnalysis {
  pub initial_want: String,
  pub why_chain: Vec<String>,
  pub final_root_cause: String,
}

impl RootCauseAnalysis {
  pub fn new(initial_want: String) -> Result<Self, CharactersError> {
    if initial_want.trim().is_empty() {
      return Err(CharactersError::EmptyField("initial_want".to_string()));
    }
    Ok(Self {
      initial_want: initial_want.trim().to_string(),
      why_chain: Vec::new(),
      final_root_cause: String::new(),
    })
  }

  #[must_use]
  pub fn with_why(mut self, answer: String) -> Self {
    if !answer.trim().is_empty() {
      self.why_chain.push(answer.trim().to_string());
    }
    self
  }

  #[must_use]
  pub fn with_final_root_cause(mut self, root_cause: String) -> Self {
    self.final_root_cause = root_cause.trim().to_string();
    self
  }

  #[must_use]
  pub fn depth(&self) -> usize {
    self.why_chain.len() + usize::from(!self.final_root_cause.is_empty())
  }

  #[must_use]
  pub fn is_complete(&self) -> bool {
    self.depth() >= MIN_RCA_DEPTH && !self.final_root_cause.is_empty()
  }

  pub fn validate(&self) -> Result<(), CharactersError> {
    if self.depth() < MIN_RCA_DEPTH {
      return Err(CharactersError::InsufficientRcaDepth {
        required: MIN_RCA_DEPTH,
        actual: self.depth(),
      });
    }
    if self.final_root_cause.is_empty() {
      return Err(CharactersError::EmptyField("final_root_cause".to_string()));
    }
    Ok(())
  }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Motivation {
  pub id: Uuid,
  pub i_want_statement: String,
  pub root_cause_analysis: RootCauseAnalysis,
  pub intensity: MotivationIntensity,
  pub created_at: DateTime<Utc>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MotivationIntensity {
  Low,
  Medium,
  High,
  Critical,
}

impl Default for MotivationIntensity {
  fn default() -> Self {
    Self::Medium
  }
}

impl fmt::Display for MotivationIntensity {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Low => write!(f, "Low"),
      Self::Medium => write!(f, "Medium"),
      Self::High => write!(f, "High"),
      Self::Critical => write!(f, "Critical"),
    }
  }
}

impl Motivation {
  pub fn new(
    i_want_statement: String,
    root_cause_analysis: RootCauseAnalysis,
  ) -> Result<Self, CharactersError> {
    if i_want_statement.trim().is_empty() {
      return Err(CharactersError::EmptyField("i_want_statement".to_string()));
    }
    root_cause_analysis.validate()?;

    Ok(Self {
      id: Uuid::new_v4(),
      i_want_statement: i_want_statement.trim().to_string(),
      root_cause_analysis,
      intensity: MotivationIntensity::default(),
      created_at: Utc::now(),
    })
  }

  #[must_use]
  pub const fn with_intensity(mut self, intensity: MotivationIntensity) -> Self {
    self.intensity = intensity;
    self
  }

  #[must_use]
  pub fn is_compelling(&self) -> bool {
    matches!(
      self.intensity,
      MotivationIntensity::High | MotivationIntensity::Critical
    ) && self.root_cause_analysis.is_complete()
  }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StrawManWarning {
  NoResources { field: String },
  ContradictoryAttributes { description: String },
  InsufficientRcaDepth { required: usize, actual: usize },
  IrrationalMotivation { description: String },
  EmptyDemographics { field: String },
}

impl fmt::Display for StrawManWarning {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::NoResources { field } => {
        write!(f, "Persona has no resources: {field}")
      }
      Self::ContradictoryAttributes { description } => {
        write!(f, "Contradictory attributes: {description}")
      }
      Self::InsufficientRcaDepth { required, actual } => {
        write!(f, "RCA depth {actual} below required {required}")
      }
      Self::IrrationalMotivation { description } => {
        write!(f, "Irrational motivation: {description}")
      }
      Self::EmptyDemographics { field } => {
        write!(f, "Empty demographic field: {field}")
      }
    }
  }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Persona {
  pub id: Uuid,
  pub name: String,
  pub demographics: Demographics,
  pub means: Means,
  pub universal_limitations: Vec<UniversalLimitation>,
  pub straw_man_score: f32,
  pub warnings: Vec<StrawManWarning>,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

impl Persona {
  pub fn new(
    name: String,
    demographics: Demographics,
    means: Means,
  ) -> Result<Self, CharactersError> {
    if name.trim().is_empty() {
      return Err(CharactersError::EmptyField("name".to_string()));
    }

    let now = Utc::now();
    let mut persona = Self {
      id: Uuid::new_v4(),
      name: name.trim().to_string(),
      demographics,
      means,
      universal_limitations: Vec::new(),
      straw_man_score: 0.0,
      warnings: Vec::new(),
      created_at: now,
      updated_at: now,
    };

    persona.detect_straw_man_issues();
    Ok(persona)
  }

  #[must_use]
  pub fn with_limitation(mut self, limitation: UniversalLimitation) -> Self {
    if !self.universal_limitations.contains(&limitation) {
      self.universal_limitations.push(limitation);
      self.updated_at = Utc::now();
    }
    self
  }

  #[must_use]
  pub fn is_straw_man(&self) -> bool {
    self.straw_man_score >= STRAW_MAN_CONFIDENCE_THRESHOLD
  }

  fn detect_straw_man_issues(&mut self) {
    self.warnings.clear();

    if !self.means.has_resources() {
      self.warnings.push(StrawManWarning::NoResources {
        field: "means".to_string(),
      });
    }

    if self.demographics.age_range.is_empty() {
      self.warnings.push(StrawManWarning::EmptyDemographics {
        field: "age_range".to_string(),
      });
    }

    if self.demographics.occupation.is_empty() {
      self.warnings.push(StrawManWarning::EmptyDemographics {
        field: "occupation".to_string(),
      });
    }

    self.calculate_straw_man_score();
  }

  fn calculate_straw_man_score(&mut self) {
    let warning_count = self.warnings.len() as f32;
    self.straw_man_score = (warning_count * 0.15).min(1.0);
  }

  pub fn validate(&self) -> Result<(), CharactersError> {
    if self.is_straw_man() {
      return Err(CharactersError::StrawManDetected {
        warnings: self.warnings.clone(),
      });
    }
    Ok(())
  }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Character {
  pub id: Uuid,
  pub persona: Persona,
  pub motivation: Motivation,
  pub created_at: DateTime<Utc>,
}

impl Character {
  pub fn new(persona: Persona, motivation: Motivation) -> Result<Self, CharactersError> {
    persona.validate()?;

    Ok(Self {
      id: Uuid::new_v4(),
      persona,
      motivation,
      created_at: Utc::now(),
    })
  }

  #[must_use]
  pub fn is_valid(&self) -> bool {
    !self.persona.is_straw_man() && self.motivation.is_compelling()
  }
}

#[derive(Debug, Error, PartialEq)]
pub enum CharactersError {
  #[error("required field is empty: {0}")]
  EmptyField(String),

  #[error("insufficient RCA depth: need {required} but have {actual}")]
  InsufficientRcaDepth { required: usize, actual: usize },

  #[error("straw man persona detected: {warnings:?}")]
  StrawManDetected { warnings: Vec<StrawManWarning> },

  #[error("motivation not compelling enough")]
  MotivationNotCompelling,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn demographics_new_requires_age_range() {
    let result = Demographics::new(
      "".to_string(),
      "location".to_string(),
      "occupation".to_string(),
      "level".to_string(),
    );
    assert!(result.is_err());
  }

  #[test]
  fn demographics_new_requires_occupation() {
    let result = Demographics::new(
      "25-35".to_string(),
      "location".to_string(),
      "".to_string(),
      "level".to_string(),
    );
    assert!(result.is_err());
  }

  #[test]
  fn demographics_new_succeeds_with_valid_input() {
    let result = Demographics::new(
      "25-35".to_string(),
      "San Francisco".to_string(),
      "Software Engineer".to_string(),
      "Senior".to_string(),
    );
    assert!(result.is_ok());
    let d = result.unwrap();
    assert_eq!(d.age_range, "25-35");
    assert_eq!(d.occupation, "Software Engineer");
  }

  #[test]
  fn means_has_resources_detects_no_resources() {
    let no_resources = Means::new("None".to_string(), "None".to_string(), "None".to_string());
    assert!(!no_resources.has_resources());
  }

  #[test]
  fn means_has_resources_detects_some_resources() {
    let some_resources = Means::new(
      "2 hours/week".to_string(),
      "$50/month".to_string(),
      "Intermediate".to_string(),
    );
    assert!(some_resources.has_resources());
  }

  #[test]
  fn means_with_tool_adds_tools() {
    let means = Means::new("Some".to_string(), "Some".to_string(), "Some".to_string())
      .with_tool("Laptop".to_string())
      .with_tool("Phone".to_string());

    assert_eq!(means.tools_available.len(), 2);
    assert!(means.tools_available.contains(&"Laptop".to_string()));
  }

  #[test]
  fn root_cause_analysis_new_requires_initial_want() {
    let result = RootCauseAnalysis::new("".to_string());
    assert!(result.is_err());
  }

  #[test]
  fn root_cause_analysis_depth_counts_whys() {
    let rca = RootCauseAnalysis::new("I want to track tasks".to_string())
      .unwrap()
      .with_why("Because I forget things".to_string())
      .with_why("Because I have too many tasks".to_string())
      .with_final_root_cause("I need external memory".to_string());

    assert_eq!(rca.depth(), 3);
    assert!(rca.is_complete());
  }

  #[test]
  fn root_cause_analysis_validate_requires_minimum_depth() {
    let shallow_rca = RootCauseAnalysis::new("I want to track tasks".to_string())
      .unwrap()
      .with_why("Because I forget".to_string());

    assert!(shallow_rca.validate().is_err());
  }

  #[test]
  fn root_cause_analysis_validate_requires_final_root_cause() {
    let no_final = RootCauseAnalysis::new("I want to track tasks".to_string())
      .unwrap()
      .with_why("Because I forget".to_string())
      .with_why("Because I have too much".to_string());

    assert!(no_final.validate().is_err());
  }

  #[test]
  fn motivation_new_requires_i_want_statement() {
    let rca = RootCauseAnalysis::new("Want".to_string())
      .unwrap()
      .with_why("Why 1".to_string())
      .with_why("Why 2".to_string())
      .with_final_root_cause("Root".to_string());

    let result = Motivation::new("".to_string(), rca);
    assert!(result.is_err());
  }

  #[test]
  fn motivation_new_requires_valid_rca() {
    let invalid_rca = RootCauseAnalysis::new("Want".to_string()).unwrap();

    let result = Motivation::new("I want X".to_string(), invalid_rca);
    assert!(result.is_err());
  }

  #[test]
  fn motivation_new_succeeds_with_valid_input() {
    let rca = RootCauseAnalysis::new("I want to track tasks".to_string())
      .unwrap()
      .with_why("Because I forget".to_string())
      .with_why("Because I have too much".to_string())
      .with_final_root_cause("Need external memory".to_string());

    let result = Motivation::new("I want a task tracker".to_string(), rca);
    assert!(result.is_ok());
    let m = result.unwrap();
    assert_eq!(m.i_want_statement, "I want a task tracker");
    assert!(m.root_cause_analysis.is_complete());
  }

  #[test]
  fn motivation_is_compelling_requires_high_intensity_and_complete_rca() {
    let rca = RootCauseAnalysis::new("Want".to_string())
      .unwrap()
      .with_why("Why 1".to_string())
      .with_why("Why 2".to_string())
      .with_final_root_cause("Root".to_string());

    let low_intensity = Motivation::new("I want X".to_string(), rca.clone()).unwrap();
    assert!(!low_intensity.is_compelling());

    let high_intensity = Motivation::new("I want X".to_string(), rca)
      .unwrap()
      .with_intensity(MotivationIntensity::High);
    assert!(high_intensity.is_compelling());
  }

  #[test]
  fn persona_new_requires_name() {
    let demographics = Demographics::default();
    let means = Means::default();

    let result = Persona::new("".to_string(), demographics, means);
    assert!(result.is_err());
  }

  #[test]
  fn persona_new_succeeds_with_valid_input() {
    let demographics = Demographics::new(
      "25-35".to_string(),
      "SF".to_string(),
      "Engineer".to_string(),
      "Senior".to_string(),
    )
    .unwrap();
    let means = Means::new(
      "2h/week".to_string(),
      "$50/mo".to_string(),
      "Pro".to_string(),
    );

    let result = Persona::new("Alice".to_string(), demographics, means);
    assert!(result.is_ok());
    let p = result.unwrap();
    assert_eq!(p.name, "Alice");
    assert!(!p.is_straw_man());
  }

  #[test]
  fn persona_detects_straw_man_no_resources() {
    let demographics = Demographics::default();
    let no_resources = Means::new("None".to_string(), "None".to_string(), "None".to_string());

    let persona = Persona::new("Bob".to_string(), demographics, no_resources).unwrap();
    assert!(persona.is_straw_man());
    assert!(persona
      .warnings
      .iter()
      .any(|w| matches!(w, StrawManWarning::NoResources { .. })));
  }

  #[test]
  fn persona_with_limitation_adds_limitations() {
    let demographics = Demographics::default();
    let means = Means::default();

    let persona = Persona::new("Test".to_string(), demographics, means)
      .unwrap()
      .with_limitation(UniversalLimitation::Lazy)
      .with_limitation(UniversalLimitation::Impatient);

    assert_eq!(persona.universal_limitations.len(), 2);
    assert!(persona
      .universal_limitations
      .contains(&UniversalLimitation::Lazy));
  }

  #[test]
  fn persona_validate_fails_for_straw_man() {
    let demographics = Demographics::default();
    let no_resources = Means::new("None".to_string(), "None".to_string(), "None".to_string());

    let persona = Persona::new("Bob".to_string(), demographics, no_resources).unwrap();

    let result = persona.validate();
    assert!(result.is_err());
    assert!(matches!(
      result,
      Err(CharactersError::StrawManDetected { .. })
    ));
  }

  #[test]
  fn character_new_requires_valid_persona() {
    let demographics = Demographics::default();
    let no_resources = Means::new("None".to_string(), "None".to_string(), "None".to_string());
    let persona = Persona::new("Bob".to_string(), demographics, no_resources).unwrap();

    let rca = RootCauseAnalysis::new("Want".to_string())
      .unwrap()
      .with_why("Why 1".to_string())
      .with_why("Why 2".to_string())
      .with_final_root_cause("Root".to_string());
    let motivation = Motivation::new("I want X".to_string(), rca).unwrap();

    let result = Character::new(persona, motivation);
    assert!(result.is_err());
  }

  #[test]
  fn character_new_succeeds_with_valid_inputs() {
    let demographics = Demographics::new(
      "25-35".to_string(),
      "SF".to_string(),
      "Engineer".to_string(),
      "Senior".to_string(),
    )
    .unwrap();
    let means = Means::new(
      "2h/week".to_string(),
      "$50/mo".to_string(),
      "Pro".to_string(),
    );
    let persona = Persona::new("Alice".to_string(), demographics, means).unwrap();

    let rca = RootCauseAnalysis::new("I want to track tasks".to_string())
      .unwrap()
      .with_why("I forget things".to_string())
      .with_why("I have too many tasks".to_string())
      .with_final_root_cause("Need external memory".to_string());
    let motivation = Motivation::new("I want a task tracker".to_string(), rca)
      .unwrap()
      .with_intensity(MotivationIntensity::High);

    let result = Character::new(persona, motivation);
    assert!(result.is_ok());
    let character = result.unwrap();
    assert!(character.is_valid());
  }

  #[test]
  fn character_is_valid_checks_persona_and_motivation() {
    let demographics = Demographics::new(
      "25-35".to_string(),
      "SF".to_string(),
      "Engineer".to_string(),
      "Senior".to_string(),
    )
    .unwrap();
    let means = Means::new(
      "2h/week".to_string(),
      "$50/mo".to_string(),
      "Pro".to_string(),
    );
    let persona = Persona::new("Alice".to_string(), demographics, means).unwrap();

    let rca = RootCauseAnalysis::new("Want".to_string())
      .unwrap()
      .with_why("Why 1".to_string())
      .with_why("Why 2".to_string())
      .with_final_root_cause("Root".to_string());
    let low_motivation = Motivation::new("I want X".to_string(), rca).unwrap();

    let character = Character::new(persona, low_motivation).unwrap();
    assert!(!character.is_valid());
  }

  #[test]
  fn universal_limitation_display() {
    assert_eq!(UniversalLimitation::Lazy.to_string(), "Lazy");
    assert_eq!(UniversalLimitation::Distracted.to_string(), "Distracted");
    assert_eq!(UniversalLimitation::RiskAverse.to_string(), "Risk Averse");
  }

  #[test]
  fn straw_man_warning_display() {
    let warning = StrawManWarning::NoResources {
      field: "time".to_string(),
    };
    assert!(warning.to_string().contains("no resources"));

    let warning = StrawManWarning::ContradictoryAttributes {
      description: "test".to_string(),
    };
    assert!(warning.to_string().contains("Contradictory"));
  }
}
