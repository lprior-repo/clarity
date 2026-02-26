#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

pub mod ai;

pub use ai::{
    load_ai_config,
    default_config,
    config_path,
    AiConfig,
    ProviderConfig,
    QualityConfig,
    ProviderType,
    ConfigError,
};
