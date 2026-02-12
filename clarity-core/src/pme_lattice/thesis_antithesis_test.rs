//! Tests for Thesis & Antithesis Generator (bead bd-16qs.1)
//!
//! TDD tests for Thesis, Antithesis, HypothesisPair, and ThesisAntithesisGenerator

#![allow(clippy::all)]
#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]
#![allow(clippy::expect_used)]
#![allow(clippy::manual_string_new)]
#![forbid(unsafe_code)]

use super::thesis_antithesis::{
  Antithesis, AntithesisBuilder, HypothesisPair, HypothesisPairBuilder, SynthesisStatus, Thesis,
  ThesisAntithesisError, ThesisAntithesisGenerator, ThesisBuilder,
};

// ============================================================================
// THESIS TESTS
// ============================================================================

#[test]
fn thesis_new_requires_non_empty_statement() {
  let result = Thesis::new("".to_string());
  assert!(matches!(result, Err(ThesisAntithesisError::EmptyStatement)));
}

#[test]
fn thesis_new_requires_non_whitespace_statement() {
  let result = Thesis::new("   ".to_string());
  assert!(matches!(result, Err(ThesisAntithesisError::EmptyStatement)));
}

#[test]
fn thesis_new_succeeds_with_valid_statement() {
  let thesis = Thesis::new("Users want automated reporting".to_string());
  assert!(thesis.is_ok());
  let t = thesis.expect("valid thesis");
  assert_eq!(t.statement(), "Users want automated reporting");
  assert!(t.supporting_evidence().is_empty());
}

#[test]
fn thesis_builder_creates_thesis() {
  let thesis = ThesisBuilder::new()
    .statement("Users want automated reporting".to_string())
    .build();

  assert!(thesis.is_ok());
  let t = thesis.expect("valid thesis");
  assert_eq!(t.statement(), "Users want automated reporting");
}

#[test]
fn thesis_builder_with_evidence() {
  let thesis = ThesisBuilder::new()
    .statement("Users want automated reporting".to_string())
    .supporting_evidence("Interview #1: 'I spend hours on reports'".to_string())
    .supporting_evidence("Survey: 80% want automation".to_string())
    .build();

  assert!(thesis.is_ok());
  let t = thesis.expect("valid thesis");
  assert_eq!(t.supporting_evidence().len(), 2);
}

#[test]
fn thesis_builder_requires_statement() {
  let thesis = ThesisBuilder::new().build();
  assert!(matches!(thesis, Err(ThesisAntithesisError::EmptyStatement)));
}

#[test]
fn thesis_has_id() {
  let thesis = Thesis::new("Test".to_string()).expect("valid");
  assert!(!thesis.id().is_nil());
}

#[test]
fn thesis_has_created_at() {
  let thesis = Thesis::new("Test".to_string()).expect("valid");
  // Just verify it's accessible (not testing exact time)
  let _ = thesis.created_at();
}

#[test]
fn thesis_add_evidence_returns_new_instance() {
  let thesis = Thesis::new("Test".to_string()).expect("valid");
  let thesis_with_evidence = thesis.clone().add_evidence("New evidence".to_string());

  assert!(thesis.supporting_evidence().is_empty());
  assert_eq!(thesis_with_evidence.supporting_evidence().len(), 1);
}

// ============================================================================
// ANTITHESIS TESTS
// ============================================================================

#[test]
fn antithesis_new_requires_non_empty_statement() {
  let result = Antithesis::new("".to_string());
  assert!(matches!(result, Err(ThesisAntithesisError::EmptyStatement)));
}

#[test]
fn antithesis_new_requires_non_whitespace_statement() {
  let result = Antithesis::new("   ".to_string());
  assert!(matches!(result, Err(ThesisAntithesisError::EmptyStatement)));
}

#[test]
fn antithesis_new_succeeds_with_valid_statement() {
  let antithesis = Antithesis::new("Users prefer manual control".to_string());
  assert!(antithesis.is_ok());
  let a = antithesis.expect("valid antithesis");
  assert_eq!(a.counter_statement(), "Users prefer manual control");
  assert!(a.attacking_evidence().is_empty());
}

