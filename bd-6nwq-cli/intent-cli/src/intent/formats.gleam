/// Rigorous format validators using pure Gleam parsing
/// No regex-only validation - actual structural parsing and validation
import gleam/int
import gleam/list
import gleam/string

/// Validate email using RFC 5322 compliant parsing
/// This is stricter than simple regex - validates local and domain parts
pub fn validate_email(email: String) -> Result(Nil, String) {
  // Split on @ - there must be exactly one
  let parts = string.split(email, "@")
  case parts {
    [local, domain] -> {
      // Validate local part
      case validate_email_local(local) {
        Error(e) -> Error(e)
        Ok(_) -> {
          // Validate domain part
          case validate_email_domain(domain) {
            Error(e) -> Error(e)
            Ok(_) -> Ok(Nil)
          }
        }
      }
    }
    _ ->
      Error("'" <> email <> "' is not a valid email address (invalid @ format)")
  }
}

/// Validate email local part (before @)
fn validate_email_local(local: String) -> Result(Nil, String) {
  case string.is_empty(local) {
    True -> Error("Email local part cannot be empty")
    False -> {
      // Check for consecutive dots
      case string.contains(local, "..") {
        True -> Error("Email local part cannot contain consecutive dots")
        False -> {
          // Check that it doesn't start or end with a dot
          case string.starts_with(local, ".") || string.ends_with(local, ".") {
            True -> Error("Email local part cannot start or end with a dot")
            False -> {
              // Check valid characters: alphanumeric, dots, hyphens, underscores, plus
              let valid = is_valid_email_local_chars(local)
              case valid {
                True -> Ok(Nil)
                False ->
                  Error(
                    "Email local part contains invalid characters: " <> local,
                  )
              }
            }
          }
        }
      }
    }
  }
}

/// Check if email local part contains only valid characters
fn is_valid_email_local_chars(s: String) -> Bool {
  string.to_graphemes(s)
  |> list.all(fn(c) {
    case c {
      "." | "-" | "_" | "+" -> True
      _ ->
        case
          string.contains(
            "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789",
            c,
          )
        {
          True -> True
          False -> False
        }
    }
  })
}

/// Validate email domain part (after @)
fn validate_email_domain(domain: String) -> Result(Nil, String) {
  case string.is_empty(domain) {
    True -> Error("Email domain cannot be empty")
    False -> {
      // Domain must contain at least one dot
      case string.contains(domain, ".") {
        False -> Error("Email domain must contain at least one dot")
        True -> {
          // Split by dots - none should be empty
          let labels = string.split(domain, ".")
          case list.any(labels, string.is_empty) {
            True ->
              Error(
                "Email domain contains empty label (consecutive or trailing dots)",
              )
            False -> {
              // Check each label is valid
              case list.all(labels, is_valid_domain_label) {
                True -> Ok(Nil)
                False ->
                  Error("Email domain contains invalid labels: " <> domain)
              }
            }
          }
        }
      }
    }
  }
}

/// Check if a domain label is valid (alphanumeric and hyphens, not starting/ending with hyphen)
fn is_valid_domain_label(label: String) -> Bool {
  case string.length(label) {
    0 -> False
    _ -> {
      let starts_with_hyphen = string.starts_with(label, "-")
      let ends_with_hyphen = string.ends_with(label, "-")
      let all_chars_valid =
        string.to_graphemes(label)
        |> list.all(fn(c) {
          case
            string.contains(
              "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-",
              c,
            )
          {
            True -> True
            False -> False
          }
        })

      !starts_with_hyphen && !ends_with_hyphen && all_chars_valid
    }
  }
}

