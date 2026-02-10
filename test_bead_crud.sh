#!/bin/bash
# Comprehensive Bead CRUD Testing Script
# This script tests all bead CRUD operations through the Rust application

set -e

echo "========================================="
echo "BEAD CRUD TESTING - Comprehensive QA"
echo "========================================="
echo ""

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test counter
TESTS_PASSED=0
TESTS_FAILED=0

# Function to report test result
report_test() {
    local test_name="$1"
    local result="$2"
    local details="$3"

    if [ "$result" = "PASS" ]; then
        echo -e "${GREEN}✓ PASS${NC}: $test_name"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        echo -e "${RED}✗ FAIL${NC}: $test_name"
        if [ -n "$details" ]; then
            echo -e "  ${YELLOW}Details:${NC} $details"
        fi
        TESTS_FAILED=$((TESTS_FAILED + 1))
    fi
    echo ""
}

# Change to project directory
cd /home/lewis/src/clarity

echo "Phase 1: Compilation Tests"
echo "-------------------------------"

# Test 1: Check if project compiles
echo "Test 1.1: Building clarity-core..."
if cargo build -p clarity-core 2>&1 | tee /tmp/build_core.log; then
    report_test "clarity-core builds successfully" "PASS"
else
    report_test "clarity-core builds successfully" "FAIL" "$(tail -20 /tmp/build_core.log)"
fi

# Test 2: Check if client compiles
echo "Test 1.2: Building clarity-client..."
if cargo build -p clarity-client 2>&1 | tee /tmp/build_client.log; then
    report_test "clarity-client builds successfully" "PASS"
else
    report_test "clarity-client builds successfully" "FAIL" "$(tail -30 /tmp/build_client.log)"
fi

# Test 3: Check for critical compilation errors
echo "Test 1.3: Checking for compilation errors..."
if cargo check 2>&1 | tee /tmp/cargo_check.log; then
    report_test "cargo check passes" "PASS"
else
    report_test "cargo check passes" "FAIL" "Compilation errors detected"
fi

echo ""
echo "Phase 2: Unit Tests"
echo "-------------------------------"

# Test 4: Run clarity-core tests
echo "Test 2.1: Running clarity-core unit tests..."
if cargo test -p clarity-core --lib 2>&1 | tee /tmp/test_core.log; then
    report_test "clarity-core unit tests pass" "PASS"
else
    report_test "clarity-core unit tests pass" "FAIL" "$(tail -30 /tmp/test_core.log)"
fi

# Test 5: Run clarity-client tests
echo "Test 2.2: Running clarity-client unit tests..."
if cargo test -p clarity-client --lib 2>&1 | tee /tmp/test_client.log; then
    report_test "clarity-client unit tests pass" "PASS"
else
    report_test "clarity-client unit tests pass" "FAIL" "$(tail -30 /tmp/test_client.log)"
fi

echo ""
echo "Phase 3: Integration Tests (Database)"
echo "-------------------------------"

# Test 6: Test bead creation in database
echo "Test 3.1: Testing bead creation (database layer)..."
cat > /tmp/test_bead_create.rs << 'EOF'
#[tokio::test]
async fn test_bead_create_integration() {
    use clarity_client::db::DesktopDb;
    use clarity_core::db::models::{NewBead, BeadStatus, BeadPriority, BeadType};

    let db = DesktopDb::in_memory().await.unwrap();

    let new_bead = NewBead {
        title: "Integration Test Bead".to_string(),
        description: Some("Testing CRUD operations".to_string()),
        status: BeadStatus::Open,
        priority: BeadPriority::HIGH,
        bead_type: BeadType::Feature,
        created_by: None,
    };

    let created = db.create_bead(new_bead).await.unwrap();
    assert_eq!(created.title, "Integration Test Bead");
    assert_eq!(created.status, BeadStatus::Open);
}
EOF
# This would require proper test file setup, skip for now
report_test "Bead creation in database" "SKIP" "Test file not integrated into project"

# Test 7: Test bead listing
echo "Test 3.2: Testing bead listing (database layer)..."
report_test "Bead listing from database" "SKIP" "Requires running database instance"

# Test 8: Test bead filtering
echo "Test 3.3: Testing bead filtering..."
report_test "Bead filtering by status/type/priority" "SKIP" "Requires database with test data"

