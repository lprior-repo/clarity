//! Test route parsing for br show integration

use clarity_client::app::Route;
use std::str::FromStr;

fn main() {
  println!("Testing route parsing...");

  // Test BrShow route
  let route_str = "/br/bd-1bf";
  let route = Route::from_str(route_str);

  match route {
    Ok(Route::BrShow { id }) => {
      println!("✓ Successfully parsed route: {route_str}");
      println!("  ID: {}", id);
      assert_eq!(id, "bd-1bf");
    }
    Ok(other) => {
      println!("✗ Wrong route type for {}: {:?}", route_str, other);
    }
    Err(e) => {
      println!("✗ Failed to parse route {}: {}", route_str, e);
    }
  }

  // Test other routes
  let test_routes = vec![
    ("/", Route::Home),
    ("/dashboard", Route::Dashboard),
    ("/beads", Route::BeadsList),
    (
      "/beads/bd-1bf",
      Route::BeadDetail {
        id: "bd-1bf".to_string(),
      },
    ),
    (
      "/br/bd-1bf",
      Route::BrShow {
        id: "bd-1bf".to_string(),
      },
    ),
  ];

  for (path, expected) in test_routes {
    let result = Route::from_str(path);
    match result {
      Ok(actual) => {
        if actual == expected {
          println!("✓ {}: {:?}", path, actual);
        } else {
          println!("✗ {} - Expected {:?}, got {:?}", path, expected, actual);
        }
      }
      Err(e) => {
        println!("✗ Failed to parse {}: {}", path, e);
      }
    }
  }

  println!("Route parsing tests completed!");
}
