#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![forbid(unsafe_code)]

mod bead;
mod error;
mod execution_plan;
mod phase;

pub use bead::PlanBead;
pub use error::PlanError;
pub use execution_plan::ExecutionPlan;
pub use phase::PlanPhase;
