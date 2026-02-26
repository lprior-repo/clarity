#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! North Star Scenario Builder
//!
//! Builds and validates user journey scenarios using a Character + Simulation framework.
//! Detects plot holes including: missing discovery mechanisms, unhandled edge cases,
//! and timeline inconsistencies.
//!
//! # Framework Components
//!
//! - **Characters**: User personas with motivations and capabilities
//! - **Scenarios**: User journey narratives with beginning, middle, and end
//! - **Timeline Events**: Ordered sequence of actions and outcomes
//! - **Discovery Mechanisms**: How users find and adopt the product
//! - **Edge Cases**: Boundary conditions and error scenarios
//!
//! # Example
//!
//! ```
//! use clarity_web::pme::discover::north_star::{NorthStarBuilder, Character, Scenario};
//!
//! let character = Character::new("Alice".to_string())
//!     .with_motivation("Save time on reporting".to_string());
//!
//! let scenario = Scenario::new("Monthly Report".to_string())
//!     .with_character(character.name.clone());
//!
//! let output = NorthStarBuilder::build(scenario);
//! ```

use itertools::Itertools;
use serde::{Deserialize, Serialize};
use thiserror::Error;

// ============================================================================
// Character Types
// ============================================================================

/// A character in a scenario (user persona).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Character {
    /// Character name
    pub name: String,
    /// Role/title
    pub role: String,
    /// Motivations (what drives them)
    pub motivations: Vec<String>,
    /// Capabilities (what they can do)
    pub capabilities: Vec<String>,
    /// Constraints (what limits them)
    pub constraints: Vec<String>,
    /// Initial state at scenario start
    pub initial_state: String,
}

impl Character {
    /// Create a new character.
    #[must_use]
    pub fn new(name: String) -> Self {
        Self {
            name,
            role: String::new(),
            motivations: Vec::new(),
            capabilities: Vec::new(),
            constraints: Vec::new(),
            initial_state: String::new(),
        }
    }

    /// Set role.
    #[must_use]
    pub fn with_role(mut self, role: String) -> Self {
        self.role = role;
        self
    }

    /// Add a motivation.
    #[must_use]
    pub fn with_motivation(mut self, motivation: String) -> Self {
        self.motivations.push(motivation);
        self
    }

    /// Add a capability.
    #[must_use]
    pub fn with_capability(mut self, capability: String) -> Self {
        self.capabilities.push(capability);
        self
    }

    /// Add a constraint.
    #[must_use]
    pub fn with_constraint(mut self, constraint: String) -> Self {
        self.constraints.push(constraint);
        self
    }

    /// Set initial state.
    #[must_use]
    pub fn with_initial_state(mut self, state: String) -> Self {
        self.initial_state = state;
        self
    }

    /// Check if character is well-defined.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        !self.name.is_empty() && !self.motivations.is_empty()
    }
}

// ============================================================================
// Timeline Event Types
// ============================================================================

/// An event in the scenario timeline.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TimelineEvent {
    /// Event identifier
    pub id: String,
    /// Event description
    pub description: String,
    /// Actor performing the event
    pub actor: String,
    /// Event type
    pub event_type: EventType,
    /// Timestamp/order in sequence
    pub order: u32,
    /// Expected outcome
    pub expected_outcome: String,
    /// Actual outcome (if known)
    pub actual_outcome: Option<String>,
    /// Dependencies (other event IDs)
    pub dependencies: Vec<String>,
}

/// Types of events in a scenario.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventType {
    /// User discovers the product
    Discovery,
    /// User takes action
    Action,
    /// System responds
    SystemResponse,
    /// Decision point
    Decision,
    /// Error or failure
    Error,
    /// Success/completion
    Success,
    /// External event
    External,
}

impl TimelineEvent {
    /// Create a new timeline event.
    #[must_use]
    pub fn new(id: String, description: String, order: u32) -> Self {
        Self {
            id,
            description,
            actor: String::new(),
            event_type: EventType::Action,
            order,
            expected_outcome: String::new(),
            actual_outcome: None,
            dependencies: Vec::new(),
        }
    }

    /// Set actor.
    #[must_use]
    pub fn with_actor(mut self, actor: String) -> Self {
        self.actor = actor;
        self
    }

