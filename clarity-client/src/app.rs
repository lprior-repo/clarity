//! Main Dioxus desktop application component with routing
//!
//! This module contains the root App component with dioxus-router routing.
//! All navigation is handled through the dioxus-router Route configuration.

// Dioxus rsx! macro internally uses unwrap, so we allow the disallowed_methods lint.
// This is a framework limitation, not our code using unwrap.
#![allow(clippy::disallowed_methods)]

use crate::components::ErrorBoundary;
use dioxus::prelude::*;
use std::str::FromStr;

/// Application route definitions using dioxus-router
///
/// All routes in the application are defined here using dioxus-router's routing system.
/// We use manual routing instead of Routable derive due to version compatibility.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Route {
  /// Home page route
  Home,

  /// About page route
  About,

  /// Dashboard route
  Dashboard,

  /// Beads list route
  BeadsList,

  /// Create new bead route
  BeadNew,

  /// Edit bead route with dynamic ID parameter
  BeadEdit { id: String },

  /// Bead detail route with dynamic ID parameter
  BeadDetail { id: String },

  /// Settings page route
  Settings,

  /// 404 Not Found route - must be last
  NotFound { route: String },
}

impl std::str::FromStr for Route {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let parts: Vec<&str> = s.trim_start_matches('/').split('/').collect();

    match parts.as_slice() {
      [] | [""] => Ok(Self::Home),
      ["about"] => Ok(Self::About),
      ["dashboard"] => Ok(Self::Dashboard),
      ["beads"] => Ok(Self::BeadsList),
      ["beads", "new"] => Ok(Self::BeadNew),
      ["beads", id] => Ok(Self::BeadDetail {
        id: (*id).to_string(),
      }),
      ["beads", id, "edit"] => Ok(Self::BeadEdit {
        id: (*id).to_string(),
      }),
      ["settings"] => Ok(Self::Settings),
      _ => Ok(Self::NotFound {
        route: s.to_string(),
      }),
    }
  }
}

// Implement traits needed for Route with dioxus-router
// Since Routable trait doesn't exist in this version, we'll use a simpler approach
impl std::convert::TryFrom<&str> for Route {
  type Error = String;

  fn try_from(path: &str) -> Result<Self, Self::Error> {
    // Use the existing from_str implementation
    Self::from_str(path)
  }
}

impl std::fmt::Display for Route {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Home => write!(f, "/"),
      Self::About => write!(f, "/about"),
      Self::Dashboard => write!(f, "/dashboard"),
      Self::BeadsList => write!(f, "/beads"),
      Self::BeadNew => write!(f, "/beads/new"),
      Self::BeadEdit { id } => write!(f, "/beads/{id}/edit"),
      Self::BeadDetail { id } => write!(f, "/beads/{id}"),
      Self::Settings => write!(f, "/settings"),
      Self::NotFound { route } => write!(f, "/{route}"),
    }
  }
}

/// Main application component with routing
///
/// This component wraps the entire application in an `ErrorBoundary` component
/// to catch and handle any errors that occur during rendering or in event handlers.
/// Routing is handled via a simple signal-based approach.
#[component]
pub fn App() -> Element {
  let current_route = use_signal(|| Route::Home);

  let route = current_route.read().clone();
  rsx! {
      ErrorBoundary {
          show_details: cfg!(debug_assertions),
          crate::providers::RouteProvider {
              route: current_route,
              children: rsx! {
                  match route {
                      Route::Home => rsx! { Home {} },
                      Route::About => rsx! { About {} },
                      Route::Dashboard => rsx! { Dashboard {} },
                      Route::BeadsList => rsx! { BeadsList {} },
                      Route::BeadNew => rsx! { BeadNew {} },
                      Route::BeadEdit { id } => rsx! { BeadEdit { id } },
                      Route::BeadDetail { id } => rsx! { BeadDetail { id } },
                      Route::Settings => rsx! { Settings {} },
                      Route::NotFound { route } => rsx! { NotFound { route } },
                  }
              }
          }
      }
  }
}

/// Home page component
#[component]
fn Home() -> Element {
  rsx! {
      div { class: "app-container",
          h1 { "Clarity" }
          div { class: "content",
              div { class: "home-page",
                  h2 { "Welcome to Clarity" }
                  p { "A modern desktop application built with Dioxus" }
                  div { class: "nav-links",
                      NavLink { to: Route::About, "Learn More" }
                      NavLink { to: Route::Dashboard, "Dashboard" }
                      NavLink { to: Route::BeadsList, "Manage Beads" }
                  }
              }
          }
      }
  }
}

