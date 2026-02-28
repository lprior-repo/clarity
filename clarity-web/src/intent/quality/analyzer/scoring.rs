use crate::intent::types::{Behavior, Feature, Spec, Verification};

use super::domain::QualityReport;
use super::issue_collection::{
  collect_ai_readiness_issues, collect_clarity_issues, collect_coverage_issues,
  collect_testability_issues,
};

#[must_use]
pub fn analyze_spec(spec: &Spec) -> QualityReport {
  let coverage_score = calculate_coverage_score(spec);
  let clarity_score = calculate_clarity_score(spec);
  let testability_score = calculate_testability_score(spec);
  let ai_readiness_score = calculate_ai_readiness_score(spec);

  let mut report = QualityReport::new(
    coverage_score,
    clarity_score,
    testability_score,
    ai_readiness_score,
  );

  collect_coverage_issues(spec, &mut report);
  collect_clarity_issues(spec, &mut report);
  collect_testability_issues(spec, &mut report);
  collect_ai_readiness_issues(spec, &mut report);
  report
}

#[must_use]
pub fn calculate_coverage_score(spec: &Spec) -> u8 {
  let score = 100_u16
    .saturating_sub(u16::from(!check_has_error_tests(spec)) * 20)
    .saturating_sub(u16::from(!check_has_auth_tests(spec)) * 15)
    .saturating_sub(u16::from(!check_has_edge_cases(spec)) * 15)
    .saturating_sub(u16::from(!check_invariants_tested(spec)) * 20)
    .saturating_add(u16::from(calculate_verification_ratio(spec) > 0.8) * 10)
    .saturating_add(u16::from(!spec.invariants.is_empty()) * 5);
  u8::try_from(score.min(100)).unwrap_or(100)
}

#[must_use]
pub fn calculate_clarity_score(spec: &Spec) -> u8 {
  let desc_ratio = calculate_description_ratio(spec);
  let description_penalty = if desc_ratio < 0.5 {
    20
  } else if desc_ratio < 0.8 {
    10
  } else {
    0
  };

  let score = 100_u16
    .saturating_sub(description_penalty)
    .saturating_sub(u16::from(spec.description.is_empty()) * 10)
    .saturating_sub(u16::from(count_vague_language(spec)).saturating_mul(5))
    .saturating_add(u16::from(desc_ratio >= 0.9) * 5);
  u8::try_from(score.min(100)).unwrap_or(100)
}

#[must_use]
pub fn calculate_testability_score(spec: &Spec) -> u8 {
  let precond_ratio = calculate_precondition_ratio(spec);
  let postcond_ratio = calculate_postcondition_ratio(spec);
  let example_ratio = calculate_example_ratio(spec);
  let deps_ratio = calculate_dependency_documentation_ratio(spec);

  let score = 100_u16
    .saturating_sub(u16::from(precond_ratio < 0.5) * 15)
    .saturating_sub(u16::from(postcond_ratio < 0.5) * 15)
    .saturating_sub(u16::from(example_ratio < 0.5) * 15)
    .saturating_sub(u16::from(deps_ratio < 0.5) * 10)
    .saturating_add(u16::from(precond_ratio >= 0.8 && postcond_ratio >= 0.8) * 10)
    .saturating_add(u16::from(example_ratio >= 0.8) * 5);
  u8::try_from(score.min(100)).unwrap_or(100)
}

#[must_use]
pub fn calculate_ai_readiness_score(spec: &Spec) -> u8 {
  let ai_hints = &spec.ai_hints;
  let has_impl_hints = !ai_hints.implementation.architecture.is_empty()
    || !ai_hints.implementation.performance_notes.is_empty()
    || !ai_hints.implementation.error_handling.is_empty();
  let has_security_hints = !ai_hints.security.password_hashing.is_empty()
    || !ai_hints.security.jwt_algorithm.is_empty()
    || !ai_hints.security.jwt_expiry.is_empty()
    || !ai_hints.security.rate_limiting.is_empty();
  let has_entity_hints = !ai_hints.entities.is_empty();
  let has_lib_hints = !ai_hints.preferred_libraries.is_empty();
  let has_style_hints = !ai_hints.style_hints.is_empty();
  let example_ratio = calculate_example_ratio(spec);

  let score = 100_u16
    .saturating_sub(u16::from(!has_impl_hints) * 20)
    .saturating_sub(u16::from(!has_security_hints) * 15)
    .saturating_sub(u16::from(!has_entity_hints) * 10)
    .saturating_sub(u16::from(!has_lib_hints) * 5)
    .saturating_sub(u16::from(!has_style_hints) * 5)
    .saturating_sub(u16::from(example_ratio < 0.5) * 15)
    .saturating_add(u16::from(has_impl_hints && has_security_hints && has_entity_hints) * 10);
  u8::try_from(score.min(100)).unwrap_or(100)
}

#[must_use]
pub fn calculate_overall_score(report: &QualityReport) -> u8 {
  calculate_overall_score_from_values(
    report.coverage_score,
    report.clarity_score,
    report.testability_score,
    report.ai_readiness_score,
  )
}

pub(super) fn calculate_overall_score_from_values(
  coverage: u8,
  clarity: u8,
  testability: u8,
  ai_readiness: u8,
) -> u8 {
  let weighted_sum = u16::from(coverage)
    .saturating_mul(30)
    .saturating_add(u16::from(clarity).saturating_mul(25))
    .saturating_add(u16::from(testability).saturating_mul(25))
    .saturating_add(u16::from(ai_readiness).saturating_mul(20));
  u8::try_from(weighted_sum / 100).unwrap_or(100)
}

