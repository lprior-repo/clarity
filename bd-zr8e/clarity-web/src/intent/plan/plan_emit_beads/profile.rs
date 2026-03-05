use crate::intent::interview::types::Profile;
use crate::intent::plan::types::PlanBead;

#[must_use]
pub fn generate_profile_beads(profile: Profile, phase: u32) -> Vec<PlanBead> {
  match profile {
    Profile::Api => tagged_tasks(
      "api",
      phase,
      &[
        ("api-design", "Design API endpoints", 2),
        ("api-auth", "Implement authentication", 3),
        ("api-validation", "Add input validation", 2),
        ("api-error-handling", "Implement error handling", 2),
        ("api-docs", "Generate API documentation", 1),
      ],
    ),
    Profile::Cli => tagged_tasks(
      "cli",
      phase,
      &[
        ("cli-args", "Parse command-line arguments", 2),
        ("cli-help", "Implement help system", 1),
        ("cli-output", "Format output", 2),
        ("cli-errors", "Handle errors gracefully", 2),
      ],
    ),
    Profile::Event => tagged_tasks(
      "event",
      phase,
      &[
        ("event-schema", "Define event schemas", 2),
        ("event-producer", "Implement event producer", 3),
        ("event-consumer", "Implement event consumer", 3),
        ("event-error", "Handle event errors", 2),
      ],
    ),
    Profile::Data => tagged_tasks(
      "data",
      phase,
      &[
        ("data-model", "Design data model", 3),
        ("data-migration", "Create migrations", 2),
        ("data-access", "Implement data access layer", 3),
        ("data-validation", "Add data validation", 2),
      ],
    ),
    Profile::Workflow => tagged_tasks(
      "workflow",
      phase,
      &[
        ("workflow-design", "Design workflow", 3),
        ("workflow-steps", "Implement workflow steps", 3),
        ("workflow-error", "Handle workflow errors", 2),
        ("workflow-monitor", "Add workflow monitoring", 2),
      ],
    ),
    Profile::Ui => tagged_tasks(
      "ui",
      phase,
      &[
        ("ui-components", "Create UI components", 3),
        ("ui-state", "Implement state management", 3),
        ("ui-events", "Handle user events", 2),
        ("ui-styling", "Apply styling", 1),
      ],
    ),
  }
}

fn tagged_tasks(tag: &str, phase: u32, tasks: &[(&str, &str, u32)]) -> Vec<PlanBead> {
  tasks
    .iter()
    .filter_map(|(id, title, effort)| {
      PlanBead::new(format!("{tag}-{id}"), (*title).to_string(), phase)
        .map(|bead| {
          bead
            .with_description(format!("{} task: {}", tag.to_uppercase(), title))
            .with_effort(*effort)
            .with_tag(tag.to_string())
        })
        .ok()
    })
    .collect()
}
