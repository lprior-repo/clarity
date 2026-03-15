#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

use dioxus::prelude::*;

use crate::components::discover::ProgressiveDiscover;

/// HomePage - Main landing page with Discover phase
///
/// Renders the ProgressiveDiscover component which orchestrates the full
/// flow: Prompt -> Extracting -> ConfirmingFields -> Preview -> KirkCompilation -> Locked
#[component]
pub fn HomePage() -> Element {
    rsx! {
        div {
            class: "min-h-screen bg-background text-foreground flex flex-col",
            // Header
            header {
                class: "border-b border-border bg-card",
                div {
                    class: "container mx-auto px-4 py-4",
                    div {
                        class: "flex items-center justify-between",
                        h1 {
                            class: "text-2xl font-bold text-foreground",
                            "Clarity Planner"
                        }
                        span {
                            class: "text-sm text-muted-foreground",
                            "Double Diamond Planning IDE"
                        }
                    }
                }
            }

            // Main content
            main {
                class: "container mx-auto px-4 py-8 flex-1",
                ProgressiveDiscover {
                    extraction_provider: None,
                    initial_prompt: None,
                    on_complete: None,
                    on_refine: None,
                }
            }

            // Footer
            footer {
                class: "border-t border-border bg-card mt-auto",
                div {
                    class: "container mx-auto px-4 py-4",
                    p {
                        class: "text-sm text-muted-foreground text-center",
                        "Clarity Planner - Intent-Driven Development"
                    }
                }
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
  //! Tests for crash recovery functionality in HomePage

  use crate::hooks::has_recoverable_session;
  use crate::kirk::progressive_discover::KirkContract16;

  /// Test that has_recoverable_session function exists and can be called
  #[test]
  fn test_has_recoverable_session_exists() {
    // This test verifies the function exists and returns a bool
    // In non-wasm32 targets, it should return false
    let result = has_recoverable_session();
    assert!(!result, "On non-wasm32 targets, has_recoverable_session should return false");
  }

  /// Test that KirkContract16 type exists and can be used in handler signatures
  #[test]
  fn test_kirk_contract16_handler_signature_type_exists() {
    // Verify that we can create a handler that receives KirkContract16
    fn _assert_handler_signature(contract: KirkContract16) {
      // Log the contract for debugging
      tracing::info!("Contract ready for Bead Factory: {:?}", contract);
    }
    // The handler signature should accept KirkContract16
    let _handler: fn(KirkContract16) = _assert_handler_signature;
  }
}