pub(super) fn check_has_error_tests(spec: &Spec) -> bool {
  spec.features.iter().any(|f| {
    f.behaviors.iter().any(|b| {
      b.verification.as_ref().is_some_and(|v| {
        let text = format!("{} {}", v.description, v.example).to_lowercase();
        ["error", "fail", "invalid"]
          .iter()
          .any(|kw| text.contains(kw))
      })
    })
  })
}

pub(super) fn check_has_auth_tests(spec: &Spec) -> bool {
  let auth_words = ["auth", "login", "permission", "unauthorized", "forbidden"];
  let in_behavior = spec.features.iter().any(|f| {
    auth_words
      .iter()
      .any(|kw| f.name.to_lowercase().contains(kw))
      || f.behaviors.iter().any(|b| {
        auth_words
          .iter()
          .any(|kw| b.name.to_lowercase().contains(kw))
          || b.verification.as_ref().is_some_and(|v| {
            auth_words
              .iter()
              .any(|kw| v.description.to_lowercase().contains(kw))
          })
      })
  });
  in_behavior
    || !spec.ai_hints.security.password_hashing.is_empty()
    || !spec.ai_hints.security.jwt_algorithm.is_empty()
    || !spec.ai_hints.security.jwt_expiry.is_empty()
    || !spec.ai_hints.security.rate_limiting.is_empty()
}

pub(super) fn check_has_edge_cases(spec: &Spec) -> bool {
  let edge_words = ["edge", "boundary", "empty", "null", "limit", "max", "min"];
  spec.features.iter().any(|f: &Feature| {
    f.behaviors.iter().any(|b: &Behavior| {
      b.preconditions.iter().any(|p| {
        let lower = p.to_lowercase();
        edge_words.iter().any(|kw| lower.contains(kw))
      }) || b.verification.as_ref().is_some_and(|v: &Verification| {
        let lower = format!("{} {}", v.description, v.example).to_lowercase();
        edge_words.iter().any(|kw| lower.contains(kw))
      })
    })
  })
}

pub(super) fn check_invariants_tested(spec: &Spec) -> bool {
  spec.invariants.is_empty()
    || spec.features.iter().any(|f| {
      f.behaviors.iter().any(|b| {
        b.verification.as_ref().is_some_and(|v| {
          let lower = format!("{} {}", v.description, v.example).to_lowercase();
          lower.contains("invariant")
        }) || b
          .postconditions
          .iter()
          .any(|p| p.to_lowercase().contains("invariant"))
      })
    })
}

pub(super) fn calculate_verification_ratio(spec: &Spec) -> f64 {
  ratio_by_behaviors(spec, |b| b.verification.is_some())
}

pub(super) fn calculate_description_ratio(spec: &Spec) -> f64 {
  ratio_by_behaviors(spec, |b| !b.description.is_empty())
}

pub(super) fn count_vague_language(spec: &Spec) -> u8 {
  let vague_words = [
    "maybe",
    "perhaps",
    "probably",
    "might",
    "could",
    "somehow",
    "something",
    "stuff",
    "things",
    "etc",
    "and so on",
    "roughly",
    "approximately",
    "usually",
    "typically",
    "generally",
    "often",
    "sometimes",
  ];
  let all_text = std::iter::once(spec.description.as_str())
    .chain(spec.features.iter().map(|f| f.description.as_str()))
    .chain(
      spec
        .features
        .iter()
        .flat_map(|f| f.behaviors.iter().map(|b| b.description.as_str())),
    )
    .map(str::to_lowercase)
    .collect::<Vec<_>>();

  all_text
    .iter()
    .map(|text| {
      u8::try_from(
        vague_words
          .iter()
          .filter(|word| text.contains(*word))
          .count(),
      )
      .unwrap_or(u8::MAX)
    })
    .fold(0_u8, u8::saturating_add)
    .min(20)
}

pub(super) fn calculate_precondition_ratio(spec: &Spec) -> f64 {
  ratio_by_behaviors(spec, |b| !b.preconditions.is_empty())
}

pub(super) fn calculate_postcondition_ratio(spec: &Spec) -> f64 {
  ratio_by_behaviors(spec, |b| !b.postconditions.is_empty())
}

pub(super) fn calculate_example_ratio(spec: &Spec) -> f64 {
  ratio_by_behaviors(spec, |b| {
    b.verification
      .as_ref()
      .is_some_and(|v| !v.example.is_empty())
  })
}

pub(super) fn calculate_dependency_documentation_ratio(spec: &Spec) -> f64 {
  if spec.features.is_empty() {
    1.0
  } else {
    let documented = spec
      .features
      .iter()
      .filter(|f| !f.depends_on.is_empty() || f.behaviors.is_empty())
      .count();
    let documented_u32 = u32::try_from(documented).unwrap_or(u32::MAX);
    let total_u32 = u32::try_from(spec.features.len()).unwrap_or(u32::MAX);
    f64::from(documented_u32) / f64::from(total_u32)
  }
}

fn ratio_by_behaviors(spec: &Spec, predicate: impl Fn(&Behavior) -> bool) -> f64 {
  let total = spec
    .features
    .iter()
    .map(|f| f.behaviors.len())
    .sum::<usize>();
  if total == 0 {
    1.0
  } else {
    let matched = spec
      .features
      .iter()
      .flat_map(|f| f.behaviors.iter())
      .filter(|b| predicate(b))
      .count();
    let matched_u32 = u32::try_from(matched).unwrap_or(u32::MAX);
    let total_u32 = u32::try_from(total).unwrap_or(u32::MAX);
    f64::from(matched_u32) / f64::from(total_u32)
  }
}
