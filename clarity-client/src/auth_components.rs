#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
// Dioxus rsx! macro internally uses unwrap, so we allow the disallowed_methods lint.
#![allow(clippy::disallowed_methods)]

//! Authentication UI components for Clarity
//!
//! Provides login and registration forms with secure password handling.

use crate::app::{NavLink, Route};
use dioxus::prelude::*;

/// Authentication state managed by the parent app
#[derive(Clone, Debug, PartialEq, Eq)]
#[derive(Default)]
pub struct AuthState {
  pub is_authenticated: bool,
  pub user_email: Option<String>,
  pub session_token: Option<String>,
}


/// Login form component
#[component]
pub fn LoginForm(
  auth_state: Signal<AuthState>,
  on_login: Callback<(String, String)>,
  error_message: Signal<String>,
) -> Element {
  let mut email = use_signal(String::new);
  let mut password = use_signal(String::new);
  let mut is_loading = use_signal(|| false);

  let handle_submit = move |evt: Event<FormData>| {
    evt.prevent_default();
    let email_val = email.read().clone();
    let password_val = password.read().clone();

    // Basic validation
    if email_val.trim().is_empty() {
      error_message.set("Please enter your email".to_string());
      return;
    }

    if password_val.is_empty() {
      error_message.set("Please enter your password".to_string());
      return;
    }

    is_loading.set(true);
    error_message.set(String::new());

    // Call the login callback
    on_login((email_val, password_val));
  };

  rsx! {
      div { class: "auth-container",
          div { class: "auth-card",
              h2 { "Sign In" }
              p { class: "auth-subtitle", "Welcome back to Clarity" }

              if !error_message.read().is_empty() {
                  div { class: "error-message",
                      {error_message.read().as_str()}
                  }
              }

              form {
                  onsubmit: handle_submit,
                  class: "auth-form",

                  div { class: "form-group",
                      label { r#for: "email", "Email" }
                      input {
                          r#type: "text",
                          id: "email",
                          name: "email",
                          value: "{email}",
                          oninput: move |evt| email.set(evt.value()),
                          placeholder: "you@example.com",
                          required: true,
                          disabled: *is_loading.read()
                      }
                  }

                  div { class: "form-group",
                      label { r#for: "password", "Password" }
                      input {
                          r#type: "password",
                          id: "password",
                          name: "password",
                          value: "{password}",
                          oninput: move |evt| password.set(evt.value()),
                          placeholder: "Enter your password",
                          required: true,
                          disabled: *is_loading.read()
                      }
                  }

                  button {
                      r#type: "submit",
                      class: "btn btn-primary",
                      disabled: *is_loading.read(),
                      if *is_loading.read() {
                          "Signing in..."
                      } else {
                          "Sign In"
                      }
                  }
              }

              div { class: "auth-footer",
                  p { "Don't have an account? " }
                  NavLink { to: Route::Register, "Register" }
              }
          }
      }
  }
}

/// Registration form component
#[component]
pub fn RegisterForm(
  auth_state: Signal<AuthState>,
  on_register: Callback<(String, String, String)>,
  error_message: Signal<String>,
) -> Element {
  let mut email = use_signal(String::new);
  let mut password = use_signal(String::new);
  let mut confirm_password = use_signal(String::new);
  let mut is_loading = use_signal(|| false);

  let validate_password = |pwd: &str| -> Result<(), String> {
    if pwd.len() < 12 {
      return Err("Password must be at least 12 characters".to_string());
    }
    if !pwd.chars().any(|c| c.is_ascii_lowercase()) {
      return Err("Password must contain at least one lowercase letter".to_string());
    }
    if !pwd.chars().any(|c| c.is_ascii_uppercase()) {
      return Err("Password must contain at least one uppercase letter".to_string());
    }
    if !pwd.chars().any(|c| c.is_ascii_digit()) {
      return Err("Password must contain at least one digit".to_string());
    }
    if !pwd.chars().any(|c| c.is_ascii_punctuation()) {
      return Err("Password must contain at least one special character".to_string());
    }
    Ok(())
  };

  let handle_submit = move |evt: Event<FormData>| {
    evt.prevent_default();
    let email_val = email.read().clone();
    let password_val = password.read().clone();
    let confirm_val = confirm_password.read().clone();

    // Basic validation
    if email_val.trim().is_empty() {
      error_message.set("Please enter your email".to_string());
      return;
    }

    if password_val.is_empty() {
      error_message.set("Please enter a password".to_string());
      return;
    }

    // Validate password strength
    if let Err(e) = validate_password(&password_val) {
      error_message.set(e);
      return;
    }

    if confirm_val != password_val {
      error_message.set("Passwords do not match".to_string());
      return;
    }

    is_loading.set(true);
    error_message.set(String::new());

    // Call the register callback
    on_register((email_val, password_val, confirm_val));
  };

  rsx! {
      div { class: "auth-container",
          div { class: "auth-card",
              h2 { "Create Account" }
              p { class: "auth-subtitle", "Get started with Clarity" }

              if !error_message.read().is_empty() {
                  div { class: "error-message",
                      {error_message.read().as_str()}
                  }
              }

              form {
                  onsubmit: handle_submit,
                  class: "auth-form",

                  div { class: "form-group",
                      label { r#for: "email", "Email" }
                      input {
                          r#type: "email",
                          id: "email",
                          name: "email",
                          value: "{email}",
                          oninput: move |evt| email.set(evt.value()),
                          placeholder: "you@example.com",
                          required: true,
                          disabled: *is_loading.read()
                      }
                  }

                  div { class: "form-group",
                      label { r#for: "password", "Password" }
                      input {
                          r#type: "password",
                          id: "password",
                          name: "password",
                          value: "{password}",
                          oninput: move |evt| password.set(evt.value()),
                          placeholder: "At least 12 characters, mixed case, numbers, symbols",
                          required: true,
                          disabled: *is_loading.read()
                      }
                      p { class: "form-hint",
                          "Must be at least 12 characters with uppercase, lowercase, numbers, and symbols"
                      }
                  }

                  div { class: "form-group",
                      label { r#for: "confirm_password", "Confirm Password" }
                      input {
                          r#type: "password",
                          id: "confirm_password",
                          name: "confirm_password",
                          value: "{confirm_password}",
                          oninput: move |evt| confirm_password.set(evt.value()),
                          placeholder: "Confirm your password",
                          required: true,
                          disabled: *is_loading.read()
                      }
                  }

                  button {
                      r#type: "submit",
                      class: "btn btn-primary",
                      disabled: *is_loading.read(),
                      if *is_loading.read() {
                          "Creating account..."
                      } else {
                          "Create Account"
                      }
                  }
              }

              div { class: "auth-footer",
                  p { "Already have an account? " }
                  NavLink { to: Route::Login, "Sign In" }
              }
          }
      }
  }
}

/// Authentication hook for managing session state
///
/// This hook provides authentication state and handlers for login/logout operations.
/// In a desktop app, session data is stored in memory.
pub fn use_auth() -> (Signal<AuthState>, Callback<(String, String)>, Callback<()>) {
  let mut auth_state = use_signal(AuthState::default);

  // Load session from storage on mount
  use_effect(move || {
    // Check for existing session in storage
    // For desktop app, we could use local storage or a simple in-memory check
  });

  let login_callback = Callback::new(move |(email, _password): (String, String)| {
    let new_auth = AuthState {
      is_authenticated: true,
      user_email: Some(email),
      session_token: Some(clarity_core::auth::generate_session_token()),
    };
    auth_state.set(new_auth);
  });

  let logout_callback = Callback::new(move |_event: ()| {
    let new_auth = AuthState {
      is_authenticated: false,
      user_email: None,
      session_token: None,
    };
    auth_state.set(new_auth);
  });

  (auth_state, login_callback, logout_callback)
}

/// Protected route component that requires authentication
///
/// Redirects to login if not authenticated, otherwise renders children.
#[component]
pub fn ProtectedRoute(auth_state: Signal<AuthState>, children: Element) -> Element {
  if auth_state.read().is_authenticated {
    children
  } else {
    rsx! {
        div { class: "auth-required",
            h2 { "Authentication Required" }
            p { "Please sign in to access this page." }
            a {
    class: "btn btn-primary nav-link",
    href: "/login",
    "Sign In"
}
        }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_auth_state_default() {
    let state = AuthState::default();
    assert!(!state.is_authenticated);
    assert!(state.user_email.is_none());
    assert!(state.session_token.is_none());
  }

  #[test]
  fn test_auth_state_authenticated() {
    let state = AuthState {
      is_authenticated: true,
      user_email: Some("test@example.com".to_string()),
      session_token: Some("token123".to_string()),
    };
    assert!(state.is_authenticated);
    assert_eq!(state.user_email, Some("test@example.com".to_string()));
    assert_eq!(state.session_token, Some("token123".to_string()));
  }
}
