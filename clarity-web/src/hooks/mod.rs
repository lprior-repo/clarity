#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

mod progressive_discover;
mod quality_scoring;

pub use progressive_discover::{
  use_progressive_discover, use_progressive_discover_actions, use_progressive_discover_with_prompt,
  ProgressiveDiscoverActions, ProgressiveDiscoverState,
};
pub use quality_scoring::{use_cached_quality_score, use_quality_score};
