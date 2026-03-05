#!/bin/bash

echo "=================================="
echo "CUE VALIDATION TEST REPORT"
echo "=================================="
echo ""

PASS=0
FAIL=0
WARN=0

for f in qa_validation_tests/*.cue; do
  filename=$(basename "$f")
  echo "Test: $filename"
  
  # Run CUE vet and capture output
  output=$(cue vet -c "$f" 2>&1)
  exit_code=$?
  
  if [ $exit_code -eq 0 ]; then
    echo "  ✓ PASSED - No validation errors"
    PASS=$((PASS + 1))
  else
    # Check if it's just incomplete values
    if echo "$output" | grep -q "incomplete value"; then
      echo "  ⚠ WARNING - Incomplete values detected"
      echo "$output" | head -5 | sed 's/^/    /'
      WARN=$((WARN + 1))
    else
      echo "  ✗ FAILED - Validation errors:"
      echo "$output" | head -10 | sed 's/^/    /'
      FAIL=$((FAIL + 1))
    fi
  fi
  echo ""
done

echo "=================================="
echo "SUMMARY"
echo "=================================="
echo "Passed: $PASS"
echo "Failed: $FAIL"
echo "Warnings: $WARN"
echo "Total: $((PASS + FAIL + WARN))"
echo ""

if [ $FAIL -gt 0 ]; then
  exit 1
fi
