use super::domain::QualityReport;

#[must_use]
pub fn format_report(report: &QualityReport) -> String {
  let header = format!(
        "=== Quality Report ===\n\nCoverage Score:     {:3}/100\nClarity Score:      {:3}/100\nTestability Score:  {:3}/100\nAI Readiness Score: {:3}/100\n------------------------\nOverall Score:      {:3}/100\n\n",
        report.coverage_score,
        report.clarity_score,
        report.testability_score,
        report.ai_readiness_score,
        report.overall_score,
    );

  let issues = if report.has_issues() {
    let lines = report
      .issues
      .iter()
      .enumerate()
      .map(|(idx, issue)| format!("  {}. {}", idx + 1, issue.description()))
      .collect::<Vec<_>>()
      .join("\n");
    format!("Issues Found ({}):\n{}\n\n", report.issue_count(), lines)
  } else {
    "No issues found. Spec quality is excellent!\n\n".to_string()
  };

  let suggestions = if report.suggestions.is_empty() {
    String::new()
  } else {
    let lines = report
      .suggestions
      .iter()
      .map(|suggestion| format!("  - {suggestion}"))
      .collect::<Vec<_>>()
      .join("\n");
    format!("Suggestions for Improvement:\n{lines}\n")
  };

  format!("{header}{issues}{suggestions}")
}

#[cfg(test)]
mod tests {
  use super::format_report;
  use crate::intent::quality::analyzer::{QualityIssue, QualityReport};

  #[test]
  fn format_includes_scores_and_issues() {
    let mut report = QualityReport::new(80, 70, 90, 60);
    report.add_issue(QualityIssue::MissingErrorTests);
    let output = format_report(&report);
    assert!(output.contains("Coverage Score:"));
    assert!(output.contains("Issues Found (1):"));
  }
}
