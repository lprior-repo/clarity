#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![allow(clippy::suspicious_else_formatting)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

/// Test mode toggle for discover phase
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiscoverMode {
  /// Express mode - user types freely
  Express,
  /// Guided mode - structured questions with AI suggestions
  Guided,
}

impl Default for DiscoverMode {
  fn default() -> Self {
    Self::Guided
  }
}
