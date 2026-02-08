#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

mod models_test;

// TODO: Re-enable integration tests when repository functions are implemented
// The integration tests require CRUD functions (create_user, get_user, etc.)
// that don't exist yet. These will be re-enabled after the repository module
// is implemented.
// #[cfg(feature = "integration-tests")]
// mod integration_test;
