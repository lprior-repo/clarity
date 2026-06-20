#![warn(clippy::pedantic)]
#![allow(clippy::suspicious_else_formatting)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
// Library crate - public items may not be used internally but are part of the public API
#![allow(dead_code)]

pub mod config;
pub mod domain;
pub mod intent;
pub mod kirk;
pub mod lattice;
pub mod providers;
pub mod server;
pub mod storage;
pub mod types;
