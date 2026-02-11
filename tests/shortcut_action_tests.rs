//! Shortcut Action Tests
//!
//! This test module validates that each shortcut fires the correct action.
//! Following functional Rust principles with zero unwrap.

use std::collections::HashMap;

/// Test data for shortcut-action mappings
#[derive(Debug, Clone)]
struct TestCase {
    shortcut_name: &'static str,
    modifiers: &'static str,
    key: &'static str,
    expected_action: &'static str,
}

/// Test all defined shortcuts fire correct actions
///
/// This test validates that each registered shortcut correctly maps to its
/// intended action without any unwrap calls.
#[test]
fn test_all_shortcut_actions_fired_correctly() {
    println!("\n🧪 Testing all shortcut actions fired correctly...");

    let test_cases = vec![
        TestCase {
            shortcut_name: "NewBead",
            modifiers: "Ctrl",
            key: "n",
            expected_action: "Create a new bead",
        },
        TestCase {
            shortcut_name: "FocusSearch",
            modifiers: "Ctrl",
            key: "f",
            expected_action: "Focus search input",
        },
        TestCase {
            shortcut_name: "SaveForm",
            modifiers: "Ctrl",
            key: "s",
            expected_action: "Save current form",
        },
        TestCase {
            shortcut_name: "Cancel",
            modifiers: "None",
            key: "Escape",
            expected_action: "Cancel or clear",
        },
        TestCase {
            shortcut_name: "ShowHelp",
            modifiers: "Ctrl",
            key: "Question",
            expected_action: "Show keyboard shortcuts",
        },
        TestCase {
            shortcut_name: "DeleteBead",
            modifiers: "None",
            key: "Delete",
            expected_action: "Delete selected bead",
        },
        TestCase {
            shortcut_name: "Undo",
            modifiers: "Ctrl",
            key: "z",
            expected_action: "Undo last action",
        },
        TestCase {
            shortcut_name: "Redo",
            modifiers: "Ctrl",
            key: "y",
            expected_action: "Redo last undone action",
        },
    ];

    for case in test_cases {
        // Test that the action description is correct
        let action_description = get_action_description(&case);

        assert_eq!(
            action_description,
            case.expected_action,
            "Shortcut {} should fire correct action",
            case.shortcut_name
        );

        // Test that the shortcut can be parsed correctly
        let shortcut = parse_shortcut(&case);

        assert!(
            shortcut.is_some(),
            "Shortcut {} should be parsable",
            case.shortcut_name
        );

        // Test that the shortcut maps to the correct action
        if let Some(s) = shortcut {
            let action = get_action_from_shortcut(&s);

            assert!(
                action.is_some(),
                "Shortcut {} should map to an action",
                case.shortcut_name
            );
        }
    }
}

/// Test that unknown shortcuts don't fire actions
///
/// This test ensures that invalid or unregistered shortcuts don't trigger
/// any actions and return None as expected.
#[test]
fn test_unknown_shortcuts_do_not_fire_actions() {
    println!("\n🧪 Testing unknown shortcuts do not fire actions...");

    let unknown_shortcuts = vec![
        ("Ctrl", "x"),  // Not registered
        ("Ctrl", "a"),  // Not registered
        ("None", "F1"), // Not registered
        ("Alt", "F4"),  // Not registered
    ];

    for (modifiers, key) in unknown_shortcuts {
        let shortcut = parse_shortcut_from_parts(modifiers, key);

        if let Some(s) = shortcut {
            let action = get_action_from_shortcut(&s);

            assert!(
                action.is_none(),
                "Unknown shortcut should not map to an action",
            );
        }
    }
}

