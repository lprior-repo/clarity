#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! UI components for the Clarity desktop application
//!
//! This module contains reusable UI components including loading indicators,
//! error displays, and the error boundary system.

pub mod error_boundary;
pub mod error_display;
pub mod keyboard_help;
pub mod loading;
pub mod settings_view;

pub use error_boundary::{use_error_handler, ErrorBoundary};
pub use error_display::{
  ErrorBanner, ErrorDisplay, ErrorDisplayProps, ErrorInline, ErrorPage, ErrorVariant, FormError,
};
pub use keyboard_help::{use_keyboard_help, KeyboardHelpDialog, ShortcutHint};
pub use loading::{
  CardSkeleton, Loading, LoadingInline, LoadingPage, LoadingProps, LoadingSize, LoadingVariant,
};
pub use settings_view::SettingsView;
