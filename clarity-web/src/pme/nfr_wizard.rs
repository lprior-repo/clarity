#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![allow(clippy::suspicious_else_formatting)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Product Architecture & NFR (Non-Functional Requirements) Wizard
//!
//! Implements the NFR categories and trade-off wizard for PME Develop Phase.
//!
//! NFR Categories:
//! - Latency/Consistency: Response time vs data freshness
//! - Availability: Uptime vs cost
//! - Scalability: Handle growth vs simplicity
//! - Maintainability: Easy changes vs optimization
//! - Security: Protection vs usability
//!
//! The trade-off wizard forces choices based on persona needs,
//! recognizing that you can't optimize for everything.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Domain errors for NFR wizard operations
#[derive(Debug, Error, PartialEq, Clone)]
pub enum NfrWizardError {
    #[error("invalid NFR category: {0}")]
    InvalidCategory(String),

    #[error("conflicting priorities: {0} and {1} cannot both be high")]
    ConflictingPriorities(String, String),

    #[error("incomplete trade-off: {0} requires a choice")]
    IncompleteTradeOff(String),

    #[error("invalid persona: {0}")]
    InvalidPersona(String),
}

/// The 5 NFR categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NfrCategory {
    /// Response time vs data freshness
    LatencyConsistency,
    /// Uptime vs cost
    Availability,
    /// Handle growth vs simplicity
    Scalability,
    /// Easy changes vs optimization
    Maintainability,
    /// Protection vs usability
    Security,
}

impl NfrCategory {
    /// All NFR categories
    pub fn all() -> &'static [NfrCategory] {
        &[
            NfrCategory::LatencyConsistency,
            NfrCategory::Availability,
            NfrCategory::Scalability,
            NfrCategory::Maintainability,
            NfrCategory::Security,
        ]
    }

    /// Human-readable label
    pub fn label(&self) -> &'static str {
        match self {
            NfrCategory::LatencyConsistency => "Latency vs Consistency",
            NfrCategory::Availability => "Availability",
            NfrCategory::Scalability => "Scalability",
            NfrCategory::Maintainability => "Maintainability",
            NfrCategory::Security => "Security",
        }
    }

    /// Description of the trade-off
    pub fn description(&self) -> &'static str {
        match self {
            NfrCategory::LatencyConsistency => {
                "Trade-off between fast responses and fresh/consistent data"
            }
            NfrCategory::Availability => "Trade-off between uptime guarantees and infrastructure cost",
            NfrCategory::Scalability => {
                "Trade-off between handling growth and architectural simplicity"
            }
            NfrCategory::Maintainability => {
                "Trade-off between ease of changes and performance optimization"
            }
            NfrCategory::Security => "Trade-off between protection level and user experience",
        }
    }

    /// Get the two poles of this trade-off
    pub fn trade_off_poles(&self) -> (&'static str, &'static str) {
        match self {
            NfrCategory::LatencyConsistency => ("Low Latency", "Strong Consistency"),
            NfrCategory::Availability => ("High Availability", "Low Cost"),
            NfrCategory::Scalability => ("Horizontal Scale", "Simplicity"),
            NfrCategory::Maintainability => ("Easy Changes", "Peak Performance"),
            NfrCategory::Security => ("Maximum Security", "Frictionless UX"),
        }
    }

    /// Get the default priority based on persona type
    pub fn default_priority_for_persona(&self, persona: &PersonaType) -> Priority {
        match (self, persona) {
            // Startup prioritizes speed and simplicity
            (NfrCategory::LatencyConsistency, PersonaType::Startup) => Priority::High,
            (NfrCategory::Availability, PersonaType::Startup) => Priority::Medium,
            (NfrCategory::Scalability, PersonaType::Startup) => Priority::Medium,
            (NfrCategory::Maintainability, PersonaType::Startup) => Priority::High,
            (NfrCategory::Security, PersonaType::Startup) => Priority::Medium,

            // Enterprise prioritizes reliability and security
            (NfrCategory::LatencyConsistency, PersonaType::Enterprise) => Priority::Medium,
            (NfrCategory::Availability, PersonaType::Enterprise) => Priority::Critical,
            (NfrCategory::Scalability, PersonaType::Enterprise) => Priority::High,
            (NfrCategory::Maintainability, PersonaType::Enterprise) => Priority::High,
            (NfrCategory::Security, PersonaType::Enterprise) => Priority::Critical,

            // Consumer app prioritizes UX and scale
            (NfrCategory::LatencyConsistency, PersonaType::ConsumerApp) => Priority::Critical,
            (NfrCategory::Availability, PersonaType::ConsumerApp) => Priority::High,
            (NfrCategory::Scalability, PersonaType::ConsumerApp) => Priority::High,
            (NfrCategory::Maintainability, PersonaType::ConsumerApp) => Priority::Medium,
            (NfrCategory::Security, PersonaType::ConsumerApp) => Priority::Medium,

            // Internal tool prioritizes maintainability
            (NfrCategory::LatencyConsistency, PersonaType::InternalTool) => Priority::Medium,
            (NfrCategory::Availability, PersonaType::InternalTool) => Priority::Medium,
            (NfrCategory::Scalability, PersonaType::InternalTool) => Priority::Low,
            (NfrCategory::Maintainability, PersonaType::InternalTool) => Priority::Critical,
            (NfrCategory::Security, PersonaType::InternalTool) => Priority::High,
        }
    }
}

