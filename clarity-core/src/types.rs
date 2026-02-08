#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use std::fmt;

/// HTTP method with validation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum HttpMethod {
  Get,
  Post,
  Put,
  Patch,
  Delete,
  Head,
  Options,
}

impl HttpMethod {
  /// Parse a string into an `HttpMethod`
  ///
  /// # Errors
  /// - Returns `HttpMethodError::InvalidMethod` if the string is not a valid HTTP method
  #[allow(clippy::should_implement_trait)]
  pub fn from_str(s: &str) -> Result<Self, HttpMethodError> {
    match s.to_lowercase().as_str() {
      "get" => Ok(Self::Get),
      "post" => Ok(Self::Post),
      "put" => Ok(Self::Put),
      "patch" => Ok(Self::Patch),
      "delete" => Ok(Self::Delete),
      "head" => Ok(Self::Head),
      "options" => Ok(Self::Options),
      _ => Err(HttpMethodError::InvalidMethod(s.to_string())),
    }
  }

  /// Get the method as a lowercase string
  #[must_use]
  pub const fn as_str(&self) -> &'static str {
    match self {
      Self::Get => "get",
      Self::Post => "post",
      Self::Put => "put",
      Self::Patch => "patch",
      Self::Delete => "delete",
      Self::Head => "head",
      Self::Options => "options",
    }
  }

  /// Check if this is a safe method (doesn't modify data)
  #[must_use]
  pub const fn is_safe(&self) -> bool {
    matches!(self, Self::Get | Self::Head | Self::Options)
  }

  /// Check if this method has a body
  #[must_use]
  pub const fn has_body(&self) -> bool {
    matches!(self, Self::Post | Self::Put | Self::Patch)
  }
}

impl fmt::Display for HttpMethod {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.as_str())
  }
}

/// Error type for HTTP method parsing
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HttpMethodError {
  InvalidMethod(String),
}

impl fmt::Display for HttpMethodError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::InvalidMethod(method) => {
        write!(f, "Invalid HTTP method: {method}")
      }
    }
  }
}

impl std::error::Error for HttpMethodError {}

/// Spec name with validation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SpecName(String);

impl SpecName {
  /// Create a new `SpecName` with validation
  ///
  /// # Errors
  /// - Returns `SpecNameError::Empty` if the spec name is empty
  /// - Returns `SpecNameError::TooLong` if the spec name exceeds 255 characters
  /// - Returns `SpecNameError::InvalidCharacters` if the spec name contains invalid characters
  pub fn new(name: String) -> Result<Self, SpecNameError> {
    if name.trim().is_empty() {
      return Err(SpecNameError::Empty);
    }

    if name.len() > 255 {
      return Err(SpecNameError::TooLong(name.len()));
    }

    if !name
      .chars()
      .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    {
      return Err(SpecNameError::InvalidCharacters(name));
    }

    Ok(Self(name))
  }

  /// Get the spec name as a string slice
  #[must_use]
  pub const fn as_str(&self) -> &str {
    self.0.as_str()
  }

  /// Convert from string (may fail)
  ///
  /// # Errors
  /// - Returns a `SpecNameError` if the spec name is invalid
  #[allow(clippy::should_implement_trait)]
  pub fn from_str(s: &str) -> Result<Self, SpecNameError> {
    Self::new(s.to_string())
  }
}

impl fmt::Display for SpecName {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

/// Error type for spec name validation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpecNameError {
  Empty,
  TooLong(usize),
  InvalidCharacters(String),
}

impl fmt::Display for SpecNameError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Empty => write!(f, "Spec name cannot be empty"),
      Self::TooLong(len) => {
        write!(f, "Spec name is too long ({len} characters, max 255)")
      }
      Self::InvalidCharacters(name) => {
        write!(f, "Spec name contains invalid characters: {name}")
      }
    }
  }
}

impl std::error::Error for SpecNameError {}

/// URL with validation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Url(String);

