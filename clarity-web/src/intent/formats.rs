#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

mod email;
mod errors;
mod helpers;
mod iso8601;
mod uri;
mod uuid;

pub use email::validate_email;
pub use errors::{EmailError, FormatError, Iso8601Error, UriError, UuidError};
pub use helpers::{get_days_in_month, is_leap_year, is_valid_hex};
pub use iso8601::validate_iso8601;
pub use uri::validate_uri;
pub use uuid::validate_uuid;
