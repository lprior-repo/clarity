use super::{ArrayIndexError, ArraySpec};

#[must_use]
pub fn split_path(path: &str) -> Vec<String> {
  let trimmed = path.trim();
  if trimmed.is_empty() {
    return Vec::new();
  }

  let mut result = Vec::new();
  let mut current = String::new();
  let mut in_brackets = false;

  for ch in trimmed.chars() {
    match ch {
      '[' => {
        in_brackets = true;
        current.push(ch);
      }
      ']' => {
        in_brackets = false;
        current.push(ch);
      }
      '.' if !in_brackets => {
        if !current.is_empty() {
          result.push(current.clone());
          current.clear();
        }
      }
      _ => current.push(ch),
    }
  }

  if !current.is_empty() {
    result.push(current);
  }

  result
}

pub fn parse_path_component(component: &str) -> Result<(String, ArraySpec), ArrayIndexError> {
  let trimmed = component.trim();
  if trimmed.is_empty() {
    return Err(ArrayIndexError::InvalidPath("empty component".into()));
  }

  match trimmed.find('[') {
    None => {
      if trimmed.contains(']') {
        return Err(ArrayIndexError::InvalidPath(format!(
          "unmatched closing bracket in: {component}"
        )));
      }
      Ok((trimmed.to_string(), ArraySpec::NoArray))
    }
    Some(open_pos) => {
      let close_pos = trimmed
        .find(']')
        .ok_or_else(|| ArrayIndexError::InvalidPath(format!("unclosed bracket in: {component}")))?;

      if close_pos <= open_pos {
        return Err(ArrayIndexError::InvalidPath(format!(
          "invalid bracket order in: {component}"
        )));
      }
      if close_pos != trimmed.len() - 1 {
        return Err(ArrayIndexError::InvalidPath(format!(
          "trailing content after bracket in: {component}"
        )));
      }

      let field_name = trimmed[..open_pos].to_string();
      if field_name.is_empty() {
        return Err(ArrayIndexError::InvalidPath(format!(
          "missing field name in: {component}"
        )));
      }

      let index_content = &trimmed[open_pos + 1..close_pos];
      let spec = parse_index_spec(index_content, component)?;
      Ok((field_name, spec))
    }
  }
}

fn parse_index_spec(content: &str, original: &str) -> Result<ArraySpec, ArrayIndexError> {
  let trimmed = content.trim();
  if trimmed.is_empty() {
    return Err(ArrayIndexError::InvalidPath(format!(
      "empty brackets in: {original}"
    )));
  }
  if trimmed == "*" {
    return Ok(ArraySpec::All);
  }

  if let Some(rest) = trimmed.strip_prefix('-') {
    let n: usize = rest.trim().parse().map_err(|_| {
      ArrayIndexError::InvalidPath(format!("invalid negative index in: {original}"))
    })?;
    if n == 0 {
      return Err(ArrayIndexError::InvalidPath(format!(
        "negative zero not allowed in: {original}"
      )));
    }
    return Ok(ArraySpec::NegativeIndex(n));
  }

  let index: usize = trimmed
    .parse()
    .map_err(|_| ArrayIndexError::InvalidPath(format!("invalid index in: {original}")))?;
  Ok(ArraySpec::Index(index))
}
