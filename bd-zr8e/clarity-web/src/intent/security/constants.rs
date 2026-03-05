#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

pub(super) const MAX_SESSION_ID_LENGTH: usize = 499;

pub(super) const SHELL_METACHARACTERS: &[char] = &[
  ';', '|', '&', '$', '`', '(', ')', '{', '}', '[', ']', '<', '>', '\\', '!', '*', '?', '"', '\'',
];

pub(super) const CONTROL_CHAR_MAX: u8 = 32;
