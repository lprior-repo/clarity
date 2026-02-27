#[derive(Debug, Clone, PartialEq)]
pub struct BehaviorPriority {
  pub path: String,
  pub dependent_count: usize,
  pub precondition_count: usize,
}

impl BehaviorPriority {
  #[must_use]
  pub fn score(&self) -> usize {
    self.dependent_count.saturating_mul(10) + (100 - self.precondition_count.min(100))
  }
}