    /// Set event type.
    #[must_use]
    pub fn with_type(mut self, event_type: EventType) -> Self {
        self.event_type = event_type;
        self
    }

    /// Set expected outcome.
    #[must_use]
    pub fn with_expected_outcome(mut self, outcome: String) -> Self {
        self.expected_outcome = outcome;
        self
    }

    /// Add dependency.
    #[must_use]
    pub fn with_dependency(mut self, event_id: String) -> Self {
        self.dependencies.push(event_id);
        self
    }

    /// Record actual outcome.
    #[must_use]
    pub fn with_actual_outcome(mut self, outcome: String) -> Self {
        self.actual_outcome = Some(outcome);
        self
    }
}

// ============================================================================
// Discovery Mechanism Types
// ============================================================================

/// A mechanism for how users discover the product.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DiscoveryMechanism {
    /// Mechanism name
    pub name: String,
    /// Description
    pub description: String,
    /// Channels (where discovery happens)
    pub channels: Vec<String>,
    /// Triggers (what initiates discovery)
    pub triggers: Vec<String>,
    /// Success probability (0.0-1.0)
    pub probability: f64,
}

impl DiscoveryMechanism {
    /// Create a new discovery mechanism.
    #[must_use]
    pub fn new(name: String, description: String) -> Self {
        Self {
            name,
            description,
            channels: Vec::new(),
            triggers: Vec::new(),
            probability: 0.5,
        }
    }

    /// Add a channel.
    #[must_use]
    pub fn with_channel(mut self, channel: String) -> Self {
        self.channels.push(channel);
        self
    }

    /// Add a trigger.
    #[must_use]
    pub fn with_trigger(mut self, trigger: String) -> Self {
        self.triggers.push(trigger);
        self
    }

    /// Set probability.
    #[must_use]
    pub fn with_probability(mut self, probability: f64) -> Self {
        self.probability = probability.clamp(0.0, 1.0);
        self
    }

    /// Check if mechanism is valid.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        !self.name.is_empty() && !self.channels.is_empty()
    }
}

// ============================================================================
// Edge Case Types
// ============================================================================

/// An edge case or boundary condition.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EdgeCase {
    /// Edge case name
    pub name: String,
    /// Description
    pub description: String,
    /// Preconditions
    pub preconditions: Vec<String>,
    /// Expected behavior
    pub expected_behavior: String,
    /// Handling status
    pub handling_status: HandlingStatus,
    /// Severity if unhandled (0.0-1.0)
    pub severity: f64,
}

/// Status of edge case handling.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HandlingStatus {
    /// Not yet addressed
    Unhandled,
    /// Partially addressed
    PartiallyHandled,
    /// Fully addressed
    Handled,
    /// Explicitly accepted as risk
    Accepted,
}

impl EdgeCase {
    /// Create a new edge case.
    #[must_use]
    pub fn new(name: String, description: String) -> Self {
        Self {
            name,
            description,
            preconditions: Vec::new(),
            expected_behavior: String::new(),
            handling_status: HandlingStatus::Unhandled,
            severity: 0.5,
        }
    }

    /// Add a precondition.
    #[must_use]
    pub fn with_precondition(mut self, precondition: String) -> Self {
        self.preconditions.push(precondition);
        self
    }

    /// Set expected behavior.
    #[must_use]
    pub fn with_expected_behavior(mut self, behavior: String) -> Self {
        self.expected_behavior = behavior;
        self
    }

    /// Set handling status.
    #[must_use]
    pub fn with_status(mut self, status: HandlingStatus) -> Self {
        self.handling_status = status;
        self
    }

    /// Set severity.
    #[must_use]
    pub fn with_severity(mut self, severity: f64) -> Self {
        self.severity = severity.clamp(0.0, 1.0);
        self
    }

    /// Check if this edge case needs attention.
    #[must_use]
    pub fn needs_attention(&self) -> bool {
        matches!(self.handling_status, HandlingStatus::Unhandled | HandlingStatus::PartiallyHandled)
            && self.severity > 0.5
    }
}

// ============================================================================
// Plot Hole Types
// ============================================================================

