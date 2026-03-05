#compdef intent
# Zsh completion script for Intent CLI

# Intent CLI completion
_intent_complete() {
    local context state line
    local -A opt_args

    # Store all possible commands (including aliases)
    local commands=(
        'interview:Run interactive interview session'
        'int:Alias for interview'
        'beads:Generate beads from completed interview session'
        'bead-status:Check the status of a bead in br'
        'history:List all interview sessions in chronological order'
        'hist:Alias for history'
        'diff:Show diff of changes for a specific session'
        'sessions:List interview sessions with optional filtering'
        'sess:Alias for sessions'
        'plan:Generate execution plan from current project context'
        'plan-next:Suggest the next task to work on based on strategy'
        'plan-approve:Approve a generated plan for execution'
        'plan-emit-beads:Emit beads from session to br (idempotent)'
        'beads-regenerate:Regenerate beads from session'
        'vision:Generate vision document from spec'
        'vis:Alias for vision'
        'ready:Generate ready document for implementation'
        'effects:Analyze behaviors for second-order effects'
        'eff:Alias for effects'
    )

    # Store all flags (including shortcuts)
    local flags=(
        '--profile:Profile type for interview questions'
        '-p:Shortcut for --profile'
        '--session:Session ID to generate beads from'
        '-s:Shortcut for --session'
        '--format:Output format for beads'
        '-f:Shortcut for --format'
        '--out:Output directory for generated beads'
        '-o:Shortcut for --out'
        '--output:Output directory (alternative to out)'
        '--resume:Resume a previous interview session by ID'
        '--answer:Answer to a specific question'
        '--bead-id:Bead ID to check'
        '--status:Status of a bead'
        '--reason:Reason for plan approval'
        '--notes:Additional context notes'
        '--strategy:Task selection strategy'
        '--name:Name of the plan'
        '--target:Target system for bead emission'
        '--feature:Feature to focus on'
        '--export-answers-template:Export answers template'
        '--vision:Generate vision document'
        '--json:Output as JSON'
        '-j:Shortcut for --json'
        '--verbose:Verbose output'
        '--quiet:Quiet output'
        '--yes:Automatic yes to prompts'
        '--draft:Generate draft version'
        '--confirm:Confirmation flag'
        '--dry-run:Show what would be created'
        '--execute:Actually create beads'
        '--force:Bypass idempotency checks'
        '--help:Show help'
        '--version:Show version'
    )

    # Store profile types
    local profiles=(
        'api:REST/GraphQL APIs (endpoints, data models, authentication)'
        'cli:Command-line tools (commands, arguments, exit codes)'
        'event:Event-driven systems (events, handlers, subscriptions)'
        'data:Data pipelines (sources, transformations, destinations)'
        'workflow:Business workflows (states, transitions, actors)'
        'ui:User interfaces (components, interactions, states)'
    )

    # Store output formats
    local formats=(
        'json:JSON array with all beads (default)'
        'jsonl:JSON Lines (one JSON object per line)'
        'markdown:Human-readable formatted list'
    )

    # Store strategies
    local strategies=(
        'page_rank:Most connected/influential task (default)'
        'critical_path:Task affecting overall timeline'
        'shortest:Quickest task to complete'
        'risk_first:Highest risk task first'
    )

    # First level completion - main commands
    if (( CURRENT == 2 )); then
        _describe 'intent commands' commands
        _alternative \
            'flags:flags:flags:compadd - $flags'
        return
    fi

    # Get the command
    local cmd="${words[2]}"

    # Complete based on the command
    case "$cmd" in
        interview|int)
            _arguments \
                '--profile[Profile type for interview questions]:profiles:_intent_profiles' \
                '-p[Profile type for interview questions]:profiles:_intent_profiles' \
                '--resume[Resume a previous interview session by ID]:session ID:_intent_sessions' \
                '--help[Show help]' \
                '--version[Show version]'
            ;;
        beads|beads-regenerate)
            _arguments \
                '--session[Session ID to generate beads from]:session ID:_intent_sessions' \
                '-s[Session ID to generate beads from]:session ID:_intent_sessions' \
                '--format[Output format for beads]:format:_intent_formats' \
                '-f[Output format for beads]:format:_intent_formats' \
                '--out[Output directory for generated beads]:directory:_files -/' \
                '-o[Output directory for generated beads]:directory:_files -/' \
                '--output[Output directory for generated beads]:directory:_files -/' \
                '--help[Show help]' \
                '--version[Show version]'
            ;;
        bead-status)
            _arguments \
                '--bead-id[Bead ID to check]:bead ID:' \
                '--help[Show help]' \
                '--version[Show version]'
            ;;
        diff)
            _arguments \
                '--session[Session ID to show diff for]:session ID:_intent_sessions' \
                '-s[Session ID to show diff for]:session ID:_intent_sessions' \
                '--help[Show help]' \
                '--version[Show version]'
            ;;
        sessions|sess)
            _arguments \
                '--profile[Filter by profile type]:profiles:_intent_profiles' \
                '-p[Filter by profile type]:profiles:_intent_profiles' \
                '--help[Show help]' \
                '--version[Show version]'
            ;;
        plan)
            _arguments \
                '--notes[Additional context notes for plan generation]:notes:' \
                '--help[Show help]' \
                '--version[Show version]'
            ;;
        plan-next)
            _arguments \
                '--strategy[Task selection strategy]:strategies:_intent_strategies' \
                '--help[Show help]' \
                '--version[Show version]'
            ;;
        plan-approve)
            _arguments \
                '1:Plan ID:' \
                '--notes[Approval notes for documentation]:notes:' \
                '--help[Show help]' \
                '--version[Show version]'
            ;;
        plan-emit-beads)
            _arguments \
                '1:Session ID:_intent_sessions' \
                '--dry-run[Show what would be created without creating beads]:boolean:(true false)' \
                '--execute[Actually create beads in br]:boolean:(true false)' \
                '--force[Bypass idempotency checks and create all beads]:boolean:(true false)' \
                '--target[Target system for bead emission]:target:(br)' \
                '--help[Show help]' \
                '--version[Show version]'
            ;;
        vision|vis)
            _arguments \
                '1:Spec file:_files -g "*.cue"' \
                '--out[Output directory for generated document]:directory:_files -/' \
                '-o[Output directory for generated document]:directory:_files -/' \
                '--output[Output directory for generated document]:directory:_files -/' \
                '--help[Show help]' \
                '--version[Show version]'
            ;;
        ready)
            _arguments \
                '1:Spec file:_files -g "*.cue"' \
                '--out[Output directory for generated document]:directory:_files -/' \
                '-o[Output directory for generated document]:directory:_files -/' \
                '--output[Output directory for generated document]:directory:_files -/' \
                '--help[Show help]' \
                '--version[Show version]'
            ;;
        effects|eff)
            _arguments \
                '1:Spec file:_files -g "*.cue"' \
                '--behavior[Analyze specific behavior only]:behavior:' \
                '--json[Output as JSON]:boolean:(true false)' \
                '-j[Output as JSON]:boolean:(true false)' \
                '--help[Show help]' \
                '--version[Show version]'
            ;;
        *)
            _alternative \
                'commands:commands:compadd - $commands' \
                'flags:flags:compadd - $flags'
            ;;
    esac
}

