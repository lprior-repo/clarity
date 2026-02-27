#!/bin/bash
# Comprehensive test for all required CUE fields
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
cd "$PROJECT_ROOT"

echo "Comprehensive CUE Required Field Validation Tests"
echo "=================================================="
echo

# Test spec-level required fields (v3.0 schema - config removed)
required_fields=("name" "description" "audience" "version" "success_criteria" "features" "invariants" "anti_patterns" "ai_hints")

echo "Testing spec-level required fields..."
for field in "${required_fields[@]}"; do
    echo -n "  Testing missing '$field' field... "
    if cue vet -c schema/intent.cue "test/test-missing-$field.cue" 2>&1 | grep -q "field is required but not present"; then
        echo "PASS"
    else
        echo "FAIL"
        exit 1
    fi
done
echo

echo "Testing valid spec acceptance..."
if cue vet schema/intent.cue test/test-valid-spec.cue 2>/dev/null; then
    echo "  PASS: Valid spec accepted"
else
    echo "  FAIL: Valid spec rejected"
    exit 1
fi
echo

echo "All comprehensive tests passed!"
