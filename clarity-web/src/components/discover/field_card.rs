#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use dioxus::prelude::*;

/// Confidence level for a field
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Confidence {
    #[default]
    Low,
    Medium,
    High,
}

impl Confidence {
    /// Get display text for confidence
    fn display(&self) -> &'static str {
        match self {
            Self::Low => "Low",
            Self::Medium => "Med",
            Self::High => "High",
        }
    }

    /// Get color classes for confidence badge
    fn badge_classes(&self) -> &'static str {
        match self {
            Self::Low => "bg-chart-4/10 text-chart-4 border-chart-4/20",
            Self::Medium => "bg-chart-3/10 text-chart-3 border-chart-3/20",
            Self::High => "bg-chart-2/10 text-chart-2 border-chart-2/20",
        }
    }

    /// Check if confidence is low (for auto-expand)
    fn is_low(&self) -> bool {
        matches!(self, Self::Low)
    }
}

/// Field data for review card
#[derive(Clone, Debug, PartialEq)]
pub struct FieldData {
    pub id: String,
    pub title: String,
    pub content: String,
    pub confidence: Confidence,
    pub locked: bool,
}

impl Default for FieldData {
    fn default() -> Self {
        Self {
            id: String::new(),
            title: String::new(),
            content: String::new(),
            confidence: Confidence::default(),
            locked: false,
        }
    }
}

/// Props for FieldCard component
#[derive(Clone, Props, PartialEq)]
pub struct FieldCardProps {
    /// Field data
    pub field: Signal<FieldData>,
    /// Callback when content is edited
    pub on_edit: Option<String>,
}

