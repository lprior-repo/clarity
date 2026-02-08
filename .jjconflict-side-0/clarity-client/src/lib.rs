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

//! Clarity Client - Dioxus Frontend Application
//!
//! This is the web frontend for Clarity, built with Dioxus.
//! It provides a modern, reactive UI for managing interviews and documentation.

pub mod app;

pub use app::{App, AppError, AppState};