/// About page component
#[component]
fn About() -> Element {
  rsx! {
      div { class: "app-container",
          h1 { "Clarity" }
          div { class: "content",
              div { class: "about-page",
                  h2 { "About Clarity" }
                  p { "Clarity is a desktop application for managing interviews and documentation." }
                  p { "Built with Dioxus and Rust, it provides a modern, reactive native UI." }
                  div { class: "nav-links",
                      NavLink { to: Route::Home, "Back Home" }
                      NavLink { to: Route::Dashboard, "Dashboard" }
                  }
              }
          }
      }
  }
}

/// Dashboard page component
#[component]
fn Dashboard() -> Element {
  rsx! {
      div { class: "app-container",
          h1 { "Clarity Dashboard" }
          div { class: "content",
              div { class: "dashboard-page",
                  h2 { "Dashboard" }
                  p { "Welcome to the Clarity Dashboard" }
                  div { class: "dashboard-content",
                      div { class: "dashboard-section",
                          h3 { "Quick Stats" }
                          p { "Overview of your Clarity workspace" }
                      }
                      div { class: "dashboard-section",
                          h3 { "Recent Activity" }
                          p { "Your latest work and updates" }
                      }
                      div { class: "dashboard-section",
                          h3 { "Quick Actions" }
                          div { class: "nav-links",
                              NavLink { to: Route::Home, "Go Home" }
                              NavLink { to: Route::About, "Learn More" }
                              NavLink { to: Route::BeadsList, "Manage Beads" }
                              NavLink { to: Route::BeadNew, "Create New Bead" }
                          }
                      }
                  }
              }
          }
      }
  }
}

/// Beads list page component wrapper
///
/// This wraps the `BeadListPage` component.
#[component]
fn BeadsList() -> Element {
  rsx! {
      div { class: "app-container",
          h1 { "Clarity" }
          div { class: "content",
              div { class: "page-header",
                  h2 { "Beads" }
                  div { class: "page-actions",
                      NavLink {
                          to: Route::BeadNew,
                          class: "btn btn-primary",
                          "Create New Bead"
                      }
                  }
              }
              super::BeadListPage {}
          }
      }
  }
}

/// Create new bead page component wrapper
///
/// This wraps the `BeadFormPage` component for creating new beads.
#[component]
fn BeadNew() -> Element {
  rsx! {
      div { class: "app-container",
          h1 { "Clarity" }
          div { class: "content",
              super::BeadFormPage { id: None }
          }
      }
  }
}

/// Edit bead page component wrapper
///
/// This wraps the `BeadFormPage` component for editing existing beads.
#[component]
fn BeadEdit(id: String) -> Element {
  // Validate the bead ID
  let bead_id_result = clarity_core::db::models::BeadId::from_str(&id);

  rsx! {
      div { class: "app-container",
          h1 { "Clarity" }
          div { class: "content",
              match bead_id_result {
                  Ok(_) => rsx! {
                      super::BeadFormPage { id: Some(id) }
                  },
                  Err(_) => rsx! {
                      div { class: "error-page",
                          h2 { "Error Loading Bead" }
                          p { "Invalid bead ID format: '{id}'" }
                          div { class: "nav-links",
                              NavLink { to: Route::BeadsList, "Back to Beads" }
                          }
                      }
                  }
              }
          }
      }
  }
}

/// Bead detail page component wrapper
///
/// This wraps the existing `BeadDetailPage` component.
/// The id parameter is automatically extracted from the route by `BeadDetailPage`.
#[component]
fn BeadDetail(id: String) -> Element {
  rsx! {
      div { class: "app-container",
          h1 { "Clarity" }
          div { class: "content",
              super::BeadDetailPage { id: id }
          }
      }
  }
}

/// Navigation link component for internal routing with active state
///
/// This component provides a styled link for navigation using the custom
/// signal-based routing system from `RouteProvider`. It adds active state
/// styling when the current route matches the link's destination.
#[component]
pub fn NavigationLink(
  to: Route,
  #[props(default)] class: String,
  #[props(default)] active_class: String,
  children: Element,
) -> Element {
  // Get the route signal from RouteProvider context
  let route = use_context::<Signal<Route>>();

  // Clone for the onclick handler
  let mut route_for_click = route;
  let target_route = to.clone();

  // Check if current route matches the destination
  let current: Route = route.read().clone();
  let is_active = current == to;

  let base_classes = if class.is_empty() {
    "nav-link".to_string()
  } else {
    format!("nav-link {class}")
  };

  let combined_class = if is_active && !active_class.is_empty() {
    format!("{base_classes} {active_class}")
  } else {
    base_classes
  };

  rsx! {
      button {
          class: "{combined_class}",
          onclick: move |_| {
              route_for_click.set(target_route.clone());
          },
          {children}
      }
  }
}

