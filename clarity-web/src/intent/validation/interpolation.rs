#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

mod context;
mod errors;
mod interpolate;
mod placeholders;
mod resolve;

pub use context::Context;
pub use errors::InterpolationError;
pub use interpolate::{interpolate_string, validate_variables};
pub use placeholders::{extract_variables, has_placeholders};
pub use resolve::resolve_path;
