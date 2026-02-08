Work through ALL beads in the repository systematically using the 15-phase TDD workflow (tdd15). The beads are stored in .beads/beads.db. Process them in priority order (lowest first), starting with foundation items then core then web features.

For each bead:
1. Read the bead details from the database using SQLite
2. Initialize tdd15 session for this bead with language "rust"
3. Execute the full 15-phase TDD workflow (phases 0-15)
4. Write PURELY FUNCTIONAL RUST code following Haskell-like principles:
   - COMPLETE IMMUTABILITY: No mutable variables, no mutation, no reference mutation
   - NO mutation: All data is immutable by default
   - NO reference mutation: Even &mut references should be avoided
   - Result types: Use Result<T, E> for all fallible operations
   - Zero unwraps/panics: Never use unwrap(), expect(), or panic!
   - Explicit error handling: Use ? operator for error propagation
   - Pure functions: No side effects, same input always produces same output
   - Function composition: Build complex operations from simple functions
   - Pattern matching: Use exhaustive pattern matching
   - Type safety: Leverage Rust's type system to prevent errors
   - No nulls: Use Option<T> instead of Option::None
   - Explicit error types: Define custom error types instead of string errors
   - Lazy evaluation: Use iterators and lazy collections where appropriate
   - Immutable collections: Use Vec, HashMap, etc. as immutable values
   - No references to mutable state: Everything should be pure and composable

5. Run `cargo test` and `cargo fmt && cargo clippy -- -D warnings` to verify code quality
6. Mark the bead as complete in the database (update status to 'closed')
7. Move to the next bead
8. Continue until ALL 42 beads are completed

The beads are organized as:
- Foundation beads (bd-2ck, bd-2b3, bd-3s0, bd-1ib, bd-26p, bd-3vg, bd-2if, bd-1ft, bd-di8)
- Core beads (bd-1my, bd-ezl, bd-3ue, bd-3ki, bd-2lb, bd-4h9, bd-2fg, bd-3ey, bd-264, bd-2yt, bd-dws, bd-1v2, bd-57v, bd-2nx)
- Web beads (bd-21h, bd-1bc, bd-z11, bd-2cg, bd-4pq, bd-3rr, bd-84g, bd-w5a, bd-1r8, bd-3n6, bd-3j3, bd-ccj, bd-3tq, bd-34e, bd-2j4, bd-4a6, bd-2pj, bd-10g)

Use completion promise "ALL BEADS COMPLETED" when done.

CRITICAL: Write ONLY purely functional Rust code. No mutation, no mutable references, no reference mutation. Everything must be immutable. Use Result types and explicit error handling. Follow Haskell-like functional programming principles.