//! Select component - shadcn-style dropdown select
//!
//! Based on shadcn-ui Select component pattern.
//! Provides a dropdown select with trigger, content, and items.
#![allow(clippy::missing_errors_doc, clippy::manual_let_else, clippy::match_wild_err_arm, clippy::match_like_matches_macro)]
#![allow(clippy::missing_panics_doc)]

use dioxus::prelude::*;
use std::collections::HashMap;

/// Select context for sharing state between components
#[derive(Clone, Copy)]
struct SelectContext {
  /// Currently selected value
  selected: Signal<String>,
  /// Display labels for each value
  labels: Signal<HashMap<String, String>>,
  /// Dropdown open state
  open: Signal<bool>,
  /// On value change handler
  on_change: Option<EventHandler<String>>,
  /// Disabled state
  disabled: Signal<bool>,
}

/// Select (root) component props
#[derive(Props, PartialEq, Clone)]
pub struct SelectProps {
  /// Currently selected value (controlled)
  #[props(default = String::new())]
  pub value: String,

  /// Default value (uncontrolled)
  #[props(default = String::new())]
  pub default_value: String,

  /// Callback when selection changes
  #[props(default)]
  pub on_value_change: Option<EventHandler<String>>,

  /// Disabled state
  #[props(default = false)]
  pub disabled: bool,

  /// Select content
  pub children: Element,

  /// Additional CSS classes
  #[props(default = String::new())]
  pub class: String,
}

/// Select root component - container for select parts
#[component]
pub fn Select(props: SelectProps) -> Element {
  let selected = use_signal(|| {
    if !props.value.is_empty() {
      props.value.clone()
    } else {
      props.default_value.clone()
    }
  });

  let labels = use_signal(HashMap::<String, String>::new);
  let open = use_signal(|| false);
  let disabled = use_signal(|| props.disabled);

  let _context = use_context_provider(|| SelectContext {
    selected,
    labels,
    open,
    on_change: props.on_value_change.clone(),
    disabled,
  });

  let class_str = if props.class.is_empty() {
    String::new()
  } else {
    format!(" {}", props.class)
  };

  rsx! {
      div {
          class: "relative inline-block w-full{class_str}",
          {props.children}
      }
  }
}

/// SelectTrigger component props
#[derive(Props, PartialEq, Clone)]
pub struct SelectTriggerProps {
  /// Trigger content (usually shows selected value)
  pub children: Element,

  /// Additional CSS classes
  #[props(default = String::new())]
  pub class: String,

  /// Placeholder text when no value selected
  #[props(default = "Select an option".to_string())]
  pub placeholder: String,
}

/// SelectTrigger - button that opens the dropdown
#[component]
pub fn SelectTrigger(props: SelectTriggerProps) -> Element {
  let mut context = use_context::<SelectContext>();
  let is_disabled = (context.disabled)();
  let is_open = (context.open)();
  let icon_class = if is_open { "rotate-180" } else { "" };

  let base_classes = "flex h-10 w-full items-center justify-between rounded-md border border-border bg-background px-3 py-2 text-sm text-foreground ring-offset-background placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50 transition-colors";

  let class_str = if props.class.is_empty() {
    base_classes.to_string()
  } else {
    format!("{} {}", base_classes, props.class)
  };

  rsx! {
      button {
          class: class_str,
          r#type: "button",
          disabled: is_disabled,
          aria_expanded: is_open,
          aria_haspopup: "listbox",
          onclick: move |_| {
              if !(context.disabled)() {
                  context.open.toggle();
              }
          },
          {props.children}
          SelectIcon {
              class: icon_class.to_string(),
          }
      }
  }
}

/// SelectValue component props
#[derive(Props, PartialEq, Clone)]
pub struct SelectValueProps {
  /// Placeholder text when no value selected
  #[props(default = "Select an option".to_string())]
  pub placeholder: String,

  /// Additional CSS classes
  #[props(default = String::new())]
  pub class: String,
}

/// SelectValue - displays the currently selected value
#[component]
pub fn SelectValue(props: SelectValueProps) -> Element {
  let context = use_context::<SelectContext>();
  let selected = (context.selected)();
  let labels = (context.labels)();

  let display_text = labels
    .get(&selected)
    .cloned()
    .unwrap_or_else(|| props.placeholder.clone());

  let class_str = if props.class.is_empty() {
    String::new()
  } else {
    format!(" {}", props.class)
  };

  rsx! {
      span {
          class: "block truncate{class_str}",
          {display_text}
      }
  }
}

/// SelectContent component props
#[derive(Props, PartialEq, Clone)]
pub struct SelectContentProps {
  /// Dropdown content (SelectItems)
  pub children: Element,

