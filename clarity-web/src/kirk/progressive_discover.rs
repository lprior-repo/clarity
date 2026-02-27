#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![allow(clippy::suspicious_else_formatting)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Progressive Discover validation types for KIRK contracts.
//!
//! This module provides types for the Progressive Discover phase validation
//! and compilation to the 16-section KIRK contract structure.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::components::discover::types::{HolePunchingResults, HoleType};

// ============================================================================
// 16-Section KIRK Contract Types (bd-2uci)
// ============================================================================

/// The 16 standard sections of a KIRK contract.
///
/// These sections represent the complete structure for documenting
/// a design-by-contract specification from the Progressive Discover phase.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct KirkSection {
  /// Section index (0-15)
  pub id: usize,
  /// Human-readable section title
  pub title: String,
  /// Content extracted from the transcript
  pub content: String,
  /// Source field in the transcript this section was derived from
  pub source_field: String,
}

impl KirkSection {
  /// Create a new section.
  #[must_use]
  pub fn new(id: usize, title: String, content: String, source_field: String) -> Self {
    Self {
      id,
      title,
      content,
      source_field,
    }
  }

  /// Create an empty section with just an ID and title.
  #[must_use]
  pub fn empty(id: usize, title: String, source_field: String) -> Self {
    Self {
      id,
      title,
      content: String::new(),
      source_field,
    }
  }

  /// Check if this section has content.
  #[must_use]
  pub fn has_content(&self) -> bool {
    !self.content.trim().is_empty()
  }
}

/// The 16-section KIRK contract structure compiled from a Progressive Discover transcript.
///
/// This represents the complete output of the Progressive Discover phase, containing
/// all extracted and validated information in a structured format.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct KirkContract16 {
  /// The 16 sections of the contract
  pub sections: [KirkSection; 16],
  /// When this contract was compiled
  pub compiled_at: DateTime<Utc>,
  /// Version of the contract schema
  pub schema_version: String,
}

impl KirkContract16 {
  /// Standard section titles in order.
  pub const SECTION_TITLES: [&'static str; 16] = [
    "Original Prompt",
    "Problem Statement",
    "Antithesis Points",
    "Target Persona",
    "Straw Man Validation",
    "Solution Description",
    "VORP Justification",
    "Non-Persona",
    "Scenario Trigger",
    "Scenario Value Moment",
    "Scenario Feeling",
    "Discovery Hole",
    "Edge Case Hole",
    "Motivation Drop-off",
    "EARS Requirements",
    "Compilation Metadata",
  ];

  /// Source fields for each section.
  pub const SOURCE_FIELDS: [&'static str; 16] = [
    "original_prompt",
    "problem",
    "antithesis",
    "persona",
    "straw_man_validation",
    "solution",
    "vorp_justification",
    "nonpersona",
    "scenario.trigger",
    "scenario.value_moment",
    "scenario.feeling",
    "scenario.hole_punching.discovery_hole",
    "scenario.hole_punching.edge_case_hole",
    "scenario.hole_punching.motivation_dropoff",
    "ears_requirements",
    "_metadata",
  ];

  /// Create a new 16-section KIRK contract with empty sections.
  #[must_use]
  pub fn new() -> Self {
    let sections = Self::create_empty_sections();
    Self {
      sections,
      compiled_at: Utc::now(),
      schema_version: "1.0.0".to_string(),
    }
  }

