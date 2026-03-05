use super::ArrayIndexError;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArraySpec {
  NoArray,
  Index(usize),
  NegativeIndex(usize),
  All,
}

impl ArraySpec {
  #[must_use]
  pub const fn is_array_access(self) -> bool {
    !matches!(self, Self::NoArray)
  }

  pub fn resolve_indices(self, length: usize) -> Result<Vec<usize>, ArrayIndexError> {
    match self {
      Self::NoArray => Ok(Vec::new()),
      Self::Index(i) => {
        if i < length {
          Ok(vec![i])
        } else {
          Err(ArrayIndexError::IndexOutOfBounds {
            index: i as isize,
            length,
          })
        }
      }
      Self::NegativeIndex(n) => {
        if n == 0 || n > length {
          Err(ArrayIndexError::IndexOutOfBounds {
            index: -(n as isize),
            length,
          })
        } else {
          Ok(vec![length - n])
        }
      }
      Self::All => Ok((0..length).collect()),
    }
  }
}

impl fmt::Display for ArraySpec {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::NoArray => write!(f, ""),
      Self::Index(i) => write!(f, "[{i}]"),
      Self::NegativeIndex(n) => write!(f, "[-{n}]"),
      Self::All => write!(f, "[*]"),
    }
  }
}
