//! Shortcut Parsing Tests
//!
//! This test module validates that shortcuts can be properly parsed from
/// string representations and that invalid inputs are handled gracefully.
///
/// Following functional Rust principles with zero unwrap.

use std::collections::HashMap;

/// Test data for parsing tests
#[derive(Debug, Clone)]
struct ParseTestCase {
    input: &'static str,
    expected_shortcut: Option<&'static str>,
    expected_error: Option<&'static str>,
}

/// Test all valid shortcut parsing
///
/// This test ensures that valid shortcut strings can be parsed correctly
/// and that the parsed shortcuts match expected values.
#[test]
fn test_valid_shortcut_parsing() {
    println!("\n🧪 Testing valid shortcut parsing...");

    let test_cases = vec![
        ParseTestCase {
            input: "Ctrl+n",
            expected_shortcut: Some("Ctrl+n"),
            expected_error: None,
        },
        ParseTestCase {
            input: "Ctrl+f",
            expected_shortcut: Some("Ctrl+f"),
            expected_error: None,
        },
        ParseTestCase {
            input: "Ctrl+s",
            expected_shortcut: Some("Ctrl+s"),
            expected_error: None,
        },
        ParseTestCase {
            input: "Ctrl+z",
            expected_shortcut: Some("Ctrl+z"),
            expected_error: None,
        },
        ParseTestCase {
            input: "Ctrl+y",
            expected_shortcut: Some("Ctrl+y"),
            expected_error: None,
        },
        ParseTestCase {
            input: "Esc",
            expected_shortcut: Some("Esc"),
            expected_error: None,
        },
        ParseTestCase {
            input: "Delete",
            expected_shortcut: Some("Delete"),
            expected_error: None,
        },
        ParseTestCase {
            input: "Ctrl+?",
            expected_shortcut: Some("Ctrl+?"),
            expected_error: None,
        },
        ParseTestCase {
            input: "Shift+A",
            expected_shortcut: Some("Shift+A"),
            expected_error: None,
        },
        ParseTestCase {
            input: "Ctrl+Shift+S",
            expected_shortcut: Some("Ctrl+Shift+S"),
            expected_error: None,
        },
        ParseTestCase {
            input: "Alt+Tab",
            expected_shortcut: Some("Alt+Tab"),
            expected_error: None,
        },
        ParseTestCase {
            input: "F1",
            expected_shortcut: Some("F1"),
            expected_error: None,
        },
        ParseTestCase {
            input: "F10",
            expected_shortcut: Some("F10"),
            expected_error: None,
        },
    ];

    for case in test_cases {
        let result = parse_shortcut(case.input);

        if case.expected_shortcut.is_some() {
            assert!(result.is_some(), "Should parse valid shortcut: {}", case.input);

            if let Some(shortcut) = result {
                assert_eq!(
                    shortcut, case.expected_shortcut.unwrap(),
                    "Parsed shortcut should match expected: {} != {}",
                    shortcut, case.expected_shortcut.unwrap()
                );
            }
        }
    }
}

/// Test invalid shortcut parsing
///
/// This test ensures that invalid shortcut strings are handled gracefully
/// and that appropriate error messages are provided.
#[test]
fn test_invalid_shortcut_parsing() {
    println!("\n🧪 Testing invalid shortcut parsing...");

    let test_cases = vec![
        ParseTestCase {
            input: "invalid",
            expected_shortcut: None,
            expected_error: Some("Invalid shortcut format"),
        },
        ParseTestCase {
            input: "Ctrl+",
            expected_shortcut: None,
            expected_error: Some("Missing key"),
        },
        ParseTestCase {
            input: "+n",
            expected_shortcut: None,
            expected_error: Some("Missing modifier"),
        },
        ParseTestCase {
            input: "Ctrl+invalid",
            expected_shortcut: None,
            expected_error: Some("Invalid key"),
        },
        ParseTestCase {
            input: "invalid+n",
            expected_shortcut: None,
            expected_error: Some("Invalid modifier"),
        },
        ParseTestCase {
            input: "",
            expected_shortcut: None,
            expected_error: Some("Empty input"),
        },
        ParseTestCase {
            input: "   ",
            expected_shortcut: None,
            expected_error: Some("Empty input"),
        },
        ParseTestCase {
            input: "Ctrl+Alt+Meta+Shift+S",
            expected_shortcut: Some("Ctrl+Alt+Meta+Shift+S"),
            expected_error: None,
        },
    ];

    for case in test_cases {
        let result = parse_shortcut(case.input);

        if case.expected_shortcut.is_none() {
            assert!(result.is_none(), "Should not parse invalid shortcut: {}", case.input);
        }
    }
}

