//! Build script for clarity-web
//! Handles Tailwind CSS asset copying for Dioxus dev server
//!
//! Dioxus 0.7 creates hashed CSS filenames (e.g., tailwind-dxh83f81826d1dae3ee.css)
//! but the HTML references non-hashed paths. This script copies the hashed file
//! to the expected non-hashed filename as a workaround.

use std::fs;
use std::path::Path;

fn main() {
  // Re-run if CSS files change
  println!("cargo:rerun-if-changed=assets/tailwind.css");
  println!("cargo:rerun-if-changed=tailwind.css");

  let dest_dirs = [
    "target/dx/clarity-web/debug/web/public/assets",
    "target/dx/clarity-web/release/web/public/assets",
  ];

  for dest_dir in &dest_dirs {
    let dest_path = Path::new(dest_dir);
    if !dest_path.exists() {
      continue;
    }

    // Find hashed CSS file (tailwind-dx*.css)
    if let Ok(entries) = fs::read_dir(dest_path) {
      for entry in entries.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str.starts_with("tailwind-dx") && name_str.ends_with(".css") {
          let hashed_file = entry.path();
          let target_file = dest_path.join("tailwind.css");
          if let Err(e) = fs::copy(&hashed_file, &target_file) {
            println!("cargo:warning=Failed to copy {name_str} to tailwind.css: {e}");
          } else {
            println!("cargo:warning=Copied {name_str} to tailwind.css");
          }
          break;
        }
      }
    }
  }
}
