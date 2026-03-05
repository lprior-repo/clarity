//! CLI UI helpers using owo-colors for rich terminal output.
//!
//! Provides colored headers, status messages, and formatted text for the Intent CLI.
//! This module is a port of the Gleam cli_ui module.

use owo_colors::OwoColorize;
use std::io::{self, Write};

/// Print a bold, colored section header.
///
/// Displays a title surrounded by decorative separator lines in bold cyan.
///
/// # Errors
///
/// Returns an error if writing to stdout fails.
pub fn print_header(title: &str) -> io::Result<()> {
    let separator = "═══════════════════════════════════════════════════════════════════";
    println!();
    println!("{}", separator.bold().cyan());
    println!("{}", title.bold().cyan());
    println!("{}", separator.bold().cyan());
    println!();
    Ok(())
}

/// Print a success message with a checkmark.
///
/// # Errors
///
/// Returns an error if writing to stdout fails.
pub fn print_success(message: &str) -> io::Result<()> {
    println!("{} {}", "\u{2713}".green(), message.green());
    Ok(())
}

/// Print a warning message with a warning symbol.
///
/// # Errors
///
/// Returns an error if writing to stdout fails.
pub fn print_warning(message: &str) -> io::Result<()> {
    println!("{}  {}", "\u{26a0}\u{fe0f}".yellow(), message.yellow());
    Ok(())
}

/// Print an error message with an X symbol to stderr.
///
/// # Errors
///
/// Returns an error if writing to stderr fails.
pub fn print_error(message: &str) -> io::Result<()> {
    writeln!(io::stderr(), "{} {}", "\u{2717}".red(), message.red())
}

/// Print an info message with an info symbol.
///
/// # Errors
///
/// Returns an error if writing to stdout fails.
pub fn print_info(message: &str) -> io::Result<()> {
    println!("{} {}", "\u{2139}".blue(), message.blue());
    Ok(())
}

/// Print a bold label with a value.
///
/// # Errors
///
/// Returns an error if writing to stdout fails.
pub fn print_labeled(label: &str, value: &str) -> io::Result<()> {
    println!("{}: {}", label.bold(), value);
    Ok(())
}

/// Print a list item with a bullet point.
///
/// # Arguments
///
/// * `item` - The text of the list item
/// * `indent` - The indentation level (each level adds 2 spaces)
///
/// # Errors
///
/// Returns an error if writing to stdout fails.
pub fn print_list_item(item: &str, indent: usize) -> io::Result<()> {
    let padding = "  ".repeat(indent);
    println!("{}{} {}", padding, "\u{2022}", item);
    Ok(())
}

/// Color specification for terminal output.
#[derive(Debug, Clone, Copy)]
pub enum Color {
    /// Cyan color
    Cyan,
    /// Green color
    Green,
    /// Yellow color
    Yellow,
    /// Red color
    Red,
    /// Blue color
    Blue,
    /// Magenta color
    Magenta,
    /// White color
    White,
}

/// Apply a color to text and return the colored string.
///
/// # Errors
///
/// Returns an error if formatting fails (should not happen with standard colors).
pub fn colorize(text: &str, color: Color) -> String {
    match color {
        Color::Cyan => format!("{}", text.cyan()),
        Color::Green => format!("{}", text.green()),
        Color::Yellow => format!("{}", text.yellow()),
        Color::Red => format!("{}", text.red()),
        Color::Blue => format!("{}", text.blue()),
        Color::Magenta => format!("{}", text.magenta()),
        Color::White => format!("{}", text.white()),
    }
}

/// Print text with a specified color.
///
/// # Errors
///
/// Returns an error if writing to stdout fails.
pub fn print_colored(text: &str, color: Color) -> io::Result<()> {
    println!("{}", colorize(text, color));
    Ok(())
}

/// Format a number as a badge with color.
///
/// Returns a string like "[label: count]" in the specified color.
pub fn badge(label: &str, count: usize, color: Color) -> String {
    colorize(&format!("[{}: {}]", label, count), color)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_colorize_cyan() {
        let result = colorize("test", Color::Cyan);
        assert!(result.contains("test"));
    }

    #[test]
    fn test_colorize_green() {
        let result = colorize("test", Color::Green);
        assert!(result.contains("test"));
    }

    #[test]
    fn test_colorize_yellow() {
        let result = colorize("test", Color::Yellow);
        assert!(result.contains("test"));
    }

    #[test]
    fn test_colorize_red() {
        let result = colorize("test", Color::Red);
        assert!(result.contains("test"));
    }

    #[test]
    fn test_colorize_blue() {
        let result = colorize("test", Color::Blue);
        assert!(result.contains("test"));
    }

    #[test]
    fn test_colorize_magenta() {
        let result = colorize("test", Color::Magenta);
        assert!(result.contains("test"));
    }

    #[test]
    fn test_colorize_white() {
        let result = colorize("test", Color::White);
        assert!(result.contains("test"));
    }

    #[test]
    fn test_badge() {
        let result = badge("items", 42, Color::Green);
        assert!(result.contains("items"));
        assert!(result.contains("42"));
    }

    #[test]
    fn test_print_functions_dont_panic() {
        // These should not panic - they return Result
        let _ = print_header("Test Header");
        let _ = print_success("Test success");
        let _ = print_warning("Test warning");
        let _ = print_error("Test error");
        let _ = print_info("Test info");
        let _ = print_labeled("Label", "value");
        let _ = print_list_item("Item", 0);
        let _ = print_list_item("Indented item", 2);
        let _ = print_colored("Colored text", Color::Cyan);
    }
}