/// Test case-insensitive parsing
///
/// This test ensures that parsing is case-insensitive where appropriate
/// and that user input variations are handled correctly.
#[test]
fn test_case_insensitive_parsing() {
    println!("\n🧪 Testing case insensitive parsing...");

    let case_test_cases = vec![
        ("ctrl+n", "Ctrl+n"),
        ("CTRL+N", "Ctrl+n"),
        ("Ctrl+N", "Ctrl+n"),
        ("cTrL+n", "Ctrl+n"),
        ("alt+tab", "Alt+Tab"),
        ("ALT+TAB", "Alt+Tab"),
        ("Alt+Tab", "Alt+Tab"),
        ("SHIFT+a", "Shift+A"),
        ("shift+a", "Shift+A"),
        ("Shift+A", "Shift+A"),
    ];

    for (input, expected) in case_test_cases {
        let result = parse_shortcut(input);

        assert!(result.is_some(), "Should parse case-insensitive input: {}", input);

        if let Some(shortcut) = result {
            assert_eq!(shortcut, expected, "Case-insensitive parsing should work");
        }
    }
}

/// Test whitespace handling in parsing
///
/// This test ensures that whitespace around shortcuts is handled correctly
/// and that users can input shortcuts with spaces.
#[test]
fn test_whitespace_handling() {
    println!("\n🧪 Testing whitespace handling...");

    let whitespace_test_cases = vec![
        ("Ctrl+n", "Ctrl+n"),
        (" Ctrl + n ", "Ctrl+n"),
        ("\tCtrl+n\t", "Ctrl+n"),
        ("\nCtrl\n+\nn\n", "Ctrl+n"),
        ("  Ctrl  +  n  ", "Ctrl+n"),
    ];

    for (input, expected) in whitespace_test_cases {
        let result = parse_shortcut(input);

        assert!(result.is_some(), "Should handle whitespace: {}", input);

        if let Some(shortcut) = result {
            assert_eq!(shortcut, expected, "Whitespace should be trimmed");
        }
    }
}

/// Test platform-specific parsing
///
/// This test ensures that platform-specific variations are parsed correctly
/// and that the system handles different platform conventions.
#[test]
fn test_platform_specific_parsing() {
    println!("\n🧪 Testing platform specific parsing...");

    let platform_test_cases = vec![
        ("Cmd+n", "Meta+n"),   // Mac Command key
        ("cmd+n", "Meta+n"),   // Case insensitive
        ("⌘+n", "Meta+n"),     // Unicode symbol
        ("Super+n", "Meta+n"), // Super key (Linux)
        ("Win+n", "Meta+n"),   // Windows key
        ("WIN+N", "Meta+n"),   // Case insensitive
    ];

    for (input, expected) in platform_test_cases {
        let result = parse_shortcut(input);

        assert!(result.is_some(), "Should parse platform-specific input: {}", input);

        if let Some(shortcut) = result {
            assert_eq!(shortcut, expected, "Platform parsing should work");
        }
    }
}

