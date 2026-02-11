//! Example: Using br show functionality
//!
//! This example demonstrates how to use the br show integration
//! to display issue details from the br command line tool.

use clarity_client::br_show;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  println!("=== Br Show Example ===");

  // Example 1: Fetch a specific issue
  println!("Fetching bd-1bf...");
  match br_show::fetch_br_issue("bd-1bf").await {
    Ok(issue) => {
      println!("✓ Found issue:");
      println!("  ID: {}", issue.id);
      println!("  Title: {}", issue.title);
      println!("  Status: {}", issue.status);
      println!("  Priority: {}", issue.priority);
      println!("  Type: {}", issue.issue_type);
      println!("  Created by: {}", issue.created_by);
      println!(
        "  Created at: {}",
        issue.created_at.format("%Y-%m-%d %H:%M:%S UTC")
      );
    }
    Err(e) => {
      println!("✗ Error fetching issue: {}", e);
    }
  }

  // Example 2: Check if an issue exists
  println!("\nChecking if bd-1bf exists...");
  match br_show::issue_exists("bd-1bf").await {
    Ok(exists) => {
      if exists {
        println!("✓ bd-1bf exists");
      } else {
        println!("✗ bd-1bf does not exist");
      }
    }
    Err(e) => {
      println!("✗ Error checking existence: {}", e);
    }
  }

  // Example 3: Get all issue IDs
  println!("\nGetting all issue IDs...");
  match br_show::get_issue_ids().await {
    Ok(ids) => {
      println!("✓ Found {} issues:", ids.len());
      for id in &ids {
        println!("  - {}", id);
      }
    }
    Err(e) => {
      println!("✗ Error getting issue IDs: {}", e);
    }
  }

  Ok(())
}
