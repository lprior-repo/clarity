//! Checkbox component - shadcn-style checkbox
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

use dioxus::prelude::*;

/// Checkbox component props
#[derive(Props, PartialEq, Clone)]
pub struct CheckboxProps {
    /// Controlled checked state
    #[props(default = false)]
    pub checked: bool,

    /// Callback when checked state changes
    #[props(default)]
    pub on_checked_change: Option<EventHandler<bool>>,

    /// Disabled state
    #[props(default = false)]
    pub disabled: bool,

    /// Additional CSS classes
    #[props(default = String::new())]
    pub class: String,

    /// Accessible name for the checkbox
    #[props(default = String::new())]
    pub aria_label: String,

    /// Unique identifier
    #[props(default = String::new())]
    pub id: String,
}

/// Checkmark icon component
#[allow(non_snake_case)]
fn CheckIcon() -> Element {
    rsx! {
        svg {
            class: "h-3 w-3",
            xmlns: "http://www.w3.org/2000/svg",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "3",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path {
                d: "M20 6L9 17l-5-5"
            }
        }
    }
}

/// Checkbox component - animated checkbox with checkmark
///
/// Follows shadcn-ui patterns with:
/// - Box: h-4 w-4 rounded border border-primary bg-background
/// - Checked: bg-primary text-primary-foreground with checkmark icon
/// - Focus: ring-2 ring-ring ring-offset-2
#[component]
pub fn Checkbox(props: CheckboxProps) -> Element {
    let mut checked = use_signal(|| props.checked);

    // Sync external prop changes to internal signal
    use_effect(move || {
        checked.set(props.checked);
    });

    let base_classes = "peer h-4 w-4 shrink-0 rounded border border-primary ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50";

    let state_class = if *checked.read() {
        "bg-primary text-primary-foreground border-primary"
    } else {
        "bg-background"
    };

    let class_str = if props.class.is_empty() {
        format!("{} {}", base_classes, state_class)
    } else {
        format!("{} {} {}", base_classes, state_class, props.class)
    };

    let aria_label = if props.aria_label.is_empty() {
        "Checkbox"
    } else {
        &props.aria_label
    };

    rsx! {
        button {
            class: class_str,
            role: "checkbox",
            aria_checked: *checked.read(),
            aria_label: aria_label,
            id: if props.id.is_empty() { None } else { Some(props.id.as_str()) },
            disabled: props.disabled,
            onclick: move |_| {
                let new_checked = !*checked.read();
                checked.set(new_checked);
                if let Some(handler) = &props.on_checked_change {
                    handler.call(new_checked);
                }
            },
            if *checked.read() {
                CheckIcon {}
            }
        }
    }
}
