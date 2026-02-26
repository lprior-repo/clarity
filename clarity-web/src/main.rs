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
    // Init logger
    dioxus_logger::init(dioxus_logger::tracing::Level::INFO).expect("Failed to init logger");

    // Launch the fullstack app
    dioxus::launch(App);
}