/// Field review card component
///
/// Displays a field with:
/// - Title header
/// - Confidence badge
/// - Editable textarea
/// - Lock/unlock button
/// - Character count
///
/// Low-confidence cards are auto-expanded.
#[component]
pub fn FieldCard(props: FieldCardProps) -> Element {
    let field = props.field;
    let mut is_editing = use_signal(|| match field.read().confidence.is_low() {
        true => true,
        false => false,
    });
    let mut local_content = use_signal(|| field.read().content.clone());

    // Sync local content when field changes externally
    use_effect({
        move || {
            let field_read = field.read();
            let local_content_val = local_content.read().to_string();
            if local_content_val != field_read.content {
                *local_content.write() = field_read.content.clone();
            }
        }
    });

    let field_read = field.read();
    let id = field_read.id.clone();
    let title = field_read.title.clone();
    let confidence = field_read.confidence;
    let locked = field_read.locked;
    let content_len = local_content.read().len();
    drop(field_read);

    let on_toggle_lock = {
        let mut field = field.clone();
        move |_| {
            let mut field_write = field.write();
            field_write.locked = !field_write.locked;
        }
    };

    let on_toggle_edit = {
        move |_| {
            is_editing.toggle();
        }
    };

    let on_content_change = {
        let mut field = field.clone();
        move |e: Event<FormData>| {
            let new_content = e.value();
            *local_content.write() = new_content.clone();
            let mut field_write = field.write();
            field_write.content = new_content;
        }
    };

    let confidence_badge = rsx! {
        span {
            class: format!(
                "inline-flex items-center rounded-md border px-2 py-0.5 text-xs font-medium {}",
                confidence.badge_classes()
            ),
            "{confidence.display()}"
        }
    };

    let lock_button = rsx! {
        button {
            "type": "button",
            onclick: on_toggle_lock,
            class: "shrink-0 rounded p-1.5 text-muted-foreground/60 transition-colors hover:bg-secondary hover:text-foreground disabled:opacity-50",
            disabled: *is_editing.read(),
            aria_label: if locked { "Unlock field" } else { "Lock field" },
            if locked {
                // Lock icon
                svg {
                    xmlns: "http://www.w3.org/2000/svg",
                    width: "16",
                    height: "16",
                    view_box: "0 0 24 24",
                    fill: "none",
                    stroke: "currentColor",
                    stroke_width: "2",
                    stroke_linecap: "round",
                    stroke_linejoin: "round",
                    rect { x: "3", y: "11", width: "18", height: "11", rx: "2", ry: "2" }
                    path { d: "M7 11V7a5 5 0 0 1 10 0v4" }
                }
            } else {
                // Unlock icon
                svg {
                    xmlns: "http://www.w3.org/2000/svg",
                    width: "16",
                    height: "16",
                    view_box: "0 0 24 24",
                    fill: "none",
                    stroke: "currentColor",
                    stroke_width: "2",
                    stroke_linecap: "round",
                    stroke_linejoin: "round",
                    rect { x: "3", y: "11", width: "18", height: "11", rx: "2", ry: "2" }
                    path { d: "M7 11V7a5 5 0 0 1 9.9-1" }
                }
            }
        }
    };

    let edit_button = rsx! {
        button {
            "type": "button",
            onclick: on_toggle_edit,
            class: "shrink-0 rounded p-1.5 text-muted-foreground/60 transition-colors hover:bg-secondary hover:text-foreground disabled:opacity-50",
            disabled: locked,
            aria_label: if *is_editing.read() { "Collapse" } else { "Expand" },
            if *is_editing.read() {
                // Chevron up (collapse)
                svg {
                    xmlns: "http://www.w3.org/2000/svg",
                    width: "16",
                    height: "16",
                    view_box: "0 0 24 24",
                    fill: "none",
                    stroke: "currentColor",
                    stroke_width: "2",
                    stroke_linecap: "round",
                    stroke_linejoin: "round",
                    path { d: "m18 15-6-6-6 6" }
                }
            } else {
                // Chevron down (expand)
                svg {
                    xmlns: "http://www.w3.org/2000/svg",
                    width: "16",
                    height: "16",
                    view_box: "0 0 24 24",
                    fill: "none",
                    stroke: "currentColor",
                    stroke_width: "2",
                    stroke_linecap: "round",
                    stroke_linejoin: "round",
                    path { d: "m6 9 6 6 6-6" }
                }
            }
        }
    };

    rsx! {
        div {
            class: "overflow-hidden rounded-lg border border-border/50 bg-card shadow-sm transition-all hover:shadow-md",
            id: "{id}",

            // Header
            div {
                class: "flex items-center justify-between border-b border-border/50 bg-muted/30 px-4 py-3",

                // Title and confidence
                div {
                    class: "flex items-center gap-3",
                    h3 {
                        class: "text-sm font-semibold text-foreground",
                        "{title}"
                    }
                    {confidence_badge}
                }

                // Actions
                div {
                    class: "flex items-center gap-1",
                    {edit_button}
                    {lock_button}
                }
            }

            // Content area
            div {
                class: "p-4",
                if *is_editing.read() {
                    // Editable textarea
                    div {
                        class: "space-y-2",
                        textarea {
                            value: "{local_content.read()}",
                            oninput: on_content_change,
                            disabled: locked,
                            placeholder: "Enter content...",
                            rows: "4",
                            class: format!(
                                "w-full resize-none rounded-md border bg-background px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground/50 focus:outline-none focus:ring-2 focus:ring-ring disabled:cursor-not-allowed disabled:opacity-50 {}",
                                if locked {
                                    "border-border/50"
                                } else {
                                    "border-primary/20"
                                }
                            ),
                        }
                        div {
                            class: "flex items-center justify-between text-xs",
                            span {
                                class: "text-muted-foreground/60",
                                if locked {
                                    "Locked — unlock to edit"
                                } else {
                                    "Editing enabled"
                                }
                            }
                            span {
                                class: "font-mono text-muted-foreground/50",
                                "{content_len} chars"
                            }
                        }
                    }
                } else {
                    // Read-only preview
                    div {
                        class: "text-sm text-foreground/80 leading-relaxed whitespace-pre-wrap",
                        if local_content.read().trim().is_empty() {
                            span {
                                class: "italic text-muted-foreground/50",
                                "No content yet"
                            }
                        } else {
                            "{local_content.read()}"
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confidence_display() {
        assert_eq!(Confidence::Low.display(), "Low");
        assert_eq!(Confidence::Medium.display(), "Med");
        assert_eq!(Confidence::High.display(), "High");
    }

    #[test]
    fn test_confidence_is_low() {
        assert!(Confidence::Low.is_low());
        assert!(!Confidence::Medium.is_low());
        assert!(!Confidence::High.is_low());
    }

    #[test]
    fn test_confidence_badge_classes() {
        assert!(Confidence::Low.badge_classes().contains("chart-4"));
        assert!(Confidence::Medium.badge_classes().contains("chart-3"));
        assert!(Confidence::High.badge_classes().contains("chart-2"));
    }

    #[test]
    fn test_field_data_default() {
        let field = FieldData::default();
        assert_eq!(field.id, "");
        assert_eq!(field.title, "");
        assert_eq!(field.content, "");
        assert_eq!(field.confidence, Confidence::Low);
        assert!(!field.locked);
    }

    #[test]
    fn test_field_data_with_values() {
        let field = FieldData {
            id: "test-id".to_string(),
            title: "Test Field".to_string(),
            content: "Test content".to_string(),
            confidence: Confidence::High,
            locked: true,
        };
        assert_eq!(field.id, "test-id");
        assert_eq!(field.title, "Test Field");
        assert_eq!(field.content, "Test content");
        assert_eq!(field.confidence, Confidence::High);
        assert!(field.locked);
    }

    #[test]
    fn test_confidence_equality() {
        assert_eq!(Confidence::Low, Confidence::Low);
        assert_eq!(Confidence::Medium, Confidence::Medium);
        assert_eq!(Confidence::High, Confidence::High);
        assert_ne!(Confidence::Low, Confidence::High);
    }

    #[test]
    fn test_field_data_equality() {
        let field1 = FieldData {
            id: "id".to_string(),
            title: "Title".to_string(),
            content: "Content".to_string(),
            confidence: Confidence::Medium,
            locked: false,
        };
        let field2 = FieldData {
            id: "id".to_string(),
            title: "Title".to_string(),
            content: "Content".to_string(),
            confidence: Confidence::Medium,
            locked: false,
        };
        assert_eq!(field1, field2);
    }
}