#[test]
fn antithesis_builder_creates_antithesis() {
  let antithesis = AntithesisBuilder::new()
    .counter_statement("Users prefer manual control".to_string())
    .build();

  assert!(antithesis.is_ok());
  let a = antithesis.expect("valid antithesis");
  assert_eq!(a.counter_statement(), "Users prefer manual control");
}

#[test]
fn antithesis_builder_with_evidence() {
  let antithesis = AntithesisBuilder::new()
    .counter_statement("Users prefer manual control".to_string())
    .attacking_evidence("Some users distrust automation".to_string())
    .attacking_evidence("Manual process allows customization".to_string())
    .build();

  assert!(antithesis.is_ok());
  let a = antithesis.expect("valid antithesis");
  assert_eq!(a.attacking_evidence().len(), 2);
}

#[test]
fn antithesis_builder_requires_statement() {
  let antithesis = AntithesisBuilder::new().build();
  assert!(matches!(
    antithesis,
    Err(ThesisAntithesisError::EmptyStatement)
  ));
}

#[test]
fn antithesis_has_id() {
  let antithesis = Antithesis::new("Test".to_string()).expect("valid");
  assert!(!antithesis.id().is_nil());
}

#[test]
fn antithesis_add_evidence_returns_new_instance() {
  let antithesis = Antithesis::new("Test".to_string()).expect("valid");
  let antithesis_with_evidence = antithesis
    .clone()
    .add_evidence("New attacking evidence".to_string());

  assert!(antithesis.attacking_evidence().is_empty());
  assert_eq!(antithesis_with_evidence.attacking_evidence().len(), 1);
}

// ============================================================================
// HYPOTHESIS PAIR TESTS
// ============================================================================

#[test]
fn hypothesis_pair_new_requires_valid_thesis_and_antithesis() {
  let thesis = Thesis::new("Users want X".to_string()).expect("valid");
  let antithesis = Antithesis::new("Users don't want X".to_string()).expect("valid");

  let pair = HypothesisPair::new(thesis, antithesis);
  assert!(pair.is_ok());
}

#[test]
fn hypothesis_pair_rejects_identical_thesis_and_antithesis() {
  let thesis = Thesis::new("Users want X".to_string()).expect("valid");
  let antithesis = Antithesis::new("Users want X".to_string()).expect("valid");

  let pair = HypothesisPair::new(thesis, antithesis);
  assert!(matches!(
    pair,
    Err(ThesisAntithesisError::ThesisEqualsAntithesis)
  ));
}

#[test]
fn hypothesis_pair_rejects_case_insensitive_identical() {
  let thesis = Thesis::new("Users want X".to_string()).expect("valid");
  let antithesis = Antithesis::new("USERS WANT X".to_string()).expect("valid");

  let pair = HypothesisPair::new(thesis, antithesis);
  assert!(matches!(
    pair,
    Err(ThesisAntithesisError::ThesisEqualsAntithesis)
  ));
}

#[test]
fn hypothesis_pair_builder_creates_pair() {
  let pair = HypothesisPairBuilder::new()
    .thesis_statement("Users want automated reporting".to_string())
    .antithesis_statement("Users prefer manual control".to_string())
    .build();

  assert!(pair.is_ok());
  let p = pair.expect("valid pair");
  assert_eq!(p.thesis().statement(), "Users want automated reporting");
  assert_eq!(
    p.antithesis().counter_statement(),
    "Users prefer manual control"
  );
}

#[test]
fn hypothesis_pair_builder_with_synthesis_notes() {
  let pair = HypothesisPairBuilder::new()
    .thesis_statement("Users want X".to_string())
    .antithesis_statement("Users don't want X".to_string())
    .synthesis_note("Interview data suggests mixed preferences".to_string())
    .build();

  assert!(pair.is_ok());
  let p = pair.expect("valid pair");
  assert_eq!(p.synthesis_notes().len(), 1);
}

#[test]
fn hypothesis_pair_synthesis_status_defaults_to_pending() {
  let pair = HypothesisPairBuilder::new()
    .thesis_statement("Users want X".to_string())
    .antithesis_statement("Users don't want X".to_string())
    .build()
    .expect("valid");

  assert_eq!(pair.synthesis_status(), SynthesisStatus::Pending);
}

