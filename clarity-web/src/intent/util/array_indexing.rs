#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

mod errors;
mod navigation;
mod parsing;
mod spec;

pub use errors::ArrayIndexError;
pub use navigation::navigate_path;
pub use parsing::{parse_path_component, split_path};
pub use spec::ArraySpec;
