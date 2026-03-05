// Example usage of QualityScoreBar component
//
// This file demonstrates how to use the QualityScoreBar component in your application.
//
// Add this to your main.rs or any component:
//
// ```rust
// use dioxus::prelude::*;
// use clarity_web::components::{QualityScoreBar, QualityScoreBarProps, QualityScore, QualityDimension};
//
// #[component]
// fn MyPage() -> Element {
//     let quality_score = use_signal(|| QualityScore::new(75).with_dimensions(vec![
//         QualityDimension::new("Clarity", 85),
//         QualityDimension::new("Completeness", 70),
//         QualityDimension::new("Accuracy", 90),
//         QualityDimension::new("Relevance", 75),
//         QualityDimension::new("Structure", 80),
//     ]));
//
//     rsx! {
//         div {
//             class: "container mx-auto p-6",
//             h1 { "Quality Assessment" }
//             QualityScoreBar {
//                 score: quality_score,
//                 expanded: false,
//             }
//         }
//     }
// }
// ```
//
// ## With Issues
//
// ```rust
// let quality_score = use_signal(|| {
//     QualityScore::new(65).with_dimensions(vec![
//         QualityDimension::new("Clarity", 60)
//             .with_issues(vec![
//                 "Add more specific examples".to_string(),
//                 "Clarify technical terms".to_string(),
//             ]),
//         QualityDimension::new("Completeness", 70),
//         QualityDimension::new("Accuracy", 75),
//         QualityDimension::new("Relevance", 65)
//             .with_issues(vec![
//                 "Better alignment with user intent needed".to_string(),
//             ]),
//         QualityDimension::new("Structure", 80),
//     ])
// });
// ```

#[cfg(test)]
mod example_tests {
    use super::*;

    #[test]
    fn example_passing_quality_score() {
        // This score passes the quality gate (>= 70)
        let score = QualityScore::new(75).with_dimensions(vec![
            QualityDimension::new("Clarity", 85),
            QualityDimension::new("Completeness", 70),
            QualityDimension::new("Accuracy", 90),
            QualityDimension::new("Relevance", 75),
            QualityDimension::new("Structure", 80),
        ]);

        assert!(score.gate_passes());
        assert_eq!(score.gate_message(), "Quality gate: PASS");
        assert_eq!(score.dimensions.len(), 5);
        assert!(score.dimensions.iter().all(|d| d.issues.is_empty()));
    }

    #[test]
    fn example_failing_quality_score_with_issues() {
        // This score fails the quality gate (< 70) and has issues
        let score = QualityScore::new(55).with_dimensions(vec![
            QualityDimension::new("Clarity", 60).with_issues(vec![
                "Add more detail".to_string(),
                "Clarify assumptions".to_string(),
            ]),
            QualityDimension::new("Completeness", 50).with_issues(vec![
                "Missing key sections".to_string(),
            ]),
            QualityDimension::new("Accuracy", 70),
            QualityDimension::new("Relevance", 55).with_issues(vec![
                "Better alignment needed".to_string(),
            ]),
            QualityDimension::new("Structure", 65),
        ]);

        assert!(!score.gate_passes());
        assert_eq!(score.gate_message(), "Quality gate: FAIL (need 70, have 55)");
        assert_eq!(score.dimensions.len(), 5);

        // Check that some dimensions have issues
        let issues_count: usize = score.dimensions.iter()
            .map(|d| d.issues.len())
            .sum();
        assert_eq!(issues_count, 4);
    }

    #[test]
    fn example_edge_case_exactly_threshold() {
        // Exactly at threshold should pass
        let score = QualityScore::new(70);
        assert!(score.gate_passes());
        assert_eq!(score.gate_message(), "Quality gate: PASS");
    }

    #[test]
    fn example_edge_case_just_below_threshold() {
        // Just below threshold should fail
        let score = QualityScore::new(69);
        assert!(!score.gate_passes());
        assert_eq!(score.gate_message(), "Quality gate: FAIL (need 70, have 69)");
    }

    #[test]
    fn example_perfect_score() {
        // Perfect score
        let score = QualityScore::new(100).with_dimensions(vec![
            QualityDimension::new("Clarity", 100),
            QualityDimension::new("Completeness", 100),
            QualityDimension::new("Accuracy", 100),
            QualityDimension::new("Relevance", 100),
            QualityDimension::new("Structure", 100),
        ]);

        assert!(score.gate_passes());
        assert_eq!(score.overall, 100);
        assert!(score.dimensions.iter().all(|d| d.score == 100));
    }

    #[test]
    fn example_minimal_score() {
        // Minimal score
        let score = QualityScore::new(0).with_dimensions(vec![
            QualityDimension::new("Clarity", 0).with_issues(vec![
                "No content".to_string(),
            ]),
            QualityDimension::new("Completeness", 0).with_issues(vec![
                "Missing all sections".to_string(),
            ]),
            QualityDimension::new("Accuracy", 0).with_issues(vec![
                "Cannot verify".to_string(),
            ]),
            QualityDimension::new("Relevance", 0).with_issues(vec![
                "Not applicable".to_string(),
            ]),
            QualityDimension::new("Structure", 0).with_issues(vec![
                "No structure".to_string(),
            ]),
        ]);

        assert!(!score.gate_passes());
        assert_eq!(score.overall, 0);
        assert_eq!(score.gate_message(), "Quality gate: FAIL (need 70, have 0)");
    }
}
