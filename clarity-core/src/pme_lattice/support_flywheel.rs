//! Support Flywheel - Support as Product Input
//!
//! Friction logging with emotional state tracking, support tickets linked to use cases.

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Eq, Error, Serialize, Deserialize)]
pub enum SupportFlywheelError {
  #[error("Log name cannot be empty")]
  EmptyLogName,
  #[error("Feature name cannot be empty")]
  EmptyFeatureName,
  #[error("Ticket ID cannot be empty")]
  EmptyTicketId,
  #[error("Ticket title cannot be empty")]
  EmptyTicketTitle,
  #[error("Use case name cannot be empty")]
  EmptyUseCaseName,
  #[error("Flywheel name cannot be empty")]
  EmptyFlywheelName,
  #[error("Ticket not found")]
  TicketNotFound,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EmotionalState {
  Frustrated,
  Confused,
  Neutral,
  Pleased,
}

impl EmotionalState {
  pub fn is_negative(&self) -> bool {
    matches!(self, Self::Frustrated | Self::Confused)
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TicketSeverity {
  Low,
  Medium,
  High,
  Critical,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TicketStatus {
  Open,
  InProgress,
  Resolved,
  Closed,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrictionEntry {
  pub feature: String,
  pub description: String,
  pub emotional_state: EmotionalState,
  pub timestamp: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrictionLog {
  pub name: String,
  pub entries: Vec<FrictionEntry>,
  pub created_at: DateTime<Utc>,
}

impl FrictionLog {
  pub fn new(name: String) -> Result<Self, SupportFlywheelError> {
    if name.is_empty() {
      return Err(SupportFlywheelError::EmptyLogName);
    }
    Ok(Self {
      name,
      entries: vec![],
      created_at: Utc::now(),
    })
  }

  pub fn add_entry(&mut self, entry: FrictionEntry) -> Result<(), SupportFlywheelError> {
    if entry.feature.is_empty() {
      return Err(SupportFlywheelError::EmptyFeatureName);
    }
    self.entries.push(entry);
    Ok(())
  }

  pub fn analyze_emotions(&self) -> HashMap<EmotionalState, usize> {
    let mut counts = HashMap::new();
    for entry in &self.entries {
      *counts.entry(entry.emotional_state).or_insert(0) += 1;
    }
    counts
  }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SupportTicket {
  pub id: String,
  pub title: String,
  pub description: String,
  pub severity: TicketSeverity,
  pub status: TicketStatus,
  pub created_at: DateTime<Utc>,
}

impl SupportTicket {
  pub fn new(
    id: String,
    title: String,
    description: String,
    severity: TicketSeverity,
  ) -> Result<Self, SupportFlywheelError> {
    if id.is_empty() {
      return Err(SupportFlywheelError::EmptyTicketId);
    }
    if title.is_empty() {
      return Err(SupportFlywheelError::EmptyTicketTitle);
    }
    Ok(Self {
      id,
      title,
      description,
      severity,
      status: TicketStatus::Open,
      created_at: Utc::now(),
    })
  }

  pub fn is_open(&self) -> bool {
    matches!(self.status, TicketStatus::Open | TicketStatus::InProgress)
  }

  pub fn is_high_priority(&self) -> bool {
    matches!(
      self.severity,
      TicketSeverity::High | TicketSeverity::Critical
    )
  }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UseCaseLink {
  pub ticket_id: String,
  pub use_case_name: String,
  pub created_at: DateTime<Utc>,
}

impl UseCaseLink {
  pub fn new(ticket_id: String, use_case_name: String) -> Result<Self, SupportFlywheelError> {
    if ticket_id.is_empty() {
      return Err(SupportFlywheelError::EmptyTicketId);
    }
    if use_case_name.is_empty() {
      return Err(SupportFlywheelError::EmptyUseCaseName);
    }
    Ok(Self {
      ticket_id,
      use_case_name,
      created_at: Utc::now(),
    })
  }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProductInsights {
  pub friction_count: HashMap<String, usize>,
  pub use_case_coverage: HashMap<String, usize>,
  pub emotional_summary: HashMap<EmotionalState, usize>,
  pub open_tickets: usize,
  pub high_priority_tickets: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SupportFlywheel {
  pub name: String,
  pub friction_logs: Vec<FrictionLog>,
  pub tickets: Vec<SupportTicket>,
  pub use_case_links: Vec<UseCaseLink>,
  pub created_at: DateTime<Utc>,
}

impl SupportFlywheel {
  pub fn new(name: String) -> Result<Self, SupportFlywheelError> {
    if name.is_empty() {
      return Err(SupportFlywheelError::EmptyFlywheelName);
    }
    Ok(Self {
      name,
      friction_logs: vec![],
      tickets: vec![],
      use_case_links: vec![],
      created_at: Utc::now(),
    })
  }

  pub fn add_friction_log(&mut self, log: FrictionLog) {
    self.friction_logs.push(log);
  }

  pub fn add_ticket(&mut self, ticket: SupportTicket) {
    self.tickets.push(ticket);
  }

  pub fn link_ticket_to_use_case(
    &mut self,
    ticket_id: &str,
    use_case_name: &str,
  ) -> Result<(), SupportFlywheelError> {
    let ticket_exists = self.tickets.iter().any(|t| t.id == ticket_id);
    if !ticket_exists {
      return Err(SupportFlywheelError::TicketNotFound);
    }
    let link = UseCaseLink::new(ticket_id.to_string(), use_case_name.to_string())?;
    self.use_case_links.push(link);
    Ok(())
  }

  pub fn generate_insights(&self) -> ProductInsights {
    let mut friction_count = HashMap::new();
    let mut emotional_summary = HashMap::new();

    for log in &self.friction_logs {
      for entry in &log.entries {
        *friction_count.entry(entry.feature.clone()).or_insert(0) += 1;
        *emotional_summary.entry(entry.emotional_state).or_insert(0) += 1;
      }
    }

    let mut use_case_coverage = HashMap::new();
    for link in &self.use_case_links {
      *use_case_coverage
        .entry(link.use_case_name.clone())
        .or_insert(0) += 1;
    }

    let open_tickets = self.tickets.iter().filter(|t| t.is_open()).count();
    let high_priority_tickets = self.tickets.iter().filter(|t| t.is_high_priority()).count();

    ProductInsights {
      friction_count,
      use_case_coverage,
      emotional_summary,
      open_tickets,
      high_priority_tickets,
    }
  }
}
