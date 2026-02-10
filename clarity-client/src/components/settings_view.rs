#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
#![allow(clippy::disallowed_methods)]

//! Settings view component for the Clarity desktop app
//!
//! Provides a comprehensive UI for managing application preferences.

use std::str::FromStr;
use dioxus::prelude::*;
use crate::hooks::use_settings::{use_settings, use_beads_per_page_validator};
use crate::settings::{Theme, BackupFrequency};
use clarity_core::db::models::{BeadPriority, BeadType};

/// Settings view component
///
/// Renders the complete settings UI with all preference controls.
#[component]
pub fn SettingsView() -> Element {
    let (settings_state, actions, save_result) = use_settings();
    let validator = use_beads_per_page_validator();
    let mut beads_per_page_error = use_signal(|| None::<String>);

    let settings = settings_state.read().settings.clone();

    rsx! {
        div { class: "settings-container",
            // Loading state
            if settings_state.read().loading {
                div { class: "settings-loading",
                    p { "Loading settings..." }
                }
            }

            // Error state
            if let Some(error) = settings_state.read().error.clone() {
                div { class: "settings-error",
                    h3 { "Error Loading Settings" }
                    p { "{error}" }
                    button {
                        class: "btn btn-secondary",
                        onclick: move |_| {
                            // Retry loading by resetting state
                            actions.reset_settings.call(());
                        },
                        "Retry"
                    }
                }
            }

            // Success notification
            if *save_result.read() == Some(true) {
                div { class: "settings-success",
                    p { "Settings saved successfully!" }
                }
            }

            // Save failure notification
            if *save_result.read() == Some(false) {
                div { class: "settings-error-inline",
                    p { "Failed to save settings. Please try again." }
                }
            }

            // Settings form
            div { class: "settings-content",
                h1 { "Settings" }
                p { class: "settings-description",
                    "Manage your Clarity application preferences"
                }

                // Appearance section
                SettingsSection {
                    title: "Appearance",
                    description: "Customize the look and feel",
                    ThemeSelector {
                        theme: settings.theme,
                        on_change: actions.update_theme
                    }
                }

                // Defaults section
                SettingsSection {
                    title: "Default Bead Settings",
                    description: "Set default values for new beads",
                    PrioritySelector {
                        priority: settings.default_priority,
                        on_change: actions.update_priority
                    }
                    TypeSelector {
                        bead_type: settings.default_type,
                        on_change: actions.update_type
                    }
                }

                // Pagination section
                SettingsSection {
                    title: "Pagination",
                    description: "Control how many beads to display per page",
                    BeadsPerPageInput {
                        value: settings.beads_per_page,
                        error: beads_per_page_error.read().clone(),
                        validator: validator,
                        on_change: move |value| {
                            if validator.call(value) {
                                actions.update_beads_per_page.call(value);
                                *beads_per_page_error.write() = None;
                            } else {
                                *beads_per_page_error.write() = Some(
                                    "Must be between 5 and 100".to_string()
                                );
                            }
                        }
                    }
                }

                // Backup section
                SettingsSection {
                    title: "Backup",
                    description: "Configure automatic backup settings",
                    AutoBackupCheckbox {
                        enabled: settings.auto_backup,
                        on_change: actions.update_auto_backup
                    }
                    if settings.auto_backup {
                        BackupFrequencySelector {
                            frequency: settings.backup_frequency,
                            on_change: actions.update_backup_frequency
                        }
                    }
                }

                // Data location section
                SettingsSection {
                    title: "Data Location",
                    description: "Where Clarity stores your data",
                    DataLocationDisplay {
                        path: settings.data_location.clone()
                    }
                }

                // Keyboard shortcuts section
                SettingsSection {
                    title: "Keyboard Shortcuts",
                    description: "View and customize keyboard shortcuts",
                    KeyboardShortcutsList {
                        shortcuts: settings.keyboard_shortcuts
                    }
                }

                // Action buttons
                div { class: "settings-actions",
                    button {
                        class: "btn btn-secondary",
                        onclick: move |_| {
                            actions.reset_settings.call(());
                        },
                        "Reset to Defaults"
                    }
                    button {
                        class: "btn btn-primary",
                        onclick: move |_| {
                            actions.save_settings.call(());
                        },
                        "Save Settings"
                    }
                }
            }
        }
    }
}

