/// Rule expression parser and types
/// Parses human-friendly rule strings like "equals foo" or "integer >= 5"
import gleam/float
import gleam/int
import gleam/list
import gleam/option.{type Option, None, Some}
import gleam/string

/// Parsed rule expression
pub type RuleExpr {
  // Equality
  Equals(String)
  EqualsVariable(String)
  EqualsInt(Int)
  EqualsFloat(Float)
  EqualsBool(Bool)

  // Types
  IsString
  IsInteger
  IsNumber
  IsBoolean
  IsArray
  IsObject
  IsNull

  // String patterns
  StringMatching(String)
  StringStartingWith(String)
  StringEndingWith(String)
  StringContaining(String)
  NonEmptyString
  IsEmail
  IsUuid
  IsUri
  IsJwt
  IsIso8601

  // Numbers
  IntegerGte(Int)
  IntegerGt(Int)
  IntegerLte(Int)
  IntegerLt(Int)
  IntegerBetween(Int, Int)
  NumberBetween(Float, Float)

  // Presence
  Present
  Absent
  NotNull

  // Arrays
  NonEmptyArray
  ArrayOfLength(Int)
  ArrayWithMinItems(Int)
  ArrayWithMaxItems(Int)
  ArrayWhereEach(RuleExpr)

  // Compound
  ValidJwt
  ValidIso8601
  OneOf(List(String))

  // Contains reference
  ContainsVariable(String)

  // Unknown/raw for rules we can't parse yet
  Raw(String)
}

/// Parse a rule string into a RuleExpr
pub fn parse(rule: String) -> RuleExpr {
  let rule = string.trim(rule)

  rule
  |> try_parse_first([
    try_parse_equals,
    try_parse_type,
    try_parse_string_pattern,
    try_parse_number,
    try_parse_presence,
    try_parse_array,
    try_parse_compound,
  ])
}

fn try_parse_first(
  rule: String,
  parsers: List(fn(String) -> Option(RuleExpr)),
) -> RuleExpr {
  case parsers {
    [] -> Raw(rule)
    [parser, ..rest] ->
      case parser(rule) {
        Some(expr) -> expr
        None -> try_parse_first(rule, rest)
      }
  }
}

fn try_parse_equals(rule: String) -> Option(RuleExpr) {
  case string.starts_with(rule, "equals ") {
    True -> {
      let value = string.drop_left(rule, 7)
      parse_equals_value(value)
    }
    False -> None
  }
}

fn parse_equals_value(value: String) -> Option(RuleExpr) {
  parse_equals_variable(value)
  |> option.lazy_or(fn() { parse_equals_bool(value) })
  |> option.lazy_or(fn() { parse_equals_number(value) })
  |> option.lazy_or(fn() { Some(Equals(value)) })
}

fn parse_equals_variable(value: String) -> Option(RuleExpr) {
  case string.starts_with(value, "${") && string.ends_with(value, "}") {
    True -> {
      let var_name =
        value
        |> string.drop_left(2)
        |> string.drop_right(1)
      Some(EqualsVariable(var_name))
    }
    False -> None
  }
}

fn parse_equals_bool(value: String) -> Option(RuleExpr) {
  case value {
    "true" -> Some(EqualsBool(True))
    "false" -> Some(EqualsBool(False))
    _ -> None
  }
}

fn parse_equals_number(value: String) -> Option(RuleExpr) {
  case int.parse(value) {
    Ok(n) -> Some(EqualsInt(n))
    Error(_) ->
      case float.parse(value) {
        Ok(f) -> Some(EqualsFloat(f))
        Error(_) -> None
      }
  }
}

fn try_parse_type(rule: String) -> Option(RuleExpr) {
  case rule {
    "string" -> Some(IsString)
    "integer" -> Some(IsInteger)
    "number" -> Some(IsNumber)
    "boolean" -> Some(IsBoolean)
    "array" -> Some(IsArray)
    "object" -> Some(IsObject)
    "null" -> Some(IsNull)
    _ -> None
  }
}

fn try_parse_string_pattern(rule: String) -> Option(RuleExpr) {
  case rule {
    "non-empty string" -> Some(NonEmptyString)
    "email" -> Some(IsEmail)
    "uuid" -> Some(IsUuid)
    "uri" -> Some(IsUri)
    "jwt" -> Some(IsJwt)
    "iso8601 datetime" -> Some(IsIso8601)
    _ -> try_parse_prefix_string_pattern(rule)
  }
}

fn try_parse_prefix_string_pattern(rule: String) -> Option(RuleExpr) {
  parse_prefix_pattern(rule, "string matching ", 16, StringMatching)
  |> option.lazy_or(fn() {
    parse_prefix_pattern(rule, "string starting with ", 21, StringStartingWith)
  })
  |> option.lazy_or(fn() {
    parse_prefix_pattern(rule, "string ending with ", 19, StringEndingWith)
  })
  |> option.lazy_or(fn() {
    parse_prefix_pattern(rule, "string containing ", 18, StringContaining)
  })
  |> option.lazy_or(fn() { parse_contains_variable(rule) })
}