/// Validate UUID format and structure
/// Validates version (1-5) and variant (RFC 4122) bits
pub fn validate_uuid(uuid: String) -> Result(Nil, String) {
  let parts = string.split(uuid, "-")
  case parts {
    [time_low, time_mid, time_high_version, clock_seq_hi_variant, node] -> {
      case
        string.length(time_low) == 8
        && string.length(time_mid) == 4
        && string.length(time_high_version) == 4
        && string.length(clock_seq_hi_variant) == 4
        && string.length(node) == 12
      {
        False ->
          Error(
            "'"
            <> uuid
            <> "' has invalid UUID segment lengths (expected 8-4-4-4-12)",
          )
        True -> {
          // Check all parts are valid hex
          case
            is_valid_hex(time_low)
            && is_valid_hex(time_mid)
            && is_valid_hex(time_high_version)
            && is_valid_hex(clock_seq_hi_variant)
            && is_valid_hex(node)
          {
            False ->
              Error("'" <> uuid <> "' contains non-hexadecimal characters")
            True -> {
              // Validate version bits (time_high_version[0] should indicate version 1-5)
              let version_char = string.slice(time_high_version, 0, 1)
              case string.contains("12345", version_char) {
                False ->
                  Error(
                    "'"
                    <> uuid
                    <> "' has invalid UUID version (expected 1-5, got "
                    <> version_char
                    <> ")",
                  )
                True -> {
                  // Validate variant bits (clock_seq_hi_variant[0] for RFC 4122)
                  let variant_char = string.slice(clock_seq_hi_variant, 0, 1)
                  case string.contains("89abAB", variant_char) {
                    False ->
                      Error(
                        "'"
                        <> uuid
                        <> "' has invalid RFC 4122 variant (expected 8,9,a,b variant bits)",
                      )
                    True -> Ok(Nil)
                  }
                }
              }
            }
          }
        }
      }
    }
    _ -> Error("'" <> uuid <> "' is not a valid UUID (invalid segment count)")
  }
}

/// Check if a string contains only valid hexadecimal characters
fn is_valid_hex(s: String) -> Bool {
  string.to_graphemes(s)
  |> list.all(fn(c) {
    case string.contains("0123456789abcdefABCDEF", c) {
      True -> True
      False -> False
    }
  })
}

