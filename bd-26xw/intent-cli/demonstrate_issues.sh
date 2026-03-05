#!/bin/bash

echo "=========================================="
echo "DEMONSTRATING CUE VALIDATION ISSUES"
echo "=========================================="
echo ""

echo "ISSUE 1: Empty behaviors list passes validation"
echo "-----------------------------------------------"
cat > /tmp/demo_empty_behaviors.cue << 'CUE'
package demo
import "github.com/intent-cli/intent/schema:intent"
spec: intent.#Spec & {
    name: "Demo"
    description: "Feature with NO behaviors"
    audience: "Test"
    version: "1.0.0"
    success_criteria: []
    config: {
        base_url: "http://localhost:8080"
        timeout_ms: 5000
        headers: {}
    }
    features: [{
        name: "Empty Feature"
        description: "This has no behaviors!"
        behaviors: []  // EMPTY - Should this be allowed?
    }]
    rules: []
    anti_patterns: []
    ai_hints: {
        implementation: {}
        entities: {}
        security: {}
        pitfalls: []
    }
}
CUE

cue vet /tmp/demo_empty_behaviors.cue 2>&1
if [ $? -eq 0 ]; then
    echo "✗ PROBLEM: Empty behaviors list PASSED validation"
else
    echo "✓ GOOD: Empty behaviors list was rejected"
fi
echo ""

echo "ISSUE 2: Negative timeout passes validation"
echo "--------------------------------------------"
cat > /tmp/demo_negative_timeout.cue << 'CUE'
package demo
import "github.com/intent-cli/intent/schema:intent"
spec: intent.#Spec & {
    name: "Demo"
    description: "Negative timeout"
    audience: "Test"
    version: "1.0.0"
    success_criteria: []
    config: {
        base_url: "http://localhost:8080"
        timeout_ms: -9999  // NEGATIVE - Should be rejected!
        headers: {}
    }
    features: [{
        name: "Test"
        description: "Test"
        behaviors: [{
            name: "test"
            intent: "Test"
            request: {
                method: "GET"
                path: "/test"
                headers: {}
                query: {}
            }
            response: {
                status: 200
                checks: {}
            }
        }]
    }]
    rules: []
    anti_patterns: []
    ai_hints: {
        implementation: {}
        entities: {}
        security: {}
        pitfalls: []
    }
}
CUE

cue vet /tmp/demo_negative_timeout.cue 2>&1
if [ $? -eq 0 ]; then
    echo "✗ PROBLEM: Negative timeout PASSED validation"
else
    echo "✓ GOOD: Negative timeout was rejected"
fi
echo ""

echo "ISSUE 3: Empty name field passes validation"
echo "--------------------------------------------"
cat > /tmp/demo_empty_name.cue << 'CUE'
package demo
import "github.com/intent-cli/intent/schema:intent"
spec: intent.#Spec & {
    name: ""  // EMPTY - Should be rejected!
    description: "Empty name test"
    audience: "Test"
    version: "1.0.0"
    success_criteria: []
    config: {
        base_url: "http://localhost:8080"
        timeout_ms: 5000
        headers: {}
    }
    features: [{
        name: "Test"
        description: "Test"
        behaviors: [{
            name: "test"
            intent: "Test"
            request: {
                method: "GET"
                path: "/test"
                headers: {}
                query: {}
            }
            response: {
                status: 200
                checks: {}
            }
        }]
    }]
    rules: []
    anti_patterns: []
    ai_hints: {
        implementation: {}
        entities: {}
        security: {}
        pitfalls: []
    }
}
CUE

cue vet /tmp/demo_empty_name.cue 2>&1
if [ $? -eq 0 ]; then
    echo "✗ PROBLEM: Empty name field PASSED validation"
else
    echo "✓ GOOD: Empty name field was rejected"
fi
echo ""

echo "ISSUE 4: Working validations (for comparison)"
echo "----------------------------------------------"
cat > /tmp/demo_invalid_method.cue << 'CUE'
package demo
import "github.com/intent-cli/intent/schema:intent"
spec: intent.#Spec & {
    name: "Demo"
    description: "Invalid method test"
    audience: "Test"
    version: "1.0.0"
    success_criteria: []
    config: {
        base_url: "http://localhost:8080"
        timeout_ms: 5000
        headers: {}
    }
    features: [{
        name: "Test"
        description: "Test"
        behaviors: [{
            name: "test"
            intent: "Test"
            request: {
                method: "INVALID_METHOD"  // This SHOULD fail
                path: "/test"
                headers: {}
                query: {}
            }
            response: {
                status: 200
                checks: {}
            }
        }]
    }]
    rules: []
    anti_patterns: []
    ai_hints: {
        implementation: {}
        entities: {}
        security: {}
        pitfalls: []
    }
}
CUE

cue vet /tmp/demo_invalid_method.cue 2>&1
if [ $? -eq 0 ]; then
    echo "✗ PROBLEM: Invalid method PASSED validation"
else
    echo "✓ GOOD: Invalid method was REJECTED (as expected)"
fi
echo ""

echo "=========================================="
echo "SUMMARY"
echo "=========================================="
echo "The CUE schema correctly catches some errors (like invalid HTTP methods)"
echo "but fails to catch others (empty lists, negative numbers, empty strings)."
echo ""
echo "Run: ./run_cue_validation.sh for full test suite"
echo "Read: CUE_VALIDATION_QA_REPORT.md for detailed analysis"