  /// Create empty sections with standard titles.
  fn create_empty_sections() -> [KirkSection; 16] {
    [
      KirkSection::empty(
        0,
        "Original Prompt".to_string(),
        "original_prompt".to_string(),
      ),
      KirkSection::empty(1, "Problem Statement".to_string(), "problem".to_string()),
      KirkSection::empty(2, "Antithesis Points".to_string(), "antithesis".to_string()),
      KirkSection::empty(3, "Target Persona".to_string(), "persona".to_string()),
      KirkSection::empty(
        4,
        "Straw Man Validation".to_string(),
        "straw_man_validation".to_string(),
      ),
      KirkSection::empty(
        5,
        "Solution Description".to_string(),
        "solution".to_string(),
      ),
      KirkSection::empty(
        6,
        "VORP Justification".to_string(),
        "vorp_justification".to_string(),
      ),
      KirkSection::empty(7, "Non-Persona".to_string(), "nonpersona".to_string()),
      KirkSection::empty(
        8,
        "Scenario Trigger".to_string(),
        "scenario.trigger".to_string(),
      ),
      KirkSection::empty(
        9,
        "Scenario Value Moment".to_string(),
        "scenario.value_moment".to_string(),
      ),
      KirkSection::empty(
        10,
        "Scenario Feeling".to_string(),
        "scenario.feeling".to_string(),
      ),
      KirkSection::empty(
        11,
        "Discovery Hole".to_string(),
        "scenario.hole_punching.discovery_hole".to_string(),
      ),
      KirkSection::empty(
        12,
        "Edge Case Hole".to_string(),
        "scenario.hole_punching.edge_case_hole".to_string(),
      ),
      KirkSection::empty(
        13,
        "Motivation Drop-off".to_string(),
        "scenario.hole_punching.motivation_dropoff".to_string(),
      ),
      KirkSection::empty(
        14,
        "EARS Requirements".to_string(),
        "ears_requirements".to_string(),
      ),
      KirkSection::empty(
        15,
        "Compilation Metadata".to_string(),
        "_metadata".to_string(),
      ),
    ]
  }

  /// Get a section by index.
  #[must_use]
  pub fn get_section(&self, index: usize) -> Option<&KirkSection> {
    self.sections.get(index)
  }

  /// Get a mutable reference to a section by index.
  pub fn get_section_mut(&mut self, index: usize) -> Option<&mut KirkSection> {
    self.sections.get_mut(index)
  }

  /// Set the content of a section by index.
  ///
  /// Returns `None` if index is out of bounds.
  #[must_use]
  pub fn with_section_content(mut self, index: usize, content: String) -> Option<Self> {
    if index < self.sections.len() {
      self.sections[index].content = content;
      Some(self)
    } else {
      None
    }
  }

  /// Count sections that have content.
  #[must_use]
  pub fn filled_section_count(&self) -> usize {
    self.sections.iter().filter(|s| s.has_content()).count()
  }

  /// Calculate completion percentage (0-100).
  #[must_use]
  pub fn completion_percentage(&self) -> u8 {
    let filled = self.filled_section_count();
    let total = self.sections.len();
    u8::try_from((filled * 100) / total).unwrap_or(0)
  }

  /// Check if all required sections are filled.
  ///
  /// Required sections are 0-13 (all except EARS and metadata).
  #[must_use]
  pub fn is_complete(&self) -> bool {
    self.sections.iter().take(14).all(KirkSection::has_content)
  }

  /// Get list of sections that are empty.
  #[must_use]
  pub fn empty_sections(&self) -> Vec<&KirkSection> {
    self.sections.iter().filter(|s| !s.has_content()).collect()
  }

  /// Update the compilation timestamp.
  #[must_use]
  pub fn with_timestamp(mut self, timestamp: DateTime<Utc>) -> Self {
    self.compiled_at = timestamp;
    self
  }
}

impl Default for KirkContract16 {
  fn default() -> Self {
    Self::new()
  }
}

// ============================================================================
// VORP Validation Types (bd-2mcc)
// ============================================================================

/// VORP (Value, Obvious, Real, Possible) validation result.
///
/// VORP is a framework for evaluating whether a solution idea is worth pursuing:
/// - **Value**: Does it provide meaningful value to users?
/// - **Obvious**: Is the value immediately apparent to users?
/// - **Real**: Are the users and problem real?
/// - **Possible**: Can we actually build this?
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VorpValidation {
  /// Overall score (0.0-1.0), average of all dimensions
  pub overall_score: f64,
  /// Individual dimension scores
  pub dimensions: Vec<(String, f64)>,
  /// Suggestions for improvement
  pub suggestions: Vec<String>,
}

