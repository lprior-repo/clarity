#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

pub(super) const MAX_SESSION_ID_LENGTH: usize = 499;

pub(super) const SHELL_METACHARACTERS: &[char] = &[
  ';', '|', '&', '$', '`', '(', ')', '{', '}', '[', ']', '<', '>', '\\', '!', '*', '?', '"', '\'',
];

pub(super) const CONTROL_CHAR_MAX: u8 = 32;
