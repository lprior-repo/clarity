/// Array indexing support for JSON path navigation
/// Enables validation of specific array elements: items[0], items[-1], items[*]
import gleam/dict
import gleam/dynamic
import gleam/int
import gleam/json.{type Json}
import gleam/list
import gleam/option.{type Option, None, Some}
import gleam/result
import gleam/string
import intent/parser

/// Parse a path component to extract field name and optional array index
/// Examples:
///   "items" -> #("items", None)
///   "items[0]" -> #("items", Some(Index(0)))
///   "items[-1]" -> #("items", Some(LastN(1)))
///   "items[*]" -> #("items", Some(All))
pub type ArraySpec {
  NoArray
  Index(Int)
  LastN(Int)
  All
}

pub fn parse_path_component(
  component: String,
) -> Result(#(String, ArraySpec), String) {
  case string.contains(component, "[") {
    False -> Ok(#(component, NoArray))
    True -> {
      case string.split_once(component, "[") {
        Error(_) -> Error("Invalid array syntax: " <> component)
        Ok(#(field_name, rest)) -> {
          case string.split_once(rest, "]") {
            Error(_) -> Error("Missing closing ] in array index: " <> component)
            Ok(#(index_str, "")) -> {
              case index_str {
                "*" -> Ok(#(field_name, All))
                "-1" -> Ok(#(field_name, LastN(1)))
                _ -> {
                  // Check if it's a negative index
                  case string.starts_with(index_str, "-") {
                    True -> {
                      case
                        parse_index(string.slice(
                          index_str,
                          1,
                          string.length(index_str),
                        ))
                      {
                        Error(e) -> Error(e)
                        Ok(n) -> Ok(#(field_name, LastN(n)))
                      }
                    }
                    False -> {
                      case parse_index(index_str) {
                        Error(e) -> Error(e)
                        Ok(n) -> Ok(#(field_name, Index(n)))
                      }
                    }
                  }
                }
              }
            }
            Ok(#(_, _)) ->
              Error("Invalid array syntax: only one ] expected: " <> component)
          }
        }
      }
    }
  }
}

/// Parse a string to integer for array index
fn parse_index(s: String) -> Result(Int, String) {
  case int.parse(s) {
    Ok(n) ->
      case n >= 0 {
        True -> Ok(n)
        False -> Error("Array index must be non-negative: " <> s)
      }
    Error(_) -> Error("Array index must be a number: " <> s)
  }
}

/// Navigate a JSON path with array indexing support
/// Examples:
///   navigate(json, ["user", "emails[0]"]) -> email at first index
///   navigate(json, ["items[*]"]) -> all items in array
pub fn navigate_path(
  json: Json,
  path_components: List(String),
) -> Result(Json, String) {
  case path_components {
    [] -> Ok(json)
    [first, ..rest] -> {
      case parse_path_component(first) {
        Error(e) -> Error(e)
        Ok(#(field_name, array_spec)) -> {
          // Navigate to the field first
          case navigate_field(json, field_name) {
            Error(e) -> Error(e)
            Ok(None) -> Error("Field '" <> field_name <> "' not found")
            Ok(Some(next_json)) -> {
              // Apply array indexing if present
              case array_spec {
                NoArray -> navigate_path(next_json, rest)
                Index(idx) -> {
                  case get_array_element(next_json, idx) {
                    Error(e) -> Error(e)
                    Ok(elem) -> navigate_path(elem, rest)
                  }
                }
                LastN(n) -> {
                  case get_array_element_last(next_json, n) {
                    Error(e) -> Error(e)
                    Ok(elem) -> navigate_path(elem, rest)
                  }
                }
                All -> {
                  // For All, we need special handling - return all elements
                  // This returns a special marker for collection validation
                  Error("Array wildcard [*] requires special handling in rules")
                }
              }
            }
          }
        }
      }
    }
  }
}

/// Navigate to a field in a JSON object
fn navigate_field(json: Json, field: String) -> Result(Option(Json), String) {
  let json_str = json.to_string(json)
  case json.decode(json_str, dynamic.dict(dynamic.string, dynamic.dynamic)) {
    Ok(obj_dict) -> {
      case dict.get(obj_dict, field) {
        Ok(dyn) -> {
          // Convert dynamic back to Json
          case dynamic_to_json(dyn) {
            Ok(j) -> Ok(Some(j))
            Error(_) -> Error("Cannot convert field value to JSON")
          }
        }
        Error(_) -> Ok(None)
      }
    }
    Error(_) ->
      Error("Cannot navigate field '" <> field <> "' in non-object JSON")
  }
}

/// Get array element by positive index
fn get_array_element(json: Json, index: Int) -> Result(Json, String) {
  let json_str = json.to_string(json)
  case json.decode(json_str, dynamic.list(dynamic.dynamic)) {
    Ok(lst) -> {
      // Get element by finding it in the list
      let maybe_elem =
        lst
        |> list.drop(index)
        |> list.first

      case maybe_elem {
        Ok(elem) -> {
          case dynamic_to_json(elem) {
            Ok(j) -> Ok(j)
            Error(_) ->
              Error(
                "Cannot convert array element to JSON at index "
                <> int.to_string(index),
              )
          }
        }
        Error(_) ->
          Error(
            "Array index "
            <> int.to_string(index)
            <> " out of bounds (length: "
            <> int.to_string(list.length(lst))
            <> ")",
          )
      }
    }
    Error(_) ->
      Error("Cannot index non-array JSON with [" <> int.to_string(index) <> "]")
  }
}

/// Get array element counting from the end (negative index)
fn get_array_element_last(json: Json, from_end: Int) -> Result(Json, String) {
  let json_str = json.to_string(json)
  case json.decode(json_str, dynamic.list(dynamic.dynamic)) {
    Ok(lst) -> {
      let length = list.length(lst)
      let actual_index = length - from_end
      case actual_index >= 0 && actual_index < length {
        False ->
          Error(
            "Array index -"
            <> int.to_string(from_end)
            <> " out of bounds (length: "
            <> int.to_string(length)
            <> ")",
          )
        True -> {
          // Get element at actual_index using drop and first
          let maybe_elem =
            lst
            |> list.drop(actual_index)
            |> list.first

          case maybe_elem {
            Ok(elem) -> {
              case dynamic_to_json(elem) {
                Ok(j) -> Ok(j)
                Error(_) ->
                  Error(
                    "Cannot convert array element to JSON at index -"
                    <> int.to_string(from_end),
                  )
              }
            }
            Error(_) -> Error("Failed to access array element")
          }
        }
      }
    }
    Error(_) -> Error("Cannot index non-array JSON with negative index")
  }
}

/// Get all elements from an array (used for "array where each" validation)
pub fn get_all_array_elements(json: Json) -> Result(List(Json), String) {
  let json_str = json.to_string(json)
  case json.decode(json_str, dynamic.list(dynamic.dynamic)) {
    Ok(lst) -> {
      // Convert all elements to JSON using parser module
      let elements =
        lst
        |> list.map(parser.dynamic_to_json)
      Ok(elements)
    }
    Error(_) -> Error("Cannot get array elements from non-array JSON")
  }
}

/// Convert dynamic to Json (helper function)
/// This uses a parser function from the intent module
fn dynamic_to_json(dyn: dynamic.Dynamic) -> Result(Json, String) {
  // Use the parser module's dynamic_to_json function
  // This safely converts a dynamic value to JSON
  let json_val = parser.dynamic_to_json(dyn)
  Ok(json_val)
}

/// Split a path string into components
/// Handles nested paths like "user.profile.emails[0].address"
pub fn split_path(path: String) -> List(String) {
  // Split by dots, but preserve array indices
  string.split(path, ".")
  |> list.map(string.trim)
  |> list.filter(fn(s) { !string.is_empty(s) })
}

/// Validate that a path string is well-formed
pub fn validate_path(path: String) -> Result(Nil, String) {
  case string.is_empty(path) {
    True -> Error("Path cannot be empty")
    False -> {
      let components = split_path(path)
      result.all(
        components
        |> list.map(fn(component) {
          case parse_path_component(component) {
            Ok(_) -> Ok(Nil)
            Error(e) -> Error(e)
          }
        }),
      )
      |> result.map(fn(_) { Nil })
    }
  }
}