impl VorpValidation {
  /// Create a new VORP validation result.
  #[must_use]
  pub fn new(value_score: f64, obvious_score: f64, real_score: f64, possible_score: f64) -> Self {
    let dimensions = vec![
      ("Value".to_string(), value_score.clamp(0.0, 1.0)),
      ("Obvious".to_string(), obvious_score.clamp(0.0, 1.0)),
      ("Real".to_string(), real_score.clamp(0.0, 1.0)),
      ("Possible".to_string(), possible_score.clamp(0.0, 1.0)),
    ];

    let overall_score = (value_score + obvious_score + real_score + possible_score) / 4.0;

    let suggestions = Self::generate_suggestions(&dimensions);

    Self {
      overall_score: overall_score.clamp(0.0, 1.0),
      dimensions,
      suggestions,
    }
  }

  /// Generate suggestions based on low-scoring dimensions.
  fn generate_suggestions(dimensions: &[(String, f64)]) -> Vec<String> {
    dimensions
      .iter()
      .filter(|(_, score)| *score < 0.7)
      .map(|(name, score)| {
        if *score < 0.4 {
          format!("{} dimension needs significant improvement", name)
        } else {
          format!("{} dimension could be strengthened", name)
        }
      })
      .collect()
  }

  /// Check if the VORP validation passes (overall >= 0.7).
  #[must_use]
  pub fn passes(&self) -> bool {
    self.overall_score >= 0.7
  }

  /// Get score for a specific dimension.
  #[must_use]
  pub fn get_dimension_score(&self, name: &str) -> Option<f64> {
    self
      .dimensions
      .iter()
      .find(|(n, _)| n == name)
      .map(|(_, score)| *score)
  }

  /// Get the lowest-scoring dimension.
  #[must_use]
  pub fn weakest_dimension(&self) -> Option<&(String, f64)> {
    self
      .dimensions
      .iter()
      .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
  }
}

impl Default for VorpValidation {
  fn default() -> Self {
    Self::new(0.0, 0.0, 0.0, 0.0)
  }
}

// ============================================================================
// Antithesis Validation Types (bd-378l)
// ============================================================================

/// Result of validating antithesis (null hypothesis) points.
///
/// Antithesis points represent realistic reasons why users might
/// reject or ignore a proposed solution.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AntithesisValidation {
  /// Quality score (0.0-1.0)
  pub score: f64,
  /// Suggestions for improving weak points
  pub suggestions: Vec<String>,
  /// Whether all 3 points are valid (non-empty and specific)
  pub is_valid: bool,
}

impl AntithesisValidation {
  /// Create a new antithesis validation result.
  #[must_use]
  pub fn new(score: f64, suggestions: Vec<String>, is_valid: bool) -> Self {
    Self {
      score: score.clamp(0.0, 1.0),
      suggestions,
      is_valid,
    }
  }

  /// Create a passing validation.
  #[must_use]
  pub fn passing(score: f64) -> Self {
    Self::new(score, vec![], true)
  }

  /// Create a failing validation with suggestions.
  #[must_use]
  pub fn failing(suggestions: Vec<String>) -> Self {
    Self::new(0.0, suggestions, false)
  }

  /// Check if quality passes threshold (>= 0.7).
  #[must_use]
  pub fn passes_quality_gate(&self) -> bool {
    self.score >= 0.7
  }
}

impl Default for AntithesisValidation {
  fn default() -> Self {
    Self::new(0.0, vec![], false)
  }
}

// ============================================================================
// EARS Extraction Types (bd-zf68)
// ============================================================================