#[test]
fn hypothesis_pair_can_set_synthesis_status() {
  let pair = HypothesisPairBuilder::new()
    .thesis_statement("Users want X".to_string())
    .antithesis_statement("Users don't want X".to_string())
    .synthesis_status(SynthesisStatus::ThesisSupported)
    .build()
    .expect("valid");

  assert_eq!(pair.synthesis_status(), SynthesisStatus::ThesisSupported);
}

#[test]
fn hypothesis_pair_has_id() {
  let pair = HypothesisPairBuilder::new()
    .thesis_statement("Users want X".to_string())
    .antithesis_statement("Users don't want X".to_string())
    .build()
    .expect("valid");

  assert!(!pair.id().is_nil());
}

#[test]
fn hypothesis_pair_add_synthesis_note_returns_new_instance() {
  let pair = HypothesisPairBuilder::new()
    .thesis_statement("Users want X".to_string())
    .antithesis_statement("Users don't want X".to_string())
    .build()
    .expect("valid");

  assert!(pair.synthesis_notes().is_empty());
  let updated = pair.add_synthesis_note("New insight".to_string());
  assert_eq!(updated.synthesis_notes().len(), 1);
}

#[test]
fn hypothesis_pair_with_status_returns_new_instance() {
  let pair = HypothesisPairBuilder::new()
    .thesis_statement("Users want X".to_string())
    .antithesis_statement("Users don't want X".to_string())
    .build()
    .expect("valid");

  assert_eq!(pair.synthesis_status(), SynthesisStatus::Pending);
  let updated = pair.with_status(SynthesisStatus::AntithesisSupported);
  assert_eq!(
    updated.synthesis_status(),
    SynthesisStatus::AntithesisSupported
  );
}

// ============================================================================
// SYNTHESIS STATUS TESTS
// ============================================================================

#[test]
fn synthesis_status_variants() {
  assert_eq!(SynthesisStatus::Pending.to_string(), "Pending");
  assert_eq!(
    SynthesisStatus::ThesisSupported.to_string(),
    "Thesis Supported"
  );
  assert_eq!(
    SynthesisStatus::AntithesisSupported.to_string(),
    "Antithesis Supported"
  );
  assert_eq!(SynthesisStatus::Inconclusive.to_string(), "Inconclusive");
  assert_eq!(
    SynthesisStatus::RequiresMoreData.to_string(),
    "Requires More Data"
  );
}

// ============================================================================
// THESIS ANTITHESIS GENERATOR TESTS
// ============================================================================

#[test]
fn generator_new_creates_empty_generator() {
  let gen = ThesisAntithesisGenerator::new();
  assert!(gen.pairs().is_empty());
}

#[test]
fn generator_create_pair_returns_hypothesis_pair() {
  let gen = ThesisAntithesisGenerator::new();
  let pair = gen.create_pair(
    "Users want automated reporting".to_string(),
    "Users prefer manual control".to_string(),
  );

  assert!(pair.is_ok());
}

#[test]
fn generator_create_pair_validates_statements() {
  let gen = ThesisAntithesisGenerator::new();

  let empty_thesis = gen.create_pair("".to_string(), "Valid antithesis".to_string());
  assert!(matches!(
    empty_thesis,
    Err(ThesisAntithesisError::EmptyStatement)
  ));

  let empty_antithesis = gen.create_pair("Valid thesis".to_string(), "".to_string());
  assert!(matches!(
    empty_antithesis,
    Err(ThesisAntithesisError::EmptyStatement)
  ));
}

#[test]
fn generator_create_pair_rejects_identical_statements() {
  let gen = ThesisAntithesisGenerator::new();
  let pair = gen.create_pair("Same statement".to_string(), "Same statement".to_string());

  assert!(matches!(
    pair,
    Err(ThesisAntithesisError::ThesisEqualsAntithesis)
  ));
}

