#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

mod engine;
mod errors;
mod parser;
mod types;

#[cfg(test)]
mod parser_tests;

pub use engine::{all_rules_pass, apply_rule, failing_rules, validate_with_rules};
pub use errors::RuleError;
pub use parser::{parse, RuleExpr, RuleParseError};
pub use types::{Rule, RuleResult};
