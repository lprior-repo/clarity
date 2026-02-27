#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

// Include the prompts module content directly
include!("lib_prompts.rs");

// Phase definition
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Phase {
  pub key: &'static str,
  pub label: &'static str,
}

/// All phases in the planning process
pub const PHASES: &[Phase] = &[
  Phase {
    key: "discover",
    label: "Discover",
  },
  Phase {
    key: "define",
    label: "Define",
  },
  Phase {
    key: "develop",
    label: "Develop",
  },
  Phase {
    key: "deliver",
    label: "Deliver",
  },
];

/// Right panel tab type
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum RightTab {
  #[default]
  Plan,
  Graph,
  State,
}

/// Tab definition with label
#[derive(Clone, Copy, Debug)]
pub struct TabDef {
  pub key: RightTab,
  pub label: &'static str,
}

/// All right panel tabs
pub const TABS: &[TabDef] = &[
  TabDef {
    key: RightTab::Plan,
    label: "Plan",
  },
  TabDef {
    key: RightTab::Graph,
    label: "Graph",
  },
  TabDef {
    key: RightTab::State,
    label: "State",
  },
];

/// User answer to a prompt step
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Answer {
  pub step_id: String,
  pub value: String,
  pub timestamp: String,
}