  /// Additional CSS classes
  #[props(default = String::new())]
  pub class: String,

  /// Position alignment
  #[props(default = String::new())]
  pub position: String,
}

/// SelectContent - dropdown panel containing options
#[component]
pub fn SelectContent(props: SelectContentProps) -> Element {
  let context = use_context::<SelectContext>();
  let is_open = (context.open)();

  if !is_open {
    return rsx! {};
  }

  let position_class = if props.position.is_empty() {
    "top-full left-0 mt-1"
  } else {
    &props.position
  };

  let base_classes = "absolute z-50 max-h-60 w-full min-w-[8rem] overflow-auto rounded-md border border-border bg-popover p-1 text-popover-foreground shadow-lg animate-in fade-in-0 zoom-in-95";

  let class_str = if props.class.is_empty() {
    format!("{} {}", base_classes, position_class)
  } else {
    format!("{} {} {}", base_classes, position_class, props.class)
  };

  rsx! {
      div {
          class: class_str,
          role: "listbox",
          onclick: move |e| e.stop_propagation(),
          {props.children}
      }
  }
}

/// SelectItem component props
#[derive(Props, PartialEq, Clone)]
pub struct SelectItemProps {
  /// Value of this item
  pub value: String,

  /// Display text (used as label)
  #[props(default = String::new())]
  pub label: String,

  /// Item content
  pub children: Element,

  /// Disabled state
  #[props(default = false)]
  pub disabled: bool,

  /// Additional CSS classes
  #[props(default = String::new())]
  pub class: String,
}

/// SelectItem - individual option in the dropdown
#[component]
pub fn SelectItem(props: SelectItemProps) -> Element {
  let mut context = use_context::<SelectContext>();
  let is_selected = (context.selected)() == props.value;
  let is_disabled = props.disabled || (context.disabled)();

  // Register this item's label in the context
  let label = if props.label.is_empty() {
    // Use value as fallback
    props.value.clone()
  } else {
    props.label.clone()
  };

  // Update labels map
  context.labels.write().insert(props.value.clone(), label);

  let base_classes = "relative flex w-full cursor-pointer select-none items-center rounded-sm py-1.5 px-2 text-sm outline-none transition-colors";

  let state_classes = if is_disabled {
    "pointer-events-none opacity-50"
  } else if is_selected {
    "bg-accent text-accent-foreground"
  } else {
    "hover:bg-accent hover:text-accent-foreground"
  };

  let class_str = if props.class.is_empty() {
    format!("{} {}", base_classes, state_classes)
  } else {
    format!("{} {} {}", base_classes, state_classes, props.class)
  };

  let span_class = if is_selected { "ml-6" } else { "" };

  rsx! {
      div {
          class: class_str,
          role: "option",
          aria_selected: is_selected,
          aria_disabled: is_disabled,
          onclick: move |_| {
              if !is_disabled {
                  context.selected.set(props.value.clone());
                  context.open.set(false);
                  if let Some(handler) = &context.on_change {
                      handler.call(props.value.clone());
                  }
              }
          },
          if is_selected {
              SelectItemIndicator {}
          }
          span {
              class: span_class,
              {props.children}
          }
      }
  }
}

/// SelectItemIndicator - check mark for selected item
#[derive(Props, PartialEq, Clone)]
pub struct SelectItemIndicatorProps {
  /// Additional CSS classes
  #[props(default = String::new())]
  pub class: String,
}

/// Indicator showing the selected item
#[component]
pub fn SelectItemIndicator(props: SelectItemIndicatorProps) -> Element {
  let class_str = if props.class.is_empty() {
    "absolute left-2 flex h-3.5 w-3.5 items-center justify-center".to_string()
  } else {
    format!(
      "absolute left-2 flex h-3.5 w-3.5 items-center justify-center {}",
      props.class
    )
  };

  rsx! {
      span {
          class: class_str,
          svg {
              class: "h-4 w-4",
              fill: "none",
              stroke: "currentColor",
              stroke_width: "2",
              stroke_linecap: "round",
              stroke_linejoin: "round",
              view_box: "0 0 24 24",
              path { d: "M20 6L9 17l-5-5" }
          }
      }
  }
}

/// SelectIcon - dropdown chevron icon
#[derive(Props, PartialEq, Clone)]
pub struct SelectIconProps {
  /// Additional CSS classes
  #[props(default = String::new())]
  pub class: String,
}

