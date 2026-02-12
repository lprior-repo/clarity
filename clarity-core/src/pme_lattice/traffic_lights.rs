//! Traffic Lights - Signifiers & Affordances Framework
//!
//! Affordance strength classification (Green/Yellow/Red) and malfunction detection.

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
pub enum TrafficLightError {
  #[error("Signifier name cannot be empty")]
  EmptySignifierName,
  #[error("Audit name cannot be empty")]
  EmptyAuditName,
  #[error("Affordance name cannot be empty")]
  EmptyAffordanceName,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrafficLight {
  Green,
  Yellow,
  Red,
}

impl TrafficLight {
  pub fn is_safe(&self) -> bool {
    matches!(self, Self::Green)
  }

  pub fn is_safer_than(&self, other: &Self) -> bool {
    match (self, other) {
      (Self::Green, Self::Yellow | Self::Red) => true,
      (Self::Yellow, Self::Red) => true,
      _ => false,
    }
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignifierType {
  Visual,
  Auditory,
  Haptic,
  Textual,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Signifier {
  pub name: String,
  pub signifier_type: SignifierType,
  pub description: String,
}

impl Signifier {
  pub fn new(
    name: String,
    signifier_type: SignifierType,
    description: String,
  ) -> Result<Self, TrafficLightError> {
    if name.is_empty() {
      return Err(TrafficLightError::EmptySignifierName);
    }
    Ok(Self {
      name,
      signifier_type,
      description,
    })
  }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Affordance {
  pub name: String,
  pub action: String,
  pub strength: TrafficLight,
  pub description: String,
}

impl Affordance {
  pub fn is_safe(&self) -> bool {
    self.strength.is_safe()
  }

  pub fn requires_caution(&self) -> bool {
    matches!(self.strength, TrafficLight::Yellow)
  }

  pub fn is_dangerous(&self) -> bool {
    matches!(self.strength, TrafficLight::Red)
  }

  pub fn action_complexity(&self) -> ActionComplexity {
    let words: Vec<&str> = self.action.split('_').collect();
    match words.len() {
      1 => ActionComplexity::Single,
      2 => ActionComplexity::Double,
      _ => ActionComplexity::Multiple,
    }
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionComplexity {
  Single,
  Double,
  Multiple,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MalfunctioningTrafficLight {
  pub dangerous_affordance: Affordance,
  pub safe_alternative: Affordance,
  pub reason: String,
}

impl MalfunctioningTrafficLight {
  pub fn is_malfunction(&self) -> bool {
    self.dangerous_affordance.is_dangerous()
      && self.safe_alternative.is_safe()
      && self.dangerous_affordance.action_complexity() < self.safe_alternative.action_complexity()
  }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrafficLightAudit {
  pub name: String,
  pub affordances: Vec<Affordance>,
  pub created_at: DateTime<Utc>,
}

impl TrafficLightAudit {
  pub fn new(name: String) -> Result<Self, TrafficLightError> {
    if name.is_empty() {
      return Err(TrafficLightError::EmptyAuditName);
    }
    Ok(Self {
      name,
      affordances: vec![],
      created_at: Utc::now(),
    })
  }

  pub fn add_affordance(&mut self, affordance: Affordance) {
    self.affordances.push(affordance);
  }

  pub fn detect_malfunctions(&self) -> Vec<MalfunctioningTrafficLight> {
    let mut malfunctions = vec![];

    let dangerous: Vec<&Affordance> = self
      .affordances
      .iter()
      .filter(|a| a.is_dangerous())
      .collect();

    let safe: Vec<&Affordance> = self.affordances.iter().filter(|a| a.is_safe()).collect();

    for d in &dangerous {
      for s in &safe {
        if d.action_complexity() < s.action_complexity() {
          malfunctions.push(MalfunctioningTrafficLight {
            dangerous_affordance: (*d).clone(),
            safe_alternative: (*s).clone(),
            reason: format!(
              "Dangerous action '{}' is easier than safe action '{}'",
              d.name, s.name
            ),
          });
        }
      }
    }

    malfunctions
  }

  pub fn count_by_strength(&self) -> HashMap<TrafficLight, usize> {
    let mut counts = HashMap::new();
    for affordance in &self.affordances {
      *counts.entry(affordance.strength).or_insert(0) += 1;
    }
    counts
  }

  pub fn generate_report(&self) -> String {
    let counts = self.count_by_strength();
    let green = counts.get(&TrafficLight::Green).copied().unwrap_or(0);
    let yellow = counts.get(&TrafficLight::Yellow).copied().unwrap_or(0);
    let red = counts.get(&TrafficLight::Red).copied().unwrap_or(0);
    let malfunctions = self.detect_malfunctions();

    format!(
      "Traffic Light Audit: {}\n\
             Green (Safe): {}\n\
             Yellow (Cautionary): {}\n\
             Red (Dangerous): {}\n\
             Malfunctions Detected: {}\n\
             Total Affordances: {}",
      self.name,
      green,
      yellow,
      red,
      malfunctions.len(),
      self.affordances.len()
    )
  }
}
