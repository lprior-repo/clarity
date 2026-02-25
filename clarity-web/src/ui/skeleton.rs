#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

use dioxus::prelude::*;

#[derive(Clone, Debug, PartialEq, Props)]
pub struct SkeletonProps {
    #[props(default)]
    pub class: String,
}

#[component]
pub fn Skeleton(props: SkeletonProps) -> Element {
    let base_classes = "animate-pulse rounded-md bg-muted h-10 w-full";
    let class = if props.class.is_empty() {
        base_classes.to_string()
    } else {
        format!("{base_classes} {}", props.class)
    };

    rsx! {
        div {
            class,
            "aria-hidden": "true",
        }
    }
}
