#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

use dioxus::prelude::*;

mod pages;

use pages::HomePage;

/// Main App component
#[component]
pub fn App() -> Element {
    rsx! {
        // Include Tailwind CSS
        document::Stylesheet { href: asset!("/assets/tailwind.css") }
        document::Title { "Clarity Planner - Double Diamond Planning IDE" }
        HomePage {}
    }
}
