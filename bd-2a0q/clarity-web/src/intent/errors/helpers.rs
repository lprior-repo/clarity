use super::Suggestion;
use itertools::Itertools;

#[must_use]
pub fn levenshtein(a: &str, b: &str) -> usize {
  let a_chars: Vec<char> = a.chars().collect();
  let b_chars: Vec<char> = b.chars().collect();

  if a_chars.is_empty() {
    return b_chars.len();
  }
  if b_chars.is_empty() {
    return a_chars.len();
  }

  let (longer, shorter) = if a_chars.len() > b_chars.len() {
    (&a_chars, &b_chars)
  } else {
    (&b_chars, &a_chars)
  };

  let mut prev_row: Vec<usize> = (0..=shorter.len()).collect();
  let mut curr_row = vec![0; shorter.len() + 1];

  for (i, long_char) in longer.iter().enumerate() {
    curr_row[0] = i + 1;

    for (j, short_char) in shorter.iter().enumerate() {
      let cost = usize::from(long_char != short_char);
      curr_row[j + 1] = (prev_row[j + 1] + 1)
        .min(curr_row[j] + 1)
        .min(prev_row[j] + cost);
    }

    std::mem::swap(&mut prev_row, &mut curr_row);
  }

  prev_row[shorter.len()]
}

#[must_use]
pub fn suggest_field_names(target: &str, available: &[String]) -> Vec<Suggestion> {
  const MAX_SUGGESTIONS: usize = 3;
  const MAX_DISTANCE: usize = 2;

  available
    .iter()
    .filter_map(|field| {
      let distance = levenshtein(target, field);
      if distance <= MAX_DISTANCE {
        Some(Suggestion::new(field.clone(), distance))
      } else {
        None
      }
    })
    .sorted()
    .take(MAX_SUGGESTIONS)
    .collect()
}

#[must_use]
pub fn extract_available_fields(json: &serde_json::Value) -> Vec<String> {
  match json {
    serde_json::Value::Object(map) => map.keys().cloned().sorted().collect(),
    serde_json::Value::Array(values) => values
      .iter()
      .filter_map(|value| {
        if let serde_json::Value::Object(inner) = value {
          Some(inner.keys().cloned().collect::<Vec<_>>())
        } else {
          None
        }
      })
      .flatten()
      .unique()
      .sorted()
      .collect(),
    _ => Vec::new(),
  }
}