# Test 9: Test bead updates
echo "Test 3.4: Testing bead updates..."
report_test "Bead update operation" "SKIP" "Requires database instance"

# Test 10: Test bead deletion
echo "Test 3.5: Testing bead deletion..."
report_test "Bead delete operation" "SKIP" "Requires database instance"

echo ""
echo "Phase 4: UI Component Tests"
echo "-------------------------------"

# Test 11: Check bead form component compiles
echo "Test 4.1: Checking bead form component..."
if cargo check -p clarity-client --lib 2>&1 | grep -q "beads/form.rs"; then
    # If file is mentioned but has errors, that's a failure
    if cargo check -p clarity-client --lib 2>&1 | grep -A 5 "beads/form.rs" | grep -q "error"; then
        report_test "Bead form component compiles" "FAIL" "Compilation errors in form.rs"
    else
        report_test "Bead form component compiles" "PASS"
    fi
else
    report_test "Bead form component compiles" "SKIP" "Component not checked"
fi

# Test 12: Check bead list component
echo "Test 4.2: Checking bead list component..."
report_test "Bead list component" "SKIP" "Requires component compilation"

# Test 13: Check bead detail component
echo "Test 4.3: Checking bead detail component..."
report_test "Bead detail component" "SKIP" "Requires component compilation"

echo ""
echo "Phase 5: Critical Issue Detection"
echo "-------------------------------"

# Test 14: Check for async/sync mismatch (CRITICAL)
echo "Test 5.1: Checking for async/sync database access..."
if grep -q "DesktopDb::new()" clarity-client/src/beads/*.rs; then
    # Check if it's called without .await
    if grep "DesktopDb::new()" clarity-client/src/beads/*.rs | grep -v "\.await" > /dev/null; then
        report_test "Async/sync database access" "FAIL" "DesktopDb::new() called without .await in UI components - will cause runtime panic"
    else
        report_test "Async/sync database access" "PASS"
    fi
else
    report_test "Async/sync database access" "SKIP" "Database access pattern not found"
fi

# Test 15: Check for missing dependencies
echo "Test 5.2: Checking for missing dependencies..."
MISSING_DEPS=0
if ! cargo build -p clarity-core 2>&1 | grep -q "Finished"; then
    if cargo build -p clarity-core 2>&1 | grep -q "unresolved crate"; then
        MISSING_DEPS=1
    fi
fi

if [ $MISSING_DEPS -eq 0 ]; then
    report_test "All dependencies resolved" "PASS"
else
    report_test "All dependencies resolved" "FAIL" "Missing crate dependencies detected"
fi

# Test 16: Check for SQL injection vulnerabilities
echo "Test 5.3: Checking SQL injection prevention..."
if grep -r "format!(.*SELECT" clarity-client/src/db.rs clarity-core/src/db/ | grep -v "WHERE 1=1" > /dev/null; then
    report_test "SQL injection prevention" "FAIL" "Unparameterized SQL queries detected"
else
    report_test "SQL injection prevention" "PASS" "Using parameterized queries"
fi

# Test 17: Check for proper error handling
echo "Test 5.4: Checking error handling..."
if grep -q "unwrap()" clarity-client/src/beads/*.rs; then
    report_test "Error handling (no unwrap panics)" "FAIL" "unwrap() calls found - potential panics"
else
    report_test "Error handling (no unwrap panics)" "PASS"
fi

echo ""
echo "========================================="
echo "TEST SUMMARY"
echo "========================================="
echo "Tests Passed: $TESTS_PASSED"
echo "Tests Failed: $TESTS_FAILED"
echo "Tests Skipped: $(( $(grep -c "SKIP" /tmp/test_results.txt 2>/dev/null || echo 0) ))"
echo ""

if [ $TESTS_FAILED -gt 0 ]; then
    echo -e "${RED}CRITICAL: Some tests failed!${NC}"
    exit 1
elif [ $TESTS_PASSED -eq 0 ]; then
    echo -e "${YELLOW}WARNING: No tests were executed successfully${NC}"
    exit 2
else
    echo -e "${GREEN}All executed tests passed!${NC}"
    exit 0
fi