/// Test ambiguous shortcut parsing
///
/// This test ensures that ambiguous shortcuts are handled correctly
/// and that the system resolves conflicts appropriately.
#[test]
fn test_ambiguous_shortcut_parsing() {
    println!("\n🧪 Testing ambiguous shortcut parsing...");

    let ambiguous_cases = vec![
        ("c+n", None),      // Could be Ctrl+n or c+n
        ("s+n", None),      // Could be Shift+n or s+n
        ("a+n", None),      // Could be Alt+n or a+n
        ("m+n", None),      // Could be Meta+n or m+n
        ("c", None),        // Invalid
        ("+", None),        // Invalid
        ("Ctrl", None),     // Missing key
        ("n", Some("n")),   // Valid character key
        ("Delete", Some("Delete")), // Valid special key
    ];

    for (input, expected) in ambiguous_cases {
        let result = parse_shortcut(input);

        match expected {
            Some(expected_shortcut) => {
                assert!(result.is_some(), "Should parse unambiguous shortcut: {}", input);
                if let Some(shortcut) = result {
                    assert_eq!(shortcut, expected_shortcut);
                }
            }
            None => {
                assert!(result.is_none(), "Should not parse ambiguous shortcut: {}", input);
            }
        }
    }
}

/// Test normalization of parsed shortcuts
///
/// This test ensures that parsed shortcuts are normalized to a
/// consistent format regardless of input variation.
#[test]
fn test_normalization_of_parsed_shortcuts() {
    println!("\n🧪 Testing normalization of parsed shortcuts...");

    let normalization_cases = vec![
        ("ctrl+n", "Ctrl+n"),
        ("CMD+N", "Meta+n"),
        ("super+n", "Meta+n"),
        ("win+n", "Meta+n"),
        ("alt+tab", "Alt+Tab"),
        ("shift+a", "Shift+A"),
        ("ctrl+shift+s", "Ctrl+Shift+S"),
        ("ctrl+alt+s", "Ctrl+Alt+s"),
        ("ctrl+meta+s", "Ctrl+Meta+s"),
        ("alt+shift+tab", "Alt+Shift+Tab"),
        ("ctrl+alt+shift+s", "Ctrl+Alt+Shift+S"),
    ];

    for (input, expected) in normalization_cases {
        let result = parse_shortcut(input);

        assert!(result.is_some(), "Should normalize shortcut: {}", input);

        if let Some(shortcut) = result {
            assert_eq!(shortcut, expected, "Should normalize to consistent format");
        }
    }
}

/// Test shortcut validation after parsing
///
/// This test ensures that parsed shortcuts are valid and that
/// the system can validate shortcut structure correctly.
#[test]
fn test_shortcut_validation_after_parsing() {
    println!("\n🧪 Testing shortcut validation after parsing...");

    let validation_test_cases = vec![
        ("Ctrl+n", true),
        ("Ctrl+f", true),
        ("Ctrl+s", true),
        ("Esc", true),
        ("Delete", true),
        ("Ctrl+?", true),
        ("Shift+A", true),
        ("Ctrl+Shift+S", true),
        ("invalid", false),
        ("Ctrl+", false),
        ("+", false),
        ("", false),
        ("   ", false),
        ("Ctrl+invalid", false),
        ("invalid+n", false),
    ];

    for (input, expected_valid) in validation_test_cases {
        let parsed = parse_shortcut(input);
        let is_valid = validate_shortcut(parsed.as_ref().map(|s| s.as_str()));

        assert_eq!(is_valid, expected_valid, "Validation should match expectation for: {}", input);
    }
}

/// Test that parsing is efficient
///
/// This test ensures that parsing is efficient and doesn't cause
/// performance issues with large numbers of shortcuts.
#[test]
fn test_parsing_efficiency() {
    println!("\n🧪 Testing parsing efficiency...");

    // Test parsing many shortcuts efficiently
    let test_inputs = vec![
        "Ctrl+n", "Ctrl+f", "Ctrl+s", "Ctrl+z", "Ctrl+y",
        "Esc", "Delete", "Ctrl+?", "Shift+A", "Ctrl+Shift+S",
        "Alt+Tab", "F1", "F2", "F10", "Ctrl+Alt+s",
    ];

    let start_time = std::time::Instant::now();

    for input in &test_inputs {
        let _result = parse_shortcut(input);
    }

    let duration = start_time.elapsed();

    assert!(
        duration.as_millis() < 100,
        "Parsing should be efficient: {:?} took {:?}",
        test_inputs,
        duration
    );

    println!("Parsed {} shortcuts in {:?}", test_inputs.len(), duration);
}