fn parse_prefix_pattern(
  rule: String,
  prefix: String,
  drop_len: Int,
  constructor: fn(String) -> RuleExpr,
) -> Option(RuleExpr) {
  case string.starts_with(rule, prefix) {
    True -> Some(constructor(string.drop_left(rule, drop_len)))
    False -> None
  }
}

fn parse_contains_variable(rule: String) -> Option(RuleExpr) {
  case string.starts_with(rule, "contains ${") {
    True -> {
      let var =
        rule
        |> string.drop_left(11)
        |> string.drop_right(1)
      Some(ContainsVariable(var))
    }
    False -> None
  }
}

fn try_parse_number(rule: String) -> Option(RuleExpr) {
  parse_integer_comparison(rule)
  |> option.lazy_or(fn() { parse_number_between(rule) })
}

fn parse_integer_comparison(rule: String) -> Option(RuleExpr) {
  parse_integer_gte(rule)
  |> option.lazy_or(fn() { parse_integer_gt(rule) })
  |> option.lazy_or(fn() { parse_integer_lte(rule) })
  |> option.lazy_or(fn() { parse_integer_lt(rule) })
}

fn parse_integer_gte(rule: String) -> Option(RuleExpr) {
  parse_int_comparison(rule, "integer >= ", 11, IntegerGte)
}

