// Answer loader module - loads pre-filled answers from files
import gleam/bool
import gleam/dict.{type Dict}
import gleam/dynamic
import gleam/int
import gleam/json
import gleam/list
import gleam/result
import gleam/string
import shellout
import simplifile

pub type AnswerLoaderError {
  FileNotFound(path: String)
  PermissionDenied(path: String)
  ParseError(path: String, message: String)
  SchemaError(message: String)
  IoError(message: String)
}

/// Load answers from a file (JSON format)
pub fn load_from_file(
  path: String,
) -> Result(Dict(String, String), AnswerLoaderError) {
  case simplifile.read(path) {
    Error(_) -> Error(FileNotFound(path))
    Ok(contents) -> parse_answers(path, contents)
  }
}

fn parse_answers(
  path: String,
  contents: String,
) -> Result(Dict(String, String), AnswerLoaderError) {
  case path_is_cue(path) {
    True -> {
      case shellout.command("cue", ["export", path, "-e", "answers"], ".", []) {
        Ok(json_str) -> parse_answers_json(path, json_str)
        Error(#(_, stderr)) -> {
          case parse_answers_json(path, contents) {
            Ok(parsed) -> Ok(parsed)
            Error(_) -> Error(ParseError(path, stderr))
          }
        }
      }
    }
    False -> parse_answers_json(path, contents)
  }
}

fn parse_answers_json(
  path: String,
  json_str: String,
) -> Result(Dict(String, String), AnswerLoaderError) {
  case json.decode(json_str, dynamic.dynamic) {
    Error(_) -> Error(ParseError(path, "Failed to decode answers JSON"))
    Ok(data) -> {
      case dynamic.dict(dynamic.string, dynamic.dynamic)(data) {
        Error(_) ->
          Error(ParseError(path, "Top-level answers must be an object/map"))
        Ok(entries) -> Ok(flatten_answers(entries))
      }
    }
  }
}

fn flatten_answers(
  entries: Dict(String, dynamic.Dynamic),
) -> Dict(String, String) {
  entries
  |> dict.to_list
  |> list.fold(dict.new(), fn(acc, entry) {
    let #(key, value) = entry
    flatten_dynamic(key, value, acc)
  })
}

fn flatten_dynamic(
  key_path: String,
  value: dynamic.Dynamic,
  acc: Dict(String, String),
) -> Dict(String, String) {
  case dynamic.dict(dynamic.string, dynamic.dynamic)(value) {
    Ok(nested) -> {
      // First, insert the JSON representation of the nested object (without short key)
      let json_repr = dynamic_value_to_string(value)
      let with_json = dict.insert(acc, key_path, json_repr)

      // Then recursively flatten the nested object
      nested
      |> dict.to_list
      |> list.fold(with_json, fn(inner_acc, entry) {
        let #(nested_key, nested_value) = entry
        flatten_dynamic(key_path <> "." <> nested_key, nested_value, inner_acc)
      })
    }
    Error(_) -> {
      // For leaf values, insert full path AND short key (last segment)
      let value_as_text = dynamic_value_to_string(value)
      let with_path = dict.insert(acc, key_path, value_as_text)

      // Also insert short key if it doesn't exist
      case last_key_segment(key_path) {
        "" -> with_path
        short_key -> {
          case dict.get(with_path, short_key) {
            Ok(_) -> with_path
            Error(_) -> dict.insert(with_path, short_key, value_as_text)
          }
        }
      }
    }
  }
}

fn last_key_segment(key_path: String) -> String {
  key_path
  |> string.split(".")
  |> list.reverse
  |> list.first
  |> result.unwrap("")
}

fn dynamic_value_to_string(value: dynamic.Dynamic) -> String {
  case dynamic.classify(value) {
    "String" | "BitArray" -> dynamic.string(value) |> result.unwrap("")
    "Int" ->
      dynamic.int(value) |> result.map(int.to_string) |> result.unwrap("")
    "Bool" ->
      dynamic.bool(value) |> result.map(bool.to_string) |> result.unwrap("")
    "Float" | "List" | "Tuple" | "Dict" | "Map" | "Nil" | _ ->
      json.to_string(dynamic_to_json(value))
  }
}

fn dynamic_to_json(value: dynamic.Dynamic) -> json.Json {
  case dynamic.classify(value) {
    "Nil" -> json.null()
    "Bool" ->
      dynamic.bool(value) |> result.map(json.bool) |> result.unwrap(json.null())
    "Int" ->
      dynamic.int(value) |> result.map(json.int) |> result.unwrap(json.null())
    "Float" ->
      dynamic.float(value)
      |> result.map(json.float)
      |> result.unwrap(json.null())
    "String" | "BitArray" ->
      dynamic.string(value)
      |> result.map(json.string)
      |> result.unwrap(json.null())
    "List" | "Tuple" -> {
      case dynamic.list(dynamic.dynamic)(value) {
        Ok(items) -> json.array(items, dynamic_to_json)
        Error(_) -> json.null()
      }
    }
    "Dict" | "Map" -> {
      case dynamic.dict(dynamic.string, dynamic.dynamic)(value) {
        Ok(entries) -> {
          entries
          |> dict.to_list
          |> list.map(fn(entry) {
            let #(key, item) = entry
            #(key, dynamic_to_json(item))
          })
          |> json.object
        }
        Error(_) -> json.null()
      }
    }
    _ -> json.null()
  }
}

fn path_is_cue(path: String) -> Bool {
  string.ends_with(path, ".cue")
}

// Test-only helper function with enhanced error reporting
pub fn parse_answers_json_for_test(
  path: String,
  json_str: String,
) -> Result(Dict(String, String), ParseErrorWithDetails) {
  case json.decode(json_str, dynamic.dynamic) {
    Error(_) -> {
      Error(ParseErrorWithDetails(
        path: path,
        decode_error: DecodeErrorDetails(
          path: "<root>",
          expected: "JSON",
          actual: "invalid",
          message: "Failed to decode JSON",
        ),
      ))
    }
    Ok(data) -> {
      case dynamic.dict(dynamic.string, dynamic.dynamic)(data) {
        Error(_) -> {
          Error(ParseErrorWithDetails(
            path: path,
            decode_error: DecodeErrorDetails(
              path: "<root>",
              expected: "Object",
              actual: dynamic_to_type_name(data),
              message: "Root value must be an object",
            ),
          ))
        }
        Ok(entries) -> Ok(flatten_answers(entries))
      }
    }
  }
}

// Helper to get type name from dynamic value
fn dynamic_to_type_name(value: dynamic.Dynamic) -> String {
  case dynamic.bool(value) {
    Ok(_) -> "Bool"
    Error(_) -> {
      case dynamic.int(value) {
        Ok(_) -> "Int"
        Error(_) -> {
          case dynamic.float(value) {
            Ok(_) -> "Float"
            Error(_) -> {
              case dynamic.string(value) {
                Ok(_) -> "String"
                Error(_) -> {
                  case dynamic.list(dynamic.dynamic)(value) {
                    Ok(_) -> "List"
                    Error(_) -> {
                      case
                        dynamic.dict(dynamic.string, dynamic.dynamic)(value)
                      {
                        Ok(_) -> "Dict"
                        Error(_) -> "Unknown"
                      }
                    }
                  }
                }
              }
            }
          }
        }
      }
    }
  }
}

// Test-only types
pub type ParseErrorWithDetails {
  ParseErrorWithDetails(path: String, decode_error: DecodeErrorDetails)
}

pub type DecodeErrorDetails {
  DecodeErrorDetails(
    path: String,
    expected: String,
    actual: String,
    message: String,
  )
}