/// A detected plot hole in the scenario.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlotHole {
    /// Plot hole type
    pub hole_type: PlotHoleType,
    /// Description
    pub description: String,
    /// Location in scenario
    pub location: String,
    /// Severity (0.0-1.0)
    pub severity: f64,
    /// Suggested fix
    pub suggested_fix: Option<String>,
}

/// Types of plot holes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PlotHoleType {
    /// Missing discovery mechanism
    MissingDiscovery,
    /// Edge case not handled
    UnhandledEdgeCase,
    /// Timeline inconsistency
    TimelineInconsistency,
    /// Character motivation unclear
    UnclearMotivation,
    /// Missing dependency
    MissingDependency,
    /// Logical contradiction
    Contradiction,
    /// Unexplained state change
    UnexplainedTransition,
}

impl PlotHole {
    /// Create a new plot hole.
    #[must_use]
    pub fn new(hole_type: PlotHoleType, description: String, location: String) -> Self {
        Self {
            hole_type,
            description,
            location,
            severity: 0.5,
            suggested_fix: None,
        }
    }

    /// Set severity.
    #[must_use]
    pub fn with_severity(mut self, severity: f64) -> Self {
        self.severity = severity.clamp(0.0, 1.0);
        self
    }

    /// Set suggested fix.
    #[must_use]
    pub fn with_fix(mut self, fix: String) -> Self {
        self.suggested_fix = Some(fix);
        self
    }
}

// ============================================================================
// Scenario Types
// ============================================================================

/// A user journey scenario.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Scenario {
    /// Scenario name
    pub name: String,
    /// Description
    pub description: String,
    /// Characters in this scenario
    pub characters: Vec<String>,
    /// Timeline of events
    pub timeline: Vec<TimelineEvent>,
    /// Discovery mechanisms
    pub discovery_mechanisms: Vec<DiscoveryMechanism>,
    /// Edge cases
    pub edge_cases: Vec<EdgeCase>,
    /// North star goal (the ideal outcome)
    pub north_star_goal: String,
}

impl Scenario {
    /// Create a new scenario.
    #[must_use]
    pub fn new(name: String) -> Self {
        Self {
            name,
            description: String::new(),
            characters: Vec::new(),
            timeline: Vec::new(),
            discovery_mechanisms: Vec::new(),
            edge_cases: Vec::new(),
            north_star_goal: String::new(),
        }
    }

    /// Set description.
    #[must_use]
    pub fn with_description(mut self, description: String) -> Self {
        self.description = description;
        self
    }

    /// Add a character.
    #[must_use]
    pub fn with_character(mut self, character: String) -> Self {
        self.characters.push(character);
        self
    }

    /// Add a timeline event.
    #[must_use]
    pub fn with_event(mut self, event: TimelineEvent) -> Self {
        self.timeline.push(event);
        self
    }

    /// Add a discovery mechanism.
    #[must_use]
    pub fn with_discovery(mut self, mechanism: DiscoveryMechanism) -> Self {
        self.discovery_mechanisms.push(mechanism);
        self
    }

    /// Add an edge case.
    #[must_use]
    pub fn with_edge_case(mut self, edge_case: EdgeCase) -> Self {
        self.edge_cases.push(edge_case);
        self
    }

    /// Set north star goal.
    #[must_use]
    pub fn with_north_star(mut self, goal: String) -> Self {
        self.north_star_goal = goal;
        self
    }

    /// Check if scenario has discovery events.
    #[must_use]
    pub fn has_discovery(&self) -> bool {
        self.timeline.iter().any(|e| e.event_type == EventType::Discovery)
            || !self.discovery_mechanisms.is_empty()
    }

    /// Get events in order.
    #[must_use]
    pub fn ordered_events(&self) -> Vec<&TimelineEvent> {
        self.timeline
            .iter()
            .sorted_by_key(|e| e.order)
            .collect()
    }
}

// ============================================================================
// Simulation Result
// ============================================================================

/// Result of simulating a scenario.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SimulationResult {
    /// Whether the scenario is coherent
    pub is_coherent: bool,
    /// Detected plot holes
    pub plot_holes: Vec<PlotHole>,
    /// Unhandled edge cases
    pub unhandled_edge_cases: Vec<EdgeCase>,
    /// Timeline issues
    pub timeline_issues: Vec<String>,
    /// Coherence score (0.0-1.0)
    pub coherence_score: f64,
    /// Recommendations
    pub recommendations: Vec<String>,
}