/// Priority level for NFR
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Priority {
    /// Nice to have, can compromise
    Low,
    /// Important but negotiable
    Medium,
    /// Must have, limited compromise
    High,
    /// Non-negotiable, hard requirement
    Critical,
}

impl Priority {
    /// All priorities in order
    pub fn all() -> &'static [Priority] {
        &[Priority::Low, Priority::Medium, Priority::High, Priority::Critical]
    }

    /// Numeric value for comparison
    pub fn value(&self) -> u8 {
        match self {
            Priority::Low => 1,
            Priority::Medium => 2,
            Priority::High => 3,
            Priority::Critical => 4,
        }
    }

    /// Label for display
    pub fn label(&self) -> &'static str {
        match self {
            Priority::Low => "Low",
            Priority::Medium => "Medium",
            Priority::High => "High",
            Priority::Critical => "Critical",
        }
    }

    /// Color class for UI display
    pub fn color_class(&self) -> &'static str {
        match self {
            Priority::Low => "bg-slate-500",
            Priority::Medium => "bg-blue-500",
            Priority::High => "bg-amber-500",
            Priority::Critical => "bg-red-500",
        }
    }
}

/// Persona type for default NFR recommendations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PersonaType {
    /// Early-stage startup, speed matters most
    Startup,
    /// Enterprise customer, reliability is key
    Enterprise,
    /// Consumer-facing app, UX is paramount
    ConsumerApp,
    /// Internal tool, maintainability matters
    InternalTool,
}

impl PersonaType {
    /// All persona types
    pub fn all() -> &'static [PersonaType] {
        &[
            PersonaType::Startup,
            PersonaType::Enterprise,
            PersonaType::ConsumerApp,
            PersonaType::InternalTool,
        ]
    }

    /// Label for display
    pub fn label(&self) -> &'static str {
        match self {
            PersonaType::Startup => "Startup",
            PersonaType::Enterprise => "Enterprise",
            PersonaType::ConsumerApp => "Consumer App",
            PersonaType::InternalTool => "Internal Tool",
        }
    }

    /// Description of this persona
    pub fn description(&self) -> &'static str {
        match self {
            PersonaType::Startup => "Early-stage company prioritizing speed and iteration",
            PersonaType::Enterprise => "Large organization requiring reliability and compliance",
            PersonaType::ConsumerApp => "Consumer-facing product where UX drives success",
            PersonaType::InternalTool => "Internal tooling where developer experience matters",
        }
    }
}

/// A trade-off choice between two poles
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TradeOffChoice {
    /// The NFR category
    pub category: NfrCategory,
    /// Position on the trade-off spectrum (0-100)
    /// 0 = first pole, 100 = second pole
    pub position: u8,
    /// The priority assigned to this NFR
    pub priority: Priority,
    /// User's rationale for this choice
    pub rationale: Option<String>,
}

impl TradeOffChoice {
    /// Create a new trade-off choice
    pub fn new(category: NfrCategory, position: u8, priority: Priority) -> Self {
        Self {
            category,
            position: position.min(100),
            priority,
            rationale: None,
        }
    }

