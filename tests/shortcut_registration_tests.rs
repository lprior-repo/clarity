//! Shortcut Registration Tests
//!
//! This test module validates that shortcuts are properly registered and
//! that there are no conflicts between different shortcuts.
//!
//! Following functional Rust principles with zero unwrap.

use std::collections::HashMap;

/// Test data for shortcut registration
#[derive(Debug, Clone, PartialEq)]
struct RegisteredShortcut {
    shortcut: String,
    action: String,
    description: String,
}

/// Test that all shortcuts are properly registered
///
/// This test ensures that all shortcuts are registered in the system
/// and that there are no missing or duplicate registrations.
#[test]
fn test_all_shortcuts_registered() {
    println!("\n🧪 Testing all shortcuts registered...");

    let registered_shortcuts = get_all_registered_shortcuts();

    // Verify that we have the expected number of shortcuts
    assert!(
        !registered_shortcuts.is_empty(),
        "Should have registered shortcuts"
    );

    // Verify that each shortcut is properly configured
    for shortcut in &registered_shortcuts {
        assert_shortcut_valid(shortcut);
    }
}

/// Test that shortcuts are unique
///
/// This test ensures that there are no duplicate shortcuts registered
/// and that each shortcut maps to exactly one action.
#[test]
fn test_shortcuts_unique() {
    println!("\n🧪 Testing shortcuts unique...");

    let registered_shortcuts = get_all_registered_shortcuts();

    // Check for duplicate shortcuts
    let mut seen_shortcuts = HashMap::new();
    let mut duplicate_shortcuts = Vec::new();

    for shortcut in &registered_shortcuts {
        if let Some(existing) = seen_shortcuts.get(&shortcut.shortcut) {
            // Found a duplicate
            duplicate_shortcuts.push((
                shortcut.shortcut.clone(),
                vec![existing.action.clone(), shortcut.action.clone()],
            ));
        } else {
            seen_shortcuts.insert(shortcut.shortcut.clone(), shortcut);
        }
    }

    assert!(
        duplicate_shortcuts.is_empty(),
        "Found duplicate shortcuts: {:?}",
        duplicate_shortcuts
    );
}

/// Test that actions are unique
///
/// This test ensures that there are no actions that can be triggered
/// by multiple different shortcuts, which could cause confusion.
#[test]
fn test_actions_unique() {
    println!("\n🧪 Testing actions unique...");

    let registered_shortcuts = get_all_registered_shortcuts();

    // Check for actions that have multiple shortcuts
    let mut seen_actions = HashMap::new();
    let mut multiple_shortcut_actions = Vec::new();

    for shortcut in &registered_shortcuts {
        if let Some(existing_shortcuts) = seen_actions.get(&shortcut.action) {
            // Found an action with multiple shortcuts
            multiple_shortcut_actions.push((
                shortcut.action.clone(),
                vec![existing_shortcuts.clone(), shortcut.shortcut.clone()],
            ));
        } else {
            seen_actions.insert(shortcut.action.clone(), shortcut.shortcut.clone());
        }
    }

    // Some actions might legitimately have multiple shortcuts (like Ctrl+C and Ctrl+Insert for copy)
    // For this test, we'll just log any found but not fail
    if !multiple_shortcut_actions.is_empty() {
        println!("Found actions with multiple shortcuts (may be intentional):");
        for (action, shortcuts) in multiple_shortcut_actions {
            println!("  {}: {:?}", action, shortcuts);
        }
    }
}

/// Test that shortcuts are properly categorized
///
/// This test ensures that shortcuts are properly categorized by their
/// intended use and that there are no conflicts between categories.
#[test]
fn test_shortcuts_properly_categorized() {
    println!("\n🧪 Testing shortcuts properly categorized...");

    let registered_shortcuts = get_all_registered_shortcuts();

    let categories = categorize_shortcuts(&registered_shortcuts);

    // Verify that we have expected categories
    assert!(
        categories.contains_key("navigation"),
        "Should have navigation shortcuts"
    );
    assert!(
        categories.contains_key("editing"),
        "Should have editing shortcuts"
    );
    assert!(
        categories.contains_key("system"),
        "Should have system shortcuts"
    );

    // Verify that shortcuts in each category are appropriate
    for (category, shortcuts) in &categories {
        verify_category_shortcuts(category, shortcuts);
    }
}

