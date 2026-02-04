use dioxus::prelude::*;

pub fn app() -> Element {
    rsx! {
        div {
            h1 { "Clarity Application" }
            p { "Welcome to the modern fullstack Dioxus application!" }
        }
    }
}
