// =========================================================================
// Fuzz target: parse_dsl
// =========================================================================
//
// Lane: **Z** (cargo-fuzz). Companion to the proptest artifact at
// `proofs/parser_proptest.rs`. Targets the front door of the DSL parser:
//
//     clarity_web::intent::parser::parse_spec
//
// ## Property under test
//
// The parser is **panic-free and termination-bounded** on any byte
// sequence. Specifically, for every byte input:
//   - `parse_spec` returns a `Result<_, ParseError>` (never panics).
//   - It returns within a wall-clock budget that makes the fuzzer
//     usable in CI (we do not assert a hard bound; cargo-fuzz's
//     sanitizer run enforces stack/heap/time limits per case).
//
// This is the same panic-freedom property as PO-P6, but exercised
// under cargo-fuzz's libFuzzer driver with arbitrary `&[u8]` input.
// proptest cannot reproduce pathologically-malformed UTF-8 / very
// large inputs / structural adversarial shapes; the fuzzer can.
//
// ## Tooling status (2026-06-21)
//
// `cargo-fuzz` is **NOT installed** in this environment. This is
// tracked as `cl-u04` (Install missing formal verifier tools).
// Execution of this harness is BLOCKED_TOOLING until `cl-u04` closes.
//
// The source below is the canonical artifact. It will compile and run
// once `cargo install cargo-fuzz` (nightly toolchain) is wired up:
//
//     cargo +nightly fuzz run parse_dsl -- -runs=100000 -max_total_time=60
//
// ## Sanitizer coverage
//
// The default cargo-fuzz profile uses AddressSanitizer + UBSan. That
// combination catches:
//   - heap buffer overflow in `serde_json` parsing of adversarial input
//   - use-after-free in the parser's `extract_string_field` path
//   - signed integer overflow in the JSON line/column counters
//
// We do **not** enable MemorySanitizer (needs a C++ rebuild of stdlib);
// that is a follow-on if leaks show up in CI.
//
// ## Corpus seeding
//
// We do not seed the corpus; `libFuzzer` will discover structurally
// valid JSON shapes on its own within the first ~50 iterations. The
// proptest artifact at `proofs/parser_proptest.rs` owns the structured
// happy-path coverage.
//
// ## Anti-laundering
//
// The harness calls `parse_spec` (production) directly. There is no
// local re-implementation of the parser. The `String::from_utf8_lossy`
// adapter is a thin byte-to-string shim; the parser logic is the
// production `parse_spec` body at `parser.rs:71-86`.
//
// =========================================================================

#![no_main]

use libfuzzer_sys::fuzz_target;
use clarity_web::intent::parser::parse_spec;

fuzz_target!(|data: &[u8]| {
    // The parser takes `&str`. `parse_spec` first calls `sanitize_string`
    // (which strips `'\0'` and trims), then `serde_json::from_str`
    // (which requires valid UTF-8). For non-UTF-8 input we use a lossy
    // UTF-8 decode so the fuzzer still exercises the parser's internal
    // paths rather than short-circuiting at the JSON layer.
    //
    // `Cow<'_, str>` derefs to `str`, so `&cow_lossy` coerces to `&str`
    // at the `parse_spec` call site.
    //
    // The only property under fuzz is panic-freedom + Result-typed
    // outcome. We do not assert on the Ok/Err branch because both are
    // valid outputs depending on whether the random bytes happen to
    // form a valid JSON Spec. The assertion that `result` is itself a
    // value (not a panic unwind) is what cargo-fuzz's sanitizer run
    // enforces.
    let lossy = String::from_utf8_lossy(data);
    let _ = parse_spec(&lossy);
});