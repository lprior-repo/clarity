#!/usr/bin/env bash

set -euo pipefail

WORKER_NAME="${WORKER_NAME:-worker-1}"
WORKSPACE_PREFIX="${WORKSPACE_PREFIX:-swarm}"
MODEL_POOL="${MODEL_POOL:-openai/gpt-5.3-codex,anthropic/claude-sonnet-4-20250514,z-ai/glm-5,z-ai/glm-4.5-air}"
QUEUE_PRIORITY="${QUEUE_PRIORITY:-5}"
POLL_SECONDS="${POLL_SECONDS:-12}"
MAX_BEADS="${MAX_BEADS:-0}"

completed=0
model_index=0

log() {
	printf '[%s][%s] %s\n' "$(date '+%Y-%m-%d %H:%M:%S')" "${WORKER_NAME}" "$*"
}

choose_model() {
	python - "$MODEL_POOL" "$model_index" <<'PY'
import sys
pool = [m.strip() for m in sys.argv[1].split(',') if m.strip()]
index = int(sys.argv[2])
if not pool:
    print("openai/gpt-5.3-codex")
else:
    print(pool[index % len(pool)])
PY
}

candidate_beads() {
	br --quiet ready --json | python - <<'PY'
import json
import sys

issues = json.load(sys.stdin)

def rank(issue):
    return (
        int(issue.get("priority", 9)),
        issue.get("created_at", ""),
        issue.get("id", ""),
    )

candidates = [
    issue["id"]
    for issue in sorted(issues, key=rank)
    if issue.get("status") == "open" and issue.get("issue_type") != "epic"
]

for bead_id in candidates:
    print(bead_id)
PY
}

claim_bead() {
	local bead_id
	while IFS= read -r bead_id; do
		[ -z "$bead_id" ] && continue
		if ! zjj claim "bead:${bead_id}" -t 2 >/dev/null 2>&1; then
			continue
		fi

		if br --quiet update "$bead_id" --status in_progress >/dev/null 2>&1; then
			printf '%s\n' "$bead_id"
			return 0
		fi

		zjj yield "bead:${bead_id}" >/dev/null 2>&1 || true
	done < <(candidate_beads)

	return 1
}

workspace_path_for() {
	local workspace="$1"
	zjj status "$workspace" --json | python - <<'PY'
import json
import sys

payload = json.load(sys.stdin)
for key in ("workspace_path", "path"):
    value = payload.get(key)
    if isinstance(value, str) and value:
        print(value)
        raise SystemExit(0)

session = payload.get("session")
if isinstance(session, dict):
    value = session.get("workspace_path")
    if isinstance(value, str) and value:
        print(value)
        raise SystemExit(0)

sessions = payload.get("sessions")
if isinstance(sessions, list) and sessions:
    value = sessions[0].get("workspace_path")
    if isinstance(value, str) and value:
        print(value)
        raise SystemExit(0)

raise SystemExit(1)
PY
}

run_one_bead() {
	local bead_id="$1"
	local safe_bead
	local workspace
	local workspace_path
	local model
	local prompt

	safe_bead="${bead_id//./-}"
	workspace="${WORKSPACE_PREFIX}-${WORKER_NAME}-${safe_bead}-$(date +%s)"

	model="$(choose_model)"
	model_index=$((model_index + 1))

	log "Claimed ${bead_id}; creating workspace ${workspace}"
	zjj work "$workspace" -b "$bead_id" --no-zellij >/dev/null
	workspace_path="$(workspace_path_for "$workspace")"

	read -r -d '' prompt <<'EOF' || true
You are implementing one bead in a Rust codebase.

Hard requirements:
1) Use zjj workspace isolation already provided.
2) Follow tdd15 discipline: tests FIRST, then minimal code, then refactor.
3) Follow functional-rust-generator rules: zero unwrap/expect/panic/todo/unimplemented.
4) Use Moon commands only (never raw cargo).
5) Implement the currently assigned bead from beads metadata.

Execution contract:
- Read acceptance criteria from beads.
- Write/adjust tests first (RED), then implement (GREEN), then refactor.
- Run `moon run :quick` and `moon run :test`.
- Keep changes scoped to this bead.
- Do not commit.

At completion, print a concise summary of what changed and why.
EOF

	log "Running opencode on ${bead_id} with model ${model}"
	if ! opencode run --agent build -m "$model" --title "${workspace}:${bead_id}" "$prompt" >/dev/null; then
		log "opencode failed for ${bead_id}; preserving workspace ${workspace_path}"
		zjj yield "bead:${bead_id}" >/dev/null 2>&1 || true
		return 1
	fi

	if ! moon run :quick >/dev/null; then
		log "moon quick failed for ${bead_id}; preserving workspace ${workspace_path}"
		zjj yield "bead:${bead_id}" >/dev/null 2>&1 || true
		return 1
	fi

	if ! moon run :test >/dev/null; then
		log "moon test failed for ${bead_id}; preserving workspace ${workspace_path}"
		zjj yield "bead:${bead_id}" >/dev/null 2>&1 || true
		return 1
	fi

	zjj queue --add "$workspace" --bead "$bead_id" --priority "$QUEUE_PRIORITY" >/dev/null
	zjj yield "bead:${bead_id}" >/dev/null 2>&1 || true

	log "Queued ${workspace} for merge"
	completed=$((completed + 1))
	return 0
}

log "Worker started with model pool: ${MODEL_POOL}"

while :; do
	if [ "$MAX_BEADS" -gt 0 ] && [ "$completed" -ge "$MAX_BEADS" ]; then
		log "Reached MAX_BEADS=${MAX_BEADS}; exiting"
		exit 0
	fi

	bead=""
	if bead="$(claim_bead)"; then
		run_one_bead "$bead" || true
	else
		sleep "$POLL_SECONDS"
	fi
done
