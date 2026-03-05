#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::struct_field_names)]
#![allow(clippy::suspicious_else_formatting)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::ptr_arg)]
#![allow(clippy::missing_fields_in_debug)]
#![allow(clippy::must_use_unit)]
#![allow(clippy::collection_is_never_read)]
#![allow(clippy::needless_collect)]
#![allow(clippy::manual_checked_ops)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::use_self)]
#![allow(clippy::derive_partial_eq_without_eq)]

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
      BadgeVariant::Default => {
        "border-transparent bg-primary text-primary-foreground hover:bg-primary/80"
      }
      BadgeVariant::Primary => "border-transparent bg-primary text-primary-foreground",
      BadgeVariant::Secondary => {
        "border-transparent bg-secondary text-secondary-foreground hover:bg-secondary/80"
      }
      BadgeVariant::Destructive => {
        "border-transparent bg-destructive text-destructive-foreground hover:bg-destructive/80"
      }
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
