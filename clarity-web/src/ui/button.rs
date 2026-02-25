#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

use dioxus::prelude::*;

/// Button component variants
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ButtonVariant {
    #[default]
    Default,
    Primary,
    Secondary,
    Destructive,
    Ghost,
}

impl ButtonVariant {
    pub fn classes(&self) -> &'static str {
        match self {
            ButtonVariant::Default => {
                "bg-secondary text-secondary-foreground hover:bg-secondary/80"
            }
            ButtonVariant::Primary => {
                "bg-primary text-primary-foreground hover:bg-primary/85"
            }
            ButtonVariant::Secondary => {
                "bg-card text-foreground border border-border hover:bg-secondary"
            }
            ButtonVariant::Destructive => {
                "bg-destructive text-destructive-foreground hover:bg-destructive/85"
            }
            ButtonVariant::Ghost => {
                "hover:bg-secondary hover:text-foreground"
            }
        }
    }
}

/// Button size variants
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ButtonSize {
    #[default]
    Default,
    Sm,
    Lg,
    Icon,
}

impl ButtonSize {
    pub fn classes(&self) -> &'static str {
        match self {
            ButtonSize::Default => "h-10 px-4 py-2",
            ButtonSize::Sm => "h-9 rounded-md px-3 text-xs",
            ButtonSize::Lg => "h-11 rounded-md px-8",
            ButtonSize::Icon => "h-10 w-10",
        }
    }
}

/// Button component properties
#[derive(Clone, Debug, PartialEq, Props)]
pub struct ButtonProps {
    #[props(default)]
    pub variant: ButtonVariant,
    #[props(default)]
    pub size: ButtonSize,
    #[props(default)]
    pub class: String,
    #[props(default)]
    pub disabled: bool,
    #[props(default)]
    pub r#type: String,
    pub onclick: Option<EventHandler<MouseEvent>>,
    pub children: Element,
}

/// Button component - shadcn-inspired for Dioxus
#[component]
pub fn Button(props: ButtonProps) -> Element {
    let base_classes = "inline-flex items-center justify-center whitespace-nowrap rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50";

    rsx! {
        button {
            class: format!("{} {} {} {}", base_classes, props.variant.classes(), props.size.classes(), props.class),
            disabled: props.disabled,
            r#type: if props.r#type.is_empty() { "button".into() } else { props.r#type.clone() },
            onclick: move |e| {
                if let Some(handler) = props.onclick {
                    handler.call(e);
                }
            },
            {props.children}
        }
    }
}
