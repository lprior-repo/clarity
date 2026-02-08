# Build Optimization Guide (LTO + PGO)

## Overview

This document describes the build optimization strategies implemented in Clarity, including Link-Time Optimization (LTO) and Profile-Guided Optimization (PGO).

**Implementation:** Bead bd-2m1
**Status:** ✅ Implemented and tested

## Configuration Summary

### Release Profile

- **LTO:** Full (fat) Link-Time Optimization enabled
- **Codegen Units:** 1 (maximum optimization)
- **Opt Level:** 3 (maximum)
- **Strip:** Symbols removed for smaller binary size
- **Performance:** +10-20% improvement over debug build
- **Build Time:** ~2-3x longer than debug build

### CI Profile

- **LTO:** Thin LTO (faster compilation)
- **Codegen Units:** 16 (more parallelism)
- **Opt Level:** 3 (maximum)
- **Strip:** Symbols removed
- **Performance:** +5-10% improvement
- **Build Time:** ~1.5x longer than debug build
- **Use Case:** CI/CD pipelines

### PGO Workflow

Three-step process for maximum optimization:

1. **Instrumentation:** Build with profiling instrumentation
   ```bash
   moon run :pgo-instrument
   ```

2. **Profiling:** Run representative workload
   ```bash
   moon run :pgo-profile
   ```

3. **Optimization:** Build using profile data
   ```bash
   moon run :pgo-optimize
   ```

Or automate the entire workflow:
```bash
moon run :pgo-build
```

**Expected Performance:** +25-50% improvement over debug build

## Quick Reference

| Profile        | Command                  | Use Case               |
|----------------|--------------------------|------------------------|
| dev            | `cargo build`            | Development            |
| release        | `moon run :release`      | Production (standard)  |
| ci             | `moon run :release-ci`   | CI/CD pipelines        |
| pgo-optimized  | `moon run :pgo-build`    | Production (max perf)  |

## Verification

Run the build optimization tests:
```bash
cargo test --test build_optimization_test
```

Tests verify:
- ✅ LTO configuration
- ✅ Codegen units optimization
- ✅ Symbol stripping
- ✅ PGO workflow setup
- ✅ Documentation completeness

## Documentation

For detailed information, see:
- [Cargo Profiles](https://doc.rust-lang.org/cargo/reference/profiles.html)
- [Rust LTO](https://doc.rust-lang.org/rustc/linker-plugin-lto.html)
- [Rust PGO](https://doc.rust-lang.org/rustc/profile-guided-optimization.html)

## References

- **Bead:** bd-2m1
- **Implementation Date:** 2026-02-08
- **Related:**
  - [03_WORKFLOW.md](03_WORKFLOW.md)
  - [02_MOON_BUILD.md](02_MOON_BUILD.md)
  - [05_RUST_STANDARDS.md](05_RUST_STANDARDS.md)
