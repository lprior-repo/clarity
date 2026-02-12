//! PME Discover State Management
//!
//! Dioxus signals and state management for the PME Discover phase.

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use crate::pme::types::*;
use dioxus::prelude::*;
use std::collections::HashMap;
use uuid::Uuid;

/// PME Discover state signal
#[derive(Clone, Debug, Default)]
pub struct PmeDiscoverSignals {
  /// All hypotheses
  pub hypotheses: Signal<Vec<Hypothesis>>,
  /// All interviews
  pub interviews: Signal<Vec<CustomerDiscoveryInterview>>,
  /// Persona evidence map (persona_id -> evidence)
  pub persona_evidence: Signal<HashMap<Uuid, PersonaEvidence>>,
  /// Detected plot holes by scenario
  pub plot_holes: Signal<Vec<ScenarioPlotHole>>,
  /// Active hypothesis being edited
  pub active_hypothesis: Signal<Option<Uuid>>,
  /// Active interview being conducted
  pub active_interview: Signal<Option<Uuid>>,
}

impl PmeDiscoverSignals {
  /// Get hypothesis count
  pub fn hypothesis_count(&self) -> usize {
    self.hypotheses.read().len()
  }

  /// Get validated hypothesis count
  pub fn validated_hypothesis_count(&self) -> usize {
    self
      .hypotheses
      .read()
      .iter()
      .filter(|h| h.status == HypothesisStatus::Validated)
      .count()
  }

  /// Get interview count
  pub fn interview_count(&self) -> usize {
    self.interviews.read().len()
  }

  /// Get total signal count
  pub fn total_signal_count(&self) -> usize {
    self.interviews.read().iter().map(|i| i.signals.len()).sum()
  }

  /// Get strong interview count
  pub fn strong_interview_count(&self) -> usize {
    self
      .interviews
      .read()
      .iter()
      .filter(|i| i.has_strong_signals())
      .count()
  }

  /// Get plot hole counts
  pub fn plot_hole_counts(&self) -> (usize, usize, usize) {
    let holes = self.plot_holes.read();
    let total = holes.len();
    let blocking = holes.iter().filter(|p| p.is_blocking()).count();
    let fatal = holes
      .iter()
      .filter(|p| p.severity == PlotHoleSeverity::Fatal)
      .count();
    (total, blocking, fatal)
  }

  /// Get blocking plot hole count
  pub fn blocking_plot_hole_count(&self) -> usize {
    self
      .plot_holes
      .read()
      .iter()
      .filter(|p| p.is_blocking())
      .count()
  }

  /// Get plot holes for a scenario (cloned)
  pub fn plot_holes_for_scenario(&self, scenario_id: Uuid) -> Vec<ScenarioPlotHole> {
    self
      .plot_holes
      .read()
      .iter()
      .filter(|p| p.scenario_id == scenario_id)
      .cloned()
      .collect()
  }

  /// Get hypotheses list (cloned)
  pub fn hypotheses_list(&self) -> Vec<Hypothesis> {
    self.hypotheses.read().iter().cloned().collect()
  }

  /// Get interviews list (cloned)
  pub fn interviews_list(&self) -> Vec<CustomerDiscoveryInterview> {
    self.interviews.read().iter().cloned().collect()
  }

  /// Get persona evidence stats for a single persona
  pub fn persona_evidence_stats(&self, persona_id: Uuid) -> (f32, bool, bool) {
    let evidence = self.persona_evidence.read();
    match evidence.get(&persona_id) {
      Some(e) => (e.confidence_level, e.is_validated(), e.is_straw_man()),
      None => (0.0, false, true),
    }
  }

  /// Get straw men count for personas
  pub fn straw_men_count(&self, personas: &[crate::planner::types::Persona]) -> usize {
    let evidence = self.persona_evidence.read();
    personas
      .iter()
      .filter(|p| evidence.get(&p.id).map_or(true, |e| e.is_straw_man()))
      .count()
  }

  /// Get refuted hypothesis count
  pub fn refuted_hypothesis_count(&self) -> usize {
    self
      .hypotheses
      .read()
      .iter()
      .filter(|h| h.status == HypothesisStatus::Refuted)
      .count()
  }

  /// Get testing hypothesis count
  pub fn testing_hypothesis_count(&self) -> usize {
    self
      .hypotheses
      .read()
      .iter()
      .filter(|h| h.status == HypothesisStatus::Testing)
      .count()
  }

  /// Get persona evidence stats
  pub fn persona_stats(
    &self,
    personas: &[crate::planner::types::Persona],
  ) -> (usize, usize, usize) {
    let evidence = self.persona_evidence.read();
    let total = personas.len();
    let validated = evidence.values().filter(|e| e.is_validated()).count();
    let straw_men = personas
      .iter()
      .filter(|p| evidence.get(&p.id).map_or(true, |e| e.is_straw_man()))
      .count();
    (total, validated, straw_men)
  }
  /// Create new PME signals
  #[must_use]
  pub fn new() -> Self {
    Self {
      hypotheses: Signal::new(Vec::new()),
      interviews: Signal::new(Vec::new()),
      persona_evidence: Signal::new(HashMap::new()),
      plot_holes: Signal::new(Vec::new()),
      active_hypothesis: Signal::new(None),
      active_interview: Signal::new(None),
    }
  }