    /// Add rationale
    pub fn with_rationale(mut self, rationale: impl Into<String>) -> Self {
        self.rationale = Some(rationale.into());
        self
    }

    /// Get the pole labels
    pub fn poles(&self) -> (&'static str, &'static str) {
        self.category.trade_off_poles()
    }

    /// Describe the current position
    pub fn position_description(&self) -> String {
        let (pole1, pole2) = self.poles();
        match self.position {
            0..=20 => format!("Strongly favoring {}", pole1),
            21..=40 => format!("Leaning towards {}", pole1),
            41..=60 => "Balanced approach".to_string(),
            61..=80 => format!("Leaning towards {}", pole2),
            81..=100 => format!("Strongly favoring {}", pole2),
            _ => "Invalid position".to_string(),
        }
    }
}

/// Architecture decision record (ADR) entry
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArchitectureDecision {
    /// Unique identifier
    pub id: String,
    /// Title of the decision
    pub title: String,
    /// Context leading to this decision
    pub context: String,
    /// The decision made
    pub decision: String,
    /// Consequences of this decision
    pub consequences: Vec<String>,
    /// Related NFR trade-offs
    pub related_nfrs: Vec<NfrCategory>,
    /// Status of this decision
    pub status: DecisionStatus,
}

impl ArchitectureDecision {
    /// Create a new architecture decision
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        context: impl Into<String>,
        decision: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            context: context.into(),
            decision: decision.into(),
            consequences: Vec::new(),
            related_nfrs: Vec::new(),
            status: DecisionStatus::Proposed,
        }
    }

    /// Add a consequence
    pub fn with_consequence(mut self, consequence: impl Into<String>) -> Self {
        self.consequences.push(consequence.into());
        self
    }

    /// Add related NFR
    pub fn with_nfr(mut self, nfr: NfrCategory) -> Self {
        self.related_nfrs.push(nfr);
        self
    }

    /// Set status
    pub fn with_status(mut self, status: DecisionStatus) -> Self {
        self.status = status;
        self
    }
}

/// Status of an architecture decision
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionStatus {
    /// Proposed but not yet accepted
    Proposed,
    /// Accepted and in effect
    Accepted,
    /// Deprecated but still in effect
    Deprecated,
    /// Superseded by another decision
    Superseded,
}

/// Complete NFR profile for a product
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NfrProfile {
    /// The persona type for this product
    pub persona: PersonaType,
    /// Trade-off choices for each category
    pub trade_offs: HashMap<NfrCategory, TradeOffChoice>,
    /// Architecture decisions made
    pub architecture_decisions: Vec<ArchitectureDecision>,
    /// Quality gates to enforce
    pub quality_gates: Vec<QualityGate>,
}

impl NfrProfile {
    /// Create a new NFR profile with defaults for the persona
    pub fn new(persona: PersonaType) -> Self {
        let trade_offs = NfrCategory::all()
            .iter()
            .map(|&cat| {
                let priority = cat.default_priority_for_persona(&persona);
                let position = Self::default_position_for_persona(cat, &persona);
                (cat, TradeOffChoice::new(cat, position, priority))
            })
            .collect();

        Self {
            persona,
            trade_offs,
            architecture_decisions: Vec::new(),
            quality_gates: Vec::new(),
        }
    }

    /// Get default position for a persona
    fn default_position_for_persona(category: NfrCategory, persona: &PersonaType) -> u8 {
        match (category, persona) {
            // Startup: favor low latency, simplicity
            (NfrCategory::LatencyConsistency, PersonaType::Startup) => 20,
            (NfrCategory::Availability, PersonaType::Startup) => 50,
            (NfrCategory::Scalability, PersonaType::Startup) => 30,
            (NfrCategory::Maintainability, PersonaType::Startup) => 20,
            (NfrCategory::Security, PersonaType::Startup) => 40,

            // Enterprise: favor consistency, availability, security
            (NfrCategory::LatencyConsistency, PersonaType::Enterprise) => 80,
            (NfrCategory::Availability, PersonaType::Enterprise) => 90,
            (NfrCategory::Scalability, PersonaType::Enterprise) => 70,
            (NfrCategory::Maintainability, PersonaType::Enterprise) => 60,
            (NfrCategory::Security, PersonaType::Enterprise) => 90,

            // Consumer: favor low latency, availability
            (NfrCategory::LatencyConsistency, PersonaType::ConsumerApp) => 15,
            (NfrCategory::Availability, PersonaType::ConsumerApp) => 85,
            (NfrCategory::Scalability, PersonaType::ConsumerApp) => 80,
            (NfrCategory::Maintainability, PersonaType::ConsumerApp) => 50,
            (NfrCategory::Security, PersonaType::ConsumerApp) => 60,

            // Internal: favor maintainability
            (NfrCategory::LatencyConsistency, PersonaType::InternalTool) => 50,
            (NfrCategory::Availability, PersonaType::InternalTool) => 50,
            (NfrCategory::Scalability, PersonaType::InternalTool) => 30,
            (NfrCategory::Maintainability, PersonaType::InternalTool) => 10,
            (NfrCategory::Security, PersonaType::InternalTool) => 70,
        }
    }

