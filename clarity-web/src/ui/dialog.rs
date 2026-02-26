//! Dialog component - shadcn-style modal dialog
//! Based on shadcn-ui Dialog pattern with Dioxus 0.7 Signal state management
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

use dioxus::prelude::*;

/// Dialog context for sharing open state between components
#[derive(Clone, Copy)]
struct DialogContext {
    open: Signal<bool>,
    on_open_change: Option<EventHandler<bool>>,
}

/// Dialog root component props
#[derive(Props, PartialEq, Clone)]
pub struct DialogProps {
    /// Controlled open state
    #[props(default = false)]
    pub open: bool,

    /// Default open state for uncontrolled mode
    #[props(default = false)]
    pub default_open: bool,

    /// Callback when open state changes
    #[props(default)]
    pub on_open_change: Option<EventHandler<bool>>,

    /// Dialog content
    pub children: Element,
}

/// Dialog root component - provides context for dialog state
#[component]
pub fn Dialog(props: DialogProps) -> Element {
    // Use controlled or uncontrolled state
    let open = use_signal(|| if props.open { true } else { props.default_open });

    let _context = use_context_provider(|| DialogContext {
        open,
        on_open_change: props.on_open_change.clone(),
    });

    rsx! {
        {props.children}
    }
}

/// DialogTrigger component props
#[derive(Props, PartialEq, Clone)]
pub struct DialogTriggerProps {
    /// Trigger content (usually a button)
    pub children: Element,

    /// Additional CSS classes
    #[props(default = String::new())]
    pub class: String,

    /// Render as child element (pass through without wrapper)
    #[props(default = false)]
    pub as_child: bool,
}

/// DialogTrigger - button that opens the dialog
#[component]
pub fn DialogTrigger(props: DialogTriggerProps) -> Element {
    let mut context = use_context::<DialogContext>();

    let class_str = if props.class.is_empty() {
        String::new()
    } else {
        format!(" {}", props.class)
    };

    if props.as_child {
        rsx! {
            div {
                class: "inline-block cursor-pointer{class_str}",
                onclick: move |_| {
                    context.open.set(true);
                    if let Some(handler) = &context.on_open_change {
                        handler.call(true);
                    }
                },
                {props.children}
            }
        }
    } else {
        rsx! {
            button {
                class: "inline-flex items-center justify-center{class_str}",
                onclick: move |_| {
                    context.open.set(true);
                    if let Some(handler) = &context.on_open_change {
                        handler.call(true);
                    }
                },
                {props.children}
            }
        }
    }
}

/// DialogContent component props
#[derive(Props, PartialEq, Clone)]
pub struct DialogContentProps {
    /// Dialog content
    pub children: Element,

    /// Additional CSS classes
    #[props(default = String::new())]
    pub class: String,

    /// Whether to show close button
    #[props(default = true)]
    pub show_close: bool,

    /// ARIA label for accessibility
    #[props(default = String::new())]
    pub aria_label: String,
}

/// Close icon SVG component
#[allow(non_snake_case)]
fn CloseIcon() -> Element {
    rsx! {
        svg {
            width: "16",
            height: "16",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            class: "h-4 w-4",
            path { d: "M18 6 6 18" }
            path { d: "m6 6 12 12" }
        }
    }
}

/// DialogContent - the main content container for the dialog
#[component]
pub fn DialogContent(props: DialogContentProps) -> Element {
    let mut context = use_context::<DialogContext>();
    let is_open = (context.open)();

    if !is_open {
        return rsx! {};
    }

    let base_classes = "fixed left-[50%] top-[50%] z-50 grid w-full max-w-lg translate-x-[-50%] translate-y-[-50%] gap-4 border bg-card p-6 shadow-lg duration-200 data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95 data-[state=closed]:slide-out-to-left-1/2 data-[state=closed]:slide-out-to-top-[48%] data-[state=open]:slide-in-from-left-1/2 data-[state=open]:slide-in-from-top-[48%] sm:rounded-lg border-border";

    let class_str = if props.class.is_empty() {
        base_classes.to_string()
    } else {
        format!("{} {}", base_classes, props.class)
    };

    let aria_label = if props.aria_label.is_empty() {
        "Dialog".to_string()
    } else {
        props.aria_label.clone()
    };

    rsx! {
        // Overlay backdrop
        div {
            class: "fixed inset-0 z-50 bg-black/80 backdrop-blur-sm data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0",
            "data-state": if is_open { "open" } else { "closed" },
            onclick: move |_| {
                context.open.set(false);
                if let Some(handler) = &context.on_open_change {
                    handler.call(false);
                }
            },
        }

        // Dialog content panel
        div {
            class: class_str,
            role: "dialog",
            "aria-modal": "true",
            "aria-label": aria_label,
            "data-state": if is_open { "open" } else { "closed" },

            // Close button
            if props.show_close {
                button {
                    class: "absolute right-4 top-4 rounded-sm opacity-70 ring-offset-background transition-opacity hover:opacity-100 focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 disabled:pointer-events-none data-[state=open]:bg-accent data-[state=open]:text-muted-foreground",
                    onclick: move |_| {
                        context.open.set(false);
                        if let Some(handler) = &context.on_open_change {
                            handler.call(false);
                        }
                    },
                    CloseIcon {}
                    span {
                        class: "sr-only",
                        "Close"
                    }
                }
            }

            {props.children}
        }
    }
}

