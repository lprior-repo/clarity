#![forbid(unsafe_code)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
// Binary crate - modules contain public items for future use
#![allow(dead_code)]
#![allow(unused_imports)]

// Import main app component
mod app;
mod components;
mod config;
mod hooks;
mod kirk;
mod lattice;
mod providers;
mod server;
mod storage;
mod types;
mod ui;

use app::App;
use dioxus::prelude::*;

fn main() {
  // Init logger with graceful fallback - logger failure should not crash the app
  // We simply proceed without structured logging if initialization fails
  if dioxus_logger::init(dioxus_logger::tracing::Level::INFO).is_err() {
    eprintln!("Warning: Failed to initialize logger, continuing without structured logging");
  }

  // Launch the fullstack app
  dioxus::launch(App);
}
mod intent;
