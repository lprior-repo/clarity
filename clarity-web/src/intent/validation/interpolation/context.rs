use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Context {
  pub variables: HashMap<String, String>,
  pub request_body: Option<Value>,
  pub response_body: Option<Value>,
}

impl Context {
  #[must_use]
  pub fn new() -> Self {
    Self {
      variables: HashMap::new(),
      request_body: None,
      response_body: None,
    }
  }

  #[must_use]
  pub fn from_variables<I, K, V>(vars: I) -> Self
  where
    I: IntoIterator<Item = (K, V)>,
    K: Into<String>,
    V: Into<String>,
  {
    Self {
      variables: vars
        .into_iter()
        .map(|(k, v)| (k.into(), v.into()))
        .collect(),
      request_body: None,
      response_body: None,
    }
  }

  #[must_use]
  pub fn with_variable(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
    self.variables.insert(key.into(), value.into());
    self
  }

  #[must_use]
  pub fn with_request_body(mut self, body: Value) -> Self {
    self.request_body = Some(body);
    self
  }

  #[must_use]
  pub fn with_response_body(mut self, body: Value) -> Self {
    self.response_body = Some(body);
    self
  }
}
