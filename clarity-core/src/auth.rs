#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Authentication module for Clarity
//!
//! Provides secure password hashing using Argon2id, password validation,
//! and session token generation for the desktop application.

use argon2::{
  password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
  Algorithm, Argon2, Params, Version,
};
use thiserror::Error;

/// Authentication errors (domain errors - use thiserror)
#[derive(Debug, Error)]
pub enum AuthError {
  #[error("password is too weak: {0}")]
  WeakPassword(String),

  #[error("password hash is invalid")]
  InvalidHash,

  #[error("password verification failed")]
  VerificationFailed,

  #[error("email is required for user creation")]
  EmailRequired,
}

/// Authentication result type
pub type AuthResult<T> = Result<T, AuthError>;

/// Minimum password length
const MIN_PASSWORD_LENGTH: usize = 12;

/// Argon2id parameters for secure password hashing
///
/// Using OWASP recommendations (2023):
/// - Algorithm: Argon2id (hybrid of Argon2i and Argon2d)
/// - Time cost: 3 iterations
/// - Memory cost: 64 `MiB` (65536 `KiB`)
/// - Parallelism: 4 lanes
/// - Output length: 32 bytes (256 bits)
const ARGON_PARAMS: Params = match Params::new(65536, 3, 4, None) {
  Ok(params) => params,
  Err(_) => Params::DEFAULT,
};

/// Hash a password using Argon2id
///
/// This function creates a secure password hash using Argon2id with OWASP-recommended
/// parameters. The hash includes a random salt and all parameters needed for verification.
///
/// # Errors
/// - Returns `AuthError::WeakPassword` if password doesn't meet strength requirements
/// - Returns `AuthError::InvalidHash` if hash generation fails (should not happen with valid input)
///
/// # Example
/// ```no_run
/// use clarity_core::auth;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let hash = auth::hash_password("SecureP@ssw0rd!123")?;
/// println!("Hash: {hash}");
/// # Ok(())
/// # }
/// ```
pub fn hash_password(password: &str) -> AuthResult<String> {
  // Validate password strength first
  validate_password_strength(password)?;

  // Generate random salt
  let salt = SaltString::generate(&mut OsRng);

  // Create Argon2id instance with secure parameters
  let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, ARGON_PARAMS);

  // Hash the password
  let password_hash = argon2
    .hash_password(password.as_bytes(), &salt)
    .map_err(|_| AuthError::InvalidHash)?;

  Ok(password_hash.to_string())
}

/// Verify a password against a hash
///
/// This function verifies that a password matches the stored hash using Argon2id.
/// It correctly handles hashes created with different parameters.
///
/// # Errors
/// - Returns `AuthError::InvalidHash` if the stored hash is malformed
/// - Returns `AuthError::VerificationFailed` if the password doesn't match
///
/// # Example
/// ```no_run
/// use clarity_core::auth;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let hash = auth::hash_password("SecureP@ssw0rd!123")?;
/// let is_valid = auth::verify_password(&hash, "SecureP@ssw0rd!123")?;
/// assert!(is_valid);
///
/// let is_invalid = auth::verify_password(&hash, "wrong-password")?;
/// assert!(!is_invalid);
/// # Ok(())
/// # }
/// ```
pub fn verify_password(hash: &str, password: &str) -> AuthResult<bool> {
  // Parse the stored password hash
  let parsed_hash = PasswordHash::new(hash).map_err(|_| AuthError::InvalidHash)?;

  // Verify the password using Argon2 (defaults to Argon2id)
  let argon2 = Argon2::default();

  // Attempt verification
  match argon2.verify_password(password.as_bytes(), &parsed_hash) {
    Ok(()) => Ok(true),
    Err(argon2::password_hash::Error::Password) => Ok(false),
    Err(_) => Err(AuthError::VerificationFailed),
  }
}

/// Validate password strength
///
/// Passwords must meet the following requirements:
/// - Minimum 12 characters
/// - Contains at least one lowercase letter
/// - Contains at least one uppercase letter
/// - Contains at least one digit
/// - Contains at least one special character
///
/// # Errors
/// - Returns `AuthError::WeakPassword` with details if password doesn't meet requirements
///
/// # Example
/// ```no_run
/// use clarity_core::auth;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // Strong password - passes validation
/// auth::validate_password_strength("SecureP@ssw0rd")?;
///
/// // Weak password - fails validation
/// assert!(auth::validate_password_strength("weak").is_err());
/// # Ok(())
/// # }
/// ```
pub fn validate_password_strength(password: &str) -> AuthResult<()> {
  let mut errors = Vec::new();

  // Check minimum length
  if password.len() < MIN_PASSWORD_LENGTH {
    errors.push(format!("must be at least {MIN_PASSWORD_LENGTH} characters"));
  }

  // Check for lowercase letter
  if !password.chars().any(|c| c.is_ascii_lowercase()) {
    errors.push("must contain at least one lowercase letter".to_string());
  }

  // Check for uppercase letter
  if !password.chars().any(|c| c.is_ascii_uppercase()) {
    errors.push("must contain at least one uppercase letter".to_string());
  }

  // Check for digit
  if !password.chars().any(|c| c.is_ascii_digit()) {
    errors.push("must contain at least one digit".to_string());
  }

  // Check for special character
  if !password.chars().any(|c| c.is_ascii_punctuation()) {
    errors.push("must contain at least one special character".to_string());
  }

  if errors.is_empty() {
    Ok(())
  } else {
    let error_msg = errors.join(", ");
    Err(AuthError::WeakPassword(error_msg))
  }
}

/// Generate a secure session token using UUID v4
///
/// This function generates a cryptographically random session token
/// suitable for desktop application session management.
///
/// # Example
/// ```no_run
/// use clarity_core::auth;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let token = auth::generate_session_token();
/// println!("Session token: {token}");
/// # Ok(())
/// # }
/// ```
#[must_use]
pub fn generate_session_token() -> String {
  uuid::Uuid::new_v4().to_string()
}
