/// CLI UI helpers using gleam_community_ansi for rich terminal output
/// Provides colored headers, status messages, and formatted text
import gleam/io
import gleam_community/ansi

/// Print a bold, colored section header
pub fn print_header(title: String) {
  io.println("")
  io.println(
    ansi.bold(ansi.cyan(
      "═══════════════════════════════════════════════════════════════════",
    )),
  )
  io.println(ansi.bold(ansi.cyan(title)))
  io.println(
    ansi.bold(ansi.cyan(
      "═══════════════════════════════════════════════════════════════════",
    )),
  )
  io.println("")
}

/// Print a success message with checkmark
pub fn print_success(message: String) {
  io.println(ansi.green("✓ " <> message))
}

/// Print a warning message with warning symbol
pub fn print_warning(message: String) {
  io.println(ansi.yellow("⚠️  " <> message))
}

/// Print an error message with X symbol
pub fn print_error(message: String) {
  io.println_error(ansi.red("✗ " <> message))
}

/// Print an info message with info symbol
pub fn print_info(message: String) {
  io.println(ansi.blue("ℹ " <> message))
}

/// Print a bold label with value
pub fn print_labeled(label: String, value: String) {
  io.println(ansi.bold(label) <> ": " <> value)
}

/// Print a list item with bullet
pub fn print_list_item(item: String, indent: Int) {
  let padding = case indent {
    0 -> ""
    n -> string.repeat(" ", n * 2)
  }
  io.println(padding <> "• " <> item)
}

/// Print a line of text with color
pub fn print_colored(color_fn: fn(String) -> String, text: String) {
  io.println(color_fn(text))
}

/// Format a number as a badge with color
pub fn badge(
  label: String,
  count: Int,
  color_fn: fn(String) -> String,
) -> String {
  color_fn("[" <> label <> ": " <> string.inspect(count) <> "]")
}

// Required imports
import gleam/string