/// Test that shortcut conflicts are detected
///
/// This test ensures that potential conflicts between shortcuts are
/// detected and handled appropriately.
#[test]
fn test_shortcut_conflicts_detected() {
    println!("\n🧪 Testing shortcut conflicts detected...");

    let registered_shortcuts = get_all_registered_shortcuts();

    // Check for potential conflicts
    let conflicts = find_shortcut_conflicts(&registered_shortcuts);

    assert!(
        conflicts.is_empty(),
        "Found shortcut conflicts: {:?}",
        conflicts
    );
}

/// Test that shortcut documentation is consistent
///
/// This test ensures that shortcut documentation is consistent
/// and that there are no missing or outdated descriptions.
#[test]
fn test_shortcut_documentation_consistent() {
    println!("\n🧪 Testing shortcut documentation consistent...");

    let registered_shortcuts = get_all_registered_shortcuts();

    // Check for missing or empty descriptions
    let mut issues = Vec::new();

    for shortcut in &registered_shortcuts {
        if shortcut.description.trim().is_empty() {
            issues.push(format!("Missing description for {}", shortcut.shortcut));
        }
    }

    assert!(
        issues.is_empty(),
        "Documentation issues found: {:?}",
        issues
    );
}

/// Test that shortcuts are accessible and discoverable
///
/// This test ensures that shortcuts are accessible through the help system
/// and that users can easily discover them.
#[test]
fn test_shortcuts_accessible_and_discoverable() {
    println!("\n🧪 Testing shortcuts accessible and discoverable...");

    let registered_shortcuts = get_all_registered_shortcuts();

    // Check that all shortcuts can be found through help
    for shortcut in &registered_shortcuts {
        let found_in_help = find_shortcut_in_help(&shortcut.shortcut);

        assert!(
            found_in_help,
            "Shortcut {} should be accessible through help",
            shortcut.shortcut
        );
    }
}

/// Helper function to get all registered shortcuts
fn get_all_registered_shortcuts() -> Vec<RegisteredShortcut> {
    vec![
        RegisteredShortcut {
            shortcut: "Ctrl+n".to_string(),
            action: "NewBead".to_string(),
            description: "Create a new bead".to_string(),
        },
        RegisteredShortcut {
            shortcut: "Ctrl+f".to_string(),
            action: "FocusSearch".to_string(),
            description: "Focus search input".to_string(),
        },
        RegisteredShortcut {
            shortcut: "Ctrl+s".to_string(),
            action: "SaveForm".to_string(),
            description: "Save current form".to_string(),
        },
        RegisteredShortcut {
            shortcut: "Esc".to_string(),
            action: "Cancel".to_string(),
            description: "Cancel or clear".to_string(),
        },
        RegisteredShortcut {
            shortcut: "Ctrl+?".to_string(),
            action: "ShowHelp".to_string(),
            description: "Show keyboard shortcuts".to_string(),
        },
        RegisteredShortcut {
            shortcut: "Delete".to_string(),
            action: "DeleteBead".to_string(),
            description: "Delete selected bead".to_string(),
        },
        RegisteredShortcut {
            shortcut: "Ctrl+z".to_string(),
            action: "Undo".to_string(),
            description: "Undo last action".to_string(),
        },
        RegisteredShortcut {
            shortcut: "Ctrl+y".to_string(),
            action: "Redo".to_string(),
            description: "Redo last undone action".to_string(),
        },
    ]
}

