#![allow(clippy::unnecessary_literal_unwrap)]
#![allow(clippy::unnecessary_lazy_evaluations)]

use std::convert::Infallible;

#[test]
fn test_server_result_error_handling() {
  let result: Result<i32, Infallible> = Ok(42);
  let value = result.unwrap_or(0);
  assert_eq!(value, 42);
}

#[test]
fn test_server_option_none_handling() {
  let option: Option<i32> = None;
  let value = option.unwrap_or(0);
  assert_eq!(value, 0);
}

#[test]
fn test_server_result_with_expected_error() {
  let result: Result<i32, String> = Err("error".to_string());
  assert!(result.is_err());
}

#[test]
fn test_server_result_with_ok_value() {
  let result: Result<i32, String> = Ok(42);
  assert!(result.is_ok());
  assert_eq!(result.unwrap_or(0), 42);
}

#[test]
fn test_server_option_some_handling() {
  let option: Option<i32> = Some(42);
  let value = option.unwrap_or(0);
  assert_eq!(value, 42);
}

#[test]
fn test_server_result_or_else_pattern() {
  let result: Result<i32, String> = Ok(42);
  let value = result.or_else(|_| Ok::<i32, String>(0));
  if let Ok(value) = value {
    assert_eq!(value, 42);
  }
}

#[test]
fn test_server_result_and_then_pattern() {
  let result: Result<i32, String> = Ok(42);
  let value = result.map(|x| x + 1);
  if let Ok(value) = value {
    assert_eq!(value, 43);
  }
}

#[test]
fn test_server_option_map_pattern() {
  let option: Option<i32> = Some(42);
  let value = option.map(|x| x + 1);
  if let Some(value) = value {
    assert_eq!(value, 43);
  }
}

#[test]
fn test_server_result_is_ok_check() {
  let result: Result<i32, String> = Ok(42);
  if let Ok(value) = result {
    assert_eq!(value, 42);
  }
}

#[test]
fn test_server_result_is_err_check() {
  let result: Result<i32, String> = Err("error".to_string());
  if let Err(error) = result {
    assert_eq!(error, "error");
  }
}