/// Chevron icon for the select trigger
#[component]
pub fn SelectIcon(props: SelectIconProps) -> Element {
  let class_str = if props.class.is_empty() {
    "ml-2 h-4 w-4 shrink-0 opacity-50 transition-transform".to_string()
  } else {
    format!(
      "ml-2 h-4 w-4 shrink-0 opacity-50 transition-transform {}",
      props.class
    )
  };

  rsx! {
      svg {
          class: class_str,
          fill: "none",
          stroke: "currentColor",
          stroke_width: "2",
          stroke_linecap: "round",
          stroke_linejoin: "round",
          view_box: "0 0 24 24",
          path { d: "M6 9l6 6 6-6" }
      }
  }
}

/// SelectGroup component props
#[derive(Props, PartialEq, Clone)]
pub struct SelectGroupProps {
  /// Group content (label + items)
  pub children: Element,

  /// Additional CSS classes
  #[props(default = String::new())]
  pub class: String,
}

/// SelectGroup - groups related items together
#[component]
pub fn SelectGroup(props: SelectGroupProps) -> Element {
  let class_str = if props.class.is_empty() {
    "w-full".to_string()
  } else {
    format!("w-full {}", props.class)
  };

  rsx! {
      div {
          class: class_str,
          role: "group",
          {props.children}
      }
  }
}

/// SelectLabel component props
#[derive(Props, PartialEq, Clone)]
pub struct SelectLabelProps {
  /// Label content
  pub children: Element,

  /// Additional CSS classes
  #[props(default = String::new())]
  pub class: String,
}

/// SelectLabel - label for a group of items
#[component]
pub fn SelectLabel(props: SelectLabelProps) -> Element {
  let base_classes = "px-2 py-1.5 text-sm font-semibold text-muted-foreground";

  let class_str = if props.class.is_empty() {
    base_classes.to_string()
  } else {
    format!("{} {}", base_classes, props.class)
  };

  rsx! {
      div {
          class: class_str,
          {props.children}
      }
  }
}

/// SelectSeparator component props
#[derive(Props, PartialEq, Clone)]
pub struct SelectSeparatorProps {
  /// Additional CSS classes
  #[props(default = String::new())]
  pub class: String,
}

/// SelectSeparator - visual divider between items
#[component]
pub fn SelectSeparator(props: SelectSeparatorProps) -> Element {
  let base_classes = "my-1 h-px w-full bg-border -mx-1 px-2";

  let class_str = if props.class.is_empty() {
    base_classes.to_string()
  } else {
    format!("{} {}", base_classes, props.class)
  };

  rsx! {
      div {
          class: class_str,
          role: "separator",
      }
  }
}

/// SelectScrollUpButton - scroll indicator for long lists
#[derive(Props, PartialEq, Clone)]
pub struct SelectScrollUpButtonProps {
  /// Additional CSS classes
  #[props(default = String::new())]
  pub class: String,
}

/// Scroll up button for long lists
#[component]
pub fn SelectScrollUpButton(props: SelectScrollUpButtonProps) -> Element {
  let class_str = if props.class.is_empty() {
    "flex cursor-default items-center justify-center py-1".to_string()
  } else {
    format!(
      "flex cursor-default items-center justify-center py-1 {}",
      props.class
    )
  };

  rsx! {
      div {
          class: class_str,
          svg {
              class: "h-4 w-4",
              fill: "none",
              stroke: "currentColor",
              stroke_width: "2",
              stroke_linecap: "round",
              stroke_linejoin: "round",
              view_box: "0 0 24 24",
              path { d: "M18 15l-6-6-6 6" }
          }
      }
  }
}

/// SelectScrollDownButton - scroll indicator for long lists
#[derive(Props, PartialEq, Clone)]
pub struct SelectScrollDownButtonProps {
  /// Additional CSS classes
  #[props(default = String::new())]
  pub class: String,
}

/// Scroll down button for long lists
#[component]
pub fn SelectScrollDownButton(props: SelectScrollDownButtonProps) -> Element {
  let class_str = if props.class.is_empty() {
    "flex cursor-default items-center justify-center py-1".to_string()
  } else {
    format!(
      "flex cursor-default items-center justify-center py-1 {}",
      props.class
    )
  };

  rsx! {
      div {
          class: class_str,
          svg {
              class: "h-4 w-4",
              fill: "none",
              stroke: "currentColor",
              stroke_width: "2",
              stroke_linecap: "round",
              stroke_linejoin: "round",
              view_box: "0 0 24 24",
              path { d: "M6 9l6 6 6-6" }
          }
      }
  }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::float_cmp, clippy::needless_collect, clippy::unnecessary_debug_formatting, clippy::match_same_arms, clippy::option_if_let_else, clippy::suspicious_else_formatting, clippy::manual_let_else, clippy::match_wild_err_arm, clippy::match_like_matches_macro, clippy::needless_pass_by_value)]
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