/// Settings section container
#[component]
fn SettingsSection(
    title: String,
    description: String,
    children: Element,
) -> Element {
    rsx! {
        div { class: "settings-section",
            h2 { class: "settings-section-title", "{title}" }
            p { class: "settings-section-description", "{description}" }
            div { class: "settings-section-content",
                {children}
            }
        }
    }
}

/// Theme selector radio buttons
#[component]
fn ThemeSelector(
    theme: Theme,
    on_change: Callback<Theme>,
) -> Element {
    rsx! {
        div { class: "setting-item",
            label { class: "setting-label", "Theme" }
            div { class: "theme-options",
                ThemeOption {
                    value: Theme::Light,
                    current: theme,
                    on_change: on_change,
                    label: "Light",
                    description: "Light color scheme"
                }
                ThemeOption {
                    value: Theme::Dark,
                    current: theme,
                    on_change: on_change,
                    label: "Dark",
                    description: "Dark color scheme"
                }
                ThemeOption {
                    value: Theme::System,
                    current: theme,
                    on_change: on_change,
                    label: "System",
                    description: "Follow system theme"
                }
            }
        }
    }
}

/// Individual theme radio option
#[component]
fn ThemeOption(
    value: Theme,
    current: Theme,
    on_change: Callback<Theme>,
    label: String,
    description: String,
) -> Element {
    let is_selected = value == current;

    rsx! {
        label { class: format!("theme-option {}", if is_selected { "selected" } else { "" }),
            input {
                r#type: "radio",
                name: "theme",
                value: value.as_str(),
                checked: is_selected,
                onchange: move |e: Event<FormData>| {
                    if let Ok(theme) = Theme::from_str(&e.value()) {
                        on_change.call(theme);
                    }
                }
            }
            div { class: "theme-option-content",
                span { class: "theme-option-label", "{label}" }
                span { class: "theme-option-description", "{description}" }
            }
        }
    }
}

/// Priority dropdown selector
#[component]
fn PrioritySelector(
    priority: BeadPriority,
    on_change: Callback<BeadPriority>,
) -> Element {
    rsx! {
        div { class: "setting-item",
            label { class: "setting-label", "for": "priority-select", "Default Priority" }
            select {
                id: "priority-select",
                class: "setting-select",
                onchange: move |e: Event<FormData>| {
                    match e.value().as_str() {
                        "1" => on_change.call(BeadPriority::HIGH),
                        "2" => on_change.call(BeadPriority::MEDIUM),
                        "3" => on_change.call(BeadPriority::LOW),
                        _ => {}
                    }
                },
                option {
                    value: "1",
                    selected: priority == BeadPriority::HIGH,
                    "High"
                }
                option {
                    value: "2",
                    selected: priority == BeadPriority::MEDIUM,
                    "Medium"
                }
                option {
                    value: "3",
                    selected: priority == BeadPriority::LOW,
                    "Low"
                }
            }
        }
    }
}

/// Bead type dropdown selector
#[component]
fn TypeSelector(
    bead_type: BeadType,
    on_change: Callback<BeadType>,
) -> Element {
    rsx! {
        div { class: "setting-item",
            label { class: "setting-label", "for": "type-select", "Default Type" }
            select {
                id: "type-select",
                class: "setting-select",
                onchange: move |e: Event<FormData>| {
                    if let Ok(parsed) = BeadType::from_str(&e.value()) {
                        on_change.call(parsed);
                    }
                },
                option {
                    value: BeadType::Feature.as_str(),
                    selected: bead_type == BeadType::Feature,
                    "Feature"
                }
                option {
                    value: BeadType::Bugfix.as_str(),
                    selected: bead_type == BeadType::Bugfix,
                    "Bugfix"
                }
                option {
                    value: BeadType::Refactor.as_str(),
                    selected: bead_type == BeadType::Refactor,
                    "Refactor"
                }
                option {
                    value: BeadType::Test.as_str(),
                    selected: bead_type == BeadType::Test,
                    "Test"
                }
                option {
                    value: BeadType::Docs.as_str(),
                    selected: bead_type == BeadType::Docs,
                    "Documentation"
                }
            }
        }
    }
}

