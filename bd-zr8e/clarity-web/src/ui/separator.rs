#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

use dioxus::prelude::*;

#[derive(Clone, Debug, PartialEq, Props)]
pub struct SeparatorProps {
  #[props(default)]
  pub class: String,
  #[props(default)]
  pub orientation: String,
  #[props(default)]
  pub decorative: bool,
}

#[component]
pub fn Separator(props: SeparatorProps) -> Element {
  let is_vertical = props.orientation == "vertical";

  rsx! {
      div {
          class: format!(
              "shrink-0 bg-border {} {}",
              if is_vertical { "h-full w-px" } else { "h-px w-full" },
              props.class
          ),
          role: if !props.decorative { "separator" } else { "none" },
          "aria-orientation": props.orientation.clone()
      }
  }
}
