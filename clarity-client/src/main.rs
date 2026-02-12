#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
#![allow(warnings)]
#![allow(clippy::all)]

//! Dioxus desktop application entry point with embedded `SQLite` database
//!
//! This is the main entry point for the Clarity desktop application.
//! It initializes an embedded `SQLite` database in the user's home directory
//! and launches the Dioxus app with local data persistence and global state management.

use anyhow::{Context, Result};
use clarity_client::db::DesktopDb;
use dioxus::prelude::*;
use std::path::PathBuf;

/// Initialize tracing subscriber for structured logging (JSONL format)
///
/// Sets up tracing with JSON Lines output (one JSON per line) for log aggregation
fn init_tracing() {
  use tracing_subscriber::{fmt, prelude::*, EnvFilter};

  let env_filter = EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into());

  tracing_subscriber::registry()
    .with(env_filter)
    .with(
      fmt::layer()
        .json()
        .with_target(false)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_file(false)
        .with_line_number(false)
        .with_writer(std::io::stdout),
    )
    .init();
}

/// Root component that wraps the entire app with providers
#[component]
fn Root() -> Element {
  // Inject CSS for the desktop app
  let css = include_str!("../public/style.css");

  rsx! {
      style { "{css}" }
      clarity_client::providers::AppProviders {
          clarity_client::app::App {}
      }
  }
}

/// Get the database path and ensure the data directory exists (blocking)
///
/// # Errors
/// Returns an error if the data directory cannot be determined or created
fn ensure_data_dir() -> Result<PathBuf> {
  let data_dir = dirs::data_local_dir()
    .ok_or_else(|| anyhow::anyhow!("Failed to determine local data directory"))?;

  let app_dir = data_dir.join("clarity");

  // Use blocking I/O for directory creation - this is safe in main()
  std::fs::create_dir_all(&app_dir).context("Failed to create data directory")?;

  let db_path = app_dir.join("clarity.db");

  eprintln!("Database path: {:?}", db_path);

  Ok(db_path)
}

/// Main entry point for the desktop application
///
/// This function:
/// 1. Initializes tracing for structured logging
/// 2. Ensures the data directory exists (using blocking I/O)
/// 3. Initializes the embedded database using `DesktopDb`
/// 4. Launches the Dioxus desktop app with global state providers
///
/// # Errors
/// Returns an error if database initialization fails
fn main() -> Result<()> {
  // Initialize tracing first for early logging
  init_tracing();

  // Ensure data directory exists using blocking I/O
  let _db_path = ensure_data_dir()?;

  // Initialize the embedded SQLite database
  // This creates the connection pool and runs migrations if needed
  let _db = DesktopDb::new().context("Failed to initialize database")?;

  tracing::info!("Application initialized successfully");

  // Launch the Dioxus desktop application with global state providers
  dioxus::LaunchBuilder::desktop()
    .with_cfg(
      dioxus::desktop::Config::new().with_window(
        dioxus::desktop::WindowBuilder::new()
          .with_title("Clarity")
          .with_inner_size(dioxus::desktop::LogicalSize::new(1400, 900)),
      ),
    )
    .launch(Root);

  Ok(())
}