impl Url {
  /// Create a new URL with validation
  ///
  /// # Errors
  /// - Returns `UrlError::Empty` if the URL is empty
  /// - Returns `UrlError::MissingScheme` if the URL doesn't start with http:// or https://
  /// - Returns `UrlError::TooLong` if the URL exceeds 2048 characters
  /// - Returns `UrlError::InvalidFormat` if the URL parsing fails
  /// - Returns `UrlError::InvalidScheme` if the URL uses an unsupported scheme
  pub fn new(url: String) -> Result<Self, UrlError> {
    if url.trim().is_empty() {
      return Err(UrlError::Empty);
    }

    if !url.starts_with("http://") && !url.starts_with("https://") {
      return Err(UrlError::MissingScheme);
    }

    if url.len() > 2048 {
      return Err(UrlError::TooLong(url.len()));
    }

    let parsed = url
      .parse::<url::Url>()
      .map_err(|_| UrlError::InvalidFormat)?;

    if parsed.scheme() != "http" && parsed.scheme() != "https" {
      return Err(UrlError::InvalidScheme);
    }

    Ok(Self(url))
  }

  /// Get the URL as a string slice
  #[must_use]
  pub const fn as_str(&self) -> &str {
    self.0.as_str()
  }

  /// Convert to string
  /// Convert from string (may fail)
  ///
  /// # Errors
  /// - Returns a `UrlError` if the URL is invalid
  #[must_use]
  #[allow(clippy::should_implement_trait, clippy::double_must_use)]
  pub fn from_str(s: &str) -> Result<Self, UrlError> {
    Self::new(s.to_string())
  }

  /// Get the scheme (http or https)
  #[must_use]
  pub fn scheme(&self) -> &'static str {
    if self.0.starts_with("https://") {
      "https"
    } else {
      "http"
    }
  }

  /// Parse the internal URL string into a `url::Url` for further processing
  fn parse_url(&self) -> Result<url::Url, UrlError> {
    self
      .0
      .parse::<url::Url>()
      .map_err(|_| UrlError::InvalidFormat)
  }

  /// Get the host
  #[must_use]
  pub fn host(&self) -> Option<String> {
    self
      .parse_url()
      .ok()
      .and_then(|u| u.host_str().map(std::string::ToString::to_string))
  }

  /// Get the path
  #[must_use]
  pub fn path(&self) -> String {
    self
      .parse_url()
      .map_or_else(|_| "/".to_string(), |u: url::Url| u.path().to_string())
  }

  /// Get the query string
  #[must_use]
  pub fn query(&self) -> Option<String> {
    self
      .parse_url()
      .ok()
      .and_then(|u| u.query().map(std::string::ToString::to_string))
  }

  /// Get the fragment
  #[must_use]
  pub fn fragment(&self) -> Option<String> {
    self
      .parse_url()
      .ok()
      .and_then(|u| u.fragment().map(std::string::ToString::to_string))
  }

  /// Get the full URL without fragment
  ///
  /// # Errors
  /// - Returns a `UrlError` if the URL without fragment is invalid
  pub fn without_fragment(&self) -> Result<Self, UrlError> {
    self.0.find('#').map_or_else(
      || Ok(self.clone()),
      |pos| Self::new(self.0[..pos].to_string()),
    )
  }

  /// Get a URL with a new path
  ///
  /// # Errors
  /// - Returns a `UrlError` if the resulting URL is invalid
  pub fn with_path(&self, path: &str) -> Result<Self, UrlError> {
    let base = self.parse_url()?;
    let new_url = base
      .join(path)
      .map_err(|_| UrlError::InvalidFormat)?
      .to_string();

    Self::new(new_url)
  }

  /// Get a URL with a new query parameter
  ///
  /// # Errors
  /// - Returns a `UrlError` if the resulting URL is invalid
  pub fn with_query(&self, key: &str, value: &str) -> Result<Self, UrlError> {
    let base = self.parse_url()?;
    let new_url = base
      .join(&format!("?{}={}", key, urlencoding::encode(value)))
      .map_err(|_| UrlError::InvalidFormat)?
      .to_string();

    Self::new(new_url)
  }
}

impl fmt::Display for Url {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

/// Error type for URL validation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UrlError {
  Empty,
  MissingScheme,
  TooLong(usize),
  InvalidFormat,
  InvalidScheme,
}

impl fmt::Display for UrlError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Empty => write!(f, "URL cannot be empty"),
      Self::MissingScheme => write!(f, "URL must start with http:// or https://"),
      Self::TooLong(len) => write!(f, "URL is too long ({len} characters, max 2048)"),
      Self::InvalidFormat => write!(f, "Invalid URL format"),
      Self::InvalidScheme => write!(f, "URL must use http or https scheme"),
    }
  }
}

