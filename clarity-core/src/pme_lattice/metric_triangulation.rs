//! Metric Triangulation - Three-Pillared Approach to Prevent Vanity Metrics
//!
//! KPI (business goal), Adoption (active users, not totals), Value (time saved, etc.)

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::missing_const_for_fn)]

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Eq, Error, Serialize, Deserialize)]
pub enum TriangulationError {
  #[error("Metric triangulation name cannot be empty")]
  EmptyName,
  #[error("KPI metric name cannot be empty")]
  EmptyKPIName,
  #[error("Adoption metric name cannot be empty")]
  EmptyAdoptionName,
  #[error("Value metric name cannot be empty")]
  EmptyValueName,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KPIMetricType {
  Profit,
  Revenue,
  Cost,
  Custom(String),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct KPIMetric {
  pub name: String,
  pub metric_type: KPIMetricType,
  pub target_value: f64,
  pub current_value: f64,
}

impl KPIMetric {
  pub fn is_healthy(&self) -> bool {
    self.current_value >= self.target_value * 0.7
  }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValueMetricType {
  TimeSaved,
  ErrorsAverted,
  InsightsGenerated,
  Custom(String),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ValueMetric {
  pub name: String,
  pub metric_type: ValueMetricType,
  pub unit: String,
  pub value: f64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdoptionMetric {
  pub name: String,
  pub active_count: u64,
  pub total_registered: u64,
  pub period_days: u32,
}

impl AdoptionMetric {
  pub fn is_vanity_metric(&self) -> bool {
    if self.total_registered == 0 {
      return false;
    }
    let ratio = self.active_count as f64 / self.total_registered as f64;
    ratio < 0.1
  }

  pub fn engagement_rate(&self) -> f64 {
    if self.total_registered == 0 {
      return 0.0;
    }
    (self.active_count as f64 / self.total_registered as f64) * 100.0
  }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TriangulationReport {
  pub name: String,
  pub has_kpi: bool,
  pub has_adoption: bool,
  pub has_value: bool,
  pub warnings: Vec<String>,
}

impl TriangulationReport {
  pub fn all_pillars_present(&self) -> bool {
    self.has_kpi && self.has_adoption && self.has_value
  }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MetricTriangulation {
  pub name: String,
  pub kpis: Vec<KPIMetric>,
  pub adoption_metrics: Vec<AdoptionMetric>,
  pub value_metrics: Vec<ValueMetric>,
}

impl MetricTriangulation {
  pub fn new(name: String) -> Result<Self, TriangulationError> {
    if name.is_empty() {
      return Err(TriangulationError::EmptyName);
    }
    Ok(Self {
      name,
      kpis: vec![],
      adoption_metrics: vec![],
      value_metrics: vec![],
    })
  }

  pub fn add_kpi(&mut self, kpi: KPIMetric) -> Result<(), TriangulationError> {
    if kpi.name.is_empty() {
      return Err(TriangulationError::EmptyKPIName);
    }
    self.kpis.push(kpi);
    Ok(())
  }

  pub fn add_adoption_metric(&mut self, metric: AdoptionMetric) -> Result<(), TriangulationError> {
    if metric.name.is_empty() {
      return Err(TriangulationError::EmptyAdoptionName);
    }
    self.adoption_metrics.push(metric);
    Ok(())
  }

  pub fn add_value_metric(&mut self, metric: ValueMetric) -> Result<(), TriangulationError> {
    if metric.name.is_empty() {
      return Err(TriangulationError::EmptyValueName);
    }
    self.value_metrics.push(metric);
    Ok(())
  }

  pub fn validate(&self) -> TriangulationReport {
    let mut warnings = vec![];

    if self.kpis.is_empty() {
      warnings.push("KPI pillar is missing".to_string());
    }
    if self.adoption_metrics.is_empty() {
      warnings.push("Adoption pillar is missing".to_string());
    } else {
      for am in &self.adoption_metrics {
        if am.is_vanity_metric() {
          warnings.push(format!("Vanity metric detected: '{}'", am.name));
        }
      }
    }
    if self.value_metrics.is_empty() {
      warnings.push("Value pillar is missing".to_string());
    }

    TriangulationReport {
      name: self.name.clone(),
      has_kpi: !self.kpis.is_empty(),
      has_adoption: !self.adoption_metrics.is_empty(),
      has_value: !self.value_metrics.is_empty(),
      warnings,
    }
  }
}
