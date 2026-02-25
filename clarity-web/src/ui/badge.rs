#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum BadgeVariant {
    #[default]
    Default,
    Primary,
    Secondary,
    Destructive,
    Outline,
}

impl BadgeVariant {
    pub fn classes(&self) -> &'static str {
        match self {
            BadgeVariant::Default => "border-transparent bg-primary text-primary-foreground hover:bg-primary/80",
            BadgeVariant::Primary => "border-transparent bg-primary text-primary-foreground",
            BadgeVariant::Secondary => "border-transparent bg-secondary text-secondary-foreground hover:bg-secondary/80",
            BadgeVariant::Destructive => "border-transparent bg-destructive text-destructive-foreground hover:bg-destructive/80",
            BadgeVariant::Outline => "text-foreground",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Props)]
pub struct BadgeProps {
    #[props(default)]
    pub variant: BadgeVariant,
    #[props(default)]
    pub class: String,
    pub children: Element,
}

#[component]
pub fn Badge(props: BadgeProps) -> Element {
    rsx! {
        div {
            class: format!(
                "inline-flex items-center rounded-full border px-2.5 py-0.5 text-xs font-semibold transition-colors focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 {} {}",
                props.variant.classes(),
                props.class
            ),
            {props.children}
        }
    }
}