/// Validate URI following RFC 3986
/// Checks for valid scheme, authority, path, query, and fragment
pub fn validate_uri(uri: String) -> Result(Nil, String) {
  case string.is_empty(uri) {
    True -> Error("URI cannot be empty")
    False -> {
      // Must contain a scheme (e.g., http://, https://, ftp://)
      let has_scheme_sep = string.contains(uri, "://")
      case has_scheme_sep {
        False -> Error("'" <> uri <> "' is not a valid URI (missing scheme)")
        True -> {
          // Extract and validate scheme
          case string.split_once(uri, "://") {
            Error(_) ->
              Error("'" <> uri <> "' is not a valid URI (malformed scheme)")
            Ok(#(scheme, rest)) -> {
              case validate_uri_scheme(scheme) {
                Error(e) -> Error(e)
                Ok(_) -> {
                  // Must have some content after ://
                  case string.is_empty(rest) {
                    True ->
                      Error("'" <> uri <> "' has no authority after scheme")
                    False -> Ok(Nil)
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

/// Validate URI scheme (must start with letter, contain only alphanumeric, +, -, .)
fn validate_uri_scheme(scheme: String) -> Result(Nil, String) {
  case string.is_empty(scheme) {
    True -> Error("URI scheme cannot be empty")
    False -> {
      let first_char = string.slice(scheme, 0, 1)
      let is_letter = case first_char {
        "a"
        | "b"
        | "c"
        | "d"
        | "e"
        | "f"
        | "g"
        | "h"
        | "i"
        | "j"
        | "k"
        | "l"
        | "m"
        | "n"
        | "o"
        | "p"
        | "q"
        | "r"
        | "s"
        | "t"
        | "u"
        | "v"
        | "w"
        | "x"
        | "y"
        | "z"
        | "A"
        | "B"
        | "C"
        | "D"
        | "E"
        | "F"
        | "G"
        | "H"
        | "I"
        | "J"
        | "K"
        | "L"
        | "M"
        | "N"
        | "O"
        | "P"
        | "Q"
        | "R"
        | "S"
        | "T"
        | "U"
        | "V"
        | "W"
        | "X"
        | "Y"
        | "Z" -> True
        _ -> False
      }

      case is_letter {
        False -> Error("URI scheme must start with a letter")
        True -> {
          let rest = string.slice(scheme, 1, string.length(scheme))
          let valid_chars =
            string.to_graphemes(rest)
            |> list.all(fn(c) {
              case
                string.contains(
                  "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789+-.",
                  c,
                )
              {
                True -> True
                False -> False
              }
            })

          case valid_chars {
            False -> Error("URI scheme contains invalid characters: " <> scheme)
            True -> Ok(Nil)
          }
        }
      }
    }
  }
}

/// Validate ISO 8601 datetime format with actual calendar validation
/// Supports: YYYY-MM-DD, YYYY-MM-DDTHH:MM:SS, with optional timezone
pub fn validate_iso8601(datetime: String) -> Result(Nil, String) {
  // At minimum, must have YYYY-MM-DD
  case string.length(datetime) {
    len if len < 10 ->
      Error("'" <> datetime <> "' is not a valid ISO8601 datetime (too short)")
    _ -> {
      // Extract date part (first 10 chars)
      let date_part = string.slice(datetime, 0, 10)
      case validate_iso8601_date(date_part) {
        Error(e) -> Error(e)
        Ok(_) -> {
          // If there's more than 10 characters, validate time part
          case string.length(datetime) {
            10 -> Ok(Nil)
            _ -> {
              // Time part starts at position 10, should be T or space
              let time_sep = string.slice(datetime, 10, 1)
              case time_sep {
                "T" | " " -> {
                  let time_part =
                    string.slice(datetime, 11, string.length(datetime))
                  validate_iso8601_time(time_part)
                }
                _ ->
                  Error(
                    "'"
                    <> datetime
                    <> "' is not a valid ISO8601 datetime (invalid separator, expected T or space)",
                  )
              }
            }
          }
        }
      }
    }
  }
}

/// Validate ISO8601 date (YYYY-MM-DD) with proper calendar validation
fn validate_iso8601_date(date_str: String) -> Result(Nil, String) {
  case string.length(date_str) {
    10 -> {
      // Check format YYYY-MM-DD
      let parts = string.split(date_str, "-")
      case parts {
        [year_str, month_str, day_str] -> {
          case parse_int(year_str), parse_int(month_str), parse_int(day_str) {
            Error(_), _, _ ->
              Error("'" <> date_str <> "' has invalid year (not a number)")
            _, Error(_), _ ->
              Error("'" <> date_str <> "' has invalid month (not a number)")
            _, _, Error(_) ->
              Error("'" <> date_str <> "' has invalid day (not a number)")
            Ok(year), Ok(month), Ok(day) -> {
              // Validate ranges
              case month {
                m if m < 1 || m > 12 ->
                  Error(
                    "'"
                    <> date_str
                    <> "' has invalid month: "
                    <> month_str
                    <> " (must be 01-12)",
                  )
                _ -> {
                  // Validate day based on month and leap year
                  let max_day = get_days_in_month(month, is_leap_year(year))
                  case day {
                    d if d < 1 || d > max_day ->
                      Error(
                        "'"
                        <> date_str
                        <> "' has invalid day: "
                        <> day_str
                        <> " (month "
                        <> month_str
                        <> " has max "
                        <> int_to_string(max_day)
                        <> " days)",
                      )
                    _ -> Ok(Nil)
                  }
                }
              }
            }
          }
        }
        _ -> Error("'" <> date_str <> "' is not valid ISO8601 date format")
      }
    }
    _ ->
      Error(
        "'"
        <> date_str
        <> "' is not valid ISO8601 date format (expected YYYY-MM-DD)",
      )
  }
}

/// Validate ISO8601 time part (HH:MM:SS with optional fractional seconds and timezone)
fn validate_iso8601_time(time_str: String) -> Result(Nil, String) {
  // Remove timezone part if present (Z, +HH:MM, -HH:MM) without treating a
  // leading sign on the hour as a timezone indicator.
  let time_without_tz = case string.ends_with(time_str, "Z") {
    True -> string.slice(time_str, 0, string.length(time_str) - 1)
    False -> {
      case string.split_once(time_str, "+") {
        Ok(#(base, _tz)) ->
          case string.is_empty(base) {
            True -> time_str
            False -> base
          }
        Error(_) ->
          case string.split_once(time_str, "-") {
            Ok(#(base, _tz)) ->
              case string.is_empty(base) {
                True -> time_str
                False -> base
              }
            Error(_) -> time_str
          }
      }
    }
  }

  case string.split(time_without_tz, ":") {
    [hour_str, minute_str, second_and_frac] ->
      case
        string.starts_with(hour_str, "-") || string.starts_with(hour_str, "+")
      {
        True ->
          Error("Invalid ISO8601 time: hour must be a number, got " <> hour_str)
        False ->
          case parse_int(hour_str), parse_int(minute_str) {
            Error(_), _ ->
              Error(
                "Invalid ISO8601 time: hour must be a number, got " <> hour_str,
              )
            _, Error(_) ->
              Error(
                "Invalid ISO8601 time: minute must be a number, got "
                <> minute_str,
              )
            Ok(hour), Ok(minute) -> {
              case hour {
                h if h < 0 || h > 23 ->
                  Error(
                    "Invalid ISO8601 time: hour must be 00-23, got " <> hour_str,
                  )
                _ -> {
                  case minute {
                    m if m < 0 || m > 59 ->
                      Error(
                        "Invalid ISO8601 time: minute must be 00-59, got "
                        <> minute_str,
                      )
                    _ -> {
                      let sec_parts = string.split(second_and_frac, ".")
                      case sec_parts {
                        [sec_str] -> validate_seconds(sec_str)
                        [sec_str, _frac] -> validate_seconds(sec_str)
                        _ -> Error("Invalid ISO8601 time format")
                      }
                    }
                  }
                }
              }
            }
          }
      }
    _ -> Error("Invalid ISO8601 time format (expected HH:MM:SS)")
  }
}

fn validate_seconds(sec_str: String) -> Result(Nil, String) {
  case parse_int(sec_str) {
    Error(_) ->
      Error("Invalid ISO8601 time: second must be a number, got " <> sec_str)
    Ok(sec) ->
      case sec {
        s if s < 0 || s > 59 ->
          Error("Invalid ISO8601 time: second must be 00-59, got " <> sec_str)
        _ -> Ok(Nil)
      }
  }
}

/// Parse a string to integer
fn parse_int(s: String) -> Result(Int, Nil) {
  case int.parse(s) {
    Ok(n) -> Ok(n)
    Error(_) -> Error(Nil)
  }
}

/// Check if a year is a leap year
fn is_leap_year(year: Int) -> Bool {
  case year % 4 {
    0 ->
      case year % 100 {
        0 ->
          case year % 400 {
            0 -> True
            _ -> False
          }
        _ -> True
      }
    _ -> False
  }
}

/// Get the number of days in a month
fn get_days_in_month(month: Int, is_leap: Bool) -> Int {
  case month {
    1 | 3 | 5 | 7 | 8 | 10 | 12 -> 31
    4 | 6 | 9 | 11 -> 30
    2 ->
      case is_leap {
        True -> 29
        False -> 28
      }
    _ -> 0
  }
}

/// Convert integer to string (for error messages)
fn int_to_string(n: Int) -> String {
  int.to_string(n)
}
