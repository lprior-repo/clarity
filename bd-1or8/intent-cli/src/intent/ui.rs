//! CLI UI helpers for rich terminal output
//!
//! Provides colored headers, status messages, and formatted text.
//! Respects the NO_COLOR environment variable for accessibility.
//!
//! # Error Handling
//!
//! All output functions return `Result<(), UiError>` to handle I/O errors gracefully.
//! This ensures that broken pipes and other I/O issues don't cause panics.

use std::env;
use std::fmt;
use std::io::{self, Write};

/// Error type for UI operations
#[derive(Debug)]
pub enum UiError {
    /// I/O error when writing to stdout/stderr
    IoError(io::Error),
    /// Broken pipe - output was closed
    BrokenPipe,
}

impl fmt::Display for UiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IoError(e) => write!(f, "I/O error: {e}"),
            Self::BrokenPipe => write!(f, "Broken pipe"),
        }
    }
}

impl std::error::Error for UiError {}

impl From<io::Error> for UiError {
    fn from(err: io::Error) -> Self {
        if err.kind() == io::ErrorKind::BrokenPipe {
            Self::BrokenPipe
        } else {
            Self::IoError(err)
        }
    }
}

/// Check if color output should be disabled
fn no_color() -> bool {
    env::var("NO_COLOR").is_ok()
}

/// Write to stdout, handling errors gracefully
fn write_stdout(s: &str) -> Result<(), UiError> {
    let mut stdout = io::stdout().lock();
    stdout.write_all(s.as_bytes())?;
    stdout.flush()?;
    Ok(())
}

/// Write to stderr, handling errors gracefully
fn write_stderr(s: &str) -> Result<(), UiError> {
    let mut stderr = io::stderr().lock();
    stderr.write_all(s.as_bytes())?;
    stderr.flush()?;
    Ok(())
}

/// Print a bold, colored section header
///
/// Outputs a header with separator lines above and below.
/// Uses cyan color when terminal supports it.
pub fn print_header(title: &str) -> Result<(), UiError> {
    let separator = "═".repeat(67);
    if no_color() {
        write_stdout(&format!("\n{separator}\n{title}\n{separator}\n\n"))?;
    } else {
        // Using simple ANSI codes instead of owo-colors to avoid dependency
        let cyan = "\x1b[36m";
        let bold = "\x1b[1m";
        let reset = "\x1b[0m";
        write_stdout(&format!(
            "\n{bold}{cyan}{separator}{reset}\n{bold}{cyan}{title}{reset}\n{bold}{cyan}{separator}{reset}\n\n"
        ))?;
    }
    Ok(())
}

/// Print a success message with checkmark
///
/// Outputs a green checkmark followed by the message.
pub fn print_success(message: &str) -> Result<(), UiError> {
    if no_color() {
        write_stdout(&format!("✓ {message}\n"))?;
    } else {
        let green = "\x1b[32m";
        let reset = "\x1b[0m";
        write_stdout(&format!("{green}✓{reset} {message}\n"))?;
    }
    Ok(())
}

/// Print a warning message with warning symbol
///
/// Outputs a yellow warning symbol followed by the message.
pub fn print_warning(message: &str) -> Result<(), UiError> {
    if no_color() {
        write_stderr(&format!("⚠️  {message}\n"))?;
    } else {
        let yellow = "\x1b[33m";
        let reset = "\x1b[0m";
        write_stderr(&format!("{yellow}⚠️{reset}  {message}\n"))?;
    }
    Ok(())
}

/// Print an error message with X symbol
///
/// Outputs a red X followed by the message to stderr.
pub fn print_error(message: &str) -> Result<(), UiError> {
    if no_color() {
        write_stderr(&format!("✗ {message}\n"))?;
    } else {
        let red = "\x1b[31m";
        let reset = "\x1b[0m";
        write_stderr(&format!("{red}✗{reset} {message}\n"))?;
    }
    Ok(())
}

/// Print an info message with info symbol
///
/// Outputs a blue info symbol followed by the message.
pub fn print_info(message: &str) -> Result<(), UiError> {
    if no_color() {
        write_stdout(&format!("ℹ {message}\n"))?;
    } else {
        let blue = "\x1b[34m";
        let reset = "\x1b[0m";
        write_stdout(&format!("{blue}ℹ{reset} {message}\n"))?;
    }
    Ok(())
}