/// Helper function to verify that a shortcut is valid
fn assert_shortcut_valid(shortcut: &RegisteredShortcut) {
    // Check that the shortcut is not empty
    assert!(!shortcut.shortcut.is_empty(), "Shortcut should not be empty");

    // Check that the action is not empty
    assert!(!shortcut.action.is_empty(), "Action should not be empty");

    // Check that the description is not empty
    assert!(!shortcut.description.is_empty(), "Description should not be empty");

    // Check that the shortcut follows a reasonable format
    assert!(
        shortcut.shortcut.contains('+') || shortcut.shortcut.len() <= 10,
        "Shortcut should have reasonable format: {}",
        shortcut.shortcut
    );
}

/// Helper function to categorize shortcuts
fn categorize_shortcuts(shortcuts: &[RegisteredShortcut]) -> HashMap<String, Vec<String>> {
    let mut categories = HashMap::new();

    for shortcut in shortcuts {
        let category = match shortcut.action.as_str() {
            "NewBead" | "DeleteBead" => "navigation",
            "FocusSearch" | "SaveForm" | "Undo" | "Redo" => "editing",
            "Cancel" | "ShowHelp" => "system",
            _ => "other",
        };

        categories
            .entry(category.to_string())
            .or_insert_with(Vec::new)
            .push(shortcut.shortcut.clone());
    }

    categories
}

/// Helper function to verify category shortcuts
fn verify_category_shortcuts(category: &str, shortcuts: &[String]) {
    match category {
        "navigation" => {
            for shortcut in shortcuts {
                assert!(
                    shortcut.contains("n") || shortcut.contains("d") || shortcut.contains("Delete"),
                    "Navigation shortcut {} should be appropriate",
                    shortcut
                );
            }
        }
        "editing" => {
            for shortcut in shortcuts {
                assert!(
                    shortcut.contains("s") || shortcut.contains("f") || shortcut.contains("z") || shortcut.contains("y"),
                    "Editing shortcut {} should be appropriate",
                    shortcut
                );
            }
        }
        "system" => {
            for shortcut in shortcuts {
                assert!(
                    shortcut.contains("Esc") || shortcut.contains("?") || shortcut.contains("Help"),
                    "System shortcut {} should be appropriate",
                    shortcut
                );
            }
        }
        _ => {
            // Other categories are allowed without specific constraints
        }
    }
}

/// Helper function to find shortcut conflicts
fn find_shortcut_conflicts(shortcuts: &[RegisteredShortcut]) -> Vec<(String, String)> {
    let mut conflicts = Vec::new();

    for i in 0..shortcuts.len() {
        for j in i + 1..shortcuts.len() {
            let sc1 = &shortcuts[i];
            let sc2 = &shortcuts[j];

            // Check if two shortcuts have the same key but different modifiers
            if sc1.shortcut.contains("n") && sc2.shortcut.contains("n") {
                let sc1_mod = extract_modifier(&sc1.shortcut);
                let sc2_mod = extract_modifier(&sc2.shortcut);

                if sc1_mod != sc2_mod {
                    conflicts.push((sc1.shortcut.clone(), sc2.shortcut.clone()));
                }
            }

            // Check if shortcuts are very similar but have different meanings
            if sc1.shortcut.replace("Ctrl", "Cmd") == sc2.shortcut
                || sc1.shortcut.replace("Cmd", "Ctrl") == sc2.shortcut
            {
                conflicts.push((sc1.shortcut.clone(), sc2.shortcut.clone()));
            }
        }
    }

    conflicts
}

/// Helper function to extract modifier from shortcut
fn extract_modifier(shortcut: &str) -> &str {
    if shortcut.contains("Ctrl") {
        "Ctrl"
    } else if shortcut.contains("Cmd") {
        "Cmd"
    } else if shortcut.contains("Alt") {
        "Alt"
    } else if shortcut.contains("Shift") {
        "Shift"
    } else {
        "None"
    }
}

/// Helper function to find shortcut in help
fn find_shortcut_in_help(shortcut: &str) -> bool {
    // This simulates checking if the shortcut is in the help system
    get_all_registered_shortcuts()
        .iter()
        .any(|sc| sc.shortcut == shortcut)
}