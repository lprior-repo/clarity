//! Standard Input Handling
//!
//! Module for reading user input from standard input.
//! Provides functions for interactive command-line prompts.
//!
//! Ported from intent-cli/src/intent/stdin.gleam

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use std::io::{self, BufRead, Write};
use thiserror::Error;

/// Errors during stdin reading
#[derive(Debug, Clone, Error)]
pub enum StdinError {
  #[error("failed to read input: {0}")]
  ReadError(String),

  #[error("input cannot be empty")]
  EmptyInput,

  #[error("no input provided")]
  NoInput,

  #[error("please enter 'y' or 'n'")]
  InvalidYesNo,
}

/// Read a line from stdin
///
/// # Errors
/// Returns `StdinError` if reading fails.
pub fn read_line() -> Result<String, StdinError> {
  let stdin = io::stdin();
  let mut line = String::new();
  stdin
    .lock()
    .read_line(&mut line)
    .map_err(|e| StdinError::ReadError(e.to_string()))?;
  Ok(line)
}

/// Read a trimmed line from stdin
///
/// # Errors
/// Returns `StdinError` if reading fails.
pub fn read_line_trimmed() -> Result<String, StdinError> {
  let line = read_line()?;
  Ok(line.trim().to_string())
}

/// Read a single line from stdin, validating it's not empty
///
/// # Errors
/// Returns `StdinError` if input is empty or whitespace-only.
pub fn read_non_empty_line() -> Result<String, StdinError> {
  let line = read_line_trimmed()?;
  if line.is_empty() {
    Err(StdinError::EmptyInput)
  } else {
    Ok(line)
  }
}

/// Read multiple lines until user enters a blank line
/// Useful for collecting multi-line responses
///
/// # Errors
/// Returns `StdinError` if reading fails or no input is provided.
pub fn read_until_blank() -> Result<String, StdinError> {
  read_until_blank_helper(Vec::new(), 0)
}

fn read_until_blank_helper(lines: Vec<String>, line_count: usize) -> Result<String, StdinError> {
  match read_line_trimmed() {
    Err(_) => {
      if line_count == 0 {
        Err(StdinError::NoInput)
      } else {
        Ok(lines.into_iter().rev().collect::<Vec<_>>().join("\n"))
      }
    }
    Ok(line) => {
      if line.is_empty() {
        // User entered blank line - stop collecting
        if line_count == 0 {
          Err(StdinError::NoInput)
        } else {
          Ok(lines.into_iter().rev().collect::<Vec<_>>().join("\n"))
        }
      } else {
        // Continue collecting
        let mut new_lines = lines;
        new_lines.push(line);
        read_until_blank_helper(new_lines, line_count + 1)
      }
    }
  }
}

/// Format and display a prompt, then read a response
///
/// # Errors
/// Returns `StdinError` if reading fails or input is empty.
pub fn prompt_for_answer(prompt_text: &str) -> Result<String, StdinError> {
  print!("{prompt_text}");
  io::stdout()
    .flush()
    .map_err(|e| StdinError::ReadError(e.to_string()))?;
  read_non_empty_line()
}

/// Display a yes/no prompt and read response
/// Returns true if user enters 'y' or 'yes' (case-insensitive)
///
/// # Errors
/// Returns `StdinError` if reading fails or input is not 'y'/'n'.
pub fn prompt_yes_no(prompt_text: &str) -> Result<bool, StdinError> {
  let response = prompt_for_answer(&format!("{prompt_text} (y/n): "))?;
  let lower = response.to_lowercase();

  match lower.as_str() {
    "y" | "yes" => Ok(true),
    "n" | "no" => Ok(false),
    _ => Err(StdinError::InvalidYesNo),
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  // Note: Most stdin functions cannot be easily unit tested without mocking stdin.
  // These tests focus on the pure logic portions.

  #[test]
  fn test_stdin_error_display() {
    let err = StdinError::EmptyInput;
    assert!(err.to_string().contains("empty"));

    let err = StdinError::NoInput;
    assert!(err.to_string().contains("no input"));

    let err = StdinError::InvalidYesNo;
    assert!(err.to_string().contains('y'));
  }

  // Integration tests would require feeding input to stdin
  // which is beyond the scope of unit tests
}
