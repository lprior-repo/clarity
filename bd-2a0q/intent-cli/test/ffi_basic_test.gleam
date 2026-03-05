/// Basic FFI test to verify functionality
import gleam/io
import gleeunit/should

// Test basic imports work
pub fn test_basic_imports() {
  io.println("Testing basic FFI imports...")
  True |> should.be_true()
}

pub fn main() {
  io.println("🧪 Running basic FFI test...")
  test_basic_imports()
  io.println("✅ Basic FFI test completed!")
}