// ============================================================================
// Output Types
// ============================================================================

/// Output from the North Star Builder.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NorthStarOutput {
    /// The scenario
    pub scenario: Scenario,
    /// Simulation result
    pub simulation: SimulationResult,
    /// Statistics
    pub stats: NorthStarStats,
    /// Validated discovery mechanisms
    pub validated_discoveries: Vec<DiscoveryMechanism>,
}

/// Statistics about the scenario.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NorthStarStats {
    /// Total events
    pub total_events: usize,
    /// Total characters
    pub total_characters: usize,
    /// Discovery mechanisms
    pub discovery_count: usize,
    /// Edge cases
    pub edge_case_count: usize,
    /// Plot holes detected
    pub plot_hole_count: usize,
    /// Critical issues
    pub critical_issue_count: usize,
}

// ============================================================================
// Error Types
// ============================================================================

/// Errors from the North Star Builder.
#[derive(Debug, Error)]
pub enum NorthStarError {
    /// Scenario name is empty
    #[error("Scenario name cannot be empty")]
    EmptyScenarioName,

    /// No characters defined
    #[error("At least one character must be defined")]
    NoCharacters,

    /// No events defined
    #[error("At least one event must be defined")]
    NoEvents,

    /// No north star goal
    #[error("North star goal must be defined")]
    NoNorthStar,

    /// Timeline order conflict
    #[error("Timeline has events with duplicate order values")]
    TimelineOrderConflict,
}

// ============================================================================
// North Star Builder Implementation
// ============================================================================

/// North Star Scenario Builder.
///
/// Builds and validates user journey scenarios with plot hole detection.
pub struct NorthStarBuilder;

impl NorthStarBuilder {
    /// Build and simulate a scenario.
    ///
    /// # Errors
    ///
    /// Returns an error if the scenario is invalid.
    pub fn build(scenario: Scenario) -> Result<NorthStarOutput, NorthStarError> {
        if scenario.name.is_empty() {
            return Err(NorthStarError::EmptyScenarioName);
        }
        if scenario.characters.is_empty() {
            return Err(NorthStarError::NoCharacters);
        }
        if scenario.timeline.is_empty() {
            return Err(NorthStarError::NoEvents);
        }
        if scenario.north_star_goal.is_empty() {
            return Err(NorthStarError::NoNorthStar);
        }

        // Check for timeline order conflicts
        let orders: Vec<u32> = scenario.timeline.iter().map(|e| e.order).sorted().collect();
        for window in orders.windows(2) {
            if window[0] == window[1] {
                return Err(NorthStarError::TimelineOrderConflict);
            }
        }

        // Run simulation
        let simulation = Self::simulate(&scenario);

        // Calculate statistics
        let stats = NorthStarStats {
            total_events: scenario.timeline.len(),
            total_characters: scenario.characters.len(),
            discovery_count: scenario.discovery_mechanisms.len(),
            edge_case_count: scenario.edge_cases.len(),
            plot_hole_count: simulation.plot_holes.len(),
            critical_issue_count: simulation
                .plot_holes
                .iter()
                .filter(|p| p.severity > 0.7)
                .count(),
        };

        // Filter validated discovery mechanisms
        let validated_discoveries = scenario
            .discovery_mechanisms
            .iter()
            .filter(|d| d.is_valid())
            .cloned()
            .collect();

        Ok(NorthStarOutput {
            scenario,
            simulation,
            stats,
            validated_discoveries,
        })
    }