/// An extracted EARS requirement from the transcript.
///
/// EARS (Easy Approach to Requirements Syntax) provides patterns for
/// writing clear, testable requirements.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtractedEarsRequirement {
  /// Unique identifier for this requirement
  pub id: String,
  /// The requirement text in EARS format
  pub text: String,
  /// Type of EARS pattern used
  pub pattern: EarsPattern,
  /// Source section this was extracted from
  pub source_section: String,
  /// Whether acceptance criteria are present
  pub has_acceptance_criteria: bool,
}

impl ExtractedEarsRequirement {
  /// Create a new extracted EARS requirement.
  #[must_use]
  pub fn new(id: String, text: String, pattern: EarsPattern, source_section: String) -> Self {
    Self {
      id,
      text,
      pattern,
      source_section,
      has_acceptance_criteria: false,
    }
  }

  /// Add acceptance criteria flag.
  #[must_use]
  pub fn with_acceptance_criteria(mut self) -> Self {
    self.has_acceptance_criteria = true;
    self
  }
}

/// EARS requirement patterns.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EarsPattern {
  /// Ubiquitous: "The system shall..."
  Ubiquitous,
  /// Event-driven: "When X, the system shall..."
  EventDriven,
  /// Unwanted behavior: "The system shall not..."
  Unwanted,
  /// State-driven: "While in state X, the system shall..."
  StateDriven,
  /// Optional feature: "Where feature X is implemented, the system shall..."
  OptionalFeature,
}

impl EarsPattern {
  /// Get the pattern name.
  #[must_use]
  pub const fn name(&self) -> &'static str {
    match self {
      Self::Ubiquitous => "Ubiquitous",
      Self::EventDriven => "Event-Driven",
      Self::Unwanted => "Unwanted",
      Self::StateDriven => "State-Driven",
      Self::OptionalFeature => "Optional Feature",
    }
  }
}

/// Result of EARS extraction from a transcript.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EarsExtraction {
  /// Extracted requirements
  pub requirements: Vec<ExtractedEarsRequirement>,
  /// Total count of requirements found
  pub total_count: usize,
  /// Count by pattern type
  pub counts_by_pattern: Vec<(EarsPattern, usize)>,
  /// Sections that were analyzed
  pub analyzed_sections: Vec<String>,
}

impl EarsExtraction {
  /// Create a new EARS extraction result.
  #[must_use]
  pub fn new(requirements: Vec<ExtractedEarsRequirement>) -> Self {
    let total_count = requirements.len();

    let counts_by_pattern = Self::count_by_pattern(&requirements);

    let analyzed_sections = requirements
      .iter()
      .map(|r| r.source_section.clone())
      .collect::<std::collections::HashSet<_>>()
      .into_iter()
      .collect();

    Self {
      requirements,
      total_count,
      counts_by_pattern,
      analyzed_sections,
    }
  }

  /// Count requirements by pattern type.
  fn count_by_pattern(requirements: &[ExtractedEarsRequirement]) -> Vec<(EarsPattern, usize)> {
    use std::collections::HashMap;
    let mut counts = HashMap::new();

    for req in requirements {
      *counts.entry(req.pattern).or_insert(0) += 1;
    }

    vec![
      (
        EarsPattern::Ubiquitous,
        *counts.get(&EarsPattern::Ubiquitous).unwrap_or(&0),
      ),
      (
        EarsPattern::EventDriven,
        *counts.get(&EarsPattern::EventDriven).unwrap_or(&0),
      ),
      (
        EarsPattern::Unwanted,
        *counts.get(&EarsPattern::Unwanted).unwrap_or(&0),
      ),
      (
        EarsPattern::StateDriven,
        *counts.get(&EarsPattern::StateDriven).unwrap_or(&0),
      ),
      (
        EarsPattern::OptionalFeature,
        *counts.get(&EarsPattern::OptionalFeature).unwrap_or(&0),
      ),
    ]
  }

  /// Create an empty extraction result.
  #[must_use]
  pub fn empty() -> Self {
    Self::new(vec![])
  }

  /// Check if any requirements were extracted.
  #[must_use]
  pub fn has_requirements(&self) -> bool {
    !self.requirements.is_empty()
  }

