#[must_use]
pub fn analyze_vague_rule(rule: &str) -> (String, String) {
  let lower = rule.to_lowercase();

  if ["fast", "quickly", "performant"]
    .iter()
    .any(|keyword| lower.contains(keyword))
  {
    (
            format!("Clarify performance requirement: '{rule}'"),
            "Replace vague performance terms with specific measurable values. Example: 'Response time must be under 200ms for 95% of requests' instead of 'fast response'.".to_string(),
        )
  } else if ["good", "appropriate", "proper"]
    .iter()
    .any(|keyword| lower.contains(keyword))
  {
    (
            format!("Clarify quality requirement: '{rule}'"),
            "Replace subjective terms with specific criteria. Example: 'Error rate must be below 0.1%' instead of 'good error handling'.".to_string(),
        )
  } else if ["some", "various", "multiple"]
    .iter()
    .any(|keyword| lower.contains(keyword))
  {
    (
            format!("Clarify scope: '{rule}'"),
            "Specify exact items or range. Example: 'Support 3-5 concurrent users' instead of 'support multiple users'.".to_string(),
        )
  } else if ["should", "may", "might"]
    .iter()
    .any(|keyword| lower.contains(keyword))
  {
    (
            format!("Clarify requirement strength: '{rule}'"),
            "Use 'must' for mandatory requirements or 'should' with explicit conditions. Avoid ambiguous modal verbs without context.".to_string(),
        )
  } else if ["etc", "and so on", "..."]
    .iter()
    .any(|keyword| lower.contains(keyword))
  {
    (
            format!("Complete the list: '{rule}'"),
            "Replace 'etc.' with a complete list of items. If the list is too long, provide a comprehensive reference or pattern.".to_string(),
        )
  } else {
    (
            format!("Add specificity to: '{rule}'"),
            "Provide concrete examples, specific values, or measurable criteria. Avoid ambiguous language and ensure the requirement can be objectively verified.".to_string(),
        )
  }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::float_cmp, clippy::needless_collect, clippy::unnecessary_debug_formatting, clippy::match_same_arms, clippy::option_if_let_else, clippy::suspicious_else_formatting, clippy::manual_let_else, clippy::match_wild_err_arm, clippy::match_like_matches_macro, clippy::needless_pass_by_value)]
mod tests {
  use super::analyze_vague_rule;

  #[test]
  fn performance_rule_is_classified() {
    let (description, action) = analyze_vague_rule("system should be fast");
    assert!(description.to_lowercase().contains("performance"));
    assert!(action.contains("200ms"));
  }
}