    /// Simulate the scenario and detect plot holes.
    fn simulate(scenario: &Scenario) -> SimulationResult {
        let mut plot_holes = Vec::new();
        let mut timeline_issues = Vec::new();
        let mut recommendations = Vec::new();

        // Check for missing discovery mechanism
        if !scenario.has_discovery() {
            plot_holes.push(
                PlotHole::new(
                    PlotHoleType::MissingDiscovery,
                    "No discovery mechanism defined - how does the user find the product?".to_string(),
                    "Scenario start".to_string(),
                )
                .with_severity(0.9)
                .with_fix("Add a discovery mechanism (search, referral, ad, etc.)".to_string()),
            );
            recommendations.push(
                "Define how users discover the product - this is critical for adoption".to_string(),
            );
        }

        // Check for unhandled edge cases
        let unhandled_edge_cases: Vec<_> = scenario
            .edge_cases
            .iter()
            .filter(|e| e.needs_attention())
            .cloned()
            .collect();

        for edge_case in &unhandled_edge_cases {
            plot_holes.push(
                PlotHole::new(
                    PlotHoleType::UnhandledEdgeCase,
                    format!("Edge case '{}' is not properly handled", edge_case.name),
                    format!("Edge case: {}", edge_case.name),
                )
                .with_severity(edge_case.severity)
                .with_fix(format!("Define behavior for: {}", edge_case.description)),
            );
        }

        // Check timeline for issues
        let ordered_events = scenario.ordered_events();

        // Check for missing dependencies
        let event_ids: Vec<&str> = scenario.timeline.iter().map(|e| e.id.as_str()).collect();
        for event in &scenario.timeline {
            for dep in &event.dependencies {
                if !event_ids.contains(&dep.as_str()) {
                    plot_holes.push(
                        PlotHole::new(
                            PlotHoleType::MissingDependency,
                            format!("Event '{}' depends on non-existent event '{}'", event.id, dep),
                            format!("Event: {}", event.id),
                        )
                        .with_severity(0.7),
                    );
                }
            }
        }

        // Check for timeline inconsistencies
        for window in ordered_events.windows(2) {
            let prev = &window[0];
            let curr = &window[1];

            // Check for logical issues
            if curr.event_type == EventType::Success && prev.event_type == EventType::Error {
                timeline_issues.push(format!(
                    "Success follows error without recovery: '{}' -> '{}'",
                    prev.description, curr.description
                ));
            }

            // Check for missing actor
            if curr.actor.is_empty() && curr.event_type == EventType::Action {
                timeline_issues.push(format!(
                    "Action event '{}' has no actor defined",
                    curr.description
                ));
            }
        }

        // Check for unexplained transitions
        for window in ordered_events.windows(2) {
            let prev = &window[0];
            let curr = &window[1];

            // State change without explanation
            if prev.actual_outcome.is_some() && curr.actual_outcome.is_some() {
                if let (Some(prev_out), Some(curr_out)) = (&prev.actual_outcome, &curr.actual_outcome) {
                    if prev_out != curr_out && !Self::are_related(prev_out, curr_out) {
                        plot_holes.push(
                            PlotHole::new(
                                PlotHoleType::UnexplainedTransition,
                                format!("State changed from '{}' to '{}' without explanation", prev_out, curr_out),
                                format!("Between '{}' and '{}'", prev.id, curr.id),
                            )
                            .with_severity(0.5),
                        );
                    }
                }
            }
        }

        // Calculate coherence score
        let coherence_score = Self::calculate_coherence_score(&plot_holes, &timeline_issues, scenario);

        // Generate recommendations
        if coherence_score < 0.7 {
            recommendations.push("Scenario has coherence issues - address plot holes before proceeding".to_string());
        }
        if !unhandled_edge_cases.is_empty() {
            recommendations.push(format!(
                "Handle {} edge case(s) before implementation",
                unhandled_edge_cases.len()
            ));
        }

        let is_coherent = coherence_score >= 0.7 && plot_holes.is_empty();

        SimulationResult {
            is_coherent,
            plot_holes,
            unhandled_edge_cases,
            timeline_issues,
            coherence_score,
            recommendations,
        }
    }

    /// Check if two outcomes are related.
    fn are_related(a: &str, b: &str) -> bool {
        // Simple heuristic - check for common words
        let a_words: Vec<&str> = a.split_whitespace().collect();
        let b_words: Vec<&str> = b.split_whitespace().collect();

        a_words.iter().any(|w| b_words.contains(w))
    }

