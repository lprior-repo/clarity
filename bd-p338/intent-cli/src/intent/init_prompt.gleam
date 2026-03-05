/// Interactive prompts for spec initialization
import gleam/io
import gleam/list
import gleam/string
import gleam/int

/// Read a line from stdin
@external(erlang, "io", "get_line")
fn read_line(prompt: String) -> String

/// Prompt user for spec name
pub fn prompt_spec_name() -> Result(String, String) {
  io.println("")
  io.println("Let's create a new Intent spec!")
  io.println("")
  io.print("Spec name (e.g., 'User Management API'): ")

  let input = read_line("")
  let trimmed = string.trim(input)

  case string.length(trimmed) {
    0 -> {
      io.println_error("Error: Spec name cannot be empty")
      Error("Spec name cannot be empty")
    }
    _ -> Ok(trimmed)
  }
}

/// Prompt user to select a template
pub fn prompt_template(
  templates: List(spec_templates.Template),
) -> Result(spec_templates.TemplateType, String) {
  io.println("")
  io.println("Available templates:")
  io.println("")

  // Display templates with numbers
  let _display =
    templates
    |> list.index_map(fn(template, index) {
      let num = index + 1
      io.println(
        "  "
        <> int.to_string(num)
        <> ". "
        <> ansi.bold(ansi.cyan(template.name))
        <> " - "
        <> template.description,
      )
      num
    })

  io.println("")
  io.print("Select a template (1-" <> int.to_string(list.length(templates)) <> "): ")

  let input = read_line("")
  let trimmed = string.trim(input)

  case int.parse(trimmed) {
    Ok(selection) -> {
      let index = selection - 1

      case index >= 0 && index < list.length(templates) {
        True -> {
          let template = list.drop(templates, index)
          case template {
            [first, ..] -> Ok(first.type_)
            _ -> Error("Invalid template selection")
          }
        }
        False -> {
          io.println_error(
            "Error: Invalid selection. Please enter a number between 1 and "
            <> int.to_string(list.length(templates)),
          )
          Error("Invalid template selection")
        }
      }
    }
    Error(_) -> {
      io.println_error("Error: Please enter a valid number")
      Error("Invalid number format")
    }
  }
}

/// Prompt for output filename
pub fn prompt_output_filename(default_name: String) -> Result(String, String) {
  io.println("")
  io.print("Output filename (default: " <> default_name <> "): ")

  let input = read_line("")
  let trimmed = string.trim(input)

  case string.length(trimmed) {
    0 -> Ok(default_name)
    _ -> {
      // Add .cue extension if not present
      let filename =
        case string.ends_with(trimmed, ".cue") {
          True -> trimmed
          False -> trimmed <> ".cue"
        }

      Ok(filename)
    }
  }
}

/// Import spec_templates module
import intent/spec_templates

/// Import ANSI for formatting
import gleam_community/ansi