# Helper functions
_intent_sessions() {
    local session_file=".interview/sessions.jsonl"
    local -a session_ids

    # Check if the sessions file exists
    if [[ -f "${session_file}" ]]; then
        # Extract session IDs from JSONL file
        while IFS= read -r line; do
            if [[ -n "$line" ]]; then
                # Extract the session ID
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

    # Describe the session IDs
    _describe 'session IDs' session_ids
}

_intent_profiles() {
    local -a profile_descriptions
    profile_descriptions=(
        'api:REST/GraphQL APIs (endpoints, data models, authentication)'
        'cli:Command-line tools (commands, arguments, exit codes)'
        'event:Event-driven systems (events, handlers, subscriptions)'
        'data:Data pipelines (sources, transformations, destinations)'
        'workflow:Business workflows (states, transitions, actors)'
        'ui:User interfaces (components, interactions, states)'
    )
    _describe 'profiles' profile_descriptions
}

_intent_formats() {
    local -a format_descriptions
    format_descriptions=(
        'json:JSON array with all beads (default)'
        'jsonl:JSON Lines (one JSON object per line)'
        'markdown:Human-readable formatted list'
    )
    _describe 'formats' format_descriptions
}

_intent_strategies() {
    local -a strategy_descriptions
    strategy_descriptions=(
        'page_rank:Most connected/influential task (default)'
        'critical_path:Task affecting overall timeline'
        'shortest:Quickest task to complete'
        'risk_first:Highest risk task first'
    )
    _describe 'strategies' strategy_descriptions
}

# Register the completion function
_intent_complete "$@"