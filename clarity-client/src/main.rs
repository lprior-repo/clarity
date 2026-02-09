#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Dioxus desktop application entry point with embedded SQLite database
//!
//! This is the main entry point for the Clarity desktop application.
//! It initializes an embedded SQLite database in the user's home directory
//! and launches the Dioxus app with local data persistence.

use anyhow::{Context, Result};
use clarity_client::db::DesktopDb;

/// Main entry point for the desktop application
///
/// This function:
/// 1. Initializes the embedded database using DesktopDb
/// 2. Launches the Dioxus desktop app
///
/// # Errors
/// Returns an error if database initialization fails
fn main() -> Result<()> {
    // Initialize the embedded SQLite database
    // This creates the database file and runs migrations if needed
    let _db = DesktopDb::new()
        .context("Failed to initialize database")?;

    eprintln!("Database initialized successfully");

    // Launch the Dioxus desktop application
    dioxus::LaunchBuilder::desktop().launch(clarity_client::App);

    Ok(())
}
