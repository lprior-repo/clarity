#[must_use]
pub fn find_placeholders(input: &str) -> Vec<(usize, usize, String)> {
  let mut placeholders = Vec::new();
  let chars: Vec<char> = input.chars().collect();
  let mut index = 0;

  while index < chars.len() {
    if index + 1 < chars.len() && chars[index] == '$' && chars[index + 1] == '{' {
      let start = index;
      index += 2;
      let mut name = String::new();
      let mut depth = 1;

      while index < chars.len() && depth > 0 {
        match chars[index] {
          '{' => {
            depth += 1;
            name.push(chars[index]);
          }
          '}' => {
            depth -= 1;
            if depth > 0 {
              name.push(chars[index]);
            }
          }
          _ => name.push(chars[index]),
        }
        index += 1;
      }

      if depth == 0 {
        let name = name.trim().to_string();
        if !name.is_empty() {
          placeholders.push((start, index, name));
        }
      }
    } else {
      index += 1;
    }
  }

  placeholders
}

#[must_use]
pub fn has_placeholders(input: &str) -> bool {
  input.contains("${")
}

#[must_use]
pub fn extract_variables(input: &str) -> Vec<String> {
  find_placeholders(input)
    .into_iter()
    .map(|(_, _, name)| name)
    .collect()
}