  /// Get requirements by pattern type.
  #[must_use]
  pub fn get_by_pattern(&self, pattern: EarsPattern) -> Vec<&ExtractedEarsRequirement> {
    self
      .requirements
      .iter()
      .filter(|r| r.pattern == pattern)
      .collect()
  }

  /// Get requirements with acceptance criteria.
  #[must_use]
  pub fn with_acceptance_criteria(&self) -> Vec<&ExtractedEarsRequirement> {
    self
      .requirements
      .iter()
      .filter(|r| r.has_acceptance_criteria)
      .collect()
  }
}

impl Default for EarsExtraction {
  fn default() -> Self {
    Self::empty()
  }
}

// ============================================================================
// Hole Punching Validation Types (bd-13yb)
// ============================================================================

/// Result of hole punching validation.
///
/// Hole punching checks that all three types of gaps in a scenario
/// have been addressed.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HolePunchingValidation {
  /// Whether all holes have been addressed
  pub is_complete: bool,
  /// Number of addressed holes (0-3)
  pub addressed_count: u8,
  /// Detailed results for each hole type
  pub results: HolePunchingResults,
}

impl HolePunchingValidation {
  /// Create a new hole punching validation.
  #[must_use]
  pub fn new(results: HolePunchingResults) -> Self {
    let is_complete = results.is_complete();
    let addressed_count = results.addressed_count() as u8;

    Self {
      is_complete,
      addressed_count,
      results,
    }
  }

  /// Create a complete validation.
  #[must_use]
  pub fn complete() -> Self {
    let results = HolePunchingResults::new()
      .address(HoleType::DiscoveryHole, "Addressed".to_string())
      .address(HoleType::EdgeCaseHole, "Addressed".to_string())
      .address(HoleType::MotivationDropOff, "Addressed".to_string());
    Self::new(results)
  }
}

impl Default for HolePunchingValidation {
  fn default() -> Self {
    Self::new(HolePunchingResults::default())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_kirk_section_new() {
    let section = KirkSection::new(
      0,
      "Test Title".to_string(),
      "Test content".to_string(),
      "test_field".to_string(),
    );

    assert_eq!(section.id, 0);
    assert_eq!(section.title, "Test Title");
    assert_eq!(section.content, "Test content");
    assert_eq!(section.source_field, "test_field");
    assert!(section.has_content());
  }

  #[test]
  fn test_kirk_section_empty() {
    let section = KirkSection::empty(1, "Empty".to_string(), "field".to_string());

    assert_eq!(section.id, 1);
    assert!(section.content.is_empty());
    assert!(!section.has_content());
  }

  #[test]
  fn test_kirk_contract_16_new() {
    let contract = KirkContract16::new();

    assert_eq!(contract.sections.len(), 16);
    assert_eq!(contract.schema_version, "1.0.0");
  }

  #[test]
  fn test_kirk_contract_16_section_titles() {
    assert_eq!(KirkContract16::SECTION_TITLES.len(), 16);
    assert_eq!(KirkContract16::SECTION_TITLES[0], "Original Prompt");
    assert_eq!(KirkContract16::SECTION_TITLES[15], "Compilation Metadata");
  }

  #[test]
  fn test_kirk_contract_16_get_section() {
    let contract = KirkContract16::new();

    let section = contract.get_section(0);
    assert!(section.is_some());
    assert_eq!(section.map(|s| s.title.as_str()), Some("Original Prompt"));

    let out_of_bounds = contract.get_section(20);
    assert!(out_of_bounds.is_none());
  }

  #[test]
  fn test_kirk_contract_16_with_section_content() -> Result<(), &'static str> {
    let contract = KirkContract16::new();

    let updated = contract.with_section_content(0, "New content".to_string());
    assert!(updated.is_some());
    assert_eq!(
      updated
        .as_ref()
        .and_then(|c| c.get_section(0))
        .map(|s| s.content.as_str()),
      Some("New content")
    );

    let updated_contract = updated.ok_or("Updated contract should exist")?;
    let out_of_bounds = updated_contract.with_section_content(20, "Invalid".to_string());
    assert!(out_of_bounds.is_none());
    Ok(())
  }