    /// Calculate coherence score based on issues.
    fn calculate_coherence_score(
        plot_holes: &[PlotHole],
        timeline_issues: &[String],
        scenario: &Scenario,
    ) -> f64 {
        let base_score = 1.0;

        // Penalty for plot holes
        let plot_hole_penalty: f64 = plot_holes
            .iter()
            .map(|p| p.severity * 0.1)
            .sum::<f64>()
            .min(0.4);

        // Penalty for timeline issues
        let timeline_penalty = f64::from(u8::try_from(timeline_issues.len()).unwrap_or(0)) * 0.05;

        // Bonus for completeness
        let completeness_bonus = {
            let mut bonus = 0.0;
            if scenario.has_discovery() { bonus += 0.1; }
            if !scenario.edge_cases.is_empty() { bonus += 0.05; }
            if scenario.timeline.len() >= 3 { bonus += 0.05; }
            bonus
        };

        (base_score - plot_hole_penalty - timeline_penalty + completeness_bonus).clamp(0.0, 1.0)
    }

    /// Add a discovery mechanism to fix missing discovery plot hole.
    #[must_use]
    pub fn add_discovery(mut scenario: Scenario, mechanism: DiscoveryMechanism) -> Scenario {
        scenario.discovery_mechanisms.push(mechanism);
        scenario
    }