/// DialogHeader component props
#[derive(Props, PartialEq, Clone)]
pub struct DialogHeaderProps {
    /// Header content
    pub children: Element,

    /// Additional CSS classes
    #[props(default = String::new())]
    pub class: String,
}

/// DialogHeader - container for dialog title and description
#[component]
pub fn DialogHeader(props: DialogHeaderProps) -> Element {
    let class_str = if props.class.is_empty() {
        "flex flex-col space-y-1.5 text-center sm:text-left".to_string()
    } else {
        format!("flex flex-col space-y-1.5 text-center sm:text-left {}", props.class)
    };

    rsx! {
        div {
            class: class_str,
            {props.children}
        }
    }
}

/// DialogFooter component props
#[derive(Props, PartialEq, Clone)]
pub struct DialogFooterProps {
    /// Footer content
    pub children: Element,

    /// Additional CSS classes
    #[props(default = String::new())]
    pub class: String,
}

/// DialogFooter - container for dialog actions
#[component]
pub fn DialogFooter(props: DialogFooterProps) -> Element {
    let class_str = if props.class.is_empty() {
        "flex flex-col-reverse sm:flex-row sm:justify-end sm:space-x-2".to_string()
    } else {
        format!("flex flex-col-reverse sm:flex-row sm:justify-end sm:space-x-2 {}", props.class)
    };

    rsx! {
        div {
            class: class_str,
            {props.children}
        }
    }
}

/// DialogTitle component props
#[derive(Props, PartialEq, Clone)]
pub struct DialogTitleProps {
    /// Title content
    pub children: Element,

    /// Additional CSS classes
    #[props(default = String::new())]
    pub class: String,
}

/// DialogTitle - the dialog's title heading
#[component]
pub fn DialogTitle(props: DialogTitleProps) -> Element {
    let class_str = if props.class.is_empty() {
        "text-lg font-semibold text-foreground".to_string()
    } else {
        format!("text-lg font-semibold text-foreground {}", props.class)
    };

    rsx! {
        h2 {
            class: class_str,
            {props.children}
        }
    }
}

/// DialogDescription component props
#[derive(Props, PartialEq, Clone)]
pub struct DialogDescriptionProps {
    /// Description content
    pub children: Element,

    /// Additional CSS classes
    #[props(default = String::new())]
    pub class: String,
}

/// DialogDescription - descriptive text for the dialog
#[component]
pub fn DialogDescription(props: DialogDescriptionProps) -> Element {
    let class_str = if props.class.is_empty() {
        "text-sm text-muted-foreground".to_string()
    } else {
        format!("text-sm text-muted-foreground {}", props.class)
    };

    rsx! {
        p {
            class: class_str,
            {props.children}
        }
    }
}

/// DialogClose component props
#[derive(Props, PartialEq, Clone)]
pub struct DialogCloseProps {
    /// Close button content
    pub children: Element,

    /// Additional CSS classes
    #[props(default = String::new())]
    pub class: String,
}

/// DialogClose - button that closes the dialog
#[component]
pub fn DialogClose(props: DialogCloseProps) -> Element {
    let mut context = use_context::<DialogContext>();

    let class_str = if props.class.is_empty() {
        String::new()
    } else {
        format!(" {}", props.class)
    };

    rsx! {
        button {
            class: "inline-flex items-center justify-center{class_str}",
            onclick: move |_| {
                context.open.set(false);
                if let Some(handler) = &context.on_open_change {
                    handler.call(false);
                }
            },
            {props.children}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_class_composition() {
        let base = "base-class";
        let custom = "custom-class";
        let result = if custom.is_empty() {
            base.to_string()
        } else {
            format!("{} {}", base, custom)
        };
        assert_eq!(result, "base-class custom-class");
    }

    #[test]
    fn test_empty_class_composition() {
        let base = "base-class";
        let custom = "";
        let result = if custom.is_empty() {
            base.to_string()
        } else {
            format!("{} {}", base, custom)
        };
        assert_eq!(result, "base-class");
    }
}