/// Number input for beads per page
#[component]
fn BeadsPerPageInput(
    value: usize,
    error: Option<String>,
    validator: Callback<usize, bool>,
    on_change: Callback<usize>,
) -> Element {
    rsx! {
        div { class: "setting-item",
            label { class: "setting-label", "for": "beads-per-page", "Beads Per Page" }
            input {
                id: "beads-per-page",
                r#type: "number",
                class: format!("setting-input {}", if error.is_some() { "invalid" } else { "" }),
                value: "{value}",
                min: 5,
                max: 100,
                onchange: move |e: Event<FormData>| {
                    if let Ok(parsed) = e.value().parse::<usize>() {
                        on_change.call(parsed);
                    }
                }
            }
            if let Some(err) = error.as_ref() {
                span { class: "setting-error", "{err}" }
            } else {
                span { class: "setting-hint", "Enter a value between 5 and 100" }
            }
        }
    }
}

/// Checkbox for auto backup
#[component]
fn AutoBackupCheckbox(
    enabled: bool,
    on_change: Callback<bool>,
) -> Element {
    rsx! {
        div { class: "setting-item",
            label { class: "setting-label checkbox-label",
                input {
                    r#type: "checkbox",
                    class: "setting-checkbox",
                    checked: enabled,
                    onchange: move |e: Event<FormData>| {
                        on_change.call(e.checked());
                    }
                }
                span { "Enable automatic backups" }
            }
        }
    }
}

/// Backup frequency selector
#[component]
fn BackupFrequencySelector(
    frequency: BackupFrequency,
    on_change: Callback<BackupFrequency>,
) -> Element {
    rsx! {
        div { class: "setting-item",
            label { class: "setting-label", "for": "backup-frequency", "Backup Frequency" }
            select {
                id: "backup-frequency",
                class: "setting-select",
                onchange: move |e: Event<FormData>| {
                    if let Ok(parsed) = BackupFrequency::from_str(&e.value()) {
                        on_change.call(parsed);
                    }
                },
                option {
                    value: BackupFrequency::Hourly.as_str(),
                    selected: frequency == BackupFrequency::Hourly,
                    "Hourly"
                }
                option {
                    value: BackupFrequency::Daily.as_str(),
                    selected: frequency == BackupFrequency::Daily,
                    "Daily"
                }
                option {
                    value: BackupFrequency::Weekly.as_str(),
                    selected: frequency == BackupFrequency::Weekly,
                    "Weekly"
                }
                option {
                    value: BackupFrequency::Never.as_str(),
                    selected: frequency == BackupFrequency::Never,
                    "Never"
                }
            }
        }
    }
}

/// Display current data location with warning
#[component]
fn DataLocationDisplay(
    path: std::path::PathBuf,
) -> Element {
    let path_str = path.display().to_string();

    rsx! {
        div { class: "setting-item",
            label { class: "setting-label", "Data Location" }
            div { class: "data-location-display",
                code { class: "data-location-path", "{path_str}" }
            }
            div { class: "setting-warning",
                strong { "Warning: " }
                span { "Changing the data location will not move existing data. "
                    "You must manually migrate your data if you change this location." }
            }
        }
    }
}

/// List of keyboard shortcuts
#[component]
fn KeyboardShortcutsList(
    shortcuts: std::collections::HashMap<String, String>,
) -> Element {
    // Convert to sorted vec for display
    let sorted_shortcuts: Vec<(String, String)> = shortcuts
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();

    rsx! {
        div { class: "keyboard-shortcuts-list",
            {sorted_shortcuts.iter().map(|(action, shortcut)| {
                rsx! {
                    div { class: "keyboard-shortcut-item",
                        span { class: "shortcut-action", "{action}" }
                        kbd { class: "shortcut-key", "{shortcut}" }
                    }
                }
            })}
        }
    }
}