  /// Add a hypothesis
  pub fn add_hypothesis(&mut self, hypothesis: Hypothesis) {
    self.hypotheses.write().push(hypothesis);
  }

  /// Update a hypothesis
  pub fn update_hypothesis(&mut self, updated: Hypothesis) {
    let mut hypotheses = self.hypotheses.write();
    if let Some(existing) = hypotheses.iter_mut().find(|h| h.id == updated.id) {
      *existing = updated;
    }
  }

  /// Remove a hypothesis
  pub fn remove_hypothesis(&mut self, id: Uuid) {
    self.hypotheses.write().retain(|h| h.id != id);
    if *self.active_hypothesis.read() == Some(id) {
      self.active_hypothesis.set(None);
    }
  }

  /// Add an interview
  pub fn add_interview(&mut self, interview: CustomerDiscoveryInterview) {
    self.interviews.write().push(interview);
  }

  /// Update an interview
  pub fn update_interview(&mut self, updated: CustomerDiscoveryInterview) {
    let mut interviews = self.interviews.write();
    if let Some(existing) = interviews.iter_mut().find(|i| i.id == updated.id) {
      *existing = updated;
    }
  }

  /// Remove an interview
  pub fn remove_interview(&mut self, id: Uuid) {
    self.interviews.write().retain(|i| i.id != id);
    if *self.active_interview.read() == Some(id) {
      self.active_interview.set(None);
    }
  }

  /// Link interview to persona evidence
  pub fn link_interview_to_persona(&mut self, persona_id: Uuid, interview_id: Uuid) {
    let mut evidence = self.persona_evidence.write();
    let entry = evidence
      .entry(persona_id)
      .or_insert_with(|| PersonaEvidence::new(persona_id));
    let updated = entry.clone().with_interview(interview_id);
    *entry = updated;
  }

  /// Add a validation check to persona
  pub fn add_persona_validation(&mut self, persona_id: Uuid, check: PersonaValidationCheck) {
    let mut evidence = self.persona_evidence.write();
    if let Some(e) = evidence.get_mut(&persona_id) {
      let updated = e.clone().with_validation_check(check);
      *e = updated;
    }
  }

  /// Add a plot hole
  pub fn add_plot_hole(&mut self, plot_hole: ScenarioPlotHole) {
    self.plot_holes.write().push(plot_hole);
  }

  /// Resolve a plot hole (remove it)
  pub fn resolve_plot_hole(&mut self, scenario_id: Uuid, hole_type: PlotHoleType) {
    self
      .plot_holes
      .write()
      .retain(|p| !(p.scenario_id == scenario_id && p.hole_type == hole_type));
  }

  /// Get health score for the discover phase
  pub fn health_score(&self) -> f32 {
    let hypotheses = self.hypotheses.read();
    let plot_holes = self.plot_holes.read();
    let evidence = self.persona_evidence.read();

    if hypotheses.is_empty() {
      return 0.0;
    }

    // Hypothesis score (40% weight)
    let validated = hypotheses
      .iter()
      .filter(|h| h.status == HypothesisStatus::Validated)
      .count();
    let hypothesis_score = validated as f32 / hypotheses.len() as f32;

    // Plot hole penalty (30% weight)
    let blocking = plot_holes.iter().filter(|p| p.is_blocking()).count();
    let hole_penalty = (blocking as f32 * 0.1).min(0.3);

    // Evidence score (30% weight)
    let total_evidence = evidence.len();
    let validated_personas = evidence.values().filter(|e| e.is_validated()).count();
    let evidence_score = if total_evidence > 0 {
      validated_personas as f32 / total_evidence as f32
    } else {
      0.0
    };

    let raw_score = (hypothesis_score * 0.4) + (evidence_score * 0.3) - hole_penalty;
    raw_score.clamp(0.0, 1.0)
  }

  /// Check if discovery phase is complete enough to proceed
  pub fn can_proceed_to_define(&self) -> bool {
    let hypotheses = self.hypotheses.read();
    let evidence = self.persona_evidence.read();

    // Need at least one validated hypothesis
    let has_validated_hypothesis = hypotheses
      .iter()
      .any(|h| h.status == HypothesisStatus::Validated);

    // Need at least 2 interviews
    let interviews = self.interviews.read();
    let has_enough_interviews = interviews.len() >= 2;

    // No blocking plot holes
    let plot_holes = self.plot_holes.read();
    let no_blocking_holes = plot_holes.iter().all(|p| !p.is_blocking());

    // No straw man personas
    let no_straw_men = evidence
      .values()
      .all(|e| !e.is_straw_man() || e.interviews_referenced.is_empty());

    has_validated_hypothesis && has_enough_interviews && no_blocking_holes && no_straw_men
  }
}

impl PartialEq for PmeDiscoverSignals {
  fn eq(&self, _other: &Self) -> bool {
    // Signals are compared by identity, not content
    // This is a simplified implementation for the type system
    true
  }
}

/// Provider component for PME Discover signals
#[component]
pub fn PmeDiscoverProvider(children: Element) -> Element {
  use_context_provider(|| PmeDiscoverSignals::new());

  rsx! {
    {children}
  }
}
