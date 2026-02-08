// Dioxus frontend application entry point
//
// This is the main entry point for the Clarity web application.
// It launches the Dioxus app with hot reload enabled for development.

fn main() {
  // Launch the Dioxus application
  // Note: Hot reload is automatically enabled in debug mode by Dioxus
  dioxus::launch(clarity_client::App);
}
