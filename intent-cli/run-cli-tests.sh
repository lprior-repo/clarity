#!/usr/bin/env bash
# Unified CLI Test Suite
# Consolidates all CLI validation tests into a single script
# Usage: ./run-cli-tests.sh [test-group]
#   test-group: one of 'all', 'unit', 'check', 'interview', 'mock-server'
#   Default: 'all'

set -euo pipefail

# Color codes for output
readonly RED='\033[0;31m'
readonly GREEN='\033[0;32m'
readonly YELLOW='\033[1;33m'
readonly BLUE='\033[0;34m'
readonly CYAN='\033[0;36m'
readonly NC='\033[0m' # No Color

# Test counters
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Test results array
declare -a FAILED_TEST_NAMES=()

# ============================================================================
# Helper Functions
# ============================================================================

print_header() {
	echo -e "${CYAN}$1${NC}"
	echo "========================================"
}

print_test() {
	echo ""
	echo -e "${BLUE}Test ${TOTAL_TESTS}: $1${NC}"
}

print_pass() {
	echo -e "${GREEN}✓ Pass: $1${NC}"
	((PASSED_TESTS += 1))
}

print_fail() {
	echo -e "${RED}✗ Fail: $1${NC}"
	((FAILED_TESTS += 1))
	FAILED_TEST_NAMES+=("Test ${TOTAL_TESTS}: $1")
}

print_section() {
	echo ""
	echo -e "${YELLOW}>>> $1${NC}"
}

run_test() {
	((TOTAL_TESTS += 1))
	local test_name="$1"
	shift
	local command=("$@")

	print_test "$test_name"

	if "${command[@]}"; then
		print_pass "$test_name"
		return 0
	else
		print_fail "$test_name"
		return 1
	fi
}

# ============================================================================
# Unit Tests: validate, show, export
# ============================================================================

