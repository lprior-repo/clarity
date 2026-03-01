//! CLI UI helpers using owo-colors for rich terminal output
//!
//! Provides colored headers, status messages, and formatted text.
//! Respects the NO_COLOR environment variable for accessibility.

use owo_colors::OwoColorize;
use std::env;
use std::io::{self, Write};

/// Check if color output should be disabled
fn no_color() -> bool {
  env::var("NO_COLOR").is_ok() || !atty::is(atty::Stream::Stdout)
}

/// Print a bold, colored section header
pub fn print_header(title: &str) {
  println!();
  if no_color() {
    println!("{}", "=".repeat(67));
    println!("{title}");
    println!("{}", "=".repeat(67));
  } else {
    let separator = "=".repeat(67);
    println!("{}", separator.bold().cyan());
    println!("{}", title.bold().cyan());
    println!("{}", separator.bold().cyan());
  }
  println!();
}

/// Print a success message with checkmark
pub fn print_success(message: &str) {
  if no_color() {
    println!("[OK] {message}");
  } else {
    println!("{} {}", "OK".green().bold(), message);
  }
}

/// Print a warning message with warning symbol
pub fn print_warning(message: &str) {
  if no_color() {
    eprintln!("[WARN] {message}");
  } else {
    eprintln!("{} {}", "WARN".yellow().bold(), message);
  }
}

/// Print an error message with X symbol
pub fn print_error(message: &str) {
  if no_color() {
    eprintln!("[ERROR] {message}");
  } else {
    eprintln!("{} {}", "ERROR".red().bold(), message);
  }
}

/// Print an info message with info symbol
pub fn print_info(message: &str) {
  if no_color() {
    println!("[INFO] {message}");
  } else {
    println!("{} {}", "INFO".blue().bold(), message);
  }
}

/// Print a bold label with value
pub fn print_labeled(label: &str, value: &str) {
  if no_color() {
    println!("{label}: {value}");
  } else {
    println!("{}: {}", label.bold(), value);
  }
}

/// Print a list item with bullet
pub fn print_list_item(item: &str, indent: usize) {
  let padding = "  ".repeat(indent);
  println!("{padding}- {item}");
}

/// Print a line of text with a color function
pub fn print_colored<F>(text: &str, color_fn: F)
where
  F: Fn(&str) -> String,
{
  println!("{}", color_fn(text));
}

/// Format a number as a badge with color
pub fn badge(label: &str, count: usize) -> String {
  if no_color() {
    format!("[{label}: {count}]")
  } else {
    format!("[{label}: {count}]").cyan().to_string()
  }
}

/// Print an error and exit with code 1
pub fn fatal(message: &str) -> ! {
  print_error(message);
  std::process::exit(1)
}

/// Prompt for user input and return the response
pub fn prompt(message: &str) -> io::Result<String> {
  print!("{message}: ");
  io::stdout().flush()?;

  let mut input = String::new();
  io::stdin().read_line(&mut input)?;
  Ok(input.trim().to_string())
}

/// Prompt for yes/no confirmation
pub fn confirm(message: &str) -> io::Result<bool> {
  let response = prompt(&format!("{message} [y/N]"))?;
  Ok(response.eq_ignore_ascii_case("y") || response.eq_ignore_ascii_case("yes"))
}

/// Format text as bold cyan (for template names, etc.)
pub fn bold_cyan(text: &str) -> String {
  if no_color() {
    text.to_string()
  } else {
    text.bold().cyan().to_string()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_no_color_detection() {
    // This test just verifies the function runs without panic
    let _ = no_color();
  }

  #[test]
  fn test_badge_format() {
    let result = badge("items", 5);
    assert!(result.contains("items"));
    assert!(result.contains('5'));
  }

  #[test]
  fn test_print_functions_dont_panic() {
    // These just verify the functions run without panicking
    print_header("Test Header");
    print_success("Test success");
    print_info("Test info");
    print_labeled("Label", "Value");
    print_list_item("Test item", 0);
    print_list_item("Nested item", 2);
  }
}
