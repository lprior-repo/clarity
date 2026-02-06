#!/bin/bash
set -e

# Enhanced Ralph Wiggum Loop for processing all 42 beads
# This script intelligently processes beads, tracks progress, and handles errors

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Bead list (all 42 beads)
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
TASK_PROMISE="READY_FOR_NEXT_TASK"

# Progress tracking
PROGRESS_FILE=".ralph/bead-progress.txt"
TOTAL_BEADS=${#BEADS[@]}

# Function to log progress
log_progress() {
	local bead_id=$1
	local status=$2
	local message=$3
	local timestamp=$(date -Iseconds)
	echo "$timestamp|$bead_id|$status|$message" >>"$PROGRESS_FILE"
}

# Function to get bead status from br
get_bead_status() {
	local bead_id=$1
	br show --json "$bead_id" 2>/dev/null | jq -r '.status' || echo "unknown"
}

# Function to get bead details
get_bead_details() {
	local bead_id=$1
	br show --json "$bead_id" 2>/dev/null || echo "{}"
}

# Function to print colored message
print_msg() {
	local color=$1
	shift
	echo -e "${color}$*${NC}"
}

# Function to process a single bead
process_bead() {
	local bead_id=$1
	local bead_number=$2

	print_msg "$BLUE" "=========================================="
	print_msg "$BLUE" "[$bead_number/$TOTAL_BEADS] Processing bead: $bead_id"
	print_msg "$BLUE" "=========================================="

	# Check if bead already closed
	local bead_status=$(get_bead_status "$bead_id")
	if [ "$bead_status" = "closed" ]; then
		print_msg "$GREEN" "✓ Bead $bead_id is already closed"
		log_progress "$bead_id" "skipped" "Already closed"
		return 0
	fi

	# Get bead details
	local bead_info=$(get_bead_details "$bead_id")
	local bead_title=$(echo "$bead_info" | jq -r '.title // "Unknown title"')
	local bead_description=$(echo "$bead_info" | jq -r '.description // ""')

	print_msg "$YELLOW" "Bead title: $bead_title"

	# Create prompt for ralph
	local prompt="Implement bead $bead_id: $bead_title

Follow these requirements:
1. Read AGENTS.md for project guidelines
2. Use TDD (Test-Driven Development) - write tests first
3. Follow functional programming principles
4. Zero unwrap/panic calls
5. Use Result types for error handling
6. Complete all verification checkpoints in the bead specification

Bead specification:
$bead_description

Complete the implementation, verify all tests pass, and say '$COMPLETION_PROMISE' when done."

	# Run ralph loop for this bead
	log_progress "$bead_id" "started" "Starting ralph loop"

	if ralph "$prompt" \
		--model "$MODEL" \
		--max-iterations "$MAX_ITERATIONS" \
		--completion-promise "$COMPLETION_PROMISE" \
		--task-promise "$TASK_PROMISE" \
		--tasks \
		--no-commit; then

		print_msg "$GREEN" "✓ Ralph loop completed for $bead_id"
		log_progress "$bead_id" "ralph_complete" "Ralph loop finished successfully"
	else
		print_msg "$RED" "✗ Ralph loop failed for $bead_id"
		log_progress "$bead_id" "ralph_failed" "Ralph loop encountered error"
		return 1
	fi

	# Check bead status after processing
	bead_status=$(get_bead_status "$bead_id")

	if [ "$bead_status" = "closed" ]; then
		print_msg "$GREEN" "✓✓ Bead $bead_id closed successfully"
		log_progress "$bead_id" "completed" "Bead closed"
	elif [ "$bead_status" = "open" ]; then
		print_msg "$YELLOW" "⚠ Bead $bead_id is still open"
		print_msg "$YELLOW" "  You may need to manually verify and close it with: br close $bead_id"
		log_progress "$bead_id" "needs_review" "Bead still open, needs manual review"
	else
		print_msg "$YELLOW" "? Bead $bead_id status: $bead_status"
		log_progress "$bead_id" "unknown_status" "Status: $bead_status"
	fi

	echo ""
}

# Main execution
main() {
	print_msg "$BLUE" "Starting Ralph Wiggum Loop for $TOTAL_BEADS beads"
	print_msg "$BLUE" "Using model: $MODEL"
	print_msg "$BLUE" "Progress tracking: $PROGRESS_FILE"
	echo ""

	# Initialize progress file
	echo "# Ralph Bead Progress Log - $(date -Iseconds)" >"$PROGRESS_FILE"
	echo "# Format: timestamp|bead_id|status|message" >>"$PROGRESS_FILE"

	# Process each bead
	local completed=0
	local failed=0
	local skipped=0
	local needs_review=0

	for i in "${!BEADS[@]}"; do
		local bead="${BEADS[$i]}"
		local bead_number=$((i + 1))

		if process_bead "$bead" "$bead_number"; then
			local status=$(get_bead_status "$bead")
			if [ "$status" = "closed" ]; then
				((completed++))
			elif [ "$status" = "open" ]; then
				((needs_review++))
			else
				((skipped++))
			fi
		else
			((failed++))
			print_msg "$RED" "Failed to process $bead. Continue? (y/n)"
			read -r continue
			if [ "$continue" != "y" ]; then
				print_msg "$RED" "Exiting loop"
				break
			fi
		fi

		# Show progress summary
		print_msg "$BLUE" "Progress: $completed completed, $needs_review need review, $failed failed, $skipped skipped"
		echo ""

		# Optional: Pause between beads
		if [ $bead_number -lt $TOTAL_BEADS ]; then
			read -p "Press Enter to continue to next bead (or Ctrl+C to stop)..."
		fi
	done

	# Final summary
	print_msg "$BLUE" "=========================================="
	print_msg "$BLUE" "Ralph Wiggum Loop Complete"
	print_msg "$BLUE" "=========================================="
	print_msg "$GREEN" "✓ Completed: $completed"
	print_msg "$YELLOW" "⚠ Need Review: $needs_review"
	print_msg "$RED" "✗ Failed: $failed"
	print_msg "$BLUE" "○ Skipped: $skipped"
	print_msg "$BLUE" "Total: $TOTAL_BEADS beads"
	print_msg "$BLUE" "=========================================="

	if [ $needs_review -gt 0 ]; then
		echo ""
		print_msg "$YELLOW" "Beads needing manual review:"
		for i in "${!BEADS[@]}"; do
			local bead="${BEADS[$i]}"
			local status=$(get_bead_status "$bead")
			if [ "$status" = "open" ]; then
				print_msg "$YELLOW" "  - $bead"
			fi
		done
	fi
}

# Run main function
main "$@"
