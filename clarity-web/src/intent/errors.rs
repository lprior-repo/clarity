#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

mod contextual;
mod formatting;
mod helpers;
mod intent;
mod validation;

pub use contextual::ContextualError;
pub use formatting::format_error;
pub use helpers::{extract_available_fields, levenshtein, suggest_field_names};
pub use intent::IntentError;
pub use validation::{FieldFailure, Suggestion, ValidationError};
