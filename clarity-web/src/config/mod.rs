#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![allow(clippy::suspicious_else_formatting)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
// Public API exports - used by library consumers
#![allow(unused_imports)]

pub mod ai;

pub use ai::{
  config_path, default_config, load_ai_config, AiConfig, ConfigError, ProviderConfig, ProviderType,
  QualityConfig,
};
