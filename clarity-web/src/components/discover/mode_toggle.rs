#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
#![warn(clippy::pedantic)]
#![allow(clippy::suspicious_else_formatting)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

/// Test mode toggle for discover phase
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiscoverMode {
  /// Express mode - user types freely
  Express,
  /// Guided mode - structured questions with AI suggestions
  #[default]
  Guided,
}
