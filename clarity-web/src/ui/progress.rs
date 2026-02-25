#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

use dioxus::prelude::*;

#[derive(Clone, Debug, PartialEq, Props)]
pub struct ProgressProps {
    /// The current progress value (0-100)
    #[props(default = 0)]
    pub value: u8,
    /// Additional CSS classes to apply to the container
    #[props(default)]
    pub class: String,
}

#[component]
pub fn Progress(props: ProgressProps) -> Element {
    let clamped_value = props.value.min(100);

    rsx! {
        div {
            class: format!(
                "relative h-2 w-full overflow-hidden rounded-full bg-secondary {}",
                props.class
            ),
            role: "progressbar",
            "aria-valuemin": "0",
            "aria-valuemax": "100",
            "aria-valuenow": clamped_value,

            div {
                class: "h-full bg-primary transition-all duration-300 ease-in-out",
                style: format!("width: {clamped_value}%"),
            }
        }
    }
}
