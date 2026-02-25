//! Switch component - shadcn-style toggle switch
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

use dioxus::prelude::*;

/// Switch component props
#[derive(Props, PartialEq, Clone)]
pub struct SwitchProps {
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

    /// Accessible name for the switch
    #[props(default = String::new())]
    pub aria_label: String,
}

/// Switch component - animated toggle switch
///
/// Follows shadcn-ui patterns with:
/// - Track: h-5 w-9 rounded-full, bg-input when unchecked, bg-primary when checked
/// - Thumb: h-4 w-4 rounded-full bg-background shadow, translates when checked
/// - Smooth transition animations
#[component]
pub fn Switch(props: SwitchProps) -> Element {
    let mut checked = use_signal(|| props.checked);

    // Sync external prop changes to internal signal
    use_effect(move || {
        checked.set(props.checked);
    });

    let track_base_classes = "inline-flex h-5 w-9 shrink-0 cursor-pointer items-center rounded-full border-2 border-transparent transition-colors duration-200 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background disabled:cursor-not-allowed disabled:opacity-50";

    let track_state_class = if *checked.read() {
        "bg-primary"
    } else {
        "bg-input"
    };

    let disabled_class = if props.disabled {
        " disabled:cursor-not-allowed disabled:opacity-50"
    } else {
        ""
    };

    let class_str = if props.class.is_empty() {
        format!("{} {}", track_base_classes, track_state_class)
    } else {
        format!(
            "{} {} {}{}",
            track_base_classes,
            track_state_class,
            props.class,
            disabled_class
        )
    };

    // Thumb position: translate-x-0.5 when unchecked, translate-x-4 when checked
    let thumb_translate_class = if *checked.read() {
        "translate-x-4"
    } else {
        "translate-x-0.5"
    };

    let thumb_classes = format!(
        "pointer-events-none block h-4 w-4 rounded-full bg-background shadow-lg ring-0 transition-transform duration-200 {}",
        thumb_translate_class
    );

    let aria_label = if props.aria_label.is_empty() {
        "Toggle switch"
    } else {
        &props.aria_label
    };

    rsx! {
        button {
            class: class_str,
            role: "switch",
            aria_checked: *checked.read(),
            aria_label: aria_label,
            disabled: props.disabled,
            onclick: move |_| {
                let new_checked = !*checked.read();
                checked.set(new_checked);
                if let Some(handler) = &props.on_checked_change {
                    handler.call(new_checked);
                }
            },
            span {
                class: thumb_classes,
            }
        }
    }
}