    /// Add an edge case with handling.
    #[must_use]
    pub fn add_edge_case(mut scenario: Scenario, edge_case: EdgeCase) -> Scenario {
        scenario.edge_cases.push(edge_case);
        scenario
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_character() -> Character {
        Character::new("Alice".to_string())
            .with_role("Analyst".to_string())
            .with_motivation("Save time on reporting".to_string())
            .with_capability("Can use Excel".to_string())
            .with_constraint("Limited budget".to_string())
            .with_initial_state("Manually creates reports".to_string())
    }

    fn create_test_scenario() -> Scenario {
        let character = create_test_character();

        let discovery = DiscoveryMechanism::new(
            "Web Search".to_string(),
            "User searches for reporting tools".to_string(),
        )
        .with_channel("Google".to_string())
        .with_trigger("Frustration with manual process".to_string())
        .with_probability(0.7);

        let event1 = TimelineEvent::new("e1".to_string(), "Searches for solution".to_string(), 1)
            .with_actor(character.name.clone())
            .with_type(EventType::Discovery)
            .with_expected_outcome("Finds product".to_string());

        let event2 = TimelineEvent::new("e2".to_string(), "Signs up".to_string(), 2)
            .with_actor(character.name.clone())
            .with_type(EventType::Action)
            .with_expected_outcome("Account created".to_string())
            .with_dependency("e1".to_string());

        let event3 = TimelineEvent::new("e3".to_string(), "Generates report".to_string(), 3)
            .with_actor(character.name.clone())
            .with_type(EventType::Success)
            .with_expected_outcome("Report generated in 5 minutes".to_string())
            .with_dependency("e2".to_string());

        Scenario::new("Monthly Reporting".to_string())
            .with_description("Alice generates monthly reports".to_string())
            .with_character(character.name)
            .with_discovery(discovery)
            .with_event(event1)
            .with_event(event2)
            .with_event(event3)
            .with_north_star("Generate any report in under 10 minutes".to_string())
    }

    #[test]
    fn test_character_creation() {
        let character = create_test_character();

        assert_eq!(character.name, "Alice");
        assert_eq!(character.role, "Analyst");
        assert!(!character.motivations.is_empty());
        assert!(character.is_valid());
    }

    #[test]
    fn test_character_invalid() {
        let character = Character::new("".to_string());
        assert!(!character.is_valid());

        let no_motivation = Character::new("Test".to_string());
        assert!(!no_motivation.is_valid());
    }

    #[test]
    fn test_timeline_event_creation() {
        let event = TimelineEvent::new("e1".to_string(), "Test event".to_string(), 1)
            .with_actor("Alice".to_string())
            .with_type(EventType::Action)
            .with_expected_outcome("Success".to_string());

        assert_eq!(event.id, "e1");
        assert_eq!(event.order, 1);
        assert_eq!(event.event_type, EventType::Action);
    }

    #[test]
    fn test_discovery_mechanism_creation() {
        let mechanism = DiscoveryMechanism::new("Test".to_string(), "Description".to_string())
            .with_channel("Web".to_string())
            .with_trigger("Need".to_string())
            .with_probability(0.8);

        assert!(mechanism.is_valid());
        assert!((mechanism.probability - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_discovery_mechanism_invalid() {
        let no_channels = DiscoveryMechanism::new("Test".to_string(), "Description".to_string());
        assert!(!no_channels.is_valid());
    }

    #[test]
    fn test_edge_case_creation() {
        let edge_case = EdgeCase::new("Network Error".to_string(), "User loses connection".to_string())
            .with_precondition("User is online".to_string())
            .with_expected_behavior("Show retry option".to_string())
            .with_status(HandlingStatus::Handled)
            .with_severity(0.6);

        assert_eq!(edge_case.name, "Network Error");
        assert!(!edge_case.needs_attention()); // Handled
    }

    #[test]
    fn test_edge_case_needs_attention() {
        let unhandled = EdgeCase::new("Error".to_string(), "Description".to_string())
            .with_status(HandlingStatus::Unhandled)
            .with_severity(0.8);

        assert!(unhandled.needs_attention());

        let low_severity = EdgeCase::new("Error".to_string(), "Description".to_string())
            .with_status(HandlingStatus::Unhandled)
            .with_severity(0.3);

        assert!(!low_severity.needs_attention());
    }

    #[test]
    fn test_plot_hole_creation() {
        let hole = PlotHole::new(
            PlotHoleType::MissingDiscovery,
            "No way to find product".to_string(),
            "Start".to_string(),
        )
        .with_severity(0.9)
        .with_fix("Add discovery mechanism".to_string());

        assert_eq!(hole.hole_type, PlotHoleType::MissingDiscovery);
        assert!(hole.suggested_fix.is_some());
    }

    #[test]
    fn test_scenario_creation() {
        let scenario = create_test_scenario();

        assert_eq!(scenario.name, "Monthly Reporting");
        assert!(!scenario.characters.is_empty());
        assert!(!scenario.timeline.is_empty());
        assert!(scenario.has_discovery());
    }

    #[test]
    fn test_scenario_ordered_events() {
        let scenario = Scenario::new("Test".to_string())
            .with_character("Alice".to_string())
            .with_event(TimelineEvent::new("e3".to_string(), "Third".to_string(), 3))
            .with_event(TimelineEvent::new("e1".to_string(), "First".to_string(), 1))
            .with_event(TimelineEvent::new("e2".to_string(), "Second".to_string(), 2))
            .with_north_star("Goal".to_string());

        let ordered = scenario.ordered_events();
        assert_eq!(ordered[0].order, 1);
        assert_eq!(ordered[1].order, 2);
        assert_eq!(ordered[2].order, 3);
    }

    #[test]
    fn test_builder_empty_scenario_name() {
        let scenario = Scenario::new("".to_string())
            .with_character("Alice".to_string())
            .with_event(TimelineEvent::new("e1".to_string(), "Test".to_string(), 1))
            .with_north_star("Goal".to_string());

        let result = NorthStarBuilder::build(scenario);
        assert!(result.is_err());
        assert!(matches!(result, Err(NorthStarError::EmptyScenarioName)));
    }

    #[test]
    fn test_builder_no_characters() {
        let scenario = Scenario::new("Test".to_string())
            .with_event(TimelineEvent::new("e1".to_string(), "Test".to_string(), 1))
            .with_north_star("Goal".to_string());

        let result = NorthStarBuilder::build(scenario);
        assert!(result.is_err());
        assert!(matches!(result, Err(NorthStarError::NoCharacters)));
    }

    #[test]
    fn test_builder_no_events() {
        let scenario = Scenario::new("Test".to_string())
            .with_character("Alice".to_string())
            .with_north_star("Goal".to_string());

        let result = NorthStarBuilder::build(scenario);
        assert!(result.is_err());
        assert!(matches!(result, Err(NorthStarError::NoEvents)));
    }

    #[test]
    fn test_builder_no_north_star() {
        let scenario = Scenario::new("Test".to_string())
            .with_character("Alice".to_string())
            .with_event(TimelineEvent::new("e1".to_string(), "Test".to_string(), 1));

        let result = NorthStarBuilder::build(scenario);
        assert!(result.is_err());
        assert!(matches!(result, Err(NorthStarError::NoNorthStar)));
    }

    #[test]
    fn test_builder_timeline_order_conflict() {
        let scenario = Scenario::new("Test".to_string())
            .with_character("Alice".to_string())
            .with_event(TimelineEvent::new("e1".to_string(), "First".to_string(), 1))
            .with_event(TimelineEvent::new("e2".to_string(), "Also first".to_string(), 1))
            .with_north_star("Goal".to_string());

        let result = NorthStarBuilder::build(scenario);
        assert!(result.is_err());
        assert!(matches!(result, Err(NorthStarError::TimelineOrderConflict)));
    }

    #[test]
    fn test_builder_success() {
        let scenario = create_test_scenario();
        let result = NorthStarBuilder::build(scenario);

        assert!(result.is_ok());
        let output = result.expect("Should succeed");

        assert!(output.simulation.is_coherent);
        assert!(output.simulation.coherence_score > 0.5);
        assert!(output.simulation.plot_holes.is_empty());
    }

    #[test]
    fn test_builder_detects_missing_discovery() {
        let scenario = Scenario::new("Test".to_string())
            .with_character("Alice".to_string())
            .with_event(
                TimelineEvent::new("e1".to_string(), "Uses product".to_string(), 1)
                    .with_type(EventType::Action),
            )
            .with_north_star("Goal".to_string());

        let result = NorthStarBuilder::build(scenario).expect("Should succeed");

        assert!(!result.simulation.plot_holes.is_empty());
        assert!(result
            .simulation
            .plot_holes
            .iter()
            .any(|p| p.hole_type == PlotHoleType::MissingDiscovery));
    }

    #[test]
    fn test_builder_detects_unhandled_edge_case() {
        let edge_case = EdgeCase::new("Error".to_string(), "System fails".to_string())
            .with_status(HandlingStatus::Unhandled)
            .with_severity(0.8);

        let scenario = Scenario::new("Test".to_string())
            .with_character("Alice".to_string())
            .with_event(
                TimelineEvent::new("e1".to_string(), "Discovery".to_string(), 1)
                    .with_type(EventType::Discovery),
            )
            .with_event(TimelineEvent::new("e2".to_string(), "Success".to_string(), 2))
            .with_edge_case(edge_case)
            .with_north_star("Goal".to_string());

        let result = NorthStarBuilder::build(scenario).expect("Should succeed");

        assert!(!result.simulation.unhandled_edge_cases.is_empty());
    }

    #[test]
    fn test_builder_detects_missing_dependency() {
        let scenario = Scenario::new("Test".to_string())
            .with_character("Alice".to_string())
            .with_event(
                TimelineEvent::new("e1".to_string(), "Discovery".to_string(), 1)
                    .with_type(EventType::Discovery),
            )
            .with_event(
                TimelineEvent::new("e2".to_string(), "Action".to_string(), 2)
                    .with_dependency("nonexistent".to_string()),
            )
            .with_north_star("Goal".to_string());

        let result = NorthStarBuilder::build(scenario).expect("Should succeed");

        assert!(result
            .simulation
            .plot_holes
            .iter()
            .any(|p| p.hole_type == PlotHoleType::MissingDependency));
    }

    #[test]
    fn test_stats_calculation() {
        let scenario = create_test_scenario();
        let output = NorthStarBuilder::build(scenario).expect("Should succeed");

        assert_eq!(output.stats.total_characters, 1);
        assert!(output.stats.total_events >= 3);
        assert!(output.stats.discovery_count >= 1);
    }

    #[test]
    fn test_coherence_score_calculation() {
        let good_scenario = create_test_scenario();
        let good_result = NorthStarBuilder::build(good_scenario).expect("Should succeed");

        let bad_scenario = Scenario::new("Test".to_string())
            .with_character("Alice".to_string())
            .with_event(TimelineEvent::new("e1".to_string(), "Test".to_string(), 1))
            .with_north_star("Goal".to_string());

        let bad_result = NorthStarBuilder::build(bad_scenario).expect("Should succeed");

        assert!(good_result.simulation.coherence_score > bad_result.simulation.coherence_score);
    }

    #[test]
    fn test_add_discovery() {
        let scenario = Scenario::new("Test".to_string())
            .with_character("Alice".to_string())
            .with_event(TimelineEvent::new("e1".to_string(), "Test".to_string(), 1))
            .with_north_star("Goal".to_string());

        let mechanism = DiscoveryMechanism::new("Search".to_string(), "Description".to_string())
            .with_channel("Google".to_string());

        let updated = NorthStarBuilder::add_discovery(scenario, mechanism);

        assert!(updated.has_discovery());
    }
}