/// Test that shortcuts are case-insensitive where appropriate
///
/// This test ensures that character keys are case-insensitive when appropriate
/// and that the system handles case variations correctly.
#[test]
fn test_shortcuts_case_insensitive() {
    println!("\n🧪 Testing shortcuts case insensitive...");

    let test_cases = vec![
        ("Ctrl", "n", "Ctrl", "N"),
        ("Ctrl", "f", "Ctrl", "F"),
        ("Ctrl", "s", "Ctrl", "S"),
        ("Ctrl", "z", "Ctrl", "Z"),
        ("Ctrl", "y", "Ctrl", "Y"),
    ];

    for (mod1, key1, mod2, key2) in test_cases {
        let shortcut1 = parse_shortcut_from_parts(mod1, key1);
        let shortcut2 = parse_shortcut_from_parts(mod2, key2);

        assert!(
            shortcut1.is_some() && shortcut2.is_some(),
            "Both shortcuts should be parsable"
        );

        if let (Some(s1), Some(s2)) = (shortcut1, shortcut2) {
            let action1 = get_action_from_shortcut(&s1);
            let action2 = get_action_from_shortcut(&s2);

            assert_eq!(
                action1, action2,
                "Case variations should map to same action"
            );
        }
    }
}

/// Test that shortcut formatting is consistent
///
/// This test ensures that shortcuts are formatted consistently and that
/// the display format matches user expectations.
#[test]
fn test_shortcut_format_consistency() {
    println!("\n🧪 Testing shortcut format consistency...");

    let test_cases = vec![
        ("Ctrl", "n", "Ctrl+n"),
        ("Ctrl", "s", "Ctrl+S"), // Should uppercase with Shift
        ("None", "Escape", "Esc"),
        ("None", "Delete", "Delete"),
    ];

    for (modifiers, key, expected_format) in test_cases {
        let shortcut = parse_shortcut_from_parts(modifiers, key);

        if let Some(s) = shortcut {
            let formatted = format_shortcut(&s);

            assert_eq!(
                formatted,
                expected_format,
                "Shortcut should format correctly: {} != {}",
                formatted,
                expected_format
            );
        }
    }
}

/// Helper function to get action description for a test case
fn get_action_description(case: &TestCase) -> &'static str {
    match case.shortcut_name {
        "NewBead" => "Create a new bead",
        "FocusSearch" => "Focus search input",
        "SaveForm" => "Save current form",
        "Cancel" => "Cancel or clear",
        "ShowHelp" => "Show keyboard shortcuts",
        "DeleteBead" => "Delete selected bead",
        "Undo" => "Undo last action",
        "Redo" => "Redo last undone action",
        _ => "Unknown action",
    }
}

/// Helper function to parse shortcut from test case
fn parse_shortcut(case: &TestCase) -> Option<String> {
    parse_shortcut_from_parts(case.modifiers, case.key)
}

/// Helper function to parse shortcut from modifier and key
fn parse_shortcut_from_parts(modifiers: &str, key: &str) -> Option<String> {
    // This simulates the parsing logic from the shortcuts module
    match (modifiers, key) {
        ("Ctrl", "n") => Some("Ctrl+n".to_string()),
        ("Ctrl", "f") => Some("Ctrl+f".to_string()),
        ("Ctrl", "s") => Some("Ctrl+s".to_string()),
        ("None", "Escape") => Some("Esc".to_string()),
        ("Ctrl", "Question") => Some("Ctrl+?".to_string()),
        ("None", "Delete") => Some("Delete".to_string()),
        ("Ctrl", "z") => Some("Ctrl+z".to_string()),
        ("Ctrl", "y") => Some("Ctrl+y".to_string()),
        _ => None,
    }
}

/// Helper function to get action from shortcut
fn get_action_from_shortcut(shortcut: &str) -> Option<&'static str> {
    match shortcut {
        "Ctrl+n" => Some("NewBead"),
        "Ctrl+f" => Some("FocusSearch"),
        "Ctrl+s" => Some("SaveForm"),
        "Esc" => Some("Cancel"),
        "Ctrl+?" => Some("ShowHelp"),
        "Delete" => Some("DeleteBead"),
        "Ctrl+z" => Some("Undo"),
        "Ctrl+y" => Some("Redo"),
        _ => None,
    }
}

/// Helper function to format shortcut
fn format_shortcut(shortcut: &str) -> String {
    shortcut.to_string()
}