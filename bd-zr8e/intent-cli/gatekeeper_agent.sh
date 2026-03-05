#!/bin/bash
# Continuous Gatekeeper Agent
# Monitors beads marked as "stage:ready-gatekeeper" and runs QA checks
# Runs continuously until manually stopped

set -euo pipefail

# Configuration
CHECK_INTERVAL=30  # seconds between checks
LOG_FILE="gatekeeper_agent.log"
PROJECT_ROOT="/home/lewis/src/intent-cli"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging function
log() {
    local level=$1
    shift
    local message="$@"
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    echo -e "${timestamp} [${level}] ${message}" | tee -a "${LOG_FILE}"
}

# Print colored output
print_success() {
    echo -e "${GREEN}✓ $*${NC}"
}

print_error() {
    echo -e "${RED}✗ $*${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠ $*${NC}"
}

print_info() {
    echo -e "${BLUE}ℹ $*${NC}"
}

# QA check functions
run_gleam_test() {
    log "INFO" "Running gleam test..."
    if gleam test > /dev/null 2>&1; then
        log "INFO" "gleam test: PASSED"
        return 0
    else
        log "ERROR" "gleam test: FAILED"
        gleam test 2>&1 | tee -a "${LOG_FILE}"
        return 1
    fi
}

check_unwrap_panic() {
    log "INFO" "Checking for dangerous unwrap/panic/expect usage..."

    # Check for problematic patterns:
    # 1. \bunwrap\(\) - standalone unwrap() calls (not result.unwrap())
    # 2. \bpanic\( - panic() function calls
    # 3. \bexpect\( - expect() function calls
    local result
    result=$(grep -rn '\bunwrap()\|\bpanic(\|\bexpect(' src/ 2>/dev/null || true)

    if [[ -n "$result" ]]; then
        log "ERROR" "Found dangerous unwrap/panic/expect in source code:"
        echo "$result" | tee -a "${LOG_FILE}"
        return 1
    fi

    log "INFO" "No dangerous unwrap/panic/expect found: PASSED"
    return 0
}

run_gleam_check() {
    log "INFO" "Running gleam check..."
    if gleam check > /dev/null 2>&1; then
        log "INFO" "gleam check: PASSED"
        return 0
    else
        log "ERROR" "gleam check: FAILED"
        gleam check 2>&1 | tee -a "${LOG_FILE}"
        return 1
    fi
}

# Run all QA checks
run_qa_checks() {
    log "INFO" "=== Starting QA Checks ==="

    local all_passed=true

    if ! run_gleam_test; then
        all_passed=false
    fi

    if ! check_unwrap_panic; then
        all_passed=false
    fi

    if ! run_gleam_check; then
        all_passed=false
    fi

    if [[ "$all_passed" == "true" ]]; then
        log "INFO" "=== All QA Checks PASSED ==="
        return 0
    else
        log "ERROR" "=== QA Checks FAILED ==="
        return 1
    fi
}

# Process a single bead
process_bead() {
    local bead_id="$1"
    local title="$2"

    log "INFO" "Processing bead: ${bead_id} - ${title}"

    # Claim the bead
    log "INFO" "Claiming bead ${bead_id}..."
    if ! br update "${bead_id}" --status in_progress --json > /dev/null 2>&1; then
        log "ERROR" "Failed to claim bead ${bead_id}"
        return 1
    fi

    # Run QA checks
    if run_qa_checks; then
        # QA passed - close the bead
        local close_reason="QA checks passed: gleam test ✓, no unwrap/panic/expect ✓, gleam check ✓"
        log "INFO" "Closing bead ${bead_id} with: ${close_reason}"

        if br close "${bead_id}" --reason "${close_reason}" --json > /dev/null 2>&1; then
            print_success "Closed: ${bead_id} - ${title}"
            return 0
        else
            log "ERROR" "Failed to close bead ${bead_id}"
            # Revert to open status
            br update "${bead_id}" --status open --json > /dev/null 2>&1 || true
            return 1
        fi
    else
        # QA failed - mark as failed
        local failure_reason="QA checks failed - see ${LOG_FILE} for details"
        log "ERROR" "Marking bead ${bead_id} as failed: ${failure_reason}"

        # Add failure label and revert to open
        br update "${bead_id}" --status open --label "qa-failed" --notes "${failure_reason}" --json > /dev/null 2>&1 || true
        print_error "QA FAILED: ${bead_id} - ${title}"
        return 1
    fi
}

# Main loop
main() {
    cd "${PROJECT_ROOT}"
    log "INFO" "=== Gatekeeper Agent Started ==="
    print_info "Gatekeeper agent running (PID: $$)"
    print_info "Checking every ${CHECK_INTERVAL} seconds for beads with label: stage:ready-gatekeeper"
    print_info "Log file: ${LOG_FILE}"
    print_info "Press Ctrl+C to stop"

    local iteration=0

    while true; do
        ((iteration++))
        log "INFO" "=== Iteration ${iteration} ==="

        # Find ready gatekeeper beads
        local ready_beads
        ready_beads=$(br ready --label "stage:ready-gatekeeper" --json 2>/dev/null || echo "[]")

        # Check if any beads found
        local bead_count
        bead_count=$(echo "$ready_beads" | jq 'length' 2>/dev/null || echo "0")

        if [[ "$bead_count" -gt 0 ]]; then
            log "INFO" "Found ${bead_count} bead(s) ready for gatekeeper review"

            # Process each bead
            echo "$ready_beads" | jq -r '.[] | @json' | while IFS= read -r bead_json; do
                local bead_id
                local title
                bead_id=$(echo "$bead_json" | jq -r '.id')
                title=$(echo "$bead_json" | jq -r '.title')

                process_bead "$bead_id" "$title"
            done
        else
            log "INFO" "No beads ready for gatekeeper review"
        fi

        # Wait before next check
        if [[ $iteration -eq 1 ]]; then
            print_info "No beads ready. Waiting ${CHECK_INTERVAL}s..."
        fi
        sleep "$CHECK_INTERVAL"
    done
}

# Trap signals for graceful shutdown
trap 'log "INFO" "Gatekeeper agent stopped"; print_info "Agent stopped"; exit 0' INT TERM

# Run main loop
main "$@"