/// Navigation link component for internal routing
///
/// This component provides a styled link using dioxus-router's Link component
/// for client-side navigation without page reloads.
#[component]
pub fn NavLink(to: Route, #[props(default)] class: String, children: Element) -> Element {
  rsx! {
      NavigationLink {
          to: to,
          class: class,
          active_class: "active".to_string(),
          {children}
      }
  }
}

/// Settings page component
#[component]
fn Settings() -> Element {
  rsx! {
      div { class: "app-container",
          h1 { "Clarity" }
          div { class: "content",
              crate::components::SettingsView {}
          }
      }
  }
}

/// 404 Not Found page component
#[component]
fn NotFound(route: String) -> Element {
  rsx! {
      div { class: "app-container",
          h1 { "Clarity" }
          div { class: "content",
              div { class: "error-page",
                  h2 { "404 - Page Not Found" }
                  p { "The page '{route}' doesn't exist." }
                  div { class: "nav-links",
                      NavLink { to: Route::Home, "Go Home" }
                      NavLink { to: Route::Dashboard, "Dashboard" }
                      NavLink { to: Route::BeadsList, "Manage Beads" }
                  }
              }
          }
      }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_route_home_exists() {
    let route = Route::Home;
    assert_eq!(format!("{route:?}"), "Home");
  }

  #[test]
  fn test_route_about_exists() {
    let route = Route::About;
    assert_eq!(format!("{route:?}"), "About");
  }

  #[test]
  fn test_route_dashboard_exists() {
    let route = Route::Dashboard;
    assert_eq!(format!("{route:?}"), "Dashboard");
  }

  #[test]
  fn test_route_beads_list_exists() {
    let route = Route::BeadsList;
    assert_eq!(format!("{route:?}"), "BeadsList");
  }

  #[test]
  fn test_route_bead_detail_with_id() {
    let route = Route::BeadDetail {
      id: "test-id-123".to_string(),
    };
    match route {
      Route::BeadDetail { id } => {
        assert_eq!(id, "test-id-123");
      }
      _ => panic!("Expected BeadDetail route"),
    }
  }

  #[test]
  fn test_route_equality() {
    let route1 = Route::Home;
    let route2 = Route::Home;
    assert_eq!(route1, route2);

    let route3 = Route::BeadDetail {
      id: "abc".to_string(),
    };
    let route4 = Route::BeadDetail {
      id: "abc".to_string(),
    };
    assert_eq!(route3, route4);

    let route5 = Route::BeadDetail {
      id: "xyz".to_string(),
    };
    assert_ne!(route3, route5);
  }

  #[test]
  fn test_all_routes_are_distinct() {
    let routes = vec![
      Route::Home,
      Route::About,
      Route::Dashboard,
      Route::BeadsList,
      Route::BeadDetail {
        id: "test".to_string(),
      },
      Route::NotFound {
        route: "test".to_string(),
      },
    ];

    // Verify all routes are distinct (except when they should be equal)
    for (i, route_a) in routes.iter().enumerate() {
      for (j, route_b) in routes.iter().enumerate() {
        if i != j {
          // Different route variants should not be equal
          let same_variant = std::mem::discriminant(route_a) == std::mem::discriminant(route_b);
          if !same_variant {
            assert_ne!(route_a, route_b);
          }
        }
      }
    }
  }

  #[test]
  fn test_route_bead_detail_with_different_ids() {
    let id1 = "bead-001";
    let id2 = "bead-002";
    let id3 = "bead-001";

    let route1 = Route::BeadDetail {
      id: id1.to_string(),
    };
    let route2 = Route::BeadDetail {
      id: id2.to_string(),
    };
    let route3 = Route::BeadDetail {
      id: id3.to_string(),
    };

    assert_ne!(
      route1, route2,
      "Routes with different IDs should not be equal"
    );
    assert_eq!(route1, route3, "Routes with same IDs should be equal");
  }

  #[test]
  fn test_route_clone() {
    let original = Route::BeadDetail {
      id: "clone-test".to_string(),
    };
    let cloned = original.clone();

    assert_eq!(original, cloned);
  }
}