    /// Update a trade-off choice
    pub fn update_trade_off(
        &mut self,
        choice: TradeOffChoice,
    ) -> Result<(), NfrWizardError> {
        // Validate for conflicts
        self.validate_trade_off(&choice)?;
        self.trade_offs.insert(choice.category, choice);
        Ok(())
    }

    /// Validate a trade-off for conflicts
    fn validate_trade_off(&self, choice: &TradeOffChoice) -> Result<(), NfrWizardError> {
        // Check for conflicting critical priorities
        // Count criticals excluding the one we're updating (if it exists)
        let existing_critical = self
            .trade_offs
            .get(&choice.category)
            .map(|t| t.priority == Priority::Critical)
            == Some(true);

        let critical_count = self
            .trade_offs
            .values()
            .filter(|t| t.priority == Priority::Critical && t.category != choice.category)
            .count();

        // If adding a new critical (not replacing one), check limit
        if choice.priority == Priority::Critical && !existing_critical && critical_count >= 2 {
            // Find existing critical to report conflict
            if let Some(existing) = self
                .trade_offs
                .values()
                .find(|t| t.priority == Priority::Critical && t.category != choice.category)
            {
                return Err(NfrWizardError::ConflictingPriorities(
                    existing.category.label().to_string(),
                    choice.category.label().to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Add an architecture decision
    pub fn add_decision(&mut self, decision: ArchitectureDecision) {
        self.architecture_decisions.push(decision);
    }

    /// Add a quality gate
    pub fn add_quality_gate(&mut self, gate: QualityGate) {
        self.quality_gates.push(gate);
    }

    /// Get the trade-off for a category
    pub fn get_trade_off(&self, category: NfrCategory) -> Option<&TradeOffChoice> {
        self.trade_offs.get(&category)
    }

    /// Generate a summary of the profile
    pub fn summary(&self) -> NfrSummary {
        let critical_nfrs: Vec<_> = self
            .trade_offs
            .values()
            .filter(|t| t.priority == Priority::Critical)
            .map(|t| t.category)
            .collect();

        let high_nfrs: Vec<_> = self
            .trade_offs
            .values()
            .filter(|t| t.priority == Priority::High)
            .map(|t| t.category)
            .collect();

        NfrSummary {
            persona: self.persona,
            critical_nfrs,
            high_nfrs,
            decision_count: self.architecture_decisions.len(),
            gate_count: self.quality_gates.len(),
        }
    }
}

/// Quality gate for NFR enforcement
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QualityGate {
    /// Gate identifier
    pub id: String,
    /// Name of the gate
    pub name: String,
    /// Related NFR category
    pub category: NfrCategory,
    /// Metric being measured
    pub metric: String,
    /// Threshold value
    pub threshold: String,
    /// Comparison operator
    pub operator: ComparisonOperator,
    /// Is this gate blocking?
    pub blocking: bool,
}

impl QualityGate {
    /// Create a new quality gate
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        category: NfrCategory,
        metric: impl Into<String>,
        threshold: impl Into<String>,
        operator: ComparisonOperator,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            category,
            metric: metric.into(),
            threshold: threshold.into(),
            operator,
            blocking: true,
        }
    }

    /// Set as non-blocking
    pub fn non_blocking(mut self) -> Self {
        self.blocking = false;
        self
    }

    /// Check if a value passes the gate
    pub fn check(&self, value: &str) -> GateResult {
        // Parse and compare based on operator
        // For simplicity, we'll do string comparison for now
        // In production, this would parse to appropriate types
        let threshold_str = self.threshold.as_str();
        let passed = match self.operator {
            ComparisonOperator::LessThan => value < threshold_str,
            ComparisonOperator::LessThanOrEqual => value <= threshold_str,
            ComparisonOperator::Equals => value == threshold_str,
            ComparisonOperator::GreaterThanOrEqual => value >= threshold_str,
            ComparisonOperator::GreaterThan => value > threshold_str,
        };

        GateResult {
            gate_id: self.id.clone(),
            passed,
            actual_value: value.to_string(),
            threshold: self.threshold.clone(),
        }
    }
}

/// Comparison operators for quality gates
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComparisonOperator {
    LessThan,
    LessThanOrEqual,
    Equals,
    GreaterThanOrEqual,
    GreaterThan,
}

impl ComparisonOperator {
    /// Symbol representation
    pub fn symbol(&self) -> &'static str {
        match self {
            ComparisonOperator::LessThan => "<",
            ComparisonOperator::LessThanOrEqual => "<=",
            ComparisonOperator::Equals => "==",
            ComparisonOperator::GreaterThanOrEqual => ">=",
            ComparisonOperator::GreaterThan => ">",
        }
    }
}

/// Result of checking a quality gate
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GateResult {
    /// Gate identifier
    pub gate_id: String,
    /// Did the check pass?
    pub passed: bool,
    /// Actual value measured
    pub actual_value: String,
    /// Threshold expected
    pub threshold: String,
}

/// Summary of an NFR profile
#[derive(Debug, Clone, PartialEq)]
pub struct NfrSummary {
    /// The persona type
    pub persona: PersonaType,
    /// NFRs marked as critical
    pub critical_nfrs: Vec<NfrCategory>,
    /// NFRs marked as high priority
    pub high_nfrs: Vec<NfrCategory>,
    /// Number of architecture decisions
    pub decision_count: usize,
    /// Number of quality gates
    pub gate_count: usize,
}

/// The NFR Wizard
///
/// Guides users through making trade-off decisions
#[derive(Debug, Clone)]
pub struct NfrWizard {
    /// Current profile being edited
    profile: Option<NfrProfile>,
    /// Wizard state
    state: WizardState,
}

/// State of the wizard
#[derive(Debug, Clone, PartialEq)]
pub enum WizardState {
    /// Selecting persona type
    SelectPersona,
    /// Setting trade-offs
    SetTradeOffs,
    /// Reviewing decisions
    Review,
    /// Wizard complete
    Complete,
}

impl Default for NfrWizard {
    fn default() -> Self {
        Self::new()
    }
}

impl NfrWizard {
    /// Create a new NFR wizard
    pub fn new() -> Self {
        Self {
            profile: None,
            state: WizardState::SelectPersona,
        }
    }

    /// Start with a persona
    pub fn start_with_persona(&mut self, persona: PersonaType) {
        self.profile = Some(NfrProfile::new(persona));
        self.state = WizardState::SetTradeOffs;
    }

    /// Get current state
    pub fn state(&self) -> &WizardState {
        &self.state
    }

    /// Get the current profile
    pub fn profile(&self) -> Option<&NfrProfile> {
        self.profile.as_ref()
    }

    /// Get mutable profile
    pub fn profile_mut(&mut self) -> Option<&mut NfrProfile> {
        self.profile.as_mut()
    }

    /// Update a trade-off
    pub fn update_trade_off(
        &mut self,
        choice: TradeOffChoice,
    ) -> Result<(), NfrWizardError> {
        match &mut self.profile {
            Some(profile) => profile.update_trade_off(choice),
            None => Err(NfrWizardError::InvalidPersona("No profile initialized".into())),
        }
    }

    /// Move to review state
    pub fn proceed_to_review(&mut self) -> Result<(), NfrWizardError> {
        if self.profile.is_none() {
            return Err(NfrWizardError::IncompleteTradeOff(
                "No profile initialized".into(),
            ));
        }

        // Check all trade-offs are set
        if let Some(ref profile) = self.profile {
            if profile.trade_offs.len() < NfrCategory::all().len() {
                return Err(NfrWizardError::IncompleteTradeOff(
                    "Not all NFR trade-offs have been set".into(),
                ));
            }
        }

        self.state = WizardState::Review;
        Ok(())
    }

    /// Complete the wizard
    pub fn complete(&mut self) -> Option<NfrProfile> {
        if self.state == WizardState::Review {
            self.state = WizardState::Complete;
            self.profile.clone()
        } else {
            None
        }
    }

    /// Get the current category to configure (first unset or in progress)
    pub fn current_category(&self) -> Option<NfrCategory> {
        match (&self.profile, &self.state) {
            (Some(profile), WizardState::SetTradeOffs) => {
                // Find first category without a rationale
                NfrCategory::all().iter().find_map(|&cat| {
                    match profile.trade_offs.get(&cat) {
                        Some(choice) if choice.rationale.is_none() => Some(cat),
                        None => Some(cat),
                        _ => None,
                    }
                })
            }
            _ => None,
        }
    }

    /// Progress percentage through the wizard
    pub fn progress(&self) -> u8 {
        match &self.state {
            WizardState::SelectPersona => 0,
            WizardState::SetTradeOffs => {
                if let Some(profile) = &self.profile {
                    let with_rationale = profile
                        .trade_offs
                        .values()
                        .filter(|t| t.rationale.is_some())
                        .count();
                    let total = NfrCategory::all().len();
                    let base = 20; // Persona selection is 20%
                    let trade_off_progress =
                        (with_rationale * 60) / total; // Trade-offs are 60%
                    (base + trade_off_progress) as u8
                } else {
                    10
                }
            }
            WizardState::Review => 90,
            WizardState::Complete => 100,
        }
    }
}

/// Create default quality gates for a persona
pub fn create_default_gates(persona: PersonaType) -> Vec<QualityGate> {
    match persona {
        PersonaType::Startup => vec![
            QualityGate::new(
                "latency_p50",
                "P50 Latency",
                NfrCategory::LatencyConsistency,
                "response_time_p50",
                "200ms",
                ComparisonOperator::LessThan,
            ),
            QualityGate::new(
                "deploy_time",
                "Deployment Time",
                NfrCategory::Maintainability,
                "deploy_duration",
                "15min",
                ComparisonOperator::LessThan,
            ),
        ],
        PersonaType::Enterprise => vec![
            QualityGate::new(
                "latency_p99",
                "P99 Latency",
                NfrCategory::LatencyConsistency,
                "response_time_p99",
                "500ms",
                ComparisonOperator::LessThan,
            ),
            QualityGate::new(
                "availability",
                "Availability",
                NfrCategory::Availability,
                "uptime_percentage",
                "99.9%",
                ComparisonOperator::GreaterThanOrEqual,
            ),
            QualityGate::new(
                "security_scan",
                "Security Scan",
                NfrCategory::Security,
                "vulnerabilities",
                "0",
                ComparisonOperator::Equals,
            ),
        ],
        PersonaType::ConsumerApp => vec![
            QualityGate::new(
                "latency_p95",
                "P95 Latency",
                NfrCategory::LatencyConsistency,
                "response_time_p95",
                "300ms",
                ComparisonOperator::LessThan,
            ),
            QualityGate::new(
                "availability",
                "Availability",
                NfrCategory::Availability,
                "uptime_percentage",
                "99.5%",
                ComparisonOperator::GreaterThanOrEqual,
            ),
        ],
        PersonaType::InternalTool => vec![
            QualityGate::new(
                "test_coverage",
                "Test Coverage",
                NfrCategory::Maintainability,
                "coverage_percentage",
                "80%",
                ComparisonOperator::GreaterThanOrEqual,
            )
            .non_blocking(),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nfr_category_trade_off_poles() {
        let (pole1, pole2) = NfrCategory::LatencyConsistency.trade_off_poles();
        assert_eq!(pole1, "Low Latency");
        assert_eq!(pole2, "Strong Consistency");
    }

    #[test]
    fn test_persona_defaults() {
        assert_eq!(
            NfrCategory::Availability.default_priority_for_persona(&PersonaType::Enterprise),
            Priority::Critical
        );
        assert_eq!(
            NfrCategory::Maintainability.default_priority_for_persona(&PersonaType::InternalTool),
            Priority::Critical
        );
        assert_eq!(
            NfrCategory::LatencyConsistency.default_priority_for_persona(&PersonaType::ConsumerApp),
            Priority::Critical
        );
    }

    #[test]
    fn test_trade_off_choice() {
        let choice = TradeOffChoice::new(NfrCategory::LatencyConsistency, 30, Priority::High)
            .with_rationale("User research shows speed is critical");

        assert_eq!(choice.position, 30);
        assert_eq!(choice.priority, Priority::High);
        assert!(choice.rationale.is_some());
        assert!(choice.position_description().contains("Low Latency"));
    }

    #[test]
    fn test_nfr_profile_creation() {
        let profile = NfrProfile::new(PersonaType::Startup);

        assert_eq!(profile.persona, PersonaType::Startup);
        assert_eq!(profile.trade_offs.len(), NfrCategory::all().len());
    }

    #[test]
    fn test_nfr_profile_conflict_detection() {
        let mut profile = NfrProfile::new(PersonaType::Enterprise);

        // Set one critical
        let choice1 =
            TradeOffChoice::new(NfrCategory::Availability, 90, Priority::Critical);
        profile.update_trade_off(choice1).map_err(|e| e.to_string()).ok();

        // Set another critical - should work (replacing existing)
        let choice2 =
            TradeOffChoice::new(NfrCategory::Security, 90, Priority::Critical);
        assert!(profile.update_trade_off(choice2).is_ok());
    }

    #[test]
    fn test_architecture_decision() {
        let decision = ArchitectureDecision::new(
            "ADR-001",
            "Use PostgreSQL for primary database",
            "Need ACID compliance for financial data",
            "PostgreSQL will be our primary data store",
        )
        .with_consequence("Team needs PostgreSQL expertise")
        .with_nfr(NfrCategory::Availability)
        .with_status(DecisionStatus::Accepted);

        assert_eq!(decision.id, "ADR-001");
        assert_eq!(decision.consequences.len(), 1);
        assert_eq!(decision.status, DecisionStatus::Accepted);
    }

    #[test]
    fn test_quality_gate() {
        let gate = QualityGate::new(
            "latency",
            "Response Time",
            NfrCategory::LatencyConsistency,
            "p99_latency",
            "500",
            ComparisonOperator::LessThan,
        );

        assert!(gate.check("400").passed);
        assert!(!gate.check("600").passed);
    }

    #[test]
    fn test_nfr_wizard_flow() {
        let mut wizard = NfrWizard::new();

        assert_eq!(wizard.state(), &WizardState::SelectPersona);
        assert_eq!(wizard.progress(), 0);

        wizard.start_with_persona(PersonaType::Startup);
        assert_eq!(wizard.state(), &WizardState::SetTradeOffs);
        assert!(wizard.profile().is_some());

        // Add rationale to all trade-offs
        if let Some(profile) = wizard.profile_mut() {
            for cat in NfrCategory::all() {
                if let Some(trade_off) = profile.trade_offs.get_mut(cat) {
                    trade_off.rationale = Some("Test rationale".to_string());
                }
            }
        }

        let result = wizard.proceed_to_review();
        assert!(result.is_ok());
        assert_eq!(wizard.state(), &WizardState::Review);
        assert!(wizard.progress() >= 90);
    }

    #[test]
    fn test_create_default_gates() {
        let enterprise_gates = create_default_gates(PersonaType::Enterprise);
        assert!(!enterprise_gates.is_empty());

        // Enterprise should have availability gate
        assert!(enterprise_gates
            .iter()
            .any(|g| g.category == NfrCategory::Availability));

        let startup_gates = create_default_gates(PersonaType::Startup);
        assert!(!startup_gates.is_empty());
    }

    #[test]
    fn test_nfr_summary() {
        let profile = NfrProfile::new(PersonaType::Enterprise);
        let summary = profile.summary();

        assert_eq!(summary.persona, PersonaType::Enterprise);
        assert!(!summary.critical_nfrs.is_empty());
    }

    #[test]
    fn test_priority_ordering() {
        assert!(Priority::Critical.value() > Priority::High.value());
        assert!(Priority::High.value() > Priority::Medium.value());
        assert!(Priority::Medium.value() > Priority::Low.value());
    }
}
