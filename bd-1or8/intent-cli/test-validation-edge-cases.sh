#!/bin/bash

echo "=== Testing Intent CLI Spec Validation Edge Cases ==="
echo

# Test case 1: Empty spec file
echo "1. Testing empty spec file:"
echo "Contents of test/test-empty-spec.cue:"
cat test/test-empty-spec.cue
echo
if cue vet test/test-empty-spec.cue; then
    echo "✓ Empty spec passed CUE validation"
else
    echo "✗ Empty spec failed CUE validation as expected"
fi
echo

# Test case 2: Minimal required fields
echo "2. Testing minimal required fields:"
echo "Contents of test/test-minimal-required.cue:"
head -20 test/test-minimal-required.cue
echo
if cue vet test/test-minimal-required.cue; then
    echo "✓ Minimal spec passed CUE validation"
else
    echo "✗ Minimal spec failed CUE validation"
fi
echo

# Test case 3: Malformed CUE syntax
echo "3. Testing malformed CUE syntax:"
echo "Contents of test/test-malformed-cue.cue:"
cat test/test-malformed-cue.cue
echo
if cue vet test/test-malformed-cue.cue; then
    echo "✗ Malformed CUE unexpectedly passed validation"
else
    echo "✓ Malformed CUE correctly failed validation"
fi
echo

# Test case 4: Invalid JSON in examples
echo "4. Testing invalid JSON in examples:"
echo "Contents of test/test-invalid-json.cue:"
cat test/test-invalid-json.cue
echo
if cue vet test/test-invalid-json.cue; then
    echo "✓ Invalid JSON spec passed CUE validation (CUE doesn't validate JSON content)"
else
    echo "✗ Invalid JSON spec failed CUE validation"
fi
echo

# Test case 5: Circular dependencies
echo "5. Testing circular dependencies:"
echo "Contents of test/test-circular-dependencies.cue:"
cat test/test-circular-dependencies.cue
echo
if cue vet test/test-circular-dependencies.cue; then
    echo "✓ Circular dependencies spec passed CUE validation"
else
    echo "✗ Circular dependencies spec failed CUE validation"
fi
echo

# Test case 6: Duplicate behavior names
echo "6. Testing duplicate behavior names:"
echo "Contents of test/test-duplicate-behavior-names.cue:"
cat test/test-duplicate-behavior-names.cue
echo
if cue vet test/test-duplicate-behavior-names.cue; then
    echo "✓ Duplicate behavior names spec passed CUE validation"
else
    echo "✗ Duplicate behavior names spec failed CUE validation"
fi
echo

# Test case 7: Invalid regex patterns
echo "7. Testing invalid regex patterns:"
echo "Contents of test/test-invalid-regex.cue:"
cat test/test-invalid-regex.cue
echo
if cue vet test/test-invalid-regex.cue; then
    echo "✓ Invalid regex spec passed CUE validation"
else
    echo "✗ Invalid regex spec failed CUE validation"
fi
echo

# Test case 8: Missing required fields
echo "8. Testing missing required fields:"
echo "Contents of test/test-missing-required-fields.cue:"
cat test/test-missing-required-fields.cue
echo
if cue vet test/test-missing-required-fields.cue; then
    echo "✗ Missing required fields spec unexpectedly passed validation"
else
    echo "✓ Missing required fields spec correctly failed validation"
fi
echo

# Test case 9: Valid spec (control test)
echo "9. Testing valid spec (control):"
echo "Contents of examples/user-api.cue (first 20 lines):"
head -20 examples/user-api.cue
echo
if cue vet examples/user-api.cue; then
    echo "✓ Valid spec passed CUE validation"
else
    echo "✗ Valid spec failed CUE validation unexpectedly"
fi
echo

echo "=== CUE Validation Test Complete ==="