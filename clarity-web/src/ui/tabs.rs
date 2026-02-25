//! Tabs component - shadcn-style tabbed interface
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

use dioxus::prelude::*;

/// Tabs component props
#[derive(Props, PartialEq, Clone)]
pub struct TabsProps {
    /// Default selected tab value
    pub default_value: String,

    /// Tab content
    pub children: Element,

    /// Additional CSS classes
    #[props(default = String::new())]
    pub class: String,

    /// On value change handler
    #[props(default)]
    pub on_value_change: Option<EventHandler<String>>,
}

/// TabsList component props
#[derive(Props, PartialEq, Clone)]
pub struct TabsListProps {
    /// Tab triggers
    pub children: Element,

    /// Additional CSS classes
    #[props(default = String::new())]
    pub class: String,
}

/// TabsTrigger component props
#[derive(Props, PartialEq, Clone)]
pub struct TabsTriggerProps {
    /// Tab value identifier
    pub value: String,

    /// Trigger content
    pub children: Element,

    /// Additional CSS classes
    #[props(default = String::new())]
    pub class: String,

    /// Disabled state
    #[props(default = false)]
    pub disabled: bool,
}

/// TabsContent component props
#[derive(Props, PartialEq, Clone)]
pub struct TabsContentProps {
    /// Tab value identifier
    pub value: String,

    /// Tab content
    pub children: Element,

    /// Additional CSS classes
    #[props(default = String::new())]
    pub class: String,
}

/// Tabs context for sharing state
#[derive(Clone, Copy)]
struct TabsContext {
    selected: Signal<String>,
    on_change: Option<EventHandler<String>>,
}

/// Tabs root component
#[component]
pub fn Tabs(props: TabsProps) -> Element {
    let selected = use_signal(|| props.default_value.clone());

    let context = use_context_provider(|| TabsContext {
        selected,
        on_change: props.on_value_change.clone(),
    });

    let class_str = if props.class.is_empty() {
        String::new()
    } else {
        format!(" {}", props.class)
    };

    rsx! {
        div {
            class: "w-full{class_str}",
            {props.children}
        }
    }
}

/// TabsList component - container for tab triggers
#[component]
pub fn TabsList(props: TabsListProps) -> Element {
    let class_str = if props.class.is_empty() {
        "inline-flex h-10 items-center justify-center rounded-md bg-muted p-1 text-muted-foreground".to_string()
    } else {
        format!("inline-flex h-10 items-center justify-center rounded-md bg-muted p-1 text-muted-foreground {}", props.class)
    };

    rsx! {
        div {
            class: class_str,
            role: "tablist",
            {props.children}
        }
    }
}

/// TabsTrigger component - individual tab button
#[component]
pub fn TabsTrigger(props: TabsTriggerProps) -> Element {
    let mut context = use_context::<TabsContext>();
    let is_selected = *context.selected.read() == props.value;

    let base_classes = "inline-flex items-center justify-center whitespace-nowrap rounded-sm px-3 py-1.5 text-sm font-medium ring-offset-background transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50";

    let state_class = if is_selected {
        "bg-background text-foreground shadow-sm"
    } else {
        ""
    };

    let class_str = if props.class.is_empty() {
        format!("{} {}", base_classes, state_class)
    } else {
        format!("{} {} {}", base_classes, state_class, props.class)
    };

    rsx! {
        button {
            class: class_str,
            role: "tab",
            aria_selected: is_selected,
            disabled: props.disabled,
            onclick: move |_| {
                context.selected.set(props.value.clone());
                if let Some(handler) = &context.on_change {
                    handler.call(props.value.clone());
                }
            },
            {props.children}
        }
    }
}

/// TabsContent component - tab panel content
#[component]
pub fn TabsContent(props: TabsContentProps) -> Element {
    let context = use_context::<TabsContext>();
    let is_selected = *context.selected.read() == props.value;

    if !is_selected {
        return rsx! {};
    }

    let base_classes = "mt-2 ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2";

    let class_str = if props.class.is_empty() {
        base_classes.to_string()
    } else {
        format!("{} {}", base_classes, props.class)
    };

    rsx! {
        div {
            class: class_str,
            role: "tabpanel",
            {props.children}
        }
    }
}
