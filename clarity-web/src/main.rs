#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

use dioxus::prelude::*;

// Import main app component
mod app;
mod components;
#[cfg(feature = "fullstack")]
mod server;
mod types;
mod ui;

use app::App;

fn main() {
    // Init logger
    dioxus_logger::init(dioxus_logger::tracing::Level::INFO).expect("Failed to init logger");

    // Web launch
    dioxus::LaunchBuilder::web().launch(App);
}