impl std::error::Error for UrlError {}

/// Question types for surveys and forms
pub mod question;

/// Planning types for project management
pub mod planning;

#[cfg(test)]
mod tests {
  #![allow(clippy::unwrap_used)]
  #![allow(clippy::expect_used)]
  #![allow(clippy::panic)]
  use super::*;
  #[allow(clippy::redundant_clone)]
  #[allow(clippy::implicit_clone)]
  #[test]
  fn test_http_method_from_str() {
    assert_eq!(HttpMethod::from_str("GET"), Ok(HttpMethod::Get));
    assert_eq!(HttpMethod::from_str("post"), Ok(HttpMethod::Post));
    assert_eq!(HttpMethod::from_str("PUT"), Ok(HttpMethod::Put));
    assert_eq!(HttpMethod::from_str("PATCH"), Ok(HttpMethod::Patch));
    assert_eq!(HttpMethod::from_str("DELETE"), Ok(HttpMethod::Delete));
    assert_eq!(HttpMethod::from_str("HEAD"), Ok(HttpMethod::Head));
    assert_eq!(HttpMethod::from_str("OPTIONS"), Ok(HttpMethod::Options));
    assert!(HttpMethod::from_str("INVALID").is_err());
  }

  #[test]
  fn test_http_method_display() {
    assert_eq!(format!("{}", HttpMethod::Get), "get");
    assert_eq!(format!("{}", HttpMethod::Post), "post");
    assert_eq!(format!("{}", HttpMethod::Put), "put");
  }

  #[test]
  fn test_http_method_is_safe() {
    assert!(HttpMethod::Get.is_safe());
    assert!(HttpMethod::Head.is_safe());
    assert!(HttpMethod::Options.is_safe());
    assert!(!HttpMethod::Post.is_safe());
    assert!(!HttpMethod::Put.is_safe());
  }

  #[test]
  fn test_http_method_has_body() {
    assert!(!HttpMethod::Get.has_body());
    assert!(!HttpMethod::Head.has_body());
    assert!(HttpMethod::Post.has_body());
    assert!(HttpMethod::Put.has_body());
    assert!(HttpMethod::Patch.has_body());
  }

  #[test]
  fn test_spec_name_new_valid() {
    assert_eq!(
      SpecName::new("test_spec".to_string()),
      Ok(SpecName("test_spec".to_string()))
    );
    assert_eq!(
      SpecName::new("spec_123".to_string()),
      Ok(SpecName("spec_123".to_string()))
    );
    assert_eq!(
      SpecName::new("spec-456".to_string()),
      Ok(SpecName("spec-456".to_string()))
    );
  }

  #[test]
  fn test_spec_name_empty() {
    assert!(SpecName::new(String::new()).is_err());
    assert!(SpecName::new("  ".to_string()).is_err());
  }

  #[test]
  fn test_spec_name_too_long() {
    assert!(SpecName::new("a".repeat(256)).is_err());
  }

  #[test]
  fn test_spec_name_invalid_characters() {
    assert!(SpecName::new("spec name".to_string()).is_err());
    assert!(SpecName::new("spec!@#".to_string()).is_err());
  }

  #[test]
  fn test_spec_name_from_str() {
    assert_eq!(
      SpecName::from_str("test_spec"),
      Ok(SpecName("test_spec".to_string()))
    );
  }

  #[test]
  fn test_url_new_valid() {
    assert_eq!(
      Url::new("http://example.com".to_string()),
      Ok(Url("http://example.com".to_string()))
    );
    assert_eq!(
      Url::new("https://example.com".to_string()),
      Ok(Url("https://example.com".to_string()))
    );
  }

  #[allow(clippy::manual_string_new)]
  #[test]
  fn test_url_empty() {
    assert!(Url::new("".to_string()).is_err());
    assert!(Url::new("  ".to_string()).is_err());
  }

  #[test]
  fn test_url_missing_scheme() {
    assert!(Url::new("example.com".to_string()).is_err());
  }

  #[test]
  fn test_url_too_long() {
    let base = "http://".to_string();
    let long_path = "/".to_string() + &"a".repeat(2050);
    let full_url = base + &long_path;
    println!("Full URL length: {}", full_url.len());
    assert!(
      full_url.len() > 2048,
      "URL should be longer than 2048 characters"
    );
    assert!(Url::new(full_url).is_err());
  }

