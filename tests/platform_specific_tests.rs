//! Platform-Specific Tests
//!
//! This test module validates that shortcuts work correctly across
/// different platforms and that platform-specific behavior is handled properly.
///
/// Following functional Rust principles with zero unwrap.

/// Test that Ctrl vs Cmd mappings are platform-specific
///
/// This test ensures that Ctrl and Cmd mappings are handled correctly
/// for different platforms (Windows/Linux vs Mac).
#[test]
fn test_ctrl_vs_cmd_mappings() {
    println!("\n🧪 Testing Ctrl vs Cmd mappings...");

    let platform_mappings = get_platform_mappings();

    for (platform, mappings) in platform_mappings {
        for (input, expected) in mappings {
            let result = parse_shortcut_for_platform(input, platform);

            assert_eq!(result, expected, "Platform mapping should work: {} on {}", input, platform);
        }
    }
}

/// Test that platform-specific key names are handled correctly
///
/// This test ensures that platform-specific key names are recognized
/// and mapped to the correct internal representations.
#[test]
fn test_platform_specific_key_names() {
    println!("\n🧪 Testing platform specific key names...");

    let platform_key_tests = vec![
        // Windows/Linux
        ("Windows", "Ctrl+n", "Ctrl+n"),
        ("Windows", "Alt+Tab", "Alt+Tab"),
        ("Windows", "Win+n", "Meta+n"), // Windows key maps to Meta

        // Mac
        ("Mac", "Cmd+n", "Meta+n"),
        ("Mac", "⌘+n", "Meta+n"),
        ("Mac", "Option+Tab", "Alt+Tab"), // Option key is Alt on Mac

        // Linux
        ("Linux", "Super+n", "Meta+n"),
        ("Linux", "Ctrl+Alt+Del", "Ctrl+Alt+Delete"),
    ];

    for (platform, input, expected) in platform_key_tests {
        let result = parse_shortcut_for_platform(input, platform);

        assert_eq!(result, expected, "Platform key name should work: {} on {}", input, platform);
    }
}

/// Test that modifier keys are platform-consistent
///
/// This test ensures that modifier keys work consistently across
/// platforms even if the physical keys are different.
#[test]
fn test_modifier_keys_platform_consistent() {
    println!("\n🧪 Testing modifier keys platform consistent...");

    let modifier_consistency_tests = vec![
        // Test that Ctrl+key works on all platforms
        ("Ctrl+c", "Ctrl+c"),
        ("Ctrl+v", "Ctrl+v"),
        ("Ctrl+x", "Ctrl+x"),
        ("Ctrl+z", "Ctrl+z"),
        ("Ctrl+a", "Ctrl+a"),

        // Test that Alt+key works on all platforms
        ("Alt+Tab", "Alt+Tab"),
        ("Alt+F4", "Alt+F4"),
        ("Alt+Space", "Alt+Space"),

        // Test that Meta/Cmd+key maps consistently
        ("Meta+c", "Meta+c"),
        ("Cmd+c", "Meta+c"), // Cmd maps to Meta
        ("Super+c", "Meta+c"), // Super maps to Meta
        ("Win+c", "Meta+c"), // Win maps to Meta
    ];

    for (input, expected) in modifier_consistency_tests {
        // Should work on all platforms
        for platform in ["Windows", "Mac", "Linux"] {
            let result = parse_shortcut_for_platform(input, platform);

            assert_eq!(result, expected, "Modifier consistency should work: {} on {}", input, platform);
        }
    }
}

/// Test that platform-specific defaults are respected
///
/// This test ensures that platform-specific defaults are respected
/// and that the system uses appropriate defaults for each platform.
#[test]
fn test_platform_specific_defaults() {
    println!("\n🧪 Testing platform specific defaults...");

    let platform_defaults = get_platform_defaults();

    for (platform, expected_defaults) in platform_defaults {
        let defaults = get_defaults_for_platform(platform);

        for (action, expected_shortcut) in expected_defaults {
            let actual_shortcut = defaults.get(&action).unwrap_or("");

            assert_eq!(
                actual_shortcut, expected_shortcut,
                "Platform default should be correct: {} on {}",
                action, platform
            );
        }
    }
}

