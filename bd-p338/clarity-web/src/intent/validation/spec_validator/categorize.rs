use super::SpecValidator;
use crate::intent::types::Spec;
use std::collections::HashMap;

impl SpecValidator {
  #[must_use]
  pub fn sort_by_category(&self, spec: &Spec) -> HashMap<String, Vec<String>> {
    let mut categories: HashMap<String, Vec<String>> = HashMap::new();

    for feature in &spec.features {
      for behavior in &feature.behaviors {
        categories
          .entry(infer_category(&behavior.name))
          .or_default()
          .push(format!("{}.{}", feature.name, behavior.name));
      }
    }

    for values in categories.values_mut() {
      values.sort();
    }

    categories
  }
}

fn infer_category(name: &str) -> String {
  name
    .split('_')
    .next()
    .map_or_else(|| "other".to_string(), ToString::to_string)
}
