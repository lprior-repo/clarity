use itertools::Itertools;
use std::collections::HashMap;

use crate::intent::interview::types::{InterviewSession, Profile};

use super::errors::SpecTemplateError;
use super::profile_templates::template_for_profile;

const PLACEHOLDER_START: &str = "{{";
const PLACEHOLDER_END: &str = "}}";
const TEMPLATE_NAME: &str = "spec_template";

pub fn generate_spec_template(profile: Profile) -> Result<String, SpecTemplateError> {
  let template = template_for_profile(profile);
  if template.trim().is_empty() {
    Err(SpecTemplateError::empty_template(format!("{profile:?}")))
  } else {
    Ok(template)
  }
}

pub fn fill_template(
  template: &str,
  session: &InterviewSession,
) -> Result<String, SpecTemplateError> {
  if template.trim().is_empty() {
    return Err(SpecTemplateError::empty_template(TEMPLATE_NAME));
  }

  if session.answers.is_empty() {
    return Err(SpecTemplateError::no_answers(&session.id));
  }

  let fields = extract_field_values(session);
  let rendered = fields
    .iter()
    .fold(template.to_string(), |current, (field, value)| {
      let placeholder = format!("{PLACEHOLDER_START}{field}{PLACEHOLDER_END}");
      current.replace(&placeholder, value)
    });

  let unresolved = extract_placeholders(&rendered);
  if unresolved.is_empty() {
    Ok(rendered)
  } else {
    Err(SpecTemplateError::unresolved_placeholders(TEMPLATE_NAME, unresolved))
  }
}

fn extract_field_values(session: &InterviewSession) -> HashMap<String, String> {
  let metadata = vec![
    ("session_id".to_string(), session.id.clone()),
    ("profile".to_string(), session.profile.as_str().to_string()),
    ("created_at".to_string(), session.created_at.clone()),
    ("updated_at".to_string(), session.updated_at.clone()),
    ("raw_notes".to_string(), session.raw_notes.clone()),
  ];

  let answer_pairs = session.answers.iter().flat_map(|answer| {
    let question_key = answer.question_id.replace(' ', "_").to_lowercase();
    let base = std::iter::once((question_key.clone(), answer.response.clone()));
    let extracted = answer
      .extracted
      .iter()
      .map(|(key, value)| (key.clone(), value.clone()));
    let notes = if answer.notes.is_empty() {
      None
    } else {
      Some((format!("{question_key}_notes"), answer.notes.clone()))
    };

    base.chain(extracted).chain(notes)
  });

  metadata.into_iter().chain(answer_pairs).collect()
}

fn extract_placeholders(template: &str) -> Vec<String> {
  template
    .split(PLACEHOLDER_START)
    .skip(1)
    .filter_map(|segment| {
      segment
        .split_once(PLACEHOLDER_END)
        .map(|(name, _)| name.trim().to_string())
    })
    .filter(|name| !name.is_empty())
    .unique()
    .collect()
}
