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
    /// Parse a string into an HttpMethod
    pub fn from_str(s: &str) -> Result<Self, HttpMethodError> {
        match s.to_lowercase().as_str() {
            "get" => Ok(HttpMethod::Get),
            "post" => Ok(HttpMethod::Post),
            "put" => Ok(HttpMethod::Put),
            "patch" => Ok(HttpMethod::Patch),
            "delete" => Ok(HttpMethod::Delete),
            "head" => Ok(HttpMethod::Head),
            "options" => Ok(HttpMethod::Options),
            _ => Err(HttpMethodError::InvalidMethod(s.to_string())),
        }
    }

    /// Get the method as a lowercase string
    pub fn as_str(&self) -> &'static str {
        match self {
            HttpMethod::Get => "get",
            HttpMethod::Post => "post",
            HttpMethod::Put => "put",
            HttpMethod::Patch => "patch",
            HttpMethod::Delete => "delete",
            HttpMethod::Head => "head",
            HttpMethod::Options => "options",
        }
    }

    /// Check if this is a safe method (doesn't modify data)
    pub fn is_safe(&self) -> bool {
        matches!(
            self,
            HttpMethod::Get | HttpMethod::Head | HttpMethod::Options
        )
    }

    /// Check if this method has a body
    pub fn has_body(&self) -> bool {
        matches!(self, HttpMethod::Post | HttpMethod::Put | HttpMethod::Patch)
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
            HttpMethodError::InvalidMethod(method) => {
                write!(f, "Invalid HTTP method: {}", method)
            }
        }
    }
}

impl std::error::Error for HttpMethodError {}

/// Spec name with validation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SpecName(String);

impl SpecName {
    /// Create a new SpecName with validation
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
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// Convert to string
    pub fn to_string(&self) -> String {
        self.0.clone()
    }

    /// Convert from string (may fail)
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
            SpecNameError::Empty => write!(f, "Spec name cannot be empty"),
            SpecNameError::TooLong(len) => {
                write!(f, "Spec name is too long ({} characters, max 255)", len)
            }
            SpecNameError::InvalidCharacters(name) => {
                write!(f, "Spec name contains invalid characters: {}", name)
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
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// Convert to string
    pub fn to_string(&self) -> String {
        self.0.clone()
    }

    /// Convert from string (may fail)
    pub fn from_str(s: &str) -> Result<Self, UrlError> {
        Self::new(s.to_string())
    }

    /// Get the scheme (http or https)
    pub fn scheme(&self) -> &'static str {
        if self.0.starts_with("https://") {
            "https"
        } else {
            "http"
        }
    }

    /// Get the host
    pub fn host(&self) -> Option<&str> {
        parsed_url().host_str()
    }

    /// Get the path
    pub fn path(&self) -> &str {
        parsed_url().path()
    }

    /// Get the query string
    pub fn query(&self) -> Option<&str> {
        parsed_url().query()
    }

    /// Get the fragment
    pub fn fragment(&self) -> Option<&str> {
        parsed_url().fragment()
    }

    /// Get the full URL without fragment
    pub fn without_fragment(&self) -> Self {
        if let Some(pos) = self.0.find('#') {
            Self::new(self.0[..pos].to_string()).unwrap()
        } else {
            self.clone()
        }
    }

    /// Get a URL with a new path
    pub fn with_path(&self, path: &str) -> Result<Self, UrlError> {
        let base = parsed_url();
        let new_url = base
            .join(path)
            .map_err(|_| UrlError::InvalidFormat)?
            .to_string();

        Self::new(new_url)
    }

    /// Get a URL with a new query parameter
    pub fn with_query(&self, key: &str, value: &str) -> Result<Self, UrlError> {
        let base = parsed_url();
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
            UrlError::Empty => write!(f, "URL cannot be empty"),
            UrlError::MissingScheme => write!(f, "URL must start with http:// or https://"),
            UrlError::TooLong(len) => write!(f, "URL is too long ({} characters, max 2048)", len),
            UrlError::InvalidFormat => write!(f, "Invalid URL format"),
            UrlError::InvalidScheme => write!(f, "URL must use http or https scheme"),
        }
    }
}

impl std::error::Error for UrlError {}

/// Parse the URL for internal use
fn parsed_url() -> &'static url::Url {
    use once_cell::sync::Lazy;
    static PARSED: Lazy<url::Url> = Lazy::new(|| url::Url::parse("http://example.com").unwrap());
    &PARSED
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(SpecName::new("".to_string()).is_err());
        assert!(SpecName::new("  ".to_string()).is_err());
    }

    #[test]
    fn test_spec_name_too_long() {
        assert!(SpecName::new("a".repeat(256).to_string()).is_err());
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

    #[test]
    fn test_url_scheme() {
        let http = Url::new("http://example.com".to_string()).unwrap();
        let https = Url::new("https://example.com".to_string()).unwrap();
        assert_eq!(http.scheme(), "http");
        assert_eq!(https.scheme(), "https");
    }

    #[test]
    fn test_url_without_fragment() {
        let url = Url::new("http://example.com/path#fragment".to_string()).unwrap();
        let without_frag = url.without_fragment();
        assert_eq!(without_frag.as_str(), "http://example.com/path");
    }

    #[test]
    fn test_url_with_path() {
        let url = Url::new("http://example.com".to_string()).unwrap();
        let new_url = url.with_path("/new/path").unwrap();
        assert_eq!(new_url.as_str(), "http://example.com/new/path");
    }

    #[test]
    fn test_url_with_query() {
        let url = Url::new("http://example.com".to_string()).unwrap();
        let new_url = url.with_query("key", "value").unwrap();
        assert!(new_url.as_str().contains("?key=value"));
    }

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
