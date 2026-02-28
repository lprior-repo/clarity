use super::{Answer, Conflict, ConflictResolution};
use super::models::ConflictState;

const FAST_KEYWORDS: [&str; 5] = ["fast", "latency", "speed", "quick", "low-latency"];
const CONSISTENT_KEYWORDS: [&str; 5] = ["consistent", "accurate", "correct", "reliable", "precise"];
const ANONYMOUS_KEYWORDS: [&str; 5] = [
  "anonymous",
  "anonymized",
  "privacy",
  "no-tracking",
  "private",
];
const AUDIT_KEYWORDS: [&str; 5] = ["audit", "log", "track", "trail", "accountability"];

#[must_use]
pub(super) fn detect_conflicts(answers: &[Answer]) -> Vec<Conflict> {
  [
    detect_cap_conflict(answers),
    detect_anonymous_audit_conflict(answers),
  ]
  .into_iter()
  .flatten()
  .collect()
}

fn contains_keywords(text: &str, keywords: &[&str]) -> bool {
  let lower_text = text.to_ascii_lowercase();
  keywords
    .iter()
    .any(|keyword| lower_text.contains(&keyword.to_ascii_lowercase()))
}

fn detect_cap_conflict(answers: &[Answer]) -> Option<Conflict> {
  let fast_answer = answers
    .iter()
    .find(|answer| contains_keywords(&answer.response, &FAST_KEYWORDS))?;

  let consistent_answer = answers.iter().find(|answer| {
    contains_keywords(&answer.response, &CONSISTENT_KEYWORDS)
      && answer.question_id != fast_answer.question_id
  })?;

  Some(Conflict {
        id: "conflict-cap-0".to_string(),
        between: (fast_answer.question_id.clone(), consistent_answer.question_id.clone()),
        description: "CAP theorem conflict: The system cannot simultaneously guarantee low latency and strong consistency. You've indicated requirements for both speed and data accuracy.".to_string(),
        impact: "Without resolution, the system may fail to meet performance expectations or data integrity requirements under load.".to_string(),
        options: vec![
            ConflictResolution {
                option: "prioritize-speed".to_string(),
                description: "Optimize for low latency with eventual consistency".to_string(),
                tradeoffs: "Faster responses but data may be temporarily stale".to_string(),
                recommendation: false,
            },
            ConflictResolution {
                option: "prioritize-consistency".to_string(),
                description: "Optimize for strong consistency with higher latency".to_string(),
                tradeoffs: "Always accurate data but slower response times".to_string(),
                recommendation: true,
            },
        ],
        state: ConflictState::Pending,
    })
}

fn detect_anonymous_audit_conflict(answers: &[Answer]) -> Option<Conflict> {
  let anonymous_answer = answers
    .iter()
    .find(|answer| contains_keywords(&answer.response, &ANONYMOUS_KEYWORDS))?;

  let audit_answer = answers.iter().find(|answer| {
    contains_keywords(&answer.response, &AUDIT_KEYWORDS)
      && answer.question_id != anonymous_answer.question_id
  })?;

  Some(Conflict {
        id: "conflict-anonymous-audit-0".to_string(),
        between: (anonymous_answer.question_id.clone(), audit_answer.question_id.clone()),
        description: "Privacy vs Accountability conflict: You've indicated requirements for user anonymity while also requiring audit trails. These requirements are fundamentally at odds.".to_string(),
        impact: "Without resolution, the system cannot provide both complete user privacy and comprehensive audit trails.".to_string(),
        options: vec![
            ConflictResolution {
                option: "prioritize-privacy".to_string(),
                description: "Remove detailed audit logging to protect user privacy".to_string(),
                tradeoffs: "Reduced accountability and harder incident investigation".to_string(),
                recommendation: false,
            },
            ConflictResolution {
                option: "pseudonymous-audit".to_string(),
                description: "Use pseudonymous identifiers in audit logs instead of real identities".to_string(),
                tradeoffs: "Partial privacy with some accountability; may not satisfy strict anonymity requirements".to_string(),
                recommendation: true,
            },
        ],
        state: ConflictState::Pending,
    })
}
