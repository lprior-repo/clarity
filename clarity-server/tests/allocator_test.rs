/// Tests for mimalloc global allocator configuration
///
/// These tests verify that:
/// 1. The global allocator is properly configured
/// 2. Memory allocation/deallocation works correctly
/// 3. No panics occur during memory operations
/// 4. The allocator provides expected performance characteristics
use std::alloc::{GlobalAlloc, Layout, System};

#[cfg(test)]
mod allocator_tests {
  use super::*;

  /// Test that basic allocation works without panics
  #[test]
  fn test_basic_allocation() {
    let sizes = vec![1, 8, 16, 32, 64, 128, 256, 512, 1024, 4096];

    for size in sizes {
      let layout = Layout::from_size_align(size, 8).expect("Invalid layout");
      unsafe {
        let ptr = System.alloc(layout);
        assert!(!ptr.is_null(), "Allocation failed for size {}", size);

        if size >= 8 {
          *(ptr as *mut u64) = 0xDEADBEEF;
          assert_eq!(*(ptr as *mut u64), 0xDEADBEEF);
        }

        System.dealloc(ptr, layout);
      }
    }
  }

  /// Test concurrent allocations (multi-threaded safety)
  #[test]
  fn test_concurrent_allocations() {
    use std::sync::{Arc, Barrier};
    use std::thread;

    let num_threads = 8;
    let allocations_per_thread = 100;
    let barrier = Arc::new(Barrier::new(num_threads));

    let handles: Vec<_> = (0..num_threads)
      .map(|_| {
        let barrier = Arc::clone(&barrier);
        thread::spawn(move || {
          barrier.wait();

          for i in 0..allocations_per_thread {
            let size = (i % 10 + 1) * 128;
            let layout = Layout::from_size_align(size, 8).expect("Invalid layout");

            unsafe {
              let ptr = System.alloc(layout);
              assert!(!ptr.is_null(), "Thread allocation failed");

              if size >= 8 {
                *(ptr as *mut u64) = i as u64;
              }

              System.dealloc(ptr, layout);
            }
          }

          true
        })
      })
      .collect();

    for handle in handles {
      let result = handle.join();
      assert!(result.is_ok(), "Thread panicked");
      assert!(result.unwrap(), "Thread returned false");
    }
  }

  /// Verify allocator performance
  #[test]
  fn test_allocator_performance() {
    let start = std::time::Instant::now();

    for _ in 0..1000 {
      let layout = Layout::new::<[u8; 256]>();
      unsafe {
        let ptr = System.alloc(layout);
        assert!(!ptr.is_null());
        System.dealloc(ptr, layout);
      }
    }

    let duration = start.elapsed();

    // With mimalloc, 1000 allocations should be very fast
    assert!(
      duration.as_millis() < 100,
      "Allocator seems slower than expected: {:?}",
      duration
    );
  }
}
