#!/bin/bash
set -e

# Ralph Wiggum Loop for processing all 42 beads
# This script processes each bead sequentially using ralph with local 4.7 model

BEADS=(
	"bd-di8"
	"bd-1ft"
	"bd-2if"
	"bd-3vg"
	"bd-26p"
	"bd-1ib"
	"bd-3s0"
	"bd-2b3"
	"bd-2ck"
	"bd-10g"
	"bd-2pj"
	"bd-4a6"
	"bd-2j4"
	"bd-34e"
	"bd-3tq"
	"bd-ccj"
	"bd-3j3"
	"bd-3n6"
	"bd-1r8"
	"bd-w5a"
	"bd-84g"
	"bd-3rr"
	"bd-4pq"
	"bd-2cg"
	"bd-z11"
	"bd-1bc"
	"bd-21h"
	"bd-2nx"
	"bd-57v"
	"bd-1v2"
	"bd-dws"
	"bd-2yt"
	"bd-264"
	"bd-3ey"
	"bd-2fg"
	"bd-4h9"
	"bd-2lb"
	"bd-3ki"
	"bd-19o"
	"bd-3ue"
	"bd-ezl"
	"bd-1my"
)

# Model configuration
MODEL="openai/zai-coding-plan/glm-4.7"
MAX_ITERATIONS=10
COMPLETION_PROMISE="BEAD_COMPLETE"

# Function to process a single bead
process_bead() {
	local bead_id=$1

	echo "=========================================="
	echo "Processing bead: $bead_id"
	echo "=========================================="

	# Get bead details
	local bead_info=$(br show --json "$bead_id")
	local bead_title=$(echo "$bead_info" | jq -r '.title')

	echo "Bead title: $bead_title"

	# Run ralph loop for this bead
	ralph "Implement bead $bead_id: $bead_title. Follow AGENTS.md guidelines. Use TDD. Complete the bead and verify with tests. Say '$COMPLETION_PROMISE' when done." \
		--model "$MODEL" \
		--max-iterations "$MAX_ITERATIONS" \
		--completion-promise "$COMPLETION_PROMISE" \
		--tasks \
		--no-commit

	# Check if bead should be closed
	local bead_status=$(br show --json "$bead_id" | jq -r '.status')

	if [ "$bead_status" = "open" ]; then
		echo "WARNING: Bead $bead_id is still open after ralph loop"
		echo "You may need to manually verify and close it"
	else
		echo "✓ Bead $bead_id completed successfully"
	fi

	echo ""
}

# Main execution
echo "Starting Ralph Wiggum Loop for $(echo "${#BEADS[@]}") beads"
echo "Using model: $MODEL"
echo ""

# Process each bead
for bead in "${BEADS[@]}"; do
	process_bead "$bead"

	# Optional: Pause between beads for review
	read -p "Press Enter to continue to next bead (or Ctrl+C to stop)..."
done

echo "=========================================="
echo "All beads processed!"
echo "=========================================="
