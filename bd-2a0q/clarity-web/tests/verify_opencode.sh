#!/usr/bin/env bash
set -e

echo "=== OpenCode Provider Verification ==="
echo ""

echo "1. Checking opencode.rs module syntax..."
cargo check -p clarity-web 2>&1 | grep -i "providers/opencode" && echo "  ❌ Errors found" || echo "  ✅ No syntax errors"

echo ""
echo "2. Checking file structure..."
test -f "/home/lewis/src/clarity/clarity-web/src/providers/opencode.rs" && echo "  ✅ opencode.rs exists" || echo "  ❌ opencode.rs missing"
test -f "/home/lewis/src/clarity/clarity-web/src/providers/trait.rs" && echo "  ✅ trait.rs exists" || echo "  ❌ trait.rs missing"
test -f "/home/lewis/src/clarity/clarity-web/src/providers/mod.rs" && echo "  ✅ mod.rs exists" || echo "  ❌ mod.rs missing"

echo ""
echo "3. Checking exports in mod.rs..."
grep -q "pub use opencode::OpenCodeProvider" /home/lewis/src/clarity/clarity-web/src/providers/mod.rs && echo "  ✅ OpenCodeProvider exported" || echo "  ❌ OpenCodeProvider not exported"

echo ""
echo "4. Checking dependencies..."
grep -q "reqwest" /home/lewis/src/clarity/Cargo.toml && echo "  ✅ reqwest in workspace Cargo.toml" || echo "  ❌ reqwest missing from workspace"
grep -q "reqwest" /home/lewis/src/clarity/clarity-web/Cargo.toml && echo "  ✅ reqwest in clarity-web Cargo.toml" || echo "  ❌ reqwest missing from clarity-web"

echo ""
echo "5. Checking implementation..."
grep -q "struct OpenCodeProvider" /home/lewis/src/clarity/clarity-web/src/providers/opencode.rs && echo "  ✅ OpenCodeProvider struct defined" || echo "  ❌ OpenCodeProvider struct missing"
grep -q "impl ExtractionProvider for OpenCodeProvider" /home/lewis/src/clarity/clarity-web/src/providers/opencode.rs && echo "  ✅ ExtractionProvider trait implemented" || echo "  ❌ ExtractionProvider trait not implemented"
grep -q "fn extract_fields" /home/lewis/src/clarity/clarity-web/src/providers/opencode.rs && echo "  ✅ extract_fields method defined" || echo "  ❌ extract_fields method missing"
grep -q "fn extract_fields_with_schema" /home/lewis/src/clarity/clarity-web/src/providers/opencode.rs && echo "  ✅ extract_fields_with_schema method defined" || echo "  ❌ extract_fields_with_schema method missing"
grep -q "fn health_check" /home/lewis/src/clarity/clarity-web/src/providers/opencode.rs && echo "  ✅ health_check method defined" || echo "  ❌ health_check method missing"
grep -q "SESSION_HEADER" /home/lewis/src/clarity/clarity-web/src/providers/opencode.rs && echo "  ✅ Session ID header handling" || echo "  ❌ Session ID header handling missing"
grep -q "DEFAULT_TIMEOUT_SECS: u64 = 30" /home/lewis/src/clarity/clarity-web/src/providers/opencode.rs && echo "  ✅ 30s timeout configured" || echo "  ❌ 30s timeout not configured"
grep -q "fn map_http_error" /home/lewis/src/clarity/clarity-web/src/providers/opencode.rs && echo "  ✅ HTTP error mapping implemented" || echo "  ❌ HTTP error mapping missing"
grep -q "fn map_status_error" /home/lewis/src/clarity/clarity-web/src/providers/opencode.rs && echo "  ✅ Status code error mapping implemented" || echo "  ❌ Status code error mapping missing"

echo ""
echo "6. Checking test coverage..."
grep -q "#\[cfg(test)\]" /home/lewis/src/clarity/clarity-web/src/providers/opencode.rs && echo "  ✅ Unit tests present" || echo "  ❌ Unit tests missing"
grep -q "test_new_provider" /home/lewis/src/clarity/clarity-web/src/providers/opencode.rs && echo "  ✅ Provider creation test" || echo "  ❌ Provider creation test missing"
grep -q "test_build_url" /home/lewis/src/clarity/clarity-web/src/providers/opencode.rs && echo "  ✅ URL building test" || echo "  ❌ URL building test missing"
grep -q "test_map_status_error" /home/lewis/src/clarity/clarity-web/src/providers/opencode.rs && echo "  ✅ Error mapping test" || echo "  ❌ Error mapping test missing"
grep -q "test_parse_response" /home/lewis/src/clarity/cl clarity-web/src/providers/opencode.rs && echo "  ✅ Response parsing test" || echo "  ❌ Response parsing test missing"

echo ""
echo "=== Verification Complete ==="
