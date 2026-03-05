#!/bin/bash
# Test that CUE schema enforces required fields
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
cd "$PROJECT_ROOT"

echo "Testing CUE required field validation..."
echo

# Test 1: Missing required 'name' field should fail
echo "Test 1: Spec missing 'name' field"
if cue vet schema/intent.cue test/test-missing-name.cue 2>/dev/null; then
    echo "FAIL: Should have rejected spec missing 'name' field"
    exit 1
else
    echo "PASS: Correctly rejected spec missing 'name' field"
fi
echo

# Test 2: Missing required 'description' field should fail
echo "Test 2: Spec missing 'description' field"
if cue vet schema/intent.cue test/test-missing-description.cue 2>/dev/null; then
    echo "FAIL: Should have rejected spec missing 'description' field"
    exit 1
else
    echo "PASS: Correctly rejected spec missing 'description' field"
fi
echo

# Test 3: Valid spec should pass
echo "Test 3: Valid spec with all required fields"
if cue vet schema/intent.cue test/test-valid-spec.cue 2>/dev/null; then
    echo "PASS: Correctly accepted valid spec"
else
    echo "FAIL: Should have accepted valid spec"
    exit 1
fi
echo

echo "All CUE required field tests passed!"
