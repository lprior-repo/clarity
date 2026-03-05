/// Flag suggestion utilities for unknown flags
/// Provides "did you mean" suggestions using Levenshtein distance
///
/// ## Example
/// ```gleam
/// let suggestion = suggest_flag("--verbsoe", max_distance: 2)
/// // Returns: "--verbose"
/// ```
import gleam/list
import gleam/order
import gleam/result
import gleam/string

/// Default maximum edit distance for suggestions
/// Flags within this distance will be suggested as corrections
const default_max_distance = 2

/// All known boolean flags
const bool_flags = [
  "help", "json", "verbose", "quiet", "yes", "draft", "confirm", "dry-run",
  "execute", "no-config", "parallel", "continue-on-error",
]

/// All known value flags
const value_flags = [
  "only", "profile", "resume", "answer", "bead-id", "status", "reason",
  "session", "format", "notes", "strategy", "output", "out", "name", "target",
  "feature", "export-answers-template", "vision", "dir",
]

/// Get all known flag names (without -- prefix)
pub fn get_all_known_flags() -> List(String) {
  list.append(bool_flags, value_flags)
}

/// Calculate Levenshtein distance between two strings
/// Uses Wagner-Fisher recursive algorithm
///
/// The distance is the minimum number of single-character edits (insertions,
/// deletions, or substitutions) required to change one string into the other.
///
/// ## Performance
/// Time complexity: O(3^n) where n is the length of the shorter string
/// This is acceptable for short strings like flag names (typically 3-20 chars)
pub fn levenshtein(a: String, b: String) -> Int {
  levenshtein_list(string.to_graphemes(a), string.to_graphemes(b))
}

/// Internal implementation that works with grapheme lists
fn levenshtein_list(a: List(String), b: List(String)) -> Int {
  case a, b {
    [], _ -> list.length(b)
    _, [] -> list.length(a)
    [a_first, ..a_rest], [b_first, ..b_rest] -> {
      // Cost of substitution: 0 if characters match, 1 otherwise
      let cost = case a_first == b_first {
        True -> 0
        False -> 1
      }

      // Three possible operations: substitute, delete from a, delete from b
      let substitute_cost = levenshtein_list(a_rest, b_rest) + cost
      let delete_a_cost = levenshtein_list(a_rest, b) + 1
      let delete_b_cost = levenshtein_list(a, b_rest) + 1

      // Return minimum of the three
      substitute_cost
      |> min(delete_a_cost)
      |> min(delete_b_cost)
    }
  }
}

/// Find minimum of two integers
fn min(a: Int, b: Int) -> Int {
  case a < b {
    True -> a
    False -> b
  }
}

/// Find flags within edit distance threshold
pub fn find_similar_flags(
  input: String,
  max_distance max_distance: Int,
) -> List(String) {
  get_all_known_flags()
  |> list.filter(fn(flag) { levenshtein(input, flag) <= max_distance })
}

/// Find the closest matching flag for an unknown flag
/// Returns the suggestion with "--" prefix, or empty string if no good match
pub fn suggest_flag(
  unknown_flag: String,
  max_distance max_distance: Int,
) -> String {
  // Check if it starts with --
  case string.starts_with(unknown_flag, "--") {
    False -> ""
    True -> {
      let flag_name = string.drop_left(unknown_flag, 2)

      // First check if it's an exact match (no suggestion needed)
      case list.contains(get_all_known_flags(), flag_name) {
        True -> ""
        False -> {
          // Find similar flags
          let similar = find_similar_flags(flag_name, max_distance)

          case similar {
            [] -> ""
            _ -> {
              // Find the closest match by sorting by distance
              similar
              |> list.sort(fn(a, b) {
                let dist_a = levenshtein(flag_name, a)
                let dist_b = levenshtein(flag_name, b)
                case dist_a < dist_b {
                  True -> order.Lt
                  False ->
                    case dist_a == dist_b {
                      True -> order.Eq
                      False -> order.Gt
                    }
                }
              })
              |> list.first
              |> result.map(fn(s) { "--" <> s })
              |> result.unwrap("")
            }
          }
        }
      }
    }
  }
}

/// Format error message with suggestion
pub fn format_suggestion(unknown_flag: String, suggestion: String) -> String {
  case suggestion {
    "" -> "Unknown flag: " <> unknown_flag
    s -> "Unknown flag '" <> unknown_flag <> "'. Did you mean '" <> s <> "'?"
  }
}

/// Extract flag name from argument (removes -- prefix)
fn extract_flag_name(arg: String) -> String {
  case string.starts_with(arg, "--") {
    False -> ""
    True -> string.drop_left(arg, 2)
  }
}

/// Check if an argument is a flag
fn is_flag(arg: String) -> Bool {
  string.starts_with(arg, "--")
}

/// Validate all flags in the argument list
/// Returns Ok(Nil) if all flags are valid, Error(message) if unknown flag found
pub fn validate_flags(args: List(String)) -> Result(Nil, String) {
  // Find all flags in the args
  let flags =
    args
    |> list.filter(is_flag)
    |> list.map(extract_flag_name)

  // Find first unknown flag
  case
    flags
    |> list.find(fn(flag) {
      case flag {
        "" -> False
        _ -> !list.contains(get_all_known_flags(), flag)
      }
    })
  {
    Ok(unknown_flag) -> {
      // Found an unknown flag
      let suggestion = suggest_flag("--" <> unknown_flag, default_max_distance)
      Error(format_suggestion("--" <> unknown_flag, suggestion))
    }
    Error(_) -> {
      // No unknown flags found (all valid)
      Ok(Nil)
    }
  }
}
