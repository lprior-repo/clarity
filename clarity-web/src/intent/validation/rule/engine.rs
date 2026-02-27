use super::errors::RuleError;
use super::types::{Comparison, Rule, RuleResult};

pub fn apply_rule(rule: &Rule, value: &str) -> Result<RuleResult, RuleError> {
  match rule {
    Rule::Required => validate_required(value),
    Rule::Pattern { pattern } => validate_pattern(value, pattern),
    Rule::Range { min, max } => validate_range(value, *min, *max),
    Rule::Custom { name, check } => validate_custom(value, name, check),
  }
}

fn validate_required(value: &str) -> Result<RuleResult, RuleError> {
  Ok(if value.trim().is_empty() {
    RuleResult::failed(
      "required",
      "value is required but was empty",
      Some(value.to_string()),
    )
  } else {
    RuleResult::passed("required", Some(value.to_string()))
  })
}

fn validate_pattern(value: &str, pattern: &str) -> Result<RuleResult, RuleError> {
  let regex =
    regex::Regex::new(pattern).map_err(|e| RuleError::InvalidPattern(format!("{pattern}: {e}")))?;

  Ok(if regex.is_match(value) {
    RuleResult::passed("pattern", Some(value.to_string()))
  } else {
    RuleResult::failed(
      "pattern",
      format!("value '{value}' does not match pattern '{pattern}'"),
      Some(value.to_string()),
    )
  })
}

fn validate_range(value: &str, min: f64, max: f64) -> Result<RuleResult, RuleError> {
  let num: f64 = value
    .trim()
    .parse()
    .map_err(|_| RuleError::NotANumber(value.to_string()))?;

  Ok(if num >= min && num <= max {
    RuleResult::passed("range", Some(value.to_string()))
  } else {
    RuleResult::failed(
      "range",
      format!("value {num} is outside range [{min}, {max}]"),
      Some(value.to_string()),
    )
  })
}

fn validate_custom(value: &str, name: &str, check: &str) -> Result<RuleResult, RuleError> {
  let passed = evaluate_custom_check(value, check.trim())?;
  Ok(if passed {
    RuleResult::passed(name, Some(value.to_string()))
  } else {
    RuleResult::failed(
      name,
      format!("custom check failed: {check}"),
      Some(value.to_string()),
    )
  })
}

fn evaluate_custom_check(value: &str, check: &str) -> Result<bool, RuleError> {
  if let Some(rest) = check.strip_prefix("length ") {
    return evaluate_length_check(value.len(), rest.trim());
  }
  if let Some(prefix) = check.strip_prefix("starts_with ") {
    return Ok(value.starts_with(&extract_quoted_string(prefix.trim())));
  }
  if let Some(suffix) = check.strip_prefix("ends_with ") {
    return Ok(value.ends_with(&extract_quoted_string(suffix.trim())));
  }
  if let Some(substr) = check.strip_prefix("contains ") {
    return Ok(value.contains(&extract_quoted_string(substr.trim())));
  }
  if let Some(list) = check.strip_prefix("one_of ") {
    return evaluate_one_of(value, list.trim());
  }

  Err(RuleError::CustomFailed {
    name: "unknown".into(),
    message: format!("unknown check expression: {check}"),
  })
}

fn evaluate_length_check(len: usize, expr: &str) -> Result<bool, RuleError> {
  let (comparison, num_str) = if let Some(rest) = expr.strip_prefix(">=") {
    (Comparison::Gte, rest.trim())
  } else if let Some(rest) = expr.strip_prefix("<=") {
    (Comparison::Lte, rest.trim())
  } else if let Some(rest) = expr.strip_prefix('>') {
    (Comparison::Gt, rest.trim())
  } else if let Some(rest) = expr.strip_prefix('<') {
    (Comparison::Lt, rest.trim())
  } else if let Some(rest) = expr.strip_prefix("==") {
    (Comparison::Eq, rest.trim())
  } else if let Some(rest) = expr.strip_prefix("!=") {
    (Comparison::Ne, rest.trim())
  } else {
    return Err(RuleError::CustomFailed {
      name: "length".into(),
      message: format!("invalid comparison: {expr}"),
    });
  };

  let target: usize = num_str.parse().map_err(|_| RuleError::CustomFailed {
    name: "length".into(),
    message: format!("not a valid number: {num_str}"),
  })?;

  Ok(match comparison {
    Comparison::Gt => len > target,
    Comparison::Lt => len < target,
    Comparison::Gte => len >= target,
    Comparison::Lte => len <= target,
    Comparison::Eq => len == target,
    Comparison::Ne => len != target,
  })
}

fn extract_quoted_string(value: &str) -> String {
  let trimmed = value.trim();
  if ((trimmed.starts_with('"') && trimmed.ends_with('"'))
    || (trimmed.starts_with('\'') && trimmed.ends_with('\'')))
    && trimmed.len() >= 2
  {
    trimmed[1..trimmed.len() - 1].to_string()
  } else {
    trimmed.to_string()
  }
}

fn evaluate_one_of(value: &str, list: &str) -> Result<bool, RuleError> {
  let trimmed = list.trim();
  if !(trimmed.starts_with('[') && trimmed.ends_with(']')) {
    return Err(RuleError::CustomFailed {
      name: "one_of".into(),
      message: format!("expected array format: {trimmed}"),
    });
  }

  let inner = &trimmed[1..trimmed.len() - 1];
  let values: Vec<String> = inner
    .split(',')
    .map(extract_quoted_string)
    .filter(|s| !s.is_empty())
    .collect();
  Ok(values.iter().any(|candidate| candidate == value))
}

pub fn validate_with_rules(value: &str, rules: &[Rule]) -> Result<Vec<RuleResult>, RuleError> {
  rules.iter().map(|rule| apply_rule(rule, value)).collect()
}

pub fn all_rules_pass(value: &str, rules: &[Rule]) -> Result<bool, RuleError> {
  Ok(
    validate_with_rules(value, rules)?
      .iter()
      .all(|result| result.passed),
  )
}

pub fn failing_rules(value: &str, rules: &[Rule]) -> Result<Vec<RuleResult>, RuleError> {
  Ok(
    validate_with_rules(value, rules)?
      .into_iter()
      .filter(|result| !result.passed)
      .collect(),
  )
}