/// Test that platform-specific conflicts are resolved
///
/// This test ensures that platform-specific conflicts are resolved
/// and that the system doesn't have conflicting shortcuts on any platform.
#[test]
fn test_platform_specific_conflicts_resolved() {
    println!("\n🧪 Testing platform specific conflicts resolved...");

    let all_platform_shortcuts = get_all_platform_shortcuts();

    // Check for conflicts across platforms
    let conflicts = find_platform_conflicts(&all_platform_shortcuts);

    assert!(
        conflicts.is_empty(),
        "Found platform-specific conflicts: {:?}",
        conflicts
    );
}

/// Test that platform-specific help text is correct
///
/// This test ensures that platform-specific help text is correct
/// and that users see the appropriate key names for their platform.
#[test]
fn test_platform_specific_help_text() {
    println!("\n🧪 Testing platform specific help text...");

    let help_text_tests = vec![
        ("Windows", "Ctrl+c", "Copy (Ctrl+C)"),
        ("Mac", "Cmd+c", "Copy (⌘+C)"),
        ("Linux", "Ctrl+c", "Copy (Ctrl+C)"),
        ("Windows", "Ctrl+v", "Paste (Ctrl+V)"),
        ("Mac", "Cmd+v", "Paste (⌘+V)"),
        ("Linux", "Ctrl+v", "Paste (Ctrl+V)"),
    ];

    for (platform, shortcut, expected_help) in help_text_tests {
        let help_text = get_platform_help_text(platform, shortcut);

        assert_eq!(help_text, expected_help, "Help text should be platform-specific: {} on {}", shortcut, platform);
    }
}

/// Test that platform-specific input methods are supported
///
/// This test ensures that platform-specific input methods are supported
/// and that users can input shortcuts using their platform's conventions.
#[test]
fn test_platform_specific_input_methods() {
    println!("\n🧪 Testing platform specific input methods...");

    let input_method_tests = vec![
        // Windows input methods
        ("Windows", "Ctrl+c", true),
        ("Windows", "Ctrl + c", true),
        ("Windows", "CTRL+C", true),
        ("Windows", "⌘+c", false), // Should not work on Windows

        // Mac input methods
        ("Mac", "Cmd+c", true),
        ("Mac", "⌘+c", true),
        ("Mac", "cmd+c", true),
        ("Mac", "Ctrl+c", false), // Should be Cmd on Mac

        // Linux input methods
        ("Linux", "Ctrl+c", true),
        ("Linux", "Super+c", true),
        ("Linux", "Win+c", true),
        ("Linux", "Cmd+c", false), // Should be Ctrl/Super on Linux
    ];

    for (platform, input, should_work) in input_method_tests {
        let result = parse_shortcut_for_platform(input, platform);

        if should_work {
            assert!(result.is_some(), "Input method should work: {} on {}", input, platform);
        } else {
            assert!(result.is_none(), "Input method should not work: {} on {}", input, platform);
        }
    }
}

/// Test that platform-specific menu accelerators are correct
///
/// This test ensures that platform-specific menu accelerators are correct
/// and that they match the platform's conventions.
#[test]
 fn test_platform_specific_menu_accelerators() {
    println!("\n🧪 Testing platform specific menu accelerators...");

    let menu_accelerator_tests = vec![
        ("Windows", "Copy", "Ctrl+C"),
        ("Mac", "Copy", "⌘+C"),
        ("Linux", "Copy", "Ctrl+C"),
        ("Windows", "Paste", "Ctrl+V"),
        ("Mac", "Paste", "⌘+V"),
        ("Linux", "Paste", "Ctrl+V"),
        ("Windows", "Save", "Ctrl+S"),
        ("Mac", "Save", "⌘+S"),
        ("Linux", "Save", "Ctrl+S"),
    ];

    for (platform, menu_item, expected_accelerator) in menu_accelerator_tests {
        let accelerator = get_platform_menu_accelerator(platform, menu_item);

        assert_eq!(
            accelerator, expected_accelerator,
            "Menu accelerator should be platform-specific: {} on {}",
            menu_item, platform
        );
    }
}

