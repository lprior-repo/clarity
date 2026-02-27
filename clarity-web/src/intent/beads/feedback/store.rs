use once_cell::sync::Lazy;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, RwLock};

use super::domain::{BeadFeedback, FeedbackError};

static FEEDBACK_STORE: Lazy<Arc<RwLock<HashMap<String, VecDeque<BeadFeedback>>>>> =
  Lazy::new(|| Arc::new(RwLock::new(HashMap::new())));

pub(super) fn store_feedback(feedback: &BeadFeedback) -> Result<(), FeedbackError> {
  FEEDBACK_STORE
    .write()
    .map_err(|_| FeedbackError::Blocked("Failed to acquire feedback store lock".to_string()))
    .map(|mut store| {
      store
        .entry(feedback.bead_id.clone())
        .or_insert_with(VecDeque::new)
        .push_back(feedback.clone());
    })
}

#[must_use]
pub(super) fn read_feedback_history(bead_id: &str) -> Vec<BeadFeedback> {
  FEEDBACK_STORE.read().ok().map_or_else(Vec::new, |store| {
    store
      .get(bead_id)
      .map_or_else(Vec::new, |items| items.iter().cloned().collect::<Vec<_>>())
  })
}

#[cfg(test)]
pub(crate) fn clear_feedback_store() {
  let _ = FEEDBACK_STORE
    .write()
    .map(|mut store| store.clear())
    .map_err(|_| ());
}