  #[test]
  fn test_url_from_str() {
    assert_eq!(
      Url::from_str("http://example.com"),
      Ok(Url("http://example.com".to_string()))
    );
  }

  #[allow(clippy::unwrap_used)]
  #[allow(clippy::panic)]
  #[test]
  fn test_url_scheme() {
    let http = Url::new("http://example.com".to_string()).unwrap();
    let https = Url::new("https://example.com".to_string()).unwrap();
    assert_eq!(http.scheme(), "http");
    assert_eq!(https.scheme(), "https");
  }

  #[allow(clippy::unwrap_used)]
  #[allow(clippy::panic)]
  #[test]
  fn test_url_without_fragment() {
    let url = Url::new("http://example.com/path#fragment".to_string()).unwrap();
    let without_frag = url.without_fragment().unwrap();
    assert_eq!(without_frag.as_str(), "http://example.com/path");

    // Test URL without fragment returns cloned self
    let url_no_frag = Url::new("http://example.com/path".to_string()).unwrap();
    let without_frag2 = url_no_frag.without_fragment().unwrap();
    assert_eq!(without_frag2.as_str(), "http://example.com/path");
  }

  #[allow(clippy::unwrap_used)]
  #[allow(clippy::panic)]
  #[test]
  fn test_url_with_path() {
    let url = Url::new("http://example.com".to_string()).unwrap();
    let new_url = url.with_path("/new/path").unwrap();
    assert_eq!(new_url.as_str(), "http://example.com/new/path");
  }

  #[allow(clippy::unwrap_used)]
  #[allow(clippy::panic)]
  #[test]
  fn test_url_with_query() {
    let url = Url::new("http://example.com".to_string()).unwrap();
    let new_url = url.with_query("key", "value").unwrap();
    assert!(new_url.as_str().contains("?key=value"));
  }

  #[allow(clippy::unwrap_used)]
  #[allow(clippy::panic)]
  #[test]
  fn test_url_host() {
    let url = Url::new("https://api.example.com:8080/path".to_string()).unwrap();
    assert_eq!(url.host(), Some("api.example.com".to_string()));
  }

  #[allow(clippy::unwrap_used)]
  #[allow(clippy::panic)]
  #[test]
  fn test_url_path() {
    let url1 = Url::new("http://example.com/path/to/resource".to_string()).unwrap();
    assert_eq!(url1.path(), "/path/to/resource".to_string());

    let url2 = Url::new("http://example.com".to_string()).unwrap();
    assert_eq!(url2.path(), "/".to_string());
  }

  #[allow(clippy::unwrap_used)]
  #[allow(clippy::panic)]
  #[test]
  fn test_url_query() {
    let url1 = Url::new("http://example.com/path?key=value&foo=bar".to_string()).unwrap();
    assert_eq!(url1.query(), Some("key=value&foo=bar".to_string()));

    let url2 = Url::new("http://example.com/path".to_string()).unwrap();
    assert_eq!(url2.query(), None);
  }

  #[allow(clippy::unwrap_used)]
  #[allow(clippy::panic)]
  #[test]
  fn test_url_fragment() {
    let url1 = Url::new("http://example.com/path#section".to_string()).unwrap();
    assert_eq!(url1.fragment(), Some("section".to_string()));

    let url2 = Url::new("http://example.com/path".to_string()).unwrap();
    assert_eq!(url2.fragment(), None);
  }

  #[allow(clippy::unwrap_used)]
  #[allow(clippy::panic)]
  #[test]
  fn test_url_complex_components() {
    let url = Url::new(
      "https://api.github.com/repos/user/repo/issues?q=labels&page=2#issue-123".to_string(),
    )
    .unwrap();
    assert_eq!(url.host(), Some("api.github.com".to_string()));
    assert_eq!(url.path(), "/repos/user/repo/issues".to_string());
    assert_eq!(url.query(), Some("q=labels&page=2".to_string()));
    assert_eq!(url.fragment(), Some("issue-123".to_string()));
  }

  #[allow(clippy::panic)]
  #[test]
  fn test_display_impls() {
    assert_eq!(format!("{}", HttpMethod::Get), "get");
    assert_eq!(format!("{}", SpecName("test".to_string())), "test");
    assert_eq!(
      format!("{}", Url("http://example.com".to_string())),
      "http://example.com"
    );
  }
}
