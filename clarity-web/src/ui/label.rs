//! Label component - shadcn-style form label
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

use dioxus::prelude::*;

/// Label component props
#[derive(Props, PartialEq, Clone)]
pub struct LabelProps {
    /// Label text
    pub children: Element,

    /// For attribute (input ID)
    #[props(default = String::new())]
    pub r#for: String,

    /// Additional CSS classes
    #[props(default = String::new())]
    pub class: String,

    /// Required indicator (e.g., "*" symbol)
    #[props(default = String::new())]
    pub required: String,
}

/// Label component - form field label
#[component]
pub fn Label(props: LabelProps) -> Element {
    let base_classes = "text-sm font-medium text-foreground leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70";

    let class_str = if props.class.is_empty() {
        base_classes.to_string()
    } else {
        format!("{} {}", base_classes, props.class)
    };

    rsx! {
        label {
            r#for: if !props.r#for.is_empty() { Some(props.r#for.clone()) } else { None },
            class: class_str,
            {props.children}
            if !props.required.is_empty() {
                span { class: "text-destructive ml-1", {props.required.clone()} }
            }
        }
    }
}
