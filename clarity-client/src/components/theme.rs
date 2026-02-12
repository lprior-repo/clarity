#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
// Dioxus rsx! macro internally uses unwrap
#![allow(clippy::disallowed_methods)]

//! Theme system for the Clarity desktop application
//!
//! This module provides a theme management system with:
//! - Light, Dark, and System (follows OS preference) themes
//! - Persistent theme choice via local storage
//! - CSS class application to root element
//! - Functional patterns with no unwrap/mut on signals
//!
//! # Example
//! ```ignore
//! // In your app root, wrap with ThemeProvider
//! ThemeProvider {
//!     App {}
//! }
//!
//! // In any component, use the hook
//! let theme = use_theme();
//! let current = theme.get();
//! theme.set(Theme::Dark);
//! theme.toggle();
//! let is_dark = theme.is_dark();
//! ```

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

/// Local storage key for persisting theme preference
const THEME_STORAGE_KEY: &str = "clarity-theme";

/// Theme variants available in the application
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Theme {
  /// Light theme
  Light,
  /// Dark theme
  Dark,
  /// Follow system preference
  #[default]
  System,
}

impl Theme {
  /// Returns the CSS class name for this theme
  #[must_use]
  pub const fn as_class(&self) -> &'static str {
    match self {
      Self::Light => "light",
      Self::Dark => "dark",
      Self::System => "system",
    }
  }

  /// Returns the display label for this theme
  #[must_use]
  pub const fn as_label(&self) -> &'static str {
    match self {
      Self::Light => "Light",
      Self::Dark => "Dark",
      Self::System => "System",
    }
  }

  /// Returns the icon path for this theme (SVG path data)
  #[must_use]
  pub const fn icon_path(&self) -> &'static str {
    match self {
            Self::Light => "M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707M16 12a4 4 0 11-8 0 4 4 0 018 0z",
            Self::Dark => "M20.354 15.354A9 9 0 018.646 3.646 9.003 9.003 0 0012 21a9.003 9.003 0 008.354-5.646z",
            Self::System => "M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z",
        }
  }

  /// Converts the theme to the resolved theme (Light or Dark)
  /// System theme resolves based on the provided system preference
  #[must_use]
  pub const fn resolve(&self, system_preference: Self) -> Self {
    match self {
      Self::Light => Self::Light,
      Self::Dark => Self::Dark,
      Self::System => match system_preference {
        Self::Light => Self::Light,
        Self::Dark | Self::System => Self::Dark, // Default System to Dark
      },
    }
  }

  /// Returns true if this theme is Dark
  #[must_use]
  pub const fn is_dark(&self) -> bool {
    matches!(self, Self::Dark)
  }

  /// Returns true if this theme is Light
  #[must_use]
  pub const fn is_light(&self) -> bool {
    matches!(self, Self::Light)
  }

  /// Returns true if this theme follows system preference
  #[must_use]
  pub const fn is_system(&self) -> bool {
    matches!(self, Self::System)
  }

  /// Returns all available themes as a slice
  #[must_use]
  pub const fn all() -> &'static [Self] {
    &[Self::Light, Self::Dark, Self::System]
  }
}

/// Global theme state
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ThemeState {
  /// The user's selected theme preference
  pub current_theme: Signal<Theme>,
  /// The detected system preference (Light or Dark)
  pub system_preference: Signal<Theme>,
}

impl ThemeState {
  /// Creates a new `ThemeState` with the given signals
  #[must_use]
  pub const fn new(current_theme: Signal<Theme>, system_preference: Signal<Theme>) -> Self {
    Self {
      current_theme,
      system_preference,
    }
  }

  /// Returns the resolved theme (Light or Dark)
  #[must_use]
  pub fn resolved(&self) -> Theme {
    self
      .current_theme
      .read()
      .resolve(*self.system_preference.read())
  }
}

/// Controller for managing theme state
///
/// This controller wraps a `ThemeState` and provides
/// methods for getting and setting the theme. The Signal
/// uses interior mutability, so methods take `&self`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ThemeController {
  state: ThemeState,
}

impl ThemeController {
  /// Creates a new `ThemeController` with the given state
  #[must_use]
  pub const fn new(state: ThemeState) -> Self {
    Self { state }
  }

