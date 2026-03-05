#!/usr/bin/env bash

set -u

PASS_COUNT=0
FAIL_COUNT=0
TOTAL_COUNT=0
INTENT_CMD="./build/erlang-shipment/entrypoint.sh run"
SCENARIO_TIMEOUT_SECONDS="${SCENARIO_TIMEOUT_SECONDS:-90}"

contains_expected_code() {
	local actual="$1"
	local expected_csv="$2"
	IFS=',' read -r -a expected <<<"$expected_csv"
	for code in "${expected[@]}"; do
		if [ "$actual" = "$code" ]; then
			return 0
		fi
	done
	return 1
}

run_case() {
	local scenario="$1"
	local given="$2"
	local when_clause="$3"
	local then_clause="$4"
	local expected_codes="$5"
	local command="$6"
	local resolved_command="${command//__INTENT__/$INTENT_CMD}"

	TOTAL_COUNT=$((TOTAL_COUNT + 1))
	local log_file
	log_file="$(mktemp)"

	echo ""
	echo "Scenario ${TOTAL_COUNT}: ${scenario}"
	echo "  GIVEN ${given}"
	echo "  WHEN  ${when_clause}"
	echo "  THEN  ${then_clause}"

	set +e
	timeout "$SCENARIO_TIMEOUT_SECONDS" bash -lc "$resolved_command" >"$log_file" 2>&1
	local exit_code=$?
	set -e

	if contains_expected_code "$exit_code" "$expected_codes"; then
		echo "  PASS (exit=${exit_code})"
		PASS_COUNT=$((PASS_COUNT + 1))
	else
		echo "  FAIL (exit=${exit_code}, expected=${expected_codes})"
		echo "  Command: ${resolved_command}"
		echo "  Output:"
		sed -n '1,20p' "$log_file"
		FAIL_COUNT=$((FAIL_COUNT + 1))
	fi

	rm -f "$log_file"
}

main() {
	set -e

	echo "CLI BDD Smoke Matrix"
	echo "===================="

	echo "Preparing shipped CLI runtime..."
	gleam export erlang-shipment >/dev/null

	run_case "root help works" \
		"the CLI is built" \
		"running intent help" \
		"it exits successfully" \
		"0" \
		"__INTENT__ --help"

	run_case "check requires spec" "no spec arg is provided" "running check" "it exits with usage error" "4" "__INTENT__ check"
	run_case "check handles unreachable target" "a valid spec and unreachable target" "running check with --json" "it exits as blocked/fail signal" "2,1" "__INTENT__ check examples/user-api.cue --target=http://127.0.0.1:1 --json=true"
	run_case "validate succeeds on example" "a valid spec file" "running validate" "it exits successfully" "0" "__INTENT__ validate examples/user-api.cue"
	run_case "show json succeeds" "a valid spec file" "running show --json" "it exits successfully" "0" "__INTENT__ show examples/user-api.cue --json=true"
	run_case "export succeeds" "a valid spec file" "running export" "it exits successfully" "0" "__INTENT__ export examples/user-api.cue"
	run_case "lint returns warning status" "example spec with lint issues" "running lint" "it exits with lint warning code" "1,0" "__INTENT__ lint examples/user-api.cue"

	run_case "analyze succeeds" "a valid spec file" "running analyze" "it exits successfully" "0" "__INTENT__ analyze examples/user-api.cue"
	run_case "improve succeeds" "a valid spec file" "running improve" "it exits successfully" "0" "__INTENT__ improve examples/user-api.cue"
	run_case "quality succeeds" "a valid spec file" "running quality" "it exits successfully" "0" "__INTENT__ quality examples/user-api.cue"
	run_case "invert succeeds" "a valid spec file" "running invert" "it exits successfully" "0" "__INTENT__ invert examples/user-api.cue"
	run_case "coverage succeeds" "a valid spec file" "running coverage" "it exits successfully" "0" "__INTENT__ coverage examples/user-api.cue"
	run_case "gaps succeeds" "a valid spec file" "running gaps" "it exits successfully" "0" "__INTENT__ gaps examples/user-api.cue"
	run_case "compact succeeds" "a valid spec file" "running compact" "it exits successfully" "0" "__INTENT__ compact examples/user-api.cue"
	run_case "prototext succeeds" "a valid spec file" "running prototext" "it exits successfully" "0" "__INTENT__ prototext examples/user-api.cue"
	run_case "effects succeeds" "a valid spec file" "running effects" "it exits successfully" "0" "__INTENT__ effects examples/user-api.cue"

	run_case "ears parser succeeds" "EARS requirements file" "running ears command" "it exits successfully" "0" "__INTENT__ ears examples/requirements.ears.md"
	run_case "sessions json works" "no session precondition" "running sessions --json" "it exits successfully" "0" "__INTENT__ sessions --json=true"

	run_case "beads requires session id" "no session argument" "running beads" "it exits with usage error" "4" "__INTENT__ beads"
	run_case "bead-status requires bead-id" "no bead-id flag" "running bead-status" "it exits with usage error" "4" "__INTENT__ bead-status"
	run_case "beads-regenerate requires session id" "no session argument" "running beads-regenerate" "it exits with usage error" "4" "__INTENT__ beads-regenerate"
	run_case "plan requires session id" "no session argument" "running plan" "it exits with usage error" "4" "__INTENT__ plan"
	run_case "plan-approve requires session id" "no session argument" "running plan-approve" "it exits with usage error" "4" "__INTENT__ plan-approve"
	run_case "history requires session id" "no session argument" "running history" "it exits with usage error" "4" "__INTENT__ history"
	run_case "diff requires two ids" "no session arguments" "running diff" "it exits with usage error" "4" "__INTENT__ diff"

	run_case "validate-bead requires bead file" "no bead path" "running validate-bead" "it exits with usage error" "4" "__INTENT__ validate-bead"
	run_case "interview help works" "interview command exists" "running interview --help" "it exits successfully" "0" "__INTENT__ interview --help"

	echo ""
	echo "Smoke Results"
	echo "-------------"
	echo "Total:  ${TOTAL_COUNT}"
	echo "Pass:   ${PASS_COUNT}"
	echo "Fail:   ${FAIL_COUNT}"

	if [ "$FAIL_COUNT" -gt 0 ]; then
		exit 1
	fi
}

main "$@"
