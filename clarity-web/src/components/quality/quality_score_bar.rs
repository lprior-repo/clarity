#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use dioxus::prelude::*;
use crate::lattice::quality::{QualityScore, IssueSeverity};

/// Minimum gate threshold for quality score
pub const MINIMUM_GATE: u8 = 70;

/// Props for QualityScoreBar component
#[derive(Clone, Debug, PartialEq, Props)]
pub struct QualityScoreBarProps {
    /// Current quality score
    pub score: Signal<Option<QualityScore>>,
    /// Minimum gate threshold
    pub minimum_gate: u8,
    /// Whether to show detailed issues
    pub show_details: Signal<bool>,
}

/// Quality score bar component
///
/// Displays:
/// - Overall score with color-coded progress bar
/// - Dimension breakdown with individual scores
/// - Issues list with severity indicators
/// - Gate status message
#[component]
pub fn QualityScoreBar(props: QualityScoreBarProps) -> Element {
    let QualityScoreBarProps {
        score,
        minimum_gate,
        mut show_details,
    } = props;

    let score_read = score.read();
    let (overall, passes_gate, dimension_scores, issues) = match score_read.as_ref() {
        Some(s) => (
            s.overall,
            s.passes(minimum_gate),
            s.dimensions.clone(),
            s.issues.clone(),
        ),
        None => (0u8, false, Vec::new(), Vec::new()),
    };
    drop(score_read);

    // Calculate color based on score
    let color_class = get_score_color_class(overall);
    let bg_color_class = get_score_bg_color_class(overall);

    // Group issues by severity
    let critical_issues: Vec<_> = issues
        .iter()
        .filter(|i| i.severity == IssueSeverity::Critical)
        .collect();
    let critical_issues_len = critical_issues.len();
    let error_issues: Vec<_> = issues
        .iter()
        .filter(|i| i.severity == IssueSeverity::Error)
        .collect();
    let error_issues_len = error_issues.len();
    let warning_issues: Vec<_> = issues
        .iter()
        .filter(|i| i.severity == IssueSeverity::Warning)
        .collect();
    let warning_issues_len = warning_issues.len();

    // Calculate dimensions passing threshold
    let dimensions_passing = dimension_scores
        .iter()
        .filter(|d| d.passes(minimum_gate))
        .count();
    let total_dimensions = dimension_scores.len();

    let show_details_val = *show_details.read();

    // Compute status message outside rsx
    let status_message = if overall == 0 {
        "Answer questions to calculate quality".to_string()
    } else if passes_gate {
        format!("Meets minimum threshold (≥{})", minimum_gate)
    } else {
        format!("Improve quality to unlock Develop phase (need ≥{})", minimum_gate)
    };

    // Compute dimensions text outside rsx
    let dimensions_text = format!("Dimensions ({} / {} passing)", dimensions_passing, total_dimensions);

    rsx! {
        div {
            class: "space-y-4",

            // Overall score section
            div {
                class: "flex items-center justify-between",

                // Score label
                div {
                    class: "space-y-1",
                    div {
                        class: "flex items-center gap-2",
                        span {
                            class: "text-sm font-semibold text-foreground",
                            "Quality Score"
                        }
                        if !passes_gate && overall > 0 {
                            span {
                                class: "inline-flex items-center rounded-full bg-chart-4/10 px-2 py-0.5 text-xs font-medium text-chart-4 border border-chart-4/20",
                                "Below threshold"
                            }
                        }
                    }
                    div {
                        class: "text-xs text-muted-foreground",
                        "{status_message}"
                    }
                }

                // Overall score badge
                div {
                    class: if overall == 0 {
                        "flex h-14 w-14 shrink-0 items-center justify-center rounded-xl border-2 border-border bg-muted text-muted-foreground".to_string()
                    } else {
                        format!("flex h-14 w-14 shrink-0 items-center justify-center rounded-xl border-2 {} {} {} border-current", bg_color_class, color_class, "border-current")
                    },
                    span {
                        class: "text-2xl font-bold tabular-nums",
                        "{overall}"
                    }
                }
            }

            // Progress bar
            if overall > 0 {
                div {
                    class: "relative h-2 w-full overflow-hidden rounded-full bg-secondary",
                    div {
                        class: format!(
                            "h-full rounded-full transition-all duration-500 {}",
                            get_score_bar_color_class(overall)
                        ),
                        style: format!("width: {}%", overall),
                    }
                    // Threshold marker
                    div {
                        class: "absolute top-0 h-full w-0.5 bg-foreground/50",
                        style: format!("left: {}%", minimum_gate),
                    }
                }
            }

            // Dimension breakdown (collapsible)
            if !dimension_scores.is_empty() {
                div {
                    class: "space-y-2",
                    div {
                        class: "flex items-center justify-between",
                        span {
                            class: "text-xs font-medium text-muted-foreground",
                            "{dimensions_text}"
                        }
                        button {
                            "type": "button",
                            onclick: move |_| {
                                show_details.toggle();
                            },
                            class: "text-xs text-primary hover:underline",
                            if show_details_val { "Hide details" } else { "Show details" }
                        }
                    }

                    // Dimension scores
                    if show_details_val {
                        div {
                            class: "space-y-2 pt-2",
                            for dimension in &dimension_scores {
                                {
                                    let label = dimension.dimension.label();
                                    let description = dimension.dimension.description();
                                    let score = dimension.score;
                                    let passes = dimension.passes(minimum_gate);
                                    let bar_color = get_score_bar_color_class(score);

                                    rsx! {
                                        div {
                                            class: "space-y-1",
                                            div {
                                                class: "flex items-center justify-between text-xs",
                                                span {
                                                    class: "font-medium text-foreground",
                                                    "{label}"
                                                }
                                                span {
                                                    class: format!(
                                                        "font-mono tabular-nums {}",
                                                        if passes {
                                                            "text-chart-2"
                                                        } else {
                                                            "text-chart-4"
                                                        }
                                                    ),
                                                    "{score}"
                                                }
                                            }
                                            div {
                                                class: "relative h-1.5 w-full overflow-hidden rounded-full bg-secondary",
                                                div {
                                                    class: format!("h-full rounded-full transition-all duration-300 {}", bar_color),
                                                    style: format!("width: {}%", score),
                                                }
                                            }
                                            p {
                                                class: "text-[10px] text-muted-foreground",
                                                "{description}"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Issues summary
            if !issues.is_empty() {
                div {
                    class: "space-y-2 pt-2 border-t border-border/50",
                    // Severity badges
                    div {
                        class: "flex flex-wrap items-center gap-2",
                        if !critical_issues.is_empty() {
                            span {
                                class: "inline-flex items-center gap-1 rounded-full bg-chart-4/10 px-2 py-0.5 text-[10px] font-medium text-chart-4 border border-chart-4/20",
                                svg {
                                    xmlns: "http://www.w3.org/2000/svg",
                                    width: "10",
                                    height: "10",
                                    view_box: "0 0 16 16",
                                    fill: "currentColor",
                                    circle { cx: "8", cy: "8", r: "6" }
                                }
                                span {
                                    "{critical_issues_len} Critical"
                                }
                            }
                        }
                        if !error_issues.is_empty() {
                            span {
                                class: "inline-flex items-center gap-1 rounded-full bg-chart-4/10 px-2 py-0.5 text-[10px] font-medium text-chart-4 border border-chart-4/20",
                                svg {
                                    xmlns: "http://www.w3.org/2000/svg",
                                    width: "10",
                                    height: "10",
                                    view_box: "0 0 16 16",
                                    fill: "currentColor",
                                    rect { x: "1", y: "1", width: "14", height: "14", rx: "2" }
                                }
                                span {
                                    "{error_issues_len} Errors"
                                }
                            }
                        }
                        if !warning_issues.is_empty() {
                            span {
                                class: "inline-flex items-center gap-1 rounded-full bg-chart-3/10 px-2 py-0.5 text-[10px] font-medium text-chart-3 border border-chart-3/20",
                                svg {
                                    xmlns: "http://www.w3.org/2000/svg",
                                    width: "10",
                                    height: "10",
                                    view_box: "0 0 16 16",
                                    fill: "currentColor",
                                    path { d: "M8 1l1 5h5l-4 4 1 5-4-3-4 3 1-5-4-4h5z" }
                                }
                                span {
                                    "{warning_issues_len} Warnings"
                                }
                            }
                        }
                    }

                    // Issue list (when details shown)
                    if show_details_val {
                        div {
                            class: "space-y-1.5 pt-2",
                            for issue in &issues {
                                {
                                    let icon = match issue.severity {
                                        IssueSeverity::Critical => rsx! {
                                            svg {
                                                xmlns: "http://www.w3.org/2000/svg",
                                                width: "12",
                                                height: "12",
                                                view_box: "0 0 16 16",
                                                fill: "currentColor",
                                                class: "text-chart-4 shrink-0",
                                                circle { cx: "8", cy: "8", r: "6" }
                                            }
                                        },
                                        IssueSeverity::Error => rsx! {
                                            svg {
                                                xmlns: "http://www.w3.org/2000/svg",
                                                width: "12",
                                                height: "12",
                                                view_box: "0 0 16 16",
                                                fill: "currentColor",
                                                class: "text-chart-4 shrink-0",
                                                rect { x: "1", y: "1", width: "14", height: "14", rx: "2" }
                                            }
                                        },
                                        IssueSeverity::Warning => rsx! {
                                            svg {
                                                xmlns: "http://www.w3.org/2000/svg",
                                                width: "12",
                                                height: "12",
                                                view_box: "0 0 16 16",
                                                fill: "currentColor",
                                                class: "text-chart-3 shrink-0",
                                                path { d: "M8 1l1 5h5l-4 4 1 5-4-3-4 3 1-5-4-4h5z" }
                                            }
                                        },
                                    };

                                    rsx! {
                                        div {
                                            class: "flex items-start gap-2 text-[10px]",
                                            {icon}
                                            div {
                                                class: "flex-1",
                                                span {
                                                    class: "font-medium text-foreground/70",
                                                    "{issue.dimension.label()}: "
                                                }
                                                span {
                                                    class: "text-muted-foreground",
                                                    "{issue.message}"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Get text color class based on score
fn get_score_color_class(score: u8) -> &'static str {
    match score {
        0..=49 => "text-chart-4",
        50..=69 => "text-chart-3",
        70..=89 => "text-chart-2",
        90..=100 => "text-chart-1",
        _ => "text-foreground",
    }
}

/// Get background color class based on score
fn get_score_bg_color_class(score: u8) -> &'static str {
    match score {
        0..=49 => "bg-chart-4/10",
        50..=69 => "bg-chart-3/10",
        70..=89 => "bg-chart-2/10",
        90..=100 => "bg-chart-1/10",
        _ => "bg-muted",
    }
}

/// Get progress bar color class based on score
fn get_score_bar_color_class(score: u8) -> &'static str {
    match score {
        0..=49 => "bg-chart-4",
        50..=69 => "bg-chart-3",
        70..=89 => "bg-chart-2",
        90..=100 => "bg-chart-1",
        _ => "bg-muted-foreground",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lattice::quality::{DimensionScore, QualityIssue, IssueSeverity};

    #[test]
    fn test_get_score_color_class() {
        assert_eq!(get_score_color_class(0), "text-chart-4");
        assert_eq!(get_score_color_class(49), "text-chart-4");
        assert_eq!(get_score_color_class(50), "text-chart-3");
        assert_eq!(get_score_color_class(69), "text-chart-3");
        assert_eq!(get_score_color_class(70), "text-chart-2");
        assert_eq!(get_score_color_class(89), "text-chart-2");
        assert_eq!(get_score_color_class(90), "text-chart-1");
        assert_eq!(get_score_color_class(100), "text-chart-1");
    }

    #[test]
    fn test_get_score_bg_color_class() {
        assert_eq!(get_score_bg_color_class(0), "bg-chart-4/10");
        assert_eq!(get_score_bg_color_class(49), "bg-chart-4/10");
        assert_eq!(get_score_bg_color_class(50), "bg-chart-3/10");
        assert_eq!(get_score_bg_color_class(70), "bg-chart-2/10");
        assert_eq!(get_score_bg_color_class(90), "bg-chart-1/10");
    }

    #[test]
    fn test_get_score_bar_color_class() {
        assert_eq!(get_score_bar_color_class(0), "bg-chart-4");
        assert_eq!(get_score_bar_color_class(50), "bg-chart-3");
        assert_eq!(get_score_bar_color_class(70), "bg-chart-2");
        assert_eq!(get_score_bar_color_class(90), "bg-chart-1");
    }

    #[test]
    fn test_minimum_gate_constant() {
        assert_eq!(MINIMUM_GATE, 70);
    }
}
