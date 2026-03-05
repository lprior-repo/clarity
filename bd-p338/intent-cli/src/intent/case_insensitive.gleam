/// Case-insensitive string operations
/// Optimized helpers that avoid repeated lowercase conversions
import gleam/list
import gleam/string

/// Check if haystack contains needle (case-insensitive)
///
/// Both strings are lowercased once before comparison.
///
/// ## Example
/// ```gleam
/// contains_ignore_case("Hello World", "world")  // True
/// contains_ignore_case("Hello World", "WORLD")  // True
/// contains_ignore_case("Hello World", "xyz")    // False
/// ```
pub fn contains_ignore_case(haystack: String, needle: String) -> Bool {
  string.contains(string.lowercase(haystack), string.lowercase(needle))
}

/// Check if haystack contains any of the needles (case-insensitive)
///
/// The haystack is lowercased once, then checked against all needles.
/// This is more efficient than calling contains_ignore_case multiple times.
///
/// ## Example
/// ```gleam
/// contains_any_ignore_case("User API", ["api", "endpoint"])  // True
/// contains_any_ignore_case("CLI Command", ["api", "endpoint"])  // False
/// ```
pub fn contains_any_ignore_case(haystack: String, needles: List(String)) -> Bool {
  let lower_haystack = string.lowercase(haystack)
  list.any(needles, fn(needle) {
    string.contains(lower_haystack, string.lowercase(needle))
  })
}

/// Check if haystack contains all of the needles (case-insensitive)
///
/// The haystack is lowercased once, then checked against all needles.
///
/// ## Example
/// ```gleam
/// contains_all_ignore_case("User API Endpoint", ["api", "endpoint"])  // True
/// contains_all_ignore_case("User API", ["api", "endpoint"])  // False
/// ```
pub fn contains_all_ignore_case(haystack: String, needles: List(String)) -> Bool {
  let lower_haystack = string.lowercase(haystack)
  list.all(needles, fn(needle) {
    string.contains(lower_haystack, string.lowercase(needle))
  })
}

/// Check if two strings are equal (case-insensitive)
///
/// ## Example
/// ```gleam
/// equals_ignore_case("Hello", "hello")  // True
/// equals_ignore_case("Hello", "HELLO")  // True
/// equals_ignore_case("Hello", "World")  // False
/// ```
pub fn equals_ignore_case(a: String, b: String) -> Bool {
  string.lowercase(a) == string.lowercase(b)
}

/// Check if haystack starts with prefix (case-insensitive)
///
/// ## Example
/// ```gleam
/// starts_with_ignore_case("HelloWorld", "hello")  // True
/// starts_with_ignore_case("HelloWorld", "world")  // False
/// ```
pub fn starts_with_ignore_case(haystack: String, prefix: String) -> Bool {
  string.starts_with(string.lowercase(haystack), string.lowercase(prefix))
}

/// Check if haystack ends with suffix (case-insensitive)
///
/// ## Example
/// ```gleam
/// ends_with_ignore_case("HelloWorld", "world")  // True
/// ends_with_ignore_case("HelloWorld", "hello")  // False
/// ```
pub fn ends_with_ignore_case(haystack: String, suffix: String) -> Bool {
  string.ends_with(string.lowercase(haystack), string.lowercase(suffix))
}
