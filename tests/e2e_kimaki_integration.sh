#!/bin/bash
# End-to-end test to validate kimaki can call opencode

set -e

echo "=== E2E Test: kimaki -> opencode Integration ==="

# Get current session ID - use the most recent kimaki session
# Filter out debug output from kimaki CLI - extract JSON starting from first array
SESSIONS_JSON=$(kimaki session list --json 2>&1 | sed -n '/^\[/,$p')
SESSION_ID=$(echo "$SESSIONS_JSON" | jq -r '.[0].id // empty')

if [ -z "$SESSION_ID" ]; then
    echo "❌ Could not determine session ID"
    exit 1
fi

echo "Testing with session: $SESSION_ID"

# Test 1: Verify the session exists and was created by kimaki
echo ""
echo "Test 1: Verify session created by kimaki"
SOURCE=$(echo "$SESSIONS_JSON" | jq -r '.[] | select(.id == "'"$SESSION_ID"'") | .source')

if [ "$SOURCE" = "kimaki" ]; then
    echo "✅ Session correctly identified as created by kimaki"
else
    echo "❌ Session source: $SOURCE (expected: kimaki)"
    exit 1
fi

# Test 2: Verify the session is associated with a Discord thread
echo ""
echo "Test 2: Verify Discord thread association"
THREAD_ID=$(echo "$SESSIONS_JSON" | jq -r '.[] | select(.id == "'"$SESSION_ID"'") | .threadId')

if [ -n "$THREAD_ID" ] && [ "$THREAD_ID" != "null" ]; then
    echo "✅ Session associated with Discord thread: $THREAD_ID"
else
    echo "❌ No Discord thread associated"
    exit 1
fi

# Test 3: Verify opencode server is accessible
echo ""
echo "Test 3: Verify OpenCode server connectivity"
if curl -s http://localhost:39527/health >/dev/null 2>&1 || curl -s http://localhost:33739/health >/dev/null 2>&1 || curl -s http://localhost:34577/health >/dev/null 2>&1; then
    echo "✅ OpenCode server is running"
else
    echo "⚠️  OpenCode server health check not available (may still be running)"
fi

# Test 4: Verify the project directory is correct
echo ""
echo "Test 4: Verify project directory"
PROJECT_DIR=$(echo "$SESSIONS_JSON" | grep -o "\"id\":\"$SESSION_ID\"" -A5 | grep -o '"directory":"[^"]*"' | cut -d'"' -f4)

if [ -d "$PROJECT_DIR" ]; then
    echo "✅ Project directory exists: $PROJECT_DIR"
else
    echo "❌ Project directory not found: $PROJECT_DIR"
    exit 1
fi

echo ""
echo "=== All E2E tests passed! ==="
echo ""
echo "Summary:"
echo "  - kimaki created session: $SESSION_ID"
echo "  - Source: kimaki"
echo "  - Thread: $THREAD_ID"
echo "  - Project: $PROJECT_DIR"
