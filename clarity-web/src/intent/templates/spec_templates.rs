#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![forbid(unsafe_code)]

mod engine;
mod errors;
mod profile_templates;

pub use engine::{fill_template, generate_spec_template};
pub use errors::SpecTemplateError;