fn parse_integer_gt(rule: String) -> Option(RuleExpr) {
  case string.starts_with(rule, "integer > ") {
    True -> {
      let rest = string.drop_left(rule, 10)
      case parse_range(rest, " and < ") {
        Some(#(low, high)) -> Some(IntegerBetween(low + 1, high - 1))
        None ->
          case int.parse(rest) {
            Ok(n) -> Some(IntegerGt(n))
            Error(_) -> None
          }
      }
    }
    False -> None
  }
}

fn parse_integer_lte(rule: String) -> Option(RuleExpr) {
  parse_int_comparison(rule, "integer <= ", 11, IntegerLte)
}

fn parse_integer_lt(rule: String) -> Option(RuleExpr) {
  parse_int_comparison(rule, "integer < ", 10, IntegerLt)
}

fn parse_int_comparison(
  rule: String,
  prefix: String,
  drop_len: Int,
  constructor: fn(Int) -> RuleExpr,
) -> Option(RuleExpr) {
  case string.starts_with(rule, prefix) {
    True -> {
      let num_str = string.drop_left(rule, drop_len)
      case int.parse(num_str) {
        Ok(n) -> Some(constructor(n))
        Error(_) -> None
      }
    }
    False -> None
  }
}

fn parse_number_between(rule: String) -> Option(RuleExpr) {
  case string.starts_with(rule, "number between ") {
    True -> {
      let rest = string.drop_left(rule, 15)
      case parse_float_range(rest) {
        Some(#(low, high)) -> Some(NumberBetween(low, high))
        None -> None
      }
    }
    False -> None
  }
}

fn parse_range(s: String, sep: String) -> Option(#(Int, Int)) {
  case string.split(s, sep) {
    [low_str, high_str] ->
      case int.parse(low_str), int.parse(high_str) {
        Ok(low), Ok(high) -> Some(#(low, high))
        _, _ -> None
      }
    _ -> None
  }
}

fn parse_float_range(s: String) -> Option(#(Float, Float)) {
  case string.split(s, " and ") {
    [low_str, high_str] ->
      case float.parse(low_str), float.parse(high_str) {
        Ok(low), Ok(high) -> Some(#(low, high))
        _, _ -> None
      }
    _ -> None
  }
}

fn try_parse_presence(rule: String) -> Option(RuleExpr) {
  case rule {
    "present" -> Some(Present)
    "absent" -> Some(Absent)
    "not null" -> Some(NotNull)
    _ -> None
  }
}

fn try_parse_array(rule: String) -> Option(RuleExpr) {
  case rule {
    "non-empty array" -> Some(NonEmptyArray)
    _ -> try_parse_prefix_array(rule)
  }
}

fn try_parse_prefix_array(rule: String) -> Option(RuleExpr) {
  parse_array_of_length(rule)
  |> option.lazy_or(fn() { parse_array_with_min(rule) })
  |> option.lazy_or(fn() { parse_array_with_max(rule) })
  |> option.lazy_or(fn() { parse_array_where_each(rule) })
}

fn parse_array_of_length(rule: String) -> Option(RuleExpr) {
  case string.starts_with(rule, "array of length ") {
    True -> {
      let num_str = string.drop_left(rule, 16)
      case int.parse(num_str) {
        Ok(n) -> Some(ArrayOfLength(n))
        Error(_) -> None
      }
    }
    False -> None
  }
}

fn parse_array_with_min(rule: String) -> Option(RuleExpr) {
  parse_array_with_items(rule, "array with min ", 15, ArrayWithMinItems)
}

fn parse_array_with_max(rule: String) -> Option(RuleExpr) {
  parse_array_with_items(rule, "array with max ", 15, ArrayWithMaxItems)
}

fn parse_array_with_items(
  rule: String,
  prefix: String,
  drop_len: Int,
  constructor: fn(Int) -> RuleExpr,
) -> Option(RuleExpr) {
  case string.starts_with(rule, prefix) {
    True -> {
      let rest = string.drop_left(rule, drop_len)
      case string.split(rest, " item") {
        [num_str, ..] ->
          case int.parse(num_str) {
            Ok(n) -> Some(constructor(n))
            Error(_) -> None
          }
        _ -> None
      }
    }
    False -> None
  }
}

fn parse_array_where_each(rule: String) -> Option(RuleExpr) {
  case string.starts_with(rule, "array where each ") {
    True -> {
      let inner = string.drop_left(rule, 17)
      let inner_rule = normalize_inner_rule(inner)
      Some(ArrayWhereEach(parse(inner_rule)))
    }
    False -> None
  }
}

fn normalize_inner_rule(inner: String) -> String {
  case string.starts_with(inner, "is ") {
    True -> string.drop_left(inner, 3)
    False ->
      case string.starts_with(inner, "matches ") {
        True -> "string matching " <> string.drop_left(inner, 8)
        False -> inner
      }
  }
}

fn try_parse_compound(rule: String) -> Option(RuleExpr) {
  case rule {
    "valid JWT" -> Some(ValidJwt)
    "valid ISO8601 datetime" -> Some(ValidIso8601)
    _ ->
      case string.starts_with(rule, "one of ") {
        True -> {
          let list_str = string.drop_left(rule, 7)
          case parse_string_list(list_str) {
            Ok(items) -> Some(OneOf(items))
            Error(_) -> None
          }
        }
        False -> None
      }
  }
}

/// Parse a list like ["a", "b", "c"]
fn parse_string_list(s: String) -> Result(List(String), Nil) {
  let s = string.trim(s)
  case string.starts_with(s, "[") && string.ends_with(s, "]") {
    True -> {
      let inner =
        s
        |> string.drop_left(1)
        |> string.drop_right(1)
        |> string.trim

      inner
      |> string.split(",")
      |> list.map(fn(item) {
        let item = string.trim(item)
        // Remove quotes
        case string.starts_with(item, "\"") && string.ends_with(item, "\"") {
          True ->
            item
            |> string.drop_left(1)
            |> string.drop_right(1)
          False -> item
        }
      })
      |> Ok
    }
    False -> Error(Nil)
  }
}

/// Format a rule expression back to a human-readable string
pub fn to_string(expr: RuleExpr) -> String {
  case expr {
    Equals(s) -> "equals " <> s
    EqualsVariable(v) -> "equals ${" <> v <> "}"
    EqualsInt(n) -> "equals " <> int.to_string(n)
    EqualsFloat(f) -> "equals " <> float.to_string(f)
    EqualsBool(True) -> "equals true"
    EqualsBool(False) -> "equals false"
    IsString -> "string"
    IsInteger -> "integer"
    IsNumber -> "number"
    IsBoolean -> "boolean"
    IsArray -> "array"
    IsObject -> "object"
    IsNull -> "null"
    StringMatching(p) -> "string matching " <> p
    StringStartingWith(p) -> "string starting with " <> p
    StringEndingWith(p) -> "string ending with " <> p
    StringContaining(p) -> "string containing " <> p
    NonEmptyString -> "non-empty string"
    IsEmail -> "email"
    IsUuid -> "uuid"
    IsUri -> "uri"
    IsJwt -> "jwt"
    IsIso8601 -> "iso8601 datetime"
    IntegerGte(n) -> "integer >= " <> int.to_string(n)
    IntegerGt(n) -> "integer > " <> int.to_string(n)
    IntegerLte(n) -> "integer <= " <> int.to_string(n)
    IntegerLt(n) -> "integer < " <> int.to_string(n)
    IntegerBetween(low, high) ->
      "integer > "
      <> int.to_string(low - 1)
      <> " and < "
      <> int.to_string(high + 1)
    NumberBetween(low, high) ->
      "number between "
      <> float.to_string(low)
      <> " and "
      <> float.to_string(high)
    Present -> "present"
    Absent -> "absent"
    NotNull -> "not null"
    NonEmptyArray -> "non-empty array"
    ArrayOfLength(n) -> "array of length " <> int.to_string(n)
    ArrayWithMinItems(n) -> "array with min " <> int.to_string(n) <> " items"
    ArrayWithMaxItems(n) -> "array with max " <> int.to_string(n) <> " items"
    ArrayWhereEach(inner) -> "array where each " <> to_string(inner)
    ValidJwt -> "valid JWT"
    ValidIso8601 -> "valid ISO8601 datetime"
    OneOf(items) ->
      "one of ["
      <> string.join(list.map(items, fn(i) { "\"" <> i <> "\"" }), ", ")
      <> "]"
    ContainsVariable(v) -> "contains ${" <> v <> "}"
    Raw(s) -> s
  }
}
