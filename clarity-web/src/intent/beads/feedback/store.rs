use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, RwLock};

use chrono::{DateTime, Utc};

use super::domain::{BeadFeedback, FeedbackError};

const MAX_FEEDBACK_ENTRIES: usize = 1000;
const FEEDBACK_TTL_HOURS: i64 = 24;

type FeedbackHistory = VecDeque<BeadFeedback>;
type FeedbackStore = HashMap<String, FeedbackHistory>;
type SharedFeedbackStore = Arc<RwLock<FeedbackStore>>;

static FEEDBACK_STORE: std::sync::LazyLock<SharedFeedbackStore> =
  std::sync::LazyLock::new(|| Arc::new(RwLock::new(HashMap::new())));

fn is_expired(timestamp: &str) -> bool {
  timestamp
    .parse::<DateTime<Utc>>()
    .map(|entry_time| {
      let now = Utc::now();
      let duration = now.signed_duration_since(entry_time);
      duration.num_hours() > FEEDBACK_TTL_HOURS
    })
    .unwrap_or(true)
}

fn evict_expired_entries(history: &mut FeedbackHistory) {
  let expired: Vec<_> = history
    .iter()
    .filter(|entry| is_expired(&entry.timestamp))
    .cloned()
    .collect();

  for entry in expired {
    if let Some(idx) = history.iter().position(|e| e.timestamp == entry.timestamp) {
      let _ = history.remove(idx);
    }
  }
}

fn remove_oldest_entry(store: &mut FeedbackStore) {
  if let Some((key, _)) = store
    .iter()
    .min_by(|(_, a), (_, b)| a.front().map_or("", |e| e.timestamp.as_str()).cmp(&b.front().map_or("", |e| e.timestamp.as_str())))
  {
    let key = key.clone();
    store.remove(&key);
  }
}

pub(super) fn store_feedback(feedback: &BeadFeedback) -> Result<(), FeedbackError> {
  FEEDBACK_STORE
    .write()
    .map_err(|_| FeedbackError::Blocked("Failed to acquire feedback store lock".to_string()))
    .map(|mut store| {
      // Evict oldest entry if at capacity
      if store.len() >= MAX_FEEDBACK_ENTRIES {
        remove_oldest_entry(&mut store);
      }

      store
        .entry(feedback.bead_id.clone())
        .or_insert_with(VecDeque::new)
        .push_back(feedback.clone());

      // Evict expired entries from this history
      if let Some(history) = store.get_mut(&feedback.bead_id) {
        evict_expired_entries(history);
      }
    })
}

pub(super) fn read_feedback_history(bead_id: &str) -> Result<Vec<BeadFeedback>, FeedbackError> {
  FEEDBACK_STORE
    .read()
    .map_err(|_| FeedbackError::Blocked("Failed to acquire feedback store lock".to_string()))
    .map(|store| {
      store
        .get(bead_id)
        .map_or_else(Vec::new, |items| items.iter().cloned().collect::<Vec<_>>())
    })
}

#[cfg(test)]
pub fn clear_feedback_store() {
  let _ = FEEDBACK_STORE
    .write()
    .map(|mut store| store.clear())
    .map_err(|_| ());
}

