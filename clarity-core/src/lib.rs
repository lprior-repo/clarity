//! Core functionality for the Clarity application

pub mod db;
pub mod error;
pub mod json_formatter;
pub mod progress;
pub mod session;
pub mod validation;

pub use error::{ExitCode, ExitCodeError, map_db_error, map_validation_error};

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