  /// Gets the current theme preference
  #[must_use]
  pub fn get(&self) -> Theme {
    *self.state.current_theme.read()
  }

  /// Gets the resolved theme (Light or Dark, never System)
  #[must_use]
  pub fn resolved(&self) -> Theme {
    self.state.resolved()
  }

  /// Gets the system preference (Light or Dark)
  #[must_use]
  pub fn system_preference(&self) -> Theme {
    *self.state.system_preference.read()
  }

  /// Sets the theme preference
  pub fn set(&self, theme: Theme) {
    let mut signal = self.state.current_theme;
    signal.write().clone_from(&theme);
    // Persist to local storage
    Self::persist_theme(theme);
    // Apply CSS class
    Self::apply_theme_to_dom(theme, *self.state.system_preference.read());
  }

  /// Toggles between Light and Dark themes
  /// If currently System, toggles based on resolved theme
  pub fn toggle(&self) {
    let resolved = self.resolved();
    let new_theme = match resolved {
      Theme::Light => Theme::Dark,
      Theme::Dark | Theme::System => Theme::Light,
    };
    self.set(new_theme);
  }

  /// Cycles through Light -> Dark -> System -> Light
  pub fn cycle(&self) {
    let current = self.get();
    let new_theme = match current {
      Theme::Light => Theme::Dark,
      Theme::Dark => Theme::System,
      Theme::System => Theme::Light,
    };
    self.set(new_theme);
  }

  /// Returns true if the resolved theme is Dark
  #[must_use]
  pub fn is_dark(&self) -> bool {
    self.resolved().is_dark()
  }

  /// Returns true if the resolved theme is Light
  #[must_use]
  pub fn is_light(&self) -> bool {
    self.resolved().is_light()
  }

  /// Returns true if the current preference is System
  #[must_use]
  pub fn is_system(&self) -> bool {
    self.get().is_system()
  }

  /// Persists theme to local storage
  fn persist_theme(theme: Theme) {
    let serialized = serde_json::to_string(&theme);
    if let Ok(json) = serialized {
      // Use Dioxus eval to call localStorage
      _ = dioxus::document::eval(&format!(
        "localStorage.setItem('{THEME_STORAGE_KEY}', '{json}');"
      ));
    }
  }

  /// Loads theme from local storage
  const fn load_persisted_theme() -> Option<Theme> {
    // We can't synchronously get localStorage in Dioxus, so we'll
    // use a JS eval that returns the value
    None // Handled via use_effect in provider
  }

  /// Applies the theme CSS class to the DOM root element
  fn apply_theme_to_dom(theme: Theme, system_preference: Theme) {
    let resolved = theme.resolve(system_preference);
    let class = resolved.as_class();
    _ = dioxus::document::eval(&format!(
      r"
            (function() {{
                const root = document.documentElement;
                root.classList.remove('light', 'dark');
                root.classList.add('{class}');
            }})();
            "
    ));
  }
}

/// Hook to access the theme controller
///
/// This hook provides access to the global theme state and methods
/// to get, set, and toggle the theme.
///
/// # Panics
/// This hook will panic if used outside of a `ThemeProvider` context.
#[must_use]
pub fn use_theme() -> ThemeController {
  use_context::<ThemeController>()
}

/// Props for `ThemeProvider` component
#[derive(Clone, Debug, PartialEq, Props)]
pub struct ThemeProviderProps {
  /// Child components
  children: Element,
}

