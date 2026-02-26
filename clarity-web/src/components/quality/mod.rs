#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

mod quality_score_bar;

pub use quality_score_bar::{QualityScoreBar, QualityScoreBarProps, MINIMUM_GATE};
