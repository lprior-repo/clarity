use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanPhase {
  pub number: u32,
  #[serde(default)]
  pub name: String,
  #[serde(default)]
  pub description: String,
  #[serde(default)]
  pub beads: Vec<String>,
  #[serde(default)]
  pub complete: bool,
}

impl Default for PlanPhase {
  fn default() -> Self {
    Self {
      number: 1,
      name: String::new(),
      description: String::new(),
      beads: Vec::new(),
      complete: false,
    }
  }
}

impl PlanPhase {
  #[must_use]
  pub fn new(number: u32, name: String) -> Self {
    Self {
      number,
      name,
      ..Self::default()
    }
  }

  pub fn add_bead(&mut self, bead_id: String) {
    if !self.beads.contains(&bead_id) {
      self.beads.push(bead_id);
    }
  }
}
