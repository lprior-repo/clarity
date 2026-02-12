// Note: We don't enforce clippy::unwrap_used at the crate level because the Dioxus rsx!
// macro internally uses unwrap(). The app module has its own lint checks for actual code.
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::cargo)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
#![warn(clippy::todo)]
#![warn(clippy::unimplemented)]
#![warn(clippy::missing_errors_doc)]
#![warn(clippy::needless_return)]
#![warn(clippy::unreadable_literal)]
#![warn(clippy::uninlined_format_args)]
#![warn(clippy::doc_markdown)]
#![warn(clippy::must_use_candidate)]
#![warn(clippy::missing_const_for_fn)]
#![warn(clippy::return_self_not_must_use)]
#![warn(clippy::should_implement_trait)]
#![warn(clippy::new_without_default)]
#![allow(clippy::cargo_common_metadata)]
#![allow(clippy::multiple_crate_versions)]

//! Clarity Client - Dioxus Desktop Application
//!
//! This is the desktop application for Clarity, built with Dioxus.
//! It provides a modern, reactive native UI for managing interviews and documentation.
//!
//! # Routing
//!
//! The application uses dioxus-router for client-side routing. All navigation
//! is handled through the `Route` enum defined in the `app` module.
//!
//! # Navigation
//!
//! Use the `Link` component from `dioxus_router` for internal navigation:
//! ```rsx
//! Link { to: Route::Dashboard {}, "Go to Dashboard" }
//! ```

pub mod app;
pub mod backup;
pub mod beads;
pub mod br_show;
pub mod components;
pub mod db;
pub mod error;
pub mod hooks;
pub mod import;
pub mod navigation;
pub mod planner;
pub mod pme;
pub mod providers;
pub mod settings;
pub mod shortcuts;
pub mod state;
pub mod undo;
pub mod validation;

pub use app::{App, NavigationLink, Route};
pub use backup::{
  auto_backup, backup_database, get_backup_directory, list_backups, restore_backup, BackupError,
  BackupInfo, BackupOptions,
};
pub use beads::{BeadDetailPage, BeadFormPage, BeadListPage};
pub use br_show::{Bd2zkShowPage, BrIssue, BrShowError, BrShowPage};
pub use components::{
  use_error_handler, ErrorBoundary, KeyboardButton, KeyboardHelpDialog, SaveButton, SettingsView,
  ShortcutHint,
};
pub use db::DesktopDb;
pub use error::{AppError, AppResult, RecoveryAction};
pub use hooks::use_keyboard::KeyEvent;
pub use hooks::{
  use_add_bead, use_bead_actions, use_bead_state, use_beads, use_beads_error, use_beads_loading,
  use_keyboard, use_theme, use_ui_actions, use_ui_state, use_undo, BeadActions, UIActions,
};
pub use import::{import_from_intent_cli, intent_cli::find_intent_cli_db};
pub use navigation::use_navigation;
pub use providers::{AppProviders, AppStateProvider, ThemeProvider};
pub use settings::{BackupFrequency, Settings, Theme as SettingsTheme};
pub use shortcuts::{Action, Key, Modifiers, Shortcut, Shortcuts};
pub use state::{AppState, BeadState, StatePersistence, Theme, UIState};
pub use undo::{
  Command, CreateBeadCommand, DatabaseAccess, DeleteBeadCommand, UndoStack, UpdateBeadCommand,
};
