#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![forbid(unsafe_code)]

mod types_bead;
mod types_error;
mod types_execution_plan;
mod types_phase;

pub use types_bead::PlanBead;
pub use types_error::PlanError;
pub use types_execution_plan::ExecutionPlan;
pub use types_phase::PlanPhase;
