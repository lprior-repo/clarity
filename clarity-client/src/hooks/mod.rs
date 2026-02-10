#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Custom hooks for Dioxus components
//!
//! Reusable stateful logic for form validation, state management,
//! loading states, and other UI patterns.

pub mod use_keyboard;
pub mod use_loading;
pub mod use_settings;
pub mod use_state;
pub mod use_undo;
pub mod use_validation;

pub use use_keyboard::{use_global_keyboard, use_keyboard, use_keyboard_with_handler, MatchResult};
pub use use_loading::{
  use_is_loading, use_is_loading_key, use_loading_batch, use_loading_manager, use_loading_message,
  use_loading_messages, use_loading_operation, use_loading_operations, use_loading_state,
  LoadingManager, LoadingOperations, LoadingState,
};
pub use use_settings::{use_settings, use_beads_per_page_validator, use_data_location_validator, SettingsActions, SettingsState};
pub use use_state::{
  use_add_bead, use_auth_actions, use_auth_state, use_bead_actions, use_bead_state, use_beads,
  use_beads_error, use_beads_loading, use_current_route, use_current_user, use_is_authenticated,
  use_theme, use_ui_actions, use_ui_state, AuthActions, BeadActions, UIActions,
};
pub use use_undo::{use_undo, use_undo_stack, UndoStackProvider};
pub use use_validation::{use_form_validation, FieldErrorState, ValidationState};
