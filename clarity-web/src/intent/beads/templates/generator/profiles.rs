use crate::intent::beads::templates::domain::BeadTemplate;

macro_rules! bead_template {
  (
    $profile:expr,
    $priority:expr,
    $summary:expr,
    $issue_type:expr,
    $labels:expr,
    $acceptance:expr $(,)?
  ) => {{
    let (title, description) = $summary;
    let candidate = $labels
      .iter()
      .fold(
        BeadTemplate::new(
          title.to_string(),
          description.to_string(),
          $profile.to_string(),
          $priority,
        )
        .map(|bead| bead.with_issue_type($issue_type.to_string()))
        .ok(),
        |bead, label| bead.map(|item| item.with_label((*label).to_string())),
      )
      .map(|template| {
        $acceptance.iter().fold(template, |current, criterion| {
          current.with_acceptance_criterion((*criterion).to_string())
        })
      });

    candidate.unwrap_or_default()
  }};
}

pub(super) fn api_beads(profile: &str) -> Vec<BeadTemplate> {
  vec![
    bead_template!(
      profile,
      1,
      (
        "Implement API Authentication",
        "Set up authentication mechanism for API endpoints.",
      ),
      "feature",
      &["api", "auth", "security"],
      &["Authentication middleware validates tokens"],
    ),
    bead_template!(
      profile,
      2,
      (
        "Define API Endpoints",
        "Design and implement API endpoints based on the spec.",
      ),
      "feature",
      &["api", "endpoints"],
      &["All endpoints return correct HTTP status codes"],
    )
    .with_dependency("Implement API Authentication".to_string()),
    bead_template!(
      profile,
      2,
      (
        "Implement API Error Handling",
        "Create comprehensive API error handling.",
      ),
      "feature",
      &["api", "error-handling"],
      &["Error responses follow RFC 7807 format"],
    ),
  ]
}

pub(super) fn cli_beads(profile: &str) -> Vec<BeadTemplate> {
  vec![
    bead_template!(
      profile,
      1,
      (
        "Implement CLI Command Parsing",
        "Set up command-line argument parsing.",
      ),
      "feature",
      &["cli", "parsing"],
      &["Parses all documented flags and options"],
    ),
    bead_template!(
      profile,
      2,
      (
        "Define CLI Exit Codes",
        "Implement standardized CLI exit codes.",
      ),
      "feature",
      &["cli", "exit-codes"],
      &["Exit code 0 for success"],
    ),
    bead_template!(
      profile,
      2,
      (
        "Implement CLI Help System",
        "Create a comprehensive CLI help system.",
      ),
      "feature",
      &["cli", "help"],
      &["Help text includes usage examples"],
    ),
  ]
}

pub(super) fn event_beads(profile: &str) -> Vec<BeadTemplate> {
  vec![
    bead_template!(
      profile,
      1,
      (
        "Define Event Types",
        "Create type definitions for event types.",
      ),
      "feature",
      &["event", "schema"],
      &["All event types have unique identifiers"],
    ),
    bead_template!(
      profile,
      2,
      (
        "Define Event Payloads",
        "Design payload structures for event types.",
      ),
      "feature",
      &["event", "payload"],
      &["Payloads are serializable to JSON"],
    ),
    bead_template!(
      profile,
      2,
      (
        "Implement Event Triggers",
        "Set up event production triggers.",
      ),
      "feature",
      &["event", "trigger"],
      &["Events are produced on defined triggers"],
    ),
  ]
}

pub(super) fn data_beads(profile: &str) -> Vec<BeadTemplate> {
  vec![
    bead_template!(
      profile,
      1,
      (
        "Design Data Model",
        "Create the data model with constraints.",
      ),
      "feature",
      &["data", "model"],
      &["All entities have primary keys"],
    ),
    bead_template!(
      profile,
      2,
      (
        "Implement Data Queries",
        "Create query functions for access patterns.",
      ),
      "feature",
      &["data", "queries"],
      &["Queries support pagination"],
    ),
    bead_template!(
      profile,
      3,
      (
        "Implement Data Retention Policy",
        "Set up data retention and archival policies.",
      ),
      "feature",
      &["data", "retention"],
      &["Old data is archived per policy"],
    ),
  ]
}

pub(super) fn workflow_beads(profile: &str) -> Vec<BeadTemplate> {
  vec![
    bead_template!(
      profile,
      1,
      (
        "Define Workflow Steps",
        "Define workflow steps with dependencies.",
      ),
      "feature",
      &["workflow", "steps"],
      &["Each step has defined inputs and outputs"],
    ),
    bead_template!(
      profile,
      2,
      (
        "Implement Workflow Transitions",
        "Create workflow transition logic.",
      ),
      "feature",
      &["workflow", "transitions"],
      &["Invalid transitions are rejected"],
    ),
    bead_template!(
      profile,
      2,
      (
        "Implement Workflow Error Recovery",
        "Add workflow error recovery mechanisms.",
      ),
      "feature",
      &["workflow", "error-recovery"],
      &["Failed workflows can be retried"],
    ),
  ]
}

pub(super) fn ui_beads(profile: &str) -> Vec<BeadTemplate> {
  vec![
    bead_template!(
      profile,
      1,
      ("Define User Flows", "Map user flows and outcomes."),
      "feature",
      &["ui", "flows"],
      &["All user flows are documented"],
    ),
    bead_template!(
      profile,
      2,
      (
        "Define UI States",
        "Define loading, error, and success states.",
      ),
      "feature",
      &["ui", "state"],
      &["Loading states show progress indicators"],
    ),
    bead_template!(
      profile,
      2,
      ("Build UI Components", "Create reusable UI components."),
      "feature",
      &["ui", "components"],
      &["Components follow design system"],
    ),
  ]
}
