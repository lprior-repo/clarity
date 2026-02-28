use crate::intent::interview::types::{Answer, InterviewSession, Profile};

use super::domain::{BeadError, BeadTemplate};
mod profiles;

pub fn generate_beads_from_session(
  session: &InterviewSession,
) -> Result<Vec<BeadTemplate>, BeadError> {
  let profile_type = session.profile.as_str().to_string();
  let answer_beads = session
    .answers
    .iter()
    .map(|answer| create_bead_from_answer(answer, &profile_type))
    .collect::<Result<Vec<_>, _>>()?;
  let profile_beads = generate_profile_beads(session)?;
  Ok(answer_beads.into_iter().chain(profile_beads).collect())
}

pub fn generate_profile_beads(session: &InterviewSession) -> Result<Vec<BeadTemplate>, BeadError> {
  let profile_type = session.profile.as_str().to_string();
  let core = match session.profile {
    Profile::Api => profiles::api_beads(&profile_type),
    Profile::Cli => profiles::cli_beads(&profile_type),
    Profile::Event => profiles::event_beads(&profile_type),
    Profile::Data => profiles::data_beads(&profile_type),
    Profile::Workflow => profiles::workflow_beads(&profile_type),
    Profile::Ui => profiles::ui_beads(&profile_type),
  };
  let config = config_beads(&profile_type, session, exclusion_keys(session.profile));
  Ok(core.into_iter().chain(config).collect())
}

fn create_bead_from_answer(answer: &Answer, profile_type: &str) -> Result<BeadTemplate, BeadError> {
  let title = format!("Implement: {}", answer.question_text);
  let description = format!(
    "Based on answer: {}\n\nResponse: {}",
    answer.question_text, answer.response
  );
  let acceptance_criteria = answer
    .extracted
    .iter()
    .map(|(key, value)| format!("{}: {}", key.replace('_', " "), value));
  let base = BeadTemplate::new(title, description, profile_type.to_string(), 3)?
    .with_issue_type(determine_issue_type(&answer.response))
    .with_label(format!("round-{}", answer.round))
    .with_label(format!("perspective-{:?}", answer.perspective).to_lowercase());
  let with_criteria = acceptance_criteria.fold(base, |bead, criterion| {
    bead.with_acceptance_criterion(criterion)
  });
  Ok(if answer.confidence < 0.7 {
    with_criteria.with_ai_hints("Low confidence answer - may need clarification".to_string())
  } else {
    with_criteria
  })
}

fn determine_issue_type(response: &str) -> String {
  let lower = response.to_lowercase();
  if ["fix", "bug", "error", "issue"]
    .iter()
    .any(|item| lower.contains(item))
  {
    "bug".to_string()
  } else if ["investigate", "research", "spike"]
    .iter()
    .any(|item| lower.contains(item))
  {
    "spike".to_string()
  } else if ["feature", "new", "add"]
    .iter()
    .any(|item| lower.contains(item))
  {
    "feature".to_string()
  } else {
    "task".to_string()
  }
}

const fn exclusion_keys(profile: Profile) -> &'static [&'static str] {
  match profile {
    Profile::Api => &["base_url", "auth_method"],
    Profile::Cli => &["command_name", "exit_codes"],
    Profile::Event | Profile::Data | Profile::Workflow | Profile::Ui => &[],
  }
}

fn config_beads(profile: &str, session: &InterviewSession, excluded: &[&str]) -> Vec<BeadTemplate> {
  session
    .answers
    .iter()
    .flat_map(|answer| answer.extracted.iter())
    .filter(|(key, _)| !excluded.iter().any(|excluded_key| excluded_key == key))
    .map(|(key, value)| {
      let title = format!(
        "Configure {} {}",
        profile.to_uppercase(),
        key.replace('_', " ")
      );
      let description = format!(
        "Set up {} for {}: {}",
        key.replace('_', " "),
        profile,
        value
      );
      BeadTemplate::new(title, description, profile.to_string(), 3).map(|bead| {
        bead
          .with_issue_type("task".to_string())
          .with_label(profile.to_string())
          .with_label("config".to_string())
      })
    })
    .filter_map(Result::ok)
    .collect()
}
