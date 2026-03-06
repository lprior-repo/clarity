#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
#![warn(clippy::pedantic)]
#![allow(clippy::suspicious_else_formatting)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
// Public API exports - used by library consumers
#![allow(unused_imports)]

mod quality_score_bar;

pub use quality_score_bar::{QualityScoreBar, QualityScoreBarProps, MINIMUM_GATE};