/// Provider component that wraps the app and provides theme state
///
/// This component must be placed at the root of your app (or near it)
/// to enable theme functionality throughout the component tree.
///
/// # Example
/// ```ignore
/// ThemeProvider {
///     Router {
///         App {}
///     }
/// }
/// ```
#[component]
pub fn ThemeProvider(props: ThemeProviderProps) -> Element {
  // Initialize theme state with defaults
  let current_theme = use_signal(|| Theme::default());
  let system_preference = use_signal(|| Theme::Light);

  // Create the state and controller
  let state = use_hook(|| ThemeState::new(current_theme, system_preference));
  let controller = use_hook(|| ThemeController::new(state));

  // Provide the controller to all child components
  use_context_provider(|| controller);

  // Effect to detect system preference and load persisted theme
  let mut current_theme_for_effect = current_theme;
  let mut system_pref_for_effect = system_preference;
  use_effect(move || {
    // Detect system preference
    spawn(async move {
      let eval_result = dioxus::document::eval(
        r"
                (function() {
                    // Check for persisted theme
                    const stored = localStorage.getItem('clarity-theme');
                    return stored;
                })();
                ",
      );

      // Get persisted theme from JS
      if let Ok(value) = eval_result.await {
        if let Some(json_str) = value.as_str() {
          if let Ok(theme) = serde_json::from_str::<Theme>(json_str) {
            current_theme_for_effect.write().clone_from(&theme);
          }
        }
      }

      // Detect system preference
      let system_eval = dioxus::document::eval(
        r#"
                window.matchMedia('(prefers-color-scheme: dark)').matches ? "Dark" : "Light"
                "#,
      );
      if let Ok(value) = system_eval.await {
        let pref = match value.as_str() {
          Some("Dark") => Theme::Dark,
          _ => Theme::Light,
        };
        system_pref_for_effect.write().clone_from(&pref);
      }

      // Apply the resolved theme
      let resolved = current_theme_for_effect
        .read()
        .resolve(*system_pref_for_effect.read());
      let class = resolved.as_class();
      _ = dioxus::document::eval(&format!(
        r"
                (function() {{
                    const root = document.documentElement;
                    root.classList.remove('light', 'dark');
                    root.classList.add('{class}');
                }})();
                "
      ));
    });
  });

  rsx! {
      {props.children}
  }
}

/// Props for `ThemeToggle` component
#[derive(Clone, Debug, PartialEq, Eq, Props)]
pub struct ThemeToggleProps {
  /// Optional additional CSS class
  #[props(default = String::new())]
  pub class: String,
  /// Show label alongside icon
  #[props(default = false)]
  pub show_label: bool,
  /// Use cycle mode (Light -> Dark -> System) instead of toggle (Light <-> Dark)
  #[props(default = false)]
  pub cycle_mode: bool,
}

/// Toggle component for switching themes
///
/// Displays a button that shows the current theme icon
/// and allows toggling between themes.
#[component]
pub fn ThemeToggle(props: ThemeToggleProps) -> Element {
  let theme = use_theme();
  let current = theme.get();
  let resolved = theme.resolved();
  let icon_path = resolved.icon_path();
  let label = current.as_label();

  let handle_click = move |_| {
    if props.cycle_mode {
      theme.cycle();
    } else {
      theme.toggle();
    }
  };

  let button_class = format!("theme-toggle {}", props.class);

  rsx! {
      button {
          class: "{button_class}",
          onclick: handle_click,
          title: "Current theme: {label}. Click to switch.",
          "aria-label": "Toggle theme",
          svg {
              xmlns: "http://www.w3.org/2000/svg",
              fill: "none",
              view_box: "0 0 24 24",
              stroke: "currentColor",
              stroke_width: 2,
              class: "theme-toggle-icon",
              path {
                  d: "{icon_path}"
              }
          }
          if props.show_label {
              span { class: "theme-toggle-label", "{label}" }
          }
      }
  }
}

/// Props for `ThemeSelector` component
#[derive(Clone, Debug, PartialEq, Eq, Props)]
pub struct ThemeSelectorProps {
  /// Optional additional CSS class
  #[props(default = String::new())]
  pub class: String,
}

/// Selector component for choosing a theme
///
/// Displays all available themes as buttons.
#[component]
pub fn ThemeSelector(props: ThemeSelectorProps) -> Element {
  let theme = use_theme();
  let current = theme.get();

  let container_class = format!("theme-selector {}", props.class);

  rsx! {
      div { class: "{container_class}",
          for theme_option in Theme::all() {
              ThemeOptionButton {
                  key: "{theme_option:?}",
                  theme: *theme_option,
                  selected: *theme_option == current,
                  controller: theme,
              }
          }
      }
  }
}

/// Props for individual theme option button
#[derive(Clone, Copy, Debug, PartialEq, Eq, Props)]
pub struct ThemeOptionButtonProps {
  /// The theme this button represents
  pub theme: Theme,
  /// Whether this theme is currently selected
  pub selected: bool,
  /// The theme controller
  pub controller: ThemeController,
}