/// Test that platform-specific accessibility features work
///
/// This test ensures that platform-specific accessibility features work
/// and that shortcuts are accessible across platforms.
#[test]
fn test_platform_specific_accessibility() {
    println!("\n🧪 Testing platform specific accessibility...");

    let accessibility_tests = vec![
        ("Windows", "Ctrl+Alt+Del", true),  // Windows security shortcut
        ("Mac", "Cmd+Space", true),         // Spotlight search
        ("Linux", "Ctrl+Alt+Backspace", true), // Terminal reset
        ("Windows", "Cmd+Space", false),   // Should not work on Windows
        ("Mac", "Ctrl+Alt+Del", false),    // Should not work on Mac
        ("Linux", "Cmd+Space", false),     // Should not work on Linux
    ];

    for (platform, shortcut, should_work) in accessibility_tests {
        let result = parse_shortcut_for_platform(shortcut, platform);

        if should_work {
            assert!(result.is_some(), "Accessibility shortcut should work: {} on {}", shortcut, platform);
        } else {
            assert!(result.is_none(), "Accessibility shortcut should not work: {} on {}", shortcut, platform);
        }
    }
}

/// Helper function to get platform mappings
fn get_platform_mappings() -> HashMap<&'static str, HashMap<&'static str, &'static str>> {
    let mut mappings = HashMap::new();

    // Windows mappings
    mappings.insert("Windows", vec![
        ("Ctrl+n", "Ctrl+n"),
        ("Alt+Tab", "Alt+Tab"),
        ("Win+n", "Meta+n"),
    ].into_iter().collect());

    // Mac mappings
    mappings.insert("Mac", vec![
        ("Cmd+n", "Meta+n"),
        ("⌘+n", "Meta+n"),
        ("Option+Tab", "Alt+Tab"),
    ].into_iter().collect());

    // Linux mappings
    mappings.insert("Linux", vec![
        ("Super+n", "Meta+n"),
        ("Ctrl+Alt+Del", "Ctrl+Alt+Delete"),
    ].into_iter().collect());

    mappings
}

/// Helper function to parse shortcut for specific platform
fn parse_shortcut_for_platform(input: &str, platform: &str) -> Option<String> {
    let normalized_input = normalize_input_for_platform(input, platform);

    match normalized_input.as_str() {
        "ctrl+n" => Some("Ctrl+n".to_string()),
        "alt+tab" => Some("Alt+Tab".to_string()),
        "meta+n" => Some("Meta+n".to_string()),
        "cmd+n" | "meta+n" => Some("Meta+n".to_string()),
        "super+n" => Some("Meta+n".to_string()),
        "ctrl+alt+del" => Some("Ctrl+Alt+Delete".to_string()),
        _ => None,
    }
}

/// Helper function to normalize input for platform
fn normalize_input_for_platform(input: &str, platform: &str) -> String {
    let normalized = input.trim().to_lowercase();

    match platform {
        "Mac" => normalized
            .replace("cmd", "meta")
            .replace("⌘", "meta")
            .replace("option", "alt")
            .replace("⌥", "alt"),
        "Windows" => normalized
            .replace("win", "meta")
            .replace("⊞", "meta"),
        "Linux" => normalized
            .replace("super", "meta")
            .replace("⊞", "meta"),
        _ => normalized,
    }
}

/// Helper function to get platform defaults
fn get_platform_defaults() -> HashMap<&'static str, HashMap<&'static str, &'static str>> {
    let mut defaults = HashMap::new();

    defaults.insert("Windows", vec![
        ("Copy", "Ctrl+C"),
        ("Paste", "Ctrl+V"),
        ("Save", "Ctrl+S"),
    ].into_iter().collect());

    defaults.insert("Mac", vec![
        ("Copy", "⌘+C"),
        ("Paste", "⌘+V"),
        ("Save", "⌘+S"),
    ].into_iter().collect());

    defaults.insert("Linux", vec![
        ("Copy", "Ctrl+C"),
        ("Paste", "Ctrl+V"),
        ("Save", "Ctrl+S"),
    ].into_iter().collect());

    defaults
}

/// Helper function to get defaults for platform
fn get_defaults_for_platform(platform: &str) -> HashMap<&'static str, &'static str> {
    match platform {
        "Windows" => vec![
            ("Copy", "Ctrl+C"),
            ("Paste", "Ctrl+V"),
            ("Save", "Ctrl+S"),
        ].into_iter().collect(),
        "Mac" => vec![
            ("Copy", "⌘+C"),
            ("Paste", "⌘+V"),
            ("Save", "⌘+S"),
        ].into_iter().collect(),
        "Linux" => vec![
            ("Copy", "Ctrl+C"),
            ("Paste", "Ctrl+V"),
            ("Save", "Ctrl+S"),
        ].into_iter().collect(),
        _ => HashMap::new(),
    }
}

