#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

pub mod app;
pub mod components;
pub mod config;
pub mod hooks;
pub mod kirk;
pub mod lattice;
pub mod providers;
pub mod server;
pub mod storage;
pub mod types;
pub mod ui;