/// Individual theme option button component
#[component]
pub fn ThemeOptionButton(props: ThemeOptionButtonProps) -> Element {
  let theme_option = props.theme;
  let icon_path = theme_option.icon_path();
  let label = theme_option.as_label();
  let is_selected = props.selected;
  let controller = props.controller;

  let handle_click = move |_| {
    controller.set(theme_option);
  };

  let selected_class = if is_selected { "selected" } else { "" };
  let button_class = format!("theme-option-button {selected_class}");

  rsx! {
      button {
          class: "{button_class}",
          onclick: handle_click,
          "aria-pressed": "{is_selected}",
          title: "Switch to {label} theme",
          svg {
              xmlns: "http://www.w3.org/2000/svg",
              fill: "none",
              view_box: "0 0 24 24",
              stroke: "currentColor",
              stroke_width: 2,
              class: "theme-option-icon",
              path {
                  d: "{icon_path}"
              }
          }
          span { class: "theme-option-label", "{label}" }
      }
  }
}

/// CSS styles for the theme system
///
/// Include these styles in your application's CSS or use the `theme_styles()` function
/// to get a CSS string that can be injected.
#[must_use]
pub const fn theme_styles() -> &'static str {
  "
/* Theme CSS Variables */
:root {
  --background: 0 0% 100%;
  --foreground: 222.2 84% 4.9%;
  --card: 0 0% 100%;
  --card-foreground: 222.2 84% 4.9%;
  --primary: 221.2 83.2% 53.3%;
  --primary-foreground: 210 40% 98%;
  --secondary: 210 40% 96.1%;
  --muted: 210 40% 96.1%;
  --border: 214.3 31.8% 91.4%;
}

.dark {
  --background: 222.2 84% 4.9%;
  --foreground: 210 40% 98%;
  --card: 222.2 84% 4.9%;
  --card-foreground: 210 40% 98%;
  --primary: 217.2 91.2% 59.8%;
  --primary-foreground: 222.2 47.4% 11.2%;
  --secondary: 217.2 32.6% 17.5%;
  --muted: 217.2 32.6% 17.5%;
  --border: 217.2 32.6% 17.5%;
}

/* Apply theme colors to body */
html.light {
  background-color: hsl(var(--background));
  color: hsl(var(--foreground));
}

html.dark {
  background-color: hsl(var(--background));
  color: hsl(var(--foreground));
}

/* Theme Toggle Button */
.theme-toggle {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  padding: 0.5rem;
  background: transparent;
  border: 1px solid hsl(var(--border));
  border-radius: 0.5rem;
  cursor: pointer;
  transition: all 0.2s ease;
  color: hsl(var(--foreground));
}

.theme-toggle:hover {
  background-color: hsl(var(--muted));
}

.theme-toggle-icon {
  width: 1.25rem;
  height: 1.25rem;
}

.theme-toggle-label {
  font-size: 0.875rem;
  font-weight: 500;
}

/* Theme Selector */
.theme-selector {
  display: flex;
  gap: 0.5rem;
  padding: 0.5rem;
  background-color: hsl(var(--muted));
  border-radius: 0.75rem;
}

/* Theme Option Button */
.theme-option-button {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.25rem;
  padding: 0.75rem 1rem;
  background: transparent;
  border: 2px solid transparent;
  border-radius: 0.5rem;
  cursor: pointer;
  transition: all 0.2s ease;
  color: hsl(var(--foreground));
  min-width: 80px;
}

.theme-option-button:hover {
  background-color: hsl(var(--background) / 0.5);
}

.theme-option-button.selected {
  background-color: hsl(var(--background));
  border-color: hsl(var(--primary));
  box-shadow: 0 0 0 1px hsl(var(--primary));
}

.theme-option-icon {
  width: 1.5rem;
  height: 1.5rem;
}

