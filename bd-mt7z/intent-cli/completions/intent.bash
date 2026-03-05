#!/usr/bin/env bash
# Bash completion script for Intent CLI

# Get the directory where this script is located
_intent_completion_dir="${BASH_SOURCE[0]%/*}"

# Intent CLI completion function
_intent_completion() {
    local cur prev words cword
    _get_comp_words_by_ref -n : cur prev words cword

    # Store all possible commands (including aliases)
    local commands=(
        "interview" "int" "beads" "bead-status" "history" "hist" "diff" "sessions" "sess"
        "plan" "plan-next" "plan-approve" "plan-emit-beads" "beads-regenerate"
        "vision" "vis" "ready" "effects" "eff"
    )

    # Store all flags (including shortcuts)
    local flags=(
        "--profile" "-p" "--session" "-s" "--format" "-f" "--out" "-o" "--output"
        "--resume" "--answer" "--bead-id" "--status" "--reason"
        "--notes" "--strategy" "--name" "--target" "--feature"
        "--export-answers-template" "--vision" "--json" "-j" "--verbose"
        "--quiet" "--yes" "--draft" "--confirm" "--dry-run"
        "--execute" "--force" "--help" "--version"
    )

    # Store profile types
    local profiles=("api" "cli" "event" "data" "workflow" "ui")

    # Store output formats
    local formats=("json" "jsonl" "markdown")

    # Store strategies
    local strategies=("page_rank" "critical_path" "shortest" "risk_first")

    # Store all file extensions
    local file_extensions=("*.cue" "*.md" "*.json" "*.jsonl")

    # If no command yet, complete commands and flags
    if [[ ${cword} -eq 1 ]]; then
        COMPREPLY=( $(compgen -W "${commands[*]}" -- "${cur}") )
        COMPREPLY+=( $(compgen -W "${flags[*]}" -- "${cur}") )
        return 0
    fi

    # Get the command (first word after intent)
    local command="${words[1]}"

    # Complete based on the command
    case "${command}" in
        interview|int)
            # Interview subcommand completion
            if [[ ${cword} -eq 2 ]]; then
                COMPREPLY=( $(compgen -W "--profile -p --resume --help" -- "${cur}") )
            elif [[ ${cword} -eq 3 && (${prev} == "--profile" || ${prev} == "-p") ]]; then
                COMPREPLY=( $(compgen -W "${profiles[*]}" -- "${cur}") )
            elif [[ ${cword} -eq 3 && ${prev} == "--resume" ]]; then
                # Try to get session IDs
                _complete_intent_sessions
            fi
            ;;
        beads|beads-regenerate)
            # Beads subcommand completion
            if [[ ${cword} -eq 2 ]]; then
                COMPREPLY=( $(compgen -W "--session -s --format -f --out --help" -- "${cur}") )
            elif [[ ${cword} -eq 3 && (${prev} == "--session" || ${prev} == "-s") ]]; then
                _complete_intent_sessions
            elif [[ ${cword} -eq 3 && (${prev} == "--format" || ${prev} == "-f") ]]; then
                COMPREPLY=( $(compgen -W "${formats[*]}" -- "${cur}") )
            elif [[ ${cword} -eq 3 && ${prev} == "--out" ]]; then
                COMPREPLY=( $(compgen -W " --" -- "${cur}") )
            fi
            ;;
        history|hist)
            # History subcommand completion
            if [[ ${cword} -eq 2 ]]; then
                COMPREPLY=( $(compgen -W "--help" -- "${cur}") )
            fi
            ;;
        diff)
            # Diff subcommand completion
            if [[ ${cword} -eq 2 ]]; then
                COMPREPLY=( $(compgen -W "--session -s --help" -- "${cur}") )
            elif [[ ${cword} -eq 3 && (${prev} == "--session" || ${prev} == "-s") ]]; then
                _complete_intent_sessions
            fi
            ;;
        sessions|sess)
            # Sessions subcommand completion
            if [[ ${cword} -eq 2 ]]; then
                COMPREPLY=( $(compgen -W "--profile -p --help" -- "${cur}") )
            elif [[ ${cword} -eq 3 && (${prev} == "--profile" || ${prev} == "-p") ]]; then
                COMPREPLY=( $(compgen -W "${profiles[*]}" -- "${cur}") )
            fi
            ;;
        effects|eff)
            # Beads subcommand completion
            if [[ ${cword} -eq 2 ]]; then
                COMPREPLY=( $(compgen -W "--session --format --out --help" -- "${cur}") )
            elif [[ ${cword} -eq 3 && ${prev} == "--session" ]]; then
                _complete_intent_sessions
            elif [[ ${cword} -eq 3 && ${prev} == "--format" ]]; then
                COMPREPLY=( $(compgen -W "${formats[*]}" -- "${cur}") )
            elif [[ ${cword} -eq 3 && ${prev} == "--out" ]]; then
                COMPREPLY=( $(compgen -W " --" -- "${cur}") )
            fi
            ;;
        bead-status)
            # Bead-status subcommand completion
            if [[ ${cword} -eq 2 ]]; then
                COMPREPLY=( $(compgen -W "--bead-id --help" -- "${cur}") )
            elif [[ ${cword} -eq 3 && ${prev} == "--bead-id" ]]; then
                # Could add bead completion here if needed
                COMPREPLY=( $(compgen -W " --" -- "${cur}") )
            fi
            ;;
        diff)
            # Diff subcommand completion
            if [[ ${cword} -eq 2 ]]; then
                COMPREPLY=( $(compgen -W "--session --help" -- "${cur}") )
            elif [[ ${cword} -eq 3 && ${prev} == "--session" ]]; then
                _complete_intent_sessions
            fi
            ;;
        sessions)
            # Sessions subcommand completion
            if [[ ${cword} -eq 2 ]]; then
                COMPREPLY=( $(compgen -W "--profile --help" -- "${cur}") )
            elif [[ ${cword} -eq 3 && ${prev} == "--profile" ]]; then
                COMPREPLY=( $(compgen -W "${profiles[*]}" -- "${cur}") )
            fi
            ;;
        plan)
            # Plan subcommand completion
            if [[ ${cword} -eq 2 ]]; then
                COMPREPLY=( $(compgen -W "--notes --help" -- "${cur}") )
            elif [[ ${cword} -eq 3 && ${prev} == "--notes" ]]; then
                COMPREPLY=( $(compgen -W " --" -- "${cur}") )
            fi
            ;;
        plan-next)
            # Plan-next subcommand completion
            if [[ ${cword} -eq 2 ]]; then
                COMPREPLY=( $(compgen -W "--strategy --help" -- "${cur}") )
            elif [[ ${cword} -eq 3 && ${prev} == "--strategy" ]]; then
                COMPREPLY=( $(compgen -W "${strategies[*]}" -- "${cur}") )
            fi
            ;;
        plan-approve)
            # Plan-approve subcommand completion
            if [[ ${cword} -eq 2 ]]; then
                COMPREPLY=( $(compgen -W "--notes --help" -- "${cur}") )
            elif [[ ${cword} -eq 3 ]]; then
                # Plan ID should be provided here
                COMPREPLY=( $(compgen -W " --" -- "${cur}") )
            elif [[ ${cword} -eq 4 && ${prev} == "--notes" ]]; then
                COMPREPLY=( $(compgen -W " --" -- "${cur}") )
            fi
            ;;
        plan-emit-beads)
            # Plan-emit-beads subcommand completion
            if [[ ${cword} -eq 2 ]]; then
                COMPREPLY=( $(compgen -W "--dry-run --execute --force --target --help" -- "${cur}") )
            elif [[ ${cword} -eq 3 ]]; then
                # Session ID should be provided here
                _complete_intent_sessions
            elif [[ ${cword} -eq 4 && ${prev} == "--dry-run" ]]; then
                COMPREPLY=( $(compgen -W "true false --" -- "${cur}") )
            elif [[ ${cword} -eq 4 && ${prev} == "--execute" ]]; then
                COMPREPLY=( $(compgen -W "true false --" -- "${cur}") )
            elif [[ ${cword} -eq 4 && ${prev} == "--force" ]]; then
                COMPREPLY=( $(compgen -W "true false --" -- "${cur}") )
            elif [[ ${cword} -eq 4 && ${prev} == "--target" ]]; then
                COMPREPLY=( $(compgen -W "br --" -- "${cur}") )
            fi
            ;;
        vision|vis)
            # Vision command completion
            if [[ ${cword} -eq 2 ]]; then
                COMPREPLY=( $(compgen -W "--out -o --help" -- "${cur}") )
            fi
            ;;
        ready)
            # Ready command completion
            if [[ ${cword} -eq 2 ]]; then
                COMPREPLY=( $(compgen -W "--out -o --help" -- "${cur}") )
            fi
            ;;
        effects|eff)
            # Effects command completion
            if [[ ${cword} -eq 2 ]]; then
                COMPREPLY=( $(compgen -W "--behavior --json -j --help" -- "${cur}") )
            fi
            ;;
        *)
            # Default completion - complete flags and files
            COMPREPLY=( $(compgen -W "${flags[*]}" -- "${cur}") )
            COMPREPLY+=( $(compgen -f -G "*.cue" -- "${cur}") )
            ;;
    esac

    return 0
}

# Complete session IDs from .interview/sessions.jsonl
_complete_intent_sessions() {
    local session_file=".interview/sessions.jsonl"
    local session_ids=()

    # Check if the sessions file exists
    if [[ -f "${session_file}" ]]; then
        # Extract session IDs from JSONL file
        while IFS= read -r line; do
            if [[ -n "$line" ]]; then
                # Extract the session ID using jq if available, otherwise fallback to grep
                if command -v jq >/dev/null 2>&1; then
                    local sid=$(echo "$line" | jq -r '.id // empty')
                    if [[ -n "$sid" ]]; then
                        session_ids+=("$sid")
                    fi
                else
                    # Fallback to grep (less accurate)
                    local sid=$(echo "$line" | grep -o '"id":"[^"]*"' | cut -d'"' -f4)
                    if [[ -n "$sid" ]]; then
                        session_ids+=("$sid")
                    fi
                fi
            fi
        done < "${session_file}"
    fi

    # Provide completions
    COMPREPLY=( $(compgen -W "${session_ids[*]}" -- "${cur}") )
}

# Register the completion function
complete -F _intent_completion intent