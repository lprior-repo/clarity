#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

pub mod ai;

pub use ai::{
  config_path, default_config, load_ai_config, AiConfig, ConfigError, ProviderConfig, ProviderType,
  QualityConfig,
};
