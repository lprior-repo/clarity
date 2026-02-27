#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

pub mod ai;

pub use ai::{
  config_path, default_config, load_ai_config, AiConfig, ConfigError, ProviderConfig, ProviderType,
  QualityConfig,
};