  #[test]
  fn test_kirk_contract_16_filled_count() -> Result<(), &'static str> {
    let contract = KirkContract16::new();
    assert_eq!(contract.filled_section_count(), 0);

    let filled = contract
      .with_section_content(0, "Content 1".to_string())
      .ok_or("Should set section 0")?
      .with_section_content(5, "Content 2".to_string())
      .ok_or("Should set section 5")?;

    assert_eq!(filled.filled_section_count(), 2);
    Ok(())
  }

  #[test]
  fn test_kirk_contract_16_completion_percentage() -> Result<(), &'static str> {
    let contract = KirkContract16::new();
    assert_eq!(contract.completion_percentage(), 0);

    let mut contract = contract;
    for i in 0..8 {
      contract = contract
        .with_section_content(i, format!("Content {}", i))
        .ok_or("Should set section content")?;
    }
    assert_eq!(contract.completion_percentage(), 50);
    Ok(())
  }

  #[test]
  fn test_kirk_contract_16_is_complete() -> Result<(), &'static str> {
    let contract = KirkContract16::new();
    assert!(!contract.is_complete());

    let mut contract = contract;
    for i in 0..14 {
      contract = contract
        .with_section_content(i, format!("Content {}", i))
        .ok_or("Should set section content")?;
    }
    assert!(contract.is_complete());
    Ok(())
  }

  #[test]
  fn test_vorp_validation_new() {
    let vorp = VorpValidation::new(0.8, 0.7, 0.9, 0.6);

    assert!((vorp.overall_score - 0.75).abs() < f64::EPSILON);
    assert_eq!(vorp.dimensions.len(), 4);
  }

  #[test]
  fn test_vorp_validation_passes() {
    let passing = VorpValidation::new(0.8, 0.8, 0.8, 0.8);
    assert!(passing.passes());

    let failing = VorpValidation::new(0.5, 0.5, 0.5, 0.5);
    assert!(!failing.passes());
  }

  #[test]
  fn test_vorp_validation_get_dimension() {
    let vorp = VorpValidation::new(0.8, 0.7, 0.9, 0.6);

    assert_eq!(vorp.get_dimension_score("Value"), Some(0.8));
    assert_eq!(vorp.get_dimension_score("Obvious"), Some(0.7));
    assert_eq!(vorp.get_dimension_score("Real"), Some(0.9));
    assert_eq!(vorp.get_dimension_score("Possible"), Some(0.6));
    assert_eq!(vorp.get_dimension_score("Invalid"), None);
  }

  #[test]
  fn test_vorp_validation_weakest() {
    let vorp = VorpValidation::new(0.8, 0.7, 0.9, 0.3);
    let weakest = vorp.weakest_dimension();

    assert!(weakest.is_some());
    assert_eq!(weakest.map(|(n, _)| n.as_str()), Some("Possible"));
  }

  #[test]
  fn test_antithesis_validation_new() {
    let validation = AntithesisValidation::new(0.8, vec!["Suggestion".to_string()], true);

    assert!((validation.score - 0.8).abs() < f64::EPSILON);
    assert_eq!(validation.suggestions.len(), 1);
    assert!(validation.is_valid);
  }

  #[test]
  fn test_antithesis_validation_passing() {
    let validation = AntithesisValidation::passing(0.9);

    assert!((validation.score - 0.9).abs() < f64::EPSILON);
    assert!(validation.suggestions.is_empty());
    assert!(validation.is_valid);
  }

  #[test]
  fn test_antithesis_validation_failing() {
    let validation = AntithesisValidation::failing(vec!["Fix this".to_string()]);

    assert_eq!(validation.score, 0.0);
    assert!(!validation.is_valid);
    assert_eq!(validation.suggestions.len(), 1);
  }

  #[test]
  fn test_antithesis_validation_quality_gate() {
    let passing = AntithesisValidation::passing(0.75);
    assert!(passing.passes_quality_gate());

    let failing = AntithesisValidation::passing(0.5);
    assert!(!failing.passes_quality_gate());
  }

  #[test]
  fn test_ears_pattern_names() {
    assert_eq!(EarsPattern::Ubiquitous.name(), "Ubiquitous");
    assert_eq!(EarsPattern::EventDriven.name(), "Event-Driven");
    assert_eq!(EarsPattern::Unwanted.name(), "Unwanted");
    assert_eq!(EarsPattern::StateDriven.name(), "State-Driven");
    assert_eq!(EarsPattern::OptionalFeature.name(), "Optional Feature");
  }

  #[test]
  fn test_ears_extraction_new() {
    let reqs = vec![
      ExtractedEarsRequirement::new(
        "1".to_string(),
        "Req 1".to_string(),
        EarsPattern::Ubiquitous,
        "section1".to_string(),
      ),
      ExtractedEarsRequirement::new(
        "2".to_string(),
        "Req 2".to_string(),
        EarsPattern::EventDriven,
        "section2".to_string(),
      ),
    ];

    let extraction = EarsExtraction::new(reqs);

    assert_eq!(extraction.total_count, 2);
    assert!(extraction.has_requirements());
    assert_eq!(extraction.analyzed_sections.len(), 2);
  }

  #[test]
  fn test_ears_extraction_empty() {
    let extraction = EarsExtraction::empty();

    assert_eq!(extraction.total_count, 0);
    assert!(!extraction.has_requirements());
  }

  #[test]
  fn test_hole_punching_validation_new() {
    let results =
      HolePunchingResults::new().address(HoleType::DiscoveryHole, "Found via search".to_string());

    let validation = HolePunchingValidation::new(results);

    assert!(!validation.is_complete);
    assert_eq!(validation.addressed_count, 1);
  }

  #[test]
  fn test_hole_punching_validation_complete() {
    let validation = HolePunchingValidation::complete();

    assert!(validation.is_complete);
    assert_eq!(validation.addressed_count, 3);
  }

  #[test]
  fn test_hole_punching_validation_default() {
    let validation = HolePunchingValidation::default();

    assert!(!validation.is_complete);
    assert_eq!(validation.addressed_count, 0);
  }

  #[test]
  fn test_kirk_contract_serialization() -> Result<(), serde_json::Error> {
    let contract = KirkContract16::new();

    let json = serde_json::to_string(&contract)?;

    let deserialized: KirkContract16 = serde_json::from_str(&json)?;
    assert_eq!(contract, deserialized);
    Ok(())
  }

  #[test]
  fn test_vorp_validation_serialization() -> Result<(), serde_json::Error> {
    let vorp = VorpValidation::new(0.8, 0.7, 0.9, 0.6);

    let json = serde_json::to_string(&vorp)?;

    let deserialized: VorpValidation = serde_json::from_str(&json)?;
    assert_eq!(vorp, deserialized);
    Ok(())
  }

  #[test]
  fn test_antithesis_validation_serialization() -> Result<(), serde_json::Error> {
    let validation = AntithesisValidation::passing(0.85);

    let json = serde_json::to_string(&validation)?;

    let deserialized: AntithesisValidation = serde_json::from_str(&json)?;
    assert_eq!(validation, deserialized);
    Ok(())
  }

  #[test]
  fn test_ears_extraction_serialization() -> Result<(), serde_json::Error> {
    let extraction = EarsExtraction::new(vec![ExtractedEarsRequirement::new(
      "1".to_string(),
      "Req".to_string(),
      EarsPattern::Ubiquitous,
      "section".to_string(),
    )]);

    let json = serde_json::to_string(&extraction)?;

    let deserialized: EarsExtraction = serde_json::from_str(&json)?;
    assert_eq!(extraction, deserialized);
    Ok(())
  }
}
