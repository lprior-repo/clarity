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
#[cfg(test)]
mod tests;

pub use context::Context;
pub use errors::InterpolationError;
pub use interpolate::{extract_capture, interpolate_headers, interpolate_string, json_to_string, validate_variables};
pub use placeholders::{extract_variables, has_placeholders};
pub use resolve::resolve_path;