.theme-option-label {
  font-size: 0.75rem;
  font-weight: 500;
}
"
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_theme_as_class() {
    assert_eq!(Theme::Light.as_class(), "light");
    assert_eq!(Theme::Dark.as_class(), "dark");
    assert_eq!(Theme::System.as_class(), "system");
  }

  #[test]
  fn test_theme_as_label() {
    assert_eq!(Theme::Light.as_label(), "Light");
    assert_eq!(Theme::Dark.as_label(), "Dark");
    assert_eq!(Theme::System.as_label(), "System");
  }

  #[test]
  fn test_theme_default() {
    assert_eq!(Theme::default(), Theme::System);
  }

  #[test]
  fn test_theme_is_dark() {
    assert!(Theme::Dark.is_dark());
    assert!(!Theme::Light.is_dark());
    assert!(!Theme::System.is_dark());
  }

  #[test]
  fn test_theme_is_light() {
    assert!(Theme::Light.is_light());
    assert!(!Theme::Dark.is_light());
    assert!(!Theme::System.is_light());
  }

  #[test]
  fn test_theme_is_system() {
    assert!(Theme::System.is_system());
    assert!(!Theme::Light.is_system());
    assert!(!Theme::Dark.is_system());
  }

  #[test]
  fn test_theme_resolve_light() {
    assert_eq!(Theme::Light.resolve(Theme::Dark), Theme::Light);
    assert_eq!(Theme::Light.resolve(Theme::Light), Theme::Light);
  }

  #[test]
  fn test_theme_resolve_dark() {
    assert_eq!(Theme::Dark.resolve(Theme::Dark), Theme::Dark);
    assert_eq!(Theme::Dark.resolve(Theme::Light), Theme::Dark);
  }

  #[test]
  fn test_theme_resolve_system() {
    assert_eq!(Theme::System.resolve(Theme::Dark), Theme::Dark);
    assert_eq!(Theme::System.resolve(Theme::Light), Theme::Light);
  }

  #[test]
  fn test_theme_all() {
    let all = Theme::all();
    assert_eq!(all.len(), 3);
    assert!(all.contains(&Theme::Light));
    assert!(all.contains(&Theme::Dark));
    assert!(all.contains(&Theme::System));
  }

  #[test]
  fn test_theme_serialization() {
    let light = Theme::Light;
    let json = serde_json::to_string(&light);
    assert!(json.is_ok());
    assert!(matches!(json.as_deref(), Ok("\"Light\"")));

    let dark = Theme::Dark;
    let json = serde_json::to_string(&dark);
    assert!(json.is_ok());
    assert!(matches!(json.as_deref(), Ok("\"Dark\"")));

    let system = Theme::System;
    let json = serde_json::to_string(&system);
    assert!(json.is_ok());
    assert!(matches!(json.as_deref(), Ok("\"System\"")));
  }

  #[test]
  fn test_theme_deserialization() {
    let light: Result<Theme, _> = serde_json::from_str("\"Light\"");
    assert!(light.is_ok());
    assert!(matches!(light, Ok(Theme::Light)));

    let dark: Result<Theme, _> = serde_json::from_str("\"Dark\"");
    assert!(dark.is_ok());
    assert!(matches!(dark, Ok(Theme::Dark)));

    let system: Result<Theme, _> = serde_json::from_str("\"System\"");
    assert!(system.is_ok());
    assert!(matches!(system, Ok(Theme::System)));
  }

  #[test]
  fn test_theme_styles_returns_string() {
    let styles = theme_styles();

    assert!(styles.contains(":root"));
    assert!(styles.contains(".dark"));
    assert!(styles.contains("--background"));
    assert!(styles.contains("--foreground"));
    assert!(styles.contains(".theme-toggle"));
    assert!(styles.contains(".theme-selector"));
    assert!(styles.contains(".theme-option-button"));
  }

  #[test]
  fn test_theme_controller_type_check() {
    // This test just ensures the ThemeController compiles with the correct types
    // Actual testing would require a Dioxus runtime
    let _ = || {
      fn component() -> Element {
        let theme = use_theme();
        let _current = theme.get();
        let _resolved = theme.resolved();
        let _system = theme.system_preference();
        theme.set(Theme::Dark);
        theme.toggle();
        theme.cycle();
        let _is_dark = theme.is_dark();
        let _is_light = theme.is_light();
        let _is_system = theme.is_system();
        rsx! { div {} }
      }
      let _ = component;
    };
  }

  #[test]
  fn test_theme_state_creation() {
    // Test that ThemeState can be created
    // In a real test, we'd need Dioxus runtime for Signal
    let _state = || {
      let current = Signal::new(Theme::Light);
      let system = Signal::new(Theme::Dark);
      ThemeState::new(current, system)
    };
  }
}
