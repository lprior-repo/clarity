//! Input component - shadcn-style text input
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

use dioxus::prelude::*;

/// Input component props
#[derive(Props, PartialEq, Clone)]
pub struct InputProps {
    /// Input type (text, email, password, etc.)
    #[props(default = "text".to_string())]
    pub r#type: String,

    /// Placeholder text
    #[props(default = String::new())]
    pub placeholder: String,

    /// Value binding
    #[props(default = String::new())]
    pub value: String,

    /// Disabled state
    #[props(default = false)]
    pub disabled: bool,

    /// Additional CSS classes
    #[props(default = String::new())]
    pub class: String,

    /// On input change handler
    #[props(default)]
    pub oninput: Option<EventHandler<String>>,

    /// On blur handler
    #[props(default)]
    pub onblur: Option<EventHandler<()>>,

    /// ID attribute
    #[props(default = String::new())]
    pub id: String,

    /// Name attribute
    #[props(default = String::new())]
    pub name: String,

    /// Required attribute
    #[props(default = false)]
    pub required: bool,

    /// Autocomplete attribute
    #[props(default = String::new())]
    pub autocomplete: String,
}

/// Input component - styled text input field
#[component]
pub fn Input(props: InputProps) -> Element {
    let base_classes = "flex h-10 w-full rounded-md border border-border bg-background px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground/50 focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 focus:ring-offset-background disabled:cursor-not-allowed disabled:opacity-50 transition-colors";

    let class_str = if props.class.is_empty() {
        base_classes.to_string()
    } else {
        format!("{} {}", base_classes, props.class)
    };

    rsx! {
        input {
            type: props.r#type.clone(),
            placeholder: if !props.placeholder.is_empty() { Some(props.placeholder.clone()) } else { None },
            value: props.value.clone(),
            disabled: props.disabled,
            required: props.required,
            id: if !props.id.is_empty() { Some(props.id.clone()) } else { None },
            name: if !props.name.is_empty() { Some(props.name.clone()) } else { None },
            autocomplete: if !props.autocomplete.is_empty() { Some(props.autocomplete.clone()) } else { None },
            class: class_str,
            oninput: move |e| {
                if let Some(handler) = &props.oninput {
                    handler.call(e.value().clone());
                }
            },
            onblur: move |_| {
                if let Some(handler) = &props.onblur {
                    handler.call(());
                }
            },
        }
    }
}