#[test]
fn generator_add_pair_adds_to_collection() {
  let gen = ThesisAntithesisGenerator::new();
  let pair = HypothesisPairBuilder::new()
    .thesis_statement("Users want X".to_string())
    .antithesis_statement("Users don't want X".to_string())
    .build()
    .expect("valid");

  let updated = gen.clone().add_pair(pair);
  assert!(gen.pairs().is_empty());
  assert_eq!(updated.pairs().len(), 1);
}

#[test]
fn generator_generate_prompts_returns_list() {
  let prompts = ThesisAntithesisGenerator::antithesis_prompts();
  assert!(!prompts.is_empty());
  assert!(prompts
    .iter()
    .any(|p| p.contains("fail") || p.contains("wrong")));
}

#[test]
fn generator_filter_by_status_returns_matching_pairs() {
  let gen = ThesisAntithesisGenerator::new()
    .add_pair(
      HypothesisPairBuilder::new()
        .thesis_statement("Thesis 1".to_string())
        .antithesis_statement("Antithesis 1".to_string())
        .synthesis_status(SynthesisStatus::ThesisSupported)
        .build()
        .expect("valid"),
    )
    .add_pair(
      HypothesisPairBuilder::new()
        .thesis_statement("Thesis 2".to_string())
        .antithesis_statement("Antithesis 2".to_string())
        .synthesis_status(SynthesisStatus::Pending)
        .build()
        .expect("valid"),
    );

  let pending = gen.filter_by_status(SynthesisStatus::Pending);
  assert_eq!(pending.len(), 1);

  let supported = gen.filter_by_status(SynthesisStatus::ThesisSupported);
  assert_eq!(supported.len(), 1);

  let inconclusive = gen.filter_by_status(SynthesisStatus::Inconclusive);
  assert!(inconclusive.is_empty());
}

#[test]
fn generator_count_by_status() {
  let gen = ThesisAntithesisGenerator::new()
    .add_pair(
      HypothesisPairBuilder::new()
        .thesis_statement("T1".to_string())
        .antithesis_statement("A1".to_string())
        .synthesis_status(SynthesisStatus::ThesisSupported)
        .build()
        .expect("valid"),
    )
    .add_pair(
      HypothesisPairBuilder::new()
        .thesis_statement("T2".to_string())
        .antithesis_statement("A2".to_string())
        .synthesis_status(SynthesisStatus::Pending)
        .build()
        .expect("valid"),
    )
    .add_pair(
      HypothesisPairBuilder::new()
        .thesis_statement("T3".to_string())
        .antithesis_statement("A3".to_string())
        .synthesis_status(SynthesisStatus::Pending)
        .build()
        .expect("valid"),
    );

  assert_eq!(gen.count_by_status(SynthesisStatus::Pending), 2);
  assert_eq!(gen.count_by_status(SynthesisStatus::ThesisSupported), 1);
  assert_eq!(gen.count_by_status(SynthesisStatus::Inconclusive), 0);
}

#[test]
fn generator_serialization() {
  let pair = HypothesisPairBuilder::new()
    .thesis_statement("Users want X".to_string())
    .antithesis_statement("Users don't want X".to_string())
    .synthesis_note("Test note".to_string())
    .build()
    .expect("valid");

  if let Ok(json) = serde_json::to_string(&pair) {
    let parsed: Result<HypothesisPair, _> = serde_json::from_str(&json);
    assert!(parsed.is_ok());
  }
}

#[test]
fn thesis_serialization() {
  let thesis = Thesis::new("Users want X".to_string()).expect("valid");

  if let Ok(json) = serde_json::to_string(&thesis) {
    let parsed: Result<Thesis, _> = serde_json::from_str(&json);
    assert!(parsed.is_ok());
  }
}

#[test]
fn antithesis_serialization() {
  let antithesis = Antithesis::new("Users don't want X".to_string()).expect("valid");

  if let Ok(json) = serde_json::to_string(&antithesis) {
    let parsed: Result<Antithesis, _> = serde_json::from_str(&json);
    assert!(parsed.is_ok());
  }
}

#[test]
fn error_display() {
  let err = ThesisAntithesisError::EmptyStatement;
  assert!(!err.to_string().is_empty());

  let err = ThesisAntithesisError::ThesisEqualsAntithesis;
  assert!(!err.to_string().is_empty());
}
