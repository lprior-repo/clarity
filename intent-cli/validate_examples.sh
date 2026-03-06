#!/bin/bash

echo "=================================="
echo "EXAMPLE FILES VALIDATION REPORT"
echo "=================================="
echo ""

PASS=0
FAIL=0
WARN=0

for f in examples/*.cue; do
  if [ -f "$f" ]; then
    filename=$(basename "$f")
    echo "Example: $filename"
    
    output=$(cue vet -c "$f" 2>&1)
    exit_code=$?
    
    if [ $exit_code -eq 0 ]; then
      echo "  ✓ PASSED"
      PASS=$((PASS + 1))
    else
      if echo "$output" | grep -q "incomplete value"; then
        echo "  ⚠ WARNING - Incomplete values"
        echo "$output" | grep "incomplete" | head -3 | sed 's/^/    /'
        WARN=$((WARN + 1))
      else
        echo "  ✗ FAILED"
        echo "$output" | head -8 | sed 's/^/    /'
        FAIL=$((FAIL + 1))
      fi
    fi
    echo ""
  fi
done

echo "=================================="
echo "EXAMPLES SUMMARY"
echo "=================================="
echo "Passed: $PASS"
echo "Failed: $FAIL"
echo "Warnings: $WARN"
echo ""

if [ $FAIL -gt 0 ] || [ $WARN -gt 0 ]; then
  exit 1
fi
