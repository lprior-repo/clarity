#!/bin/bash
# Single-run Gatekeeper Agent
# Checks for ready gatekeeper beads once and processes them

set -euo pipefail

PROJECT_ROOT="/home/lewis/src/intent-cli"
cd "${PROJECT_ROOT}"

echo "=== Gatekeeper Agent (Single Run) ==="
echo "Looking for beads with label: stage:ready-gatekeeper"
echo ""

# Find ready gatekeeper beads
ready_beads=$(br ready --label "stage:ready-gatekeeper" --json 2>/dev/null || echo "[]")

# Check if any beads found
bead_count=$(echo "$ready_beads" | jq 'length' 2>/dev/null || echo "0")

if [[ "$bead_count" -gt 0 ]]; then
    echo "Found ${bead_count} bead(s) ready for gatekeeper review:"
    echo "$ready_beads" | jq -r '.[] | "  - \(.id): \(.title)"'
    echo ""
    echo "To run the continuous gatekeeper agent, execute:"
    echo "  ./gatekeeper_agent.sh"
else
    echo "No beads ready for gatekeeper review"
    echo ""
    echo "To mark a bead as ready for gatekeeper, run:"
    echo "  br update <bead-id> --label 'stage:ready-gatekeeper'"
    echo ""
    echo "Example:"
    echo "  br update bd-30lt.27 --label 'stage:ready-gatekeeper'"
fi