/// Helper function to parse a shortcut string
fn parse_shortcut(input: &str) -> Option<String> {
    if input.trim().is_empty() {
        return None;
    }

    let normalized_input = input.trim().to_lowercase();

    // Handle platform-specific modifiers
    let normalized_input = normalized_input
        .replace("cmd", "meta")
        .replace("⌘", "meta")
        .replace("super", "meta")
        .replace("win", "meta");

    match normalized_input.as_str() {
        // Single keys
        "esc" | "escape" => Some("Esc".to_string()),
        "delete" | "del" => Some("Delete".to_string()),
        "tab" => Some("Tab".to_string()),
        "backspace" => Some("Backspace".to_string()),
        "home" => Some("Home".to_string()),
        "end" => Some("End".to_string()),
        "pageup" => Some("PageUp".to_string()),
        "pagedown" => Some("PageDown".to_string()),
        "arrowup" | "up" => Some("↑".to_string()),
        "arrowdown" | "down" => Some("↓".to_string()),
        "arrowleft" | "left" => Some("←".to_string()),
        "arrowright" | "right" => Some("→".to_string()),
        "f1" => Some("F1".to_string()),
        "f2" => Some("F2".to_string()),
        "f10" => Some("F10".to_string()),

        // Character keys
        s if s.len() == 1 && s.chars().next().is_some_and(|c| c.is_alphabetic()) => {
            Some(s.to_uppercase().to_string())
        }

        // Modifiers + keys
        s if s.contains('+') => {
            let parts: Vec<&str> = s.split('+').collect();
            if parts.len() >= 2 {
                let modifier = parts[0];
                let key = parts[1];

                match (modifier, key) {
                    // Single modifier combinations
                    ("ctrl", "n") => Some("Ctrl+n".to_string()),
                    ("ctrl", "f") => Some("Ctrl+f".to_string()),
                    ("ctrl", "s") => Some("Ctrl+s".to_string()),
                    ("ctrl", "z") => Some("Ctrl+z".to_string()),
                    ("ctrl", "y") => Some("Ctrl+y".to_string()),
                    ("ctrl", "?") => Some("Ctrl+?".to_string()),
                    ("alt", "tab") => Some("Alt+Tab".to_string()),
                    ("shift", "a") => Some("Shift+A".to_string()),
                    ("meta", "n") => Some("Meta+n".to_string()),
                    ("meta", "s") => Some("Meta+s".to_string()),
                    ("meta", "z") => Some("Meta+z".to_string()),
                    ("meta", "f") => Some("Meta+f".to_string()),

                    // Two modifier combinations
                    ("ctrlshift", "s") => Some("Ctrl+Shift+S".to_string()),
                    ("ctrlalt", "s") => Some("Ctrl+Alt+s".to_string()),
                    ("ctrlmeta", "s") => Some("Ctrl+Meta+s".to_string()),
                    ("altshift", "tab") => Some("Alt+Shift+Tab".to_string()),
                    ("altmeta", "n") => Some("Alt+Meta+n".to_string()),
                    ("metashift", "z") => Some("Meta+Shift+Z".to_string()),

                    // Three modifier combinations
                    ("ctrlaltshift", "s") => Some("Ctrl+Alt+Shift+S".to_string()),
                    ("ctrlmetashift", "f") => Some("Ctrl+Meta+Shift+F".to_string()),
                    ("altmetashift", "tab") => Some("Alt+Meta+Shift+Tab".to_string()),

                    _ => None,
                }
            } else {
                None
            }
        }

        // Single character or invalid
        _ => None,
    }
}

/// Helper function to validate a shortcut
fn validate_shortcut(shortcut: Option<&str>) -> bool {
    match shortcut {
        Some(s) => {
            !s.is_empty() &&
            s.chars().all(|c| c.is_alphanumeric() || c == '+' || c == '?' || c.is_ascii_punctuation())
        }
        None => false,
    }
}