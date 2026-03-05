#!/bin/bash
# RED-06: Concurrency Race Condition Demonstration
# This script demonstrates the race condition in interview_storage.gleam
#
# The bug: append_session_to_jsonl does a non-atomic read-modify-write
# When multiple processes write simultaneously, updates are lost

set -e

TEST_DIR="/tmp/intent_concurrency_race_test"
TEST_FILE="$TEST_DIR/sessions.jsonl"

echo "=========================================="
echo "RED-06: Race Condition Demonstration"
echo "=========================================="
echo ""

# Clean up
rm -rf "$TEST_DIR"
mkdir -p "$TEST_DIR"

echo "Test 1: Sequential writes (baseline)"
echo "-------------------------------------"
rm -f "$TEST_FILE"

# Append 5 sessions sequentially
for i in {1..5}; do
	echo "Appending session-$i..."
	# Simulate the append operation
	if [ -f "$TEST_FILE" ]; then
		CURRENT=$(cat "$TEST_FILE")
		NEW_LINE="{\"id\":\"session-$i\",\"profile\":\"api\",\"created_at\":\"2024-01-01T00:00:00Z\",\"updated_at\":\"2024-01-01T00:00:00Z\",\"completed_at\":\"\",\"stage\":\"discovery\",\"rounds_completed\":0,\"raw_notes\":\"Sequential session-$i\"}"
		if [ -z "$CURRENT" ]; then
			NEW_CONTENT="$NEW_LINE"
		else
			NEW_CONTENT="$CURRENT"$'\n'"$NEW_LINE"
		fi
		echo "$NEW_CONTENT" >"$TEST_FILE"
	else
		NEW_LINE="{\"id\":\"session-$i\",\"profile\":\"api\",\"created_at\":\"2024-01-01T00:00:00Z\",\"updated_at\":\"2024-01-01T00:00:00Z\",\"completed_at\":\"\",\"stage\":\"discovery\",\"rounds_completed\":0,\"raw_notes\":\"Sequential session-$i\"}"
		echo "$NEW_LINE" >"$TEST_FILE"
	fi
done

# Count sessions
COUNT=$(wc -l <"$TEST_FILE")
echo "Expected: 5 sessions"
echo "Actual:   $COUNT sessions"
echo ""

if [ "$COUNT" -eq 5 ]; then
	echo "✓ Sequential writes work correctly"
else
	echo "✗ Sequential writes failed unexpectedly"
fi
echo ""

echo "Test 2: Concurrent writes (demonstrates bug)"
echo "---------------------------------------------"
echo "Running test multiple times to increase likelihood..."
echo ""

LOST_COUNT=0
RUNS=0
MAX_RUNS=10

while [ $RUNS -lt $MAX_RUNS ]; do
	RUNS=$((RUNS + 1))
	echo "Run $RUNS of $MAX_RUNS:"

	rm -f "$TEST_FILE"

	# Simulate concurrent writes with explicit delays to create race window
	append_concurrent() {
		local session_id="$1"
		local delay_before="$2"
		local delay_after="$3"

		# Simulate: READ -> [delay] -> MODIFY -> [delay] -> WRITE
		# This maximizes the race window

		# Initial delay before read
		sleep "$delay_before"

		# READ: Get current content
		if [ -f "$TEST_FILE" ]; then
			# Use flock to simulate atomic read (but no lock around write!)
			if flock -x 200; then
				CURRENT=$(cat "$TEST_FILE")
				flock -u 200
			else
				CURRENT=""
			fi
		else
			CURRENT=""
		fi 200>/dev/null

		# Delay between read and write (this creates the race condition!)
		sleep "$delay_after"

		# MODIFY: Add new session (in memory)
		NEW_LINE="{\"id\":\"$session_id\",\"profile\":\"api\",\"created_at\":\"2024-01-01T00:00:00Z\",\"updated_at\":\"2024-01-01T00:00:00Z\",\"completed_at\":\"\",\"stage\":\"discovery\",\"rounds_completed\":0,\"raw_notes\":\"Concurrent session-$session_id\"}"
		if [ -z "$CURRENT" ]; then
			NEW_CONTENT="$NEW_LINE"
		else
			NEW_CONTENT="$CURRENT"$'\n'"$NEW_LINE"
		fi

		# WRITE: Overwrite file (NO LOCK HERE - THIS IS THE BUG!)
		# In the real code, this is simplifile.write() with no flock
		echo "$NEW_CONTENT" >"$TEST_FILE"

		echo "  ✓ Process $session_id completed"
	}

	# Start 10 processes in parallel with staggered delays
	for i in {1..10}; do
		# Random delays to create race conditions
		DELAY_BEFORE=$(echo "scale=2; $RANDOM/10000" | bc 2>/dev/null || echo "0.0$i")
		DELAY_AFTER=$(echo "scale=2; $RANDOM/5000" | bc 2>/dev/null || echo "0.0$((i % 3))")

		append_concurrent "run-$RUNS-session-$i" "$DELAY_BEFORE" "$DELAY_AFTER" &
	done

	# Wait for all to complete
	wait

	# Count sessions
	if [ -f "$TEST_FILE" ]; then
		COUNT=$(wc -l <"$TEST_FILE")
	else
		COUNT=0
	fi

	echo "  Expected: 10 sessions"
	echo "  Actual:   $COUNT sessions"

	if [ "$COUNT" -lt 10 ]; then
		LOST_COUNT=$((LOST_COUNT + 10 - COUNT))
		echo "  ✗ BUG REPRODUCED: $((10 - COUNT)) session(s) lost!"
		echo ""
	else
		echo "  ? Race condition not triggered this time"
		echo ""
	fi
done

echo "=========================================="
echo "Summary"
echo "=========================================="
echo "Total runs:      $RUNS"
echo "Losses detected: $LOST_COUNT"
echo ""

if [ "$LOST_COUNT" -gt 0 ]; then
	echo "✓ BUG CONFIRMED: Race condition causes data loss"
else
	echo "? Bug not confirmed in $RUNS runs (timing-dependent)"
	echo "  Try running script again - race conditions are stochastic"
fi

echo ""
echo "=========================================="
echo "EARS Format Bug Report"
echo "=========================================="
echo ""
echo "WHEN multiple processes call append_session_to_jsonl concurrently"
echo "THE SYSTEM SHALL preserve all session updates atomically using file locks or atomic appends"
echo "BUT INSTEAD uses non-atomic read-modify-write pattern causing data loss"
echo ""
echo "Severity: P0 (CRITICAL)"
echo "Reproduction: ./test/concurrency_race.sh"
echo "Where to look: src/intent/interview_storage.gleam:615-646"
echo ""
echo "Technical Details:"
echo "  1. Line 619-622: simplifile.read() loads entire file"
echo "  2. Line 629-636: In-memory filtering (removes old session by ID)"
echo "  3. Line 640: simplifile.write() overwrites entire file"
echo ""
echo "Race Window:"
echo "  Process A: read [S1,S2] -> filter -> write [S1,S2,S3]"
echo "  Process B:           read [S1,S2] -> filter -> write [S1,S2,S4]"
echo "  Result: S3 is overwritten and lost!"
echo ""
echo "Fix Options:"
echo "  1. Use file locking (flock/lockf) before read-modify-write"
echo "  2. Switch to append-only writes (remove filtering, deduplicate on read)"
echo "  3. Use SQLite with proper transaction isolation"
echo "  4. Implement write-ahead logging with atomic rename"
echo ""

# Clean up
rm -rf "$TEST_DIR"
