#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

use dioxus::prelude::*;

#[derive(Clone, Debug, PartialEq, Props)]
pub struct ScrollAreaProps {
    #[props(default)]
    pub class: String,
    pub children: Element,
}

#[component]
pub fn ScrollArea(props: ScrollAreaProps) -> Element {
    rsx! {
        div {
            class: format!("relative overflow-auto {}", props.class),
            {props.children}
        }
    }
}