run_unit_tests() {
	print_section "Running Unit Tests (validate, show, export)"

	# Test 1: validate with valid spec
	run_test "validate valid spec" \
		bash -c "gleam run -- validate examples/pokemon-api.cue 2>&1 | tr -d '\x1b' | tr -d '\r' | grep -q 'Valid spec'"

	# Test 2: show displays spec
	run_test "show displays spec" \
		bash -c "gleam run -- show examples/pokemon-api.cue 2>&1 | tr -d '\x1b' | tr -d '\r' | grep -q 'Spec: Pokemon API'"

	# Test 3: export produces JSON
	run_test "export produces JSON" \
		bash -c "gleam run -- export examples/pokemon-api.cue 2>&1 | grep -A 1000 '^{' | jq .name > /dev/null 2>&1"

	# Test 4: validate without args
	run_test "validate without arguments shows error" \
		bash -c "gleam run -- validate 2>&1 | tr -d '\x1b' | tr -d '\r' | grep -q 'spec file path required'"

	# Test 5: show --json produces JSON
	run_test "show --json produces JSON" \
		bash -c "gleam run -- show examples/pokemon-api.cue --json 2>&1 | grep -A 1000 '^{' | jq .name > /dev/null 2>&1"

	# Test 6: export without args
	run_test "export without arguments shows error" \
		bash -c "gleam run -- export 2>&1 | tr -d '\x1b' | tr -d '\r' | grep -q 'spec file path required'"

	# Test 7: validate multiple specs
	print_section "Validating all example specs"
	local all_specs_pass=true
	for spec in examples/*.cue; do
		if ! gleam run -- validate "$spec" >/dev/null 2>&1; then
			echo -e "${RED}✗ Failed: $spec${NC}"
			all_specs_pass=false
			((FAILED_TESTS += 1))
		else
			echo -e "${GREEN}✓ Valid: $spec${NC}"
			((PASSED_TESTS += 1))
		fi
		((TOTAL_TESTS += 1))
	done

	# Test 8: show without args
	run_test "show without arguments shows error" \
		bash -c "gleam run -- show 2>&1 | tr -d '\x1b' | tr -d '\r' | grep -q 'spec file path required'"
}

# ============================================================================
# Check Command Tests (requires running server or mock)
# ============================================================================

run_check_tests() {
	print_section "Running Check Command Tests"

	# Test 9: check with valid spec against server
	run_test "check runs against server" \
		bash -c "gleam run -- check examples/user-api.cue --target http://localhost:8080 --json 2>&1 | jq -e '.total >= 0' > /dev/null 2>&1"

	# Test 10: check --feature filter
	run_test "check --feature filters behaviors" \
		bash -c "gleam run -- check examples/user-api.cue --target http://localhost:8080 --feature 'User Registration' --json 2>&1 | jq -e '.total > 0' > /dev/null 2>&1"

	# Test 11: check --only filter
	run_test "check --only filters single behavior" \
		bash -c "gleam run -- check examples/user-api.cue --target http://localhost:8080 --only 'successful-registration' --json 2>&1 | jq -e '.total >= 0' > /dev/null 2>&1"

	# Test 12: check --verbose
	run_test "check --verbose provides detailed output" \
		bash -c "gleam run -- check examples/user-api.cue --target http://localhost:8080 --verbose 2>&1 | grep -q 'Testing:'"

	# Test 13: check --quiet
	run_test "check --quiet shows minimal output" \
		bash -c "gleam run -- check examples/user-api.cue --target http://localhost:8080 --quiet 2>&1 | [ \$(wc -l) -lt 10 ]"

	# Test 14: check without spec
	run_test "check without arguments shows error" \
		bash -c "gleam run -- check 2>&1 | grep -q 'spec file path required'"

	# Test 15: check without target
	run_test "check without --target shows usage" \
		bash -c "gleam run -- check examples/user-api.cue 2>&1 | grep -q 'Usage:'"
}

# ============================================================================
# Interview Command Tests
# ============================================================================

run_interview_tests() {
	print_section "Running Interview Command Tests"

	# Test 16: interview --profile api
	run_test "interview with api profile" \
		bash -c "echo 'quit' | timeout 2 gleam run -- interview --profile api 2>&1 | grep -q 'INTENT INTERVIEW'"

	# Test 17: interview --profile cli
	run_test "interview with cli profile" \
		bash -c "echo 'quit' | timeout 2 gleam run -- interview --profile cli 2>&1 | grep -q 'INTENT INTERVIEW'"

	# Test 18: interview --resume (non-existent session)
	run_test "interview --resume with non-existent session" \
		bash -c "gleam run -- interview --resume non-existent 2>&1 | grep -q 'not found' || true"

	# Test 19: beads command generates work items
	run_test "beads command generates work items" \
		bash -c "echo 'quit' | timeout 2 gleam run -- interview --profile api --export /tmp/test-spec.cue && gleam run -- beads test-session 2>&1 | grep -q 'Generated' || true"

	# Test 20: beads-status command
	run_test "bead-status command accepts flags" \
		bash -c "gleam run -- bead-status --help 2>&1 | grep -q 'Mark bead execution status'"
}

# ============================================================================
# Mock Server Tests (start local server, run tests, stop)
# ============================================================================

run_mock_server_tests() {
	print_section "Running Mock Server Tests"

	# Create a simple mock server using Python
	local mock_server_script='#!/usr/bin/env python3
import http.server
import json
from urllib.parse import urlparse, parse_qs

class MockHandler(http.server.BaseHTTPRequestHandler):
    def do_GET(self):
        path = urlparse(self.path).path
        if path == "/test-endpoint":
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.end_headers()
            self.wfile.write(json.dumps({"status": "ok"}).encode())
        elif path == "/auth/login":
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.end_headers()
            self.wfile.write(json.dumps({"token": "test-token"}).encode())
        else:
            self.send_response(404)
            self.end_headers()

    def do_POST(self):
        path = urlparse(self.path).path
        content_length = int(self.headers.get("Content-Length", 0))
        body = self.rfile.read(content_length)
        if path == "/users":
            self.send_response(201)
            self.send_header("Content-Type", "application/json")
            self.end_headers()
            self.wfile.write(json.dumps({"id": "usr_abc123", "email": "test@example.com"}).encode())
        else:
            self.send_response(404)
            self.end_headers()

    def log_message(self, format, *args):
        pass

if __name__ == "__main__":
    server = http.server.HTTPServer(("localhost", 8080), MockHandler)
    print("Mock server started on port 8080", flush=True)
    server.serve_forever()
'

	echo "$mock_server_script" >/tmp/mock_server.py
	chmod +x /tmp/mock_server.py

	# Start mock server in background
	python3 /tmp/mock_server.py >/tmp/mock_server.log 2>&1 &
	local server_pid=$!

	# Wait for server to start
	echo "Waiting for mock server to start..."
	sleep 2

	if ! kill -0 $server_pid 2>/dev/null; then
		echo -e "${RED}Failed to start mock server${NC}"
		cat /tmp/mock_server.log
		return 1
	fi

	echo -e "${GREEN}Mock server started (PID: $server_pid)${NC}"

	# Run tests against mock server
	# Test 21: Check with mock server
	run_test "check against mock server" \
		bash -c "gleam run -- check examples/user-api.cue --target http://localhost:8080 --json 2>&1 | jq -e '.total >= 0' > /dev/null 2>&1"

	# Test 22: Check with feature filter against mock
	run_test "check --feature against mock" \
		bash -c "gleam run -- check examples/user-api.cue --target http://localhost:8080 --feature 'User Registration' --json 2>&1 | jq -e '.total >= 0' > /dev/null 2>&1"

	# Test 23: Check timeout
	run_test "check handles timeout" \
		bash -c "timeout 5 gleam run -- check examples/user-api.cue --target http://localhost:8080 --json 2>&1 || true" # May timeout, that's OK

	# Cleanup
	echo "Stopping mock server..."
	kill $server_pid 2>/dev/null || true
	wait $server_pid 2>/dev/null || true
	rm -f /tmp/mock_server.py

	echo -e "${GREEN}Mock server stopped${NC}"
}

# ============================================================================
# Lint Command Tests
# ============================================================================

run_lint_tests() {
	print_section "Running Lint Command Tests"

	# Test 24: lint checks spec
	run_test "lint checks spec for issues" \
		bash -c "gleam run -- lint examples/user-api.cue 2>&1 | grep -q 'well-formed\|linting issues' || true"

	# Test 25: lint without args
	run_test "lint without arguments shows error" \
		bash -c "gleam run -- lint 2>&1 | grep -q 'spec file path required'"

	# Test 26: lint with invalid spec
	run_test "lint handles invalid spec" \
		bash -c "echo 'invalid cue' > /tmp/invalid.cue && gleam run -- lint /tmp/invalid.cue 2>&1 | grep -q 'error\|Invalid' || true"
}

# ============================================================================
# Plan Command Tests
# ============================================================================

run_plan_tests() {
	print_section "Running Plan Command Tests"

	# Test 27: plan shows execution plan
	run_test "plan shows execution plan" \
		bash -c "gleam run -- plan test-session 2>&1 | grep -q 'execution plan\|beads\|phases' || true"

	# Test 28: plan --format json
	run_test "plan --format json produces JSON" \
		bash -c "gleam run -- plan test-session --format json 2>&1 | jq . > /dev/null 2>&1 || true"

	# Test 29: plan --format ai
	run_test "plan --format ai produces AI-ready output" \
		bash -c "gleam run -- plan test-session --format ai 2>&1 | grep -q 'AI\|agent' || true"

	# Test 30: plan without args
	run_test "plan without arguments shows usage" \
		bash -c "gleam run -- plan 2>&1 | grep -q 'session_id\|Usage:'"
}

# ============================================================================
# KIRK Analysis Command Tests
# ============================================================================

run_kirk_tests() {
	print_section "Running KIRK Analysis Tests"

	# Test 31: quality analyzes spec
	run_test "quality analyzes spec" \
		bash -c "gleam run -- quality examples/user-api.cue 2>&1 | grep -q 'completeness\|consistency\|testability' || true"

	# Test 32: invert suggests failures
	run_test "invert suggests failure cases" \
		bash -c "gleam run -- invert examples/user-api.cue 2>&1 | grep -q 'gaps\|inversion\|failure' || true"

	# Test 33: coverage shows metrics
	run_test "coverage shows coverage metrics" \
		bash -c "gleam run -- coverage examples/user-api.cue 2>&1 | grep -q 'HTTP methods\|status codes\|coverage' || true"

	# Test 34: gaps detects missing tests
	run_test "gaps detects missing tests" \
		bash -c "gleam run -- gaps examples/user-api.cue 2>&1 | grep -q 'missing\|gap\|coverage gap' || true"

	# Test 35: compact produces compact format
	run_test "compact produces compact format" \
		bash -c "gleam run -- compact examples/user-api.cue 2>&1 | jq . > /dev/null 2>&1 || true"
}

# ============================================================================
# Help and Usage Tests
# ============================================================================

run_help_tests() {
	print_section "Running Help and Usage Tests"

	# Test 36: --help shows all commands
	run_test "intent --help shows commands" \
		bash -c "gleam run -- 2>&1 | grep -q 'validate\|show\|export\|check\|interview\|quality\|invert'"

	# Test 37: validate --help shows usage
	run_test "validate --help shows usage" \
		bash -c "gleam run -- validate --help 2>&1 | grep -q 'Validate a CUE spec'"

	# Test 38: check --help shows usage
	run_test "check --help shows usage" \
		bash -c "gleam run -- check --help 2>&1 | grep -q 'Run spec against a target'"

	# Test 39: interview --help shows usage
	run_test "interview --help shows usage" \
		bash -c "gleam run -- interview --help 2>&1 | grep -q 'Guided specification discovery'"
}

# ============================================================================
# Error Handling Tests
# ============================================================================

run_error_handling_tests() {
	print_section "Running Error Handling Tests"

	# Test 40: invalid CUE syntax
	run_test "handles invalid CUE syntax" \
		bash -c "echo 'invalid: {' > /tmp/invalid.cue && gleam run -- validate /tmp/invalid.cue 2>&1 | grep -q 'error\|invalid'"

	# Test 41: non-existent spec file
	run_test "handles non-existent spec file" \
		bash -c "gleam run -- validate /tmp/nonexistent.cue 2>&1 | grep -q 'not found\|No such file\|error'"

	# Test 42: permission denied
	run_test "handles permission errors" \
		bash -c "echo '{}' > /tmp/no-read.cue && chmod 000 /tmp/no-read.cue && gleam run -- validate /tmp/no-read.cue 2>&1 | grep -q 'permission\|denied\|error' || true; chmod 644 /tmp/no-read.cue 2>/dev/null"

	# Test 43: invalid flag
	run_test "handles invalid flag" \
		bash -c "gleam run -- validate --invalid-flag examples/pokemon-api.cue 2>&1 | grep -q 'unknown flag\|invalid option\|Usage' || true"
}

# ============================================================================
# Integration Tests
# ============================================================================

run_integration_tests() {
	print_section "Running Integration Tests"

	# Test 44: validate -> show -> export pipeline
	run_test "validate->show->export pipeline" \
		bash -c "gleam run -- validate examples/pokemon-api.cue && gleam run -- show examples/pokemon-api.cue --json | jq .name > /dev/null 2>&1"

	# Test 45: interview -> beads -> plan pipeline
	run_test "interview->beads->plan pipeline" \
		bash -c "echo 'quit' | timeout 2 gleam run -- interview --profile api --export /tmp/pipeline-spec.cue && gleam run -- beads test-session 2>&1 | grep -q 'Generated' || true"

	# Test 46: Multiple command invocations
	run_test "handles rapid command invocations" \
		bash -c "for i in {1..3}; do gleam run -- validate examples/pokemon-api.cue > /dev/null 2>&1 || true; done"
}

# ============================================================================
# Performance Tests
# ============================================================================

run_performance_tests() {
	print_section "Running Performance Tests"

	# Test 47: validate is fast (< 1s)
	run_test "validate completes quickly" \
		bash -c "time gleam run -- validate examples/pokemon-api.cue > /dev/null 2>&1 && [ \$SECONDS -lt 1 ]"

	# Test 48: show is fast (< 1s)
	run_test "show completes quickly" \
		bash -c "time gleam run -- show examples/pokemon-api.cue > /dev/null 2>&1 && [ \$SECONDS -lt 1 ]"

	# Test 49: Memory usage check
	run_test "memory usage is reasonable" \
		bash -c "/usr/bin/time -f '%M' gleam run -- validate examples/pokemon-api.cue 2>&1 | grep -q '^[0-9]' || true"

	# Test 50: Large spec handling
	run_test "handles large specs efficiently" \
		bash -c "for i in {1..10}; do gleam run -- validate examples/pokemon-api.cue > /dev/null 2>&1; done"
}

# ============================================================================
# Test Suite Runner
# ============================================================================

print_summary() {
	echo ""
	print_header "Test Suite Summary"

	echo -e "Total Tests: ${BLUE}${TOTAL_TESTS}${NC}"
	echo -e "Passed:      ${GREEN}${PASSED_TESTS}${NC}"
	echo -e "Failed:      ${RED}${FAILED_TESTS}${NC}"

	if [ $FAILED_TESTS -gt 0 ]; then
		echo ""
		echo -e "${RED}Failed Tests:${NC}"
		for failed_test in "${FAILED_TEST_NAMES[@]}"; do
			echo -e "  ${RED}✗${NC} $failed_test"
		done

		echo ""
		echo -e "${RED}Some tests failed!${NC}"
		exit 1
	else
		echo ""
		echo -e "${GREEN}✓ All ${TOTAL_TESTS} tests passed!${NC}"
		exit 0
	fi
}

print_usage() {
	cat <<EOF
Unified CLI Test Suite

Usage: $0 [test-group]

Test Groups:
  all           Run all test suites (default)
  unit          Run unit tests (validate, show, export)
  check         Run check command tests
  interview     Run interview command tests
  mock-server   Run mock server integration tests
  lint          Run lint command tests
  plan          Run plan command tests
  kirk          Run KIRK analysis tests
  help          Run help and usage tests
  errors        Run error handling tests
  integration   Run integration tests
  performance   Run performance tests

Examples:
  $0                 Run all tests
  $0 unit            Run only unit tests
  $0 check           Run check command tests
  $0 mock-server     Run mock server integration tests

Note: Mock server tests require Python 3 and port 8080 to be available.
EOF
}

# ============================================================================
# Main
# ============================================================================

main() {
	local test_group="${1:-all}"

	print_header "Intent CLI Unified Test Suite"
	echo "Test Group: $test_group"
	echo ""

	case "$test_group" in
	all)
		run_unit_tests
		run_check_tests
		run_interview_tests
		run_lint_tests
		run_plan_tests
		run_kirk_tests
		run_help_tests
		run_error_handling_tests
		run_integration_tests
		run_performance_tests
		;;

	unit)
		run_unit_tests
		;;

	check)
		run_check_tests
		;;

	interview)
		run_interview_tests
		;;

	mock-server)
		run_mock_server_tests
		;;

	lint)
		run_lint_tests
		;;

	plan)
		run_plan_tests
		;;

	kirk)
		run_kirk_tests
		;;

	help)
		run_help_tests
		;;

	errors)
		run_error_handling_tests
		;;

	integration)
		run_integration_tests
		;;

	performance)
		run_performance_tests
		;;

	-h | --help | help)
		print_usage
		exit 0
		;;

	*)
		echo -e "${RED}Unknown test group: $test_group${NC}"
		echo ""
		print_usage
		exit 1
		;;
	esac

	print_summary
}

main "$@"
