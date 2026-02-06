//! Core functionality for the Clarity application

pub mod db;
pub mod error;
pub mod json_formatter;
pub mod progress;
pub mod session;
pub mod types;
pub mod validation;

pub use error::{map_db_error, map_validation_error, ExitCode, ExitCodeError};
pub use types::{HttpMethod, HttpMethodError, SpecName, SpecNameError, Url, UrlError};

/// A simple function to demonstrate core functionality
pub fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_greet() {
        assert_eq!(greet("World"), "Hello, World!");
    }
}