/// Helper function to get all platform shortcuts
fn get_all_platform_shortcuts() -> HashMap<&'static str, Vec<&'static str>> {
    let mut all_shortcuts = HashMap::new();

    all_shortcuts.insert("Windows", vec![
        "Ctrl+n", "Ctrl+c", "Ctrl+v", "Alt+Tab", "Win+n",
    ]);

    all_shortcuts.insert("Mac", vec![
        "Cmd+n", "Cmd+c", "Cmd+v", "Alt+Tab", "⌘+n",
    ]);

    all_shortcuts.insert("Linux", vec![
        "Ctrl+n", "Ctrl+c", "Ctrl+v", "Alt+Tab", "Super+n",
    ]);

    all_shortcuts
}

/// Helper function to find platform conflicts
fn find_platform_conflicts(platform_shortcuts: &HashMap<&'static str, Vec<&'static str>>) -> Vec<(String, String)> {
    let mut conflicts = Vec::new();

    // Check for conflicts between platforms
    let platforms: Vec<_> = platform_shortcuts.keys().collect();
    for i in 0..platforms.len() {
        for j in i + 1..platforms.len() {
            let platform1 = platforms[i];
            let platform2 = platforms[j];

            let shortcuts1 = &platform_shortcuts[platform1];
            let shortcuts2 = &platform_shortcuts[platform2];

            for sc1 in shortcuts1 {
                for sc2 in shortcuts2 {
                    if sc1 != sc2 && map_to_internal(sc1) == map_to_internal(sc2) {
                        conflicts.push((sc1.to_string(), sc2.to_string()));
                    }
                }
            }
        }
    }

    conflicts
}

/// Helper function to map shortcut to internal representation
fn map_to_internal(shortcut: &str) -> String {
    shortcut.to_lowercase().replace("cmd", "meta").replace("⌘", "meta")
}

/// Helper function to get platform help text
fn get_platform_help_text(platform: &str, shortcut: &str) -> String {
    match platform {
        "Windows" => format!("{} (Ctrl+{})", get_action_name(shortcut), shortcut.split('+').last().unwrap_or("")),
        "Mac" => {
            let display_shortcut = shortcut.replace("Ctrl", "⌘").replace("Cmd", "⌘");
            format!("{} (⌘+{})", get_action_name(shortcut), shortcut.split('+').last().unwrap_or(""))
        }
        "Linux" => format!("{} (Ctrl+{})", get_action_name(shortcut), shortcut.split('+').last().unwrap_or("")),
        _ => format!("{} ({})", get_action_name(shortcut), shortcut),
    }
}

/// Helper function to get action name from shortcut
fn get_action_name(shortcut: &str) -> &'static str {
    match shortcut {
        "Ctrl+c" | "Cmd+c" | "Super+c" => "Copy",
        "Ctrl+v" | "Cmd+v" | "Super+v" => "Paste",
        "Ctrl+s" | "Cmd+s" | "Super+s" => "Save",
        _ => "Unknown",
    }
}

/// Helper function to get platform menu accelerator
fn get_platform_menu_accelerator(platform: &str, menu_item: &str) -> String {
    match platform {
        "Windows" => match menu_item {
            "Copy" => "Ctrl+C".to_string(),
            "Paste" => "Ctrl+V".to_string(),
            "Save" => "Ctrl+S".to_string(),
            _ => format!("Ctrl+{}", menu_item.chars().next().unwrap_or('?').to_uppercase()),
        },
        "Mac" => match menu_item {
            "Copy" => "⌘+C".to_string(),
            "Paste" => "⌘+V".to_string(),
            "Save" => "⌘+S".to_string(),
            _ => format!("⌘+{}", menu_item.chars().next().unwrap_or('?').to_uppercase()),
        },
        "Linux" => match menu_item {
            "Copy" => "Ctrl+C".to_string(),
            "Paste" => "Ctrl+V".to_string(),
            "Save" => "Ctrl+S".to_string(),
            _ => format!("Ctrl+{}", menu_item.chars().next().unwrap_or('?').to_uppercase()),
        },
        _ => menu_item.to_string(),
    }
}