// Simple test for br route parsing
fn main() {
    // Test that we can create the Route enum
    println!("Testing BrShow route creation...");

    let route = clarity_client::app::Route::BrShow { id: "bd-1bf".to_string() };

    match route {
        clarity_client::app::Route::BrShow { id } => {
            println!("✓ Successfully created BrShow route with id: {}", id);
            assert_eq!(id, "bd-1bf");
        }
        _ => {
            println!("✗ Failed to create BrShow route");
            return;
        }
    }

    // Test display
    let display = format!("{}", route);
    println!("✓ Route display: {}", display);
    assert_eq!(display, "/br/bd-1bf");

    println!("All tests passed!");
}