/// Print a bold label with value
///
/// Outputs "label: value" format with the label bolded.
pub fn print_labeled(label: &str, value: &str) -> Result<(), UiError> {
    if no_color() {
        write_stdout(&format!("{label}: {value}\n"))?;
    } else {
        let bold = "\x1b[1m";
        let reset = "\x1b[0m";
        write_stdout(&format!("{bold}{label}{reset}: {value}\n"))?;
    }
    Ok(())
}

/// Print a list item with bullet
///
/// Outputs an indented bullet point with the item text.
/// The indent parameter specifies the nesting level (0 = no indent).
pub fn print_list_item(item: &str, indent: usize) -> Result<(), UiError> {
    let padding = "  ".repeat(indent);
    write_stdout(&format!("{padding}• {item}\n"))?;
    Ok(())
}

/// Print a line of text with a color function
///
/// Applies the provided color function to the text and prints it.
pub fn print_colored<F>(text: &str, color_fn: F) -> Result<(), UiError>
where
    F: Fn(&str) -> String,
{
    write_stdout(&format!("{}\n", color_fn(text)))?;
    Ok(())
}

/// Format a number as a badge with color
///
/// Returns a formatted badge string like "[label: count]" with the
/// provided color function applied.
pub fn badge<F>(label: &str, count: usize, color_fn: F) -> String
where
    F: Fn(&str) -> String,
{
    let badge_text = format!("[{label}: {count}]");
    color_fn(&badge_text)
}

/// Create a cyan color function for use with badge()
pub fn cyan_color(text: &str) -> String {
    if no_color() {
        text.to_string()
    } else {
        format!("\x1b[36m{text}\x1b[0m")
    }
}

/// Create a green color function for use with badge()
pub fn green_color(text: &str) -> String {
    if no_color() {
        text.to_string()
    } else {
        format!("\x1b[32m{text}\x1b[0m")
    }
}

/// Create a yellow color function for use with badge()
pub fn yellow_color(text: &str) -> String {
    if no_color() {
        text.to_string()
    } else {
        format!("\x1b[33m{text}\x1b[0m")
    }
}

/// Create a red color function for use with badge()
pub fn red_color(text: &str) -> String {
    if no_color() {
        text.to_string()
    } else {
        format!("\x1b[31m{text}\x1b[0m")
    }
}

/// Print an error and exit with code 1
///
/// This function will print the error message and terminate the process.
/// It does not return.
pub fn fatal(message: &str) -> ! {
    // Ignore errors since we're exiting anyway
    let _ = print_error(message);
    std::process::exit(1)
}

/// Prompt for user input and return the response
///
/// Displays the message and waits for user input.
/// Returns the trimmed input string.
pub fn prompt(message: &str) -> io::Result<String> {
    print!("{message}: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

/// Prompt for yes/no confirmation
///
/// Displays the message with [y/N] suffix and returns true if user enters 'y' or 'yes'.
pub fn confirm(message: &str) -> io::Result<bool> {
    let response = prompt(&format!("{message} [y/N]"))?;
    Ok(response.eq_ignore_ascii_case("y") || response.eq_ignore_ascii_case("yes"))
}

/// Format text as bold cyan (for template names, etc.)
pub fn bold_cyan(text: &str) -> String {
    if no_color() {
        text.to_string()
    } else {
        format!("\x1b[1m\x1b[36m{text}\x1b[0m")
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
    fn test_badge_format_with_color_fn() {
        let result = badge("items", 5, cyan_color);
        assert!(result.contains("items"));
        assert!(result.contains('5'));
    }

    #[test]
    fn test_badge_format_with_custom_fn() {
        let result = badge("count", 42, |s| format!("CUSTOM:{s}"));
        assert!(result.contains("CUSTOM:"));
        assert!(result.contains("count"));
        assert!(result.contains("42"));
    }

    #[test]
    fn test_print_functions_dont_panic() {
        // These just verify the functions run without panicking
        // They return Result but we ignore errors in tests
        let _ = print_header("Test Header");
        let _ = print_success("Test success");
        let _ = print_info("Test info");
        let _ = print_labeled("Label", "Value");
        let _ = print_list_item("Test item", 0);
        let _ = print_list_item("Nested item", 2);
    }

    #[test]
    fn test_color_functions() {
        // Verify color functions produce output
        let cyan = cyan_color("test");
        let green = green_color("test");
        let yellow = yellow_color("test");
        let red = red_color("test");

        // All should contain "test"
        assert!(cyan.contains("test"));
        assert!(green.contains("test"));
        assert!(yellow.contains("test"));
        assert!(red.contains("test"));
    }

    #[test]
    fn test_bold_cyan() {
        let result = bold_cyan("text");
        assert!(result.contains("text"));
    }
}
