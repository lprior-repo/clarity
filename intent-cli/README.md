# Intent CLI

**AI-powered planning and bead generation tool v0.1.0**

Intent transforms vague requirements into crystal-clear, atomic work items through interactive interviews and semantic validation.

## The Vision

```
Human writes requirements → CLI interviews systematically → CUE schemas validate → Beads track work → br manages execution
```

**CUE is the center of the universe.** Everything flows through typed, validated CUE schemas:
- Requirements are captured through interviews
- Specifications are validated as CUE
- Behaviors are documented declaratively
- Beads (work items) are generated from specifications
- Work is tracked via br (beads_rust)

## How It Works

### 1. Interactive Interview

Start an interview to capture requirements systematically:

```bash
gleam run -- interview --profile api
```

The CLI guides you through structured questions about:
- System overview and goals
- User roles and actors
- Features and behaviors
- Success criteria
- Constraints and anti-patterns

### 2. Specification Generation

After the interview, Intent generates a structured CUE specification:

```cue
spec: {
    name: "User Management API"
    description: "API for user authentication and profile management"
    audience: "Mobile and web clients"
    version: "1.0.0"
    success_criteria: [
        "Users can register and login",
        "Passwords never exposed in responses",
    ]
    features: [...]
    invariants: [...]
    anti_patterns: [...]
}
```

### 3. Bead Generation

Convert specifications into atomic work items (beads):

```bash
gleam run -- beads --session <session-id> --format json
```

Each behavior becomes a bead with:
- ID and title
- Description and acceptance criteria
- Dependencies (blocks/blocked_by)
- Priority and tags

### 4. Work Management

Emit beads to br (beads_rust) for tracking:

```bash
gleam run -- plan-emit-beads <session-id> --execute
```

## Key Concepts

### Declarative Behaviors

Behaviors describe what a system does, not how to test it:

```cue
{
    name: "successful-login"
    intent: "User can authenticate with valid credentials"
    preconditions: [
        "User account exists",
        "User provides valid credentials",
    ]
    postconditions: [
        "User is authenticated",
        "JWT token is issued",
    ]
    verifications: [{
        description: "Authentication succeeds"
        criteria: [
            "Valid credentials return 200 with JWT",
            "Invalid credentials return 401",
        ]
    }]
}
```

### Invariants

Global invariants apply to all behaviors:

```cue
invariants: [{
    name: "no-password-exposure"
    description: "Passwords never appear in responses"
    criteria: [
        "Password field absent from all user responses",
        "Password hash never returned",
    ]
}]
```

### Anti-Patterns

Document common mistakes to avoid:

```cue
anti_patterns: [{
    name: "missing-timestamps"
    description: "Responses should include created_at and updated_at"
    bad_example: { id: "123", name: "Item" }
    good_example: {
        id: "123"
        name: "Item"
        created_at: "2024-01-04T12:00:00Z"
        updated_at: "2024-01-04T12:00:00Z"
    }
    why: "Timestamps are essential for auditing and debugging"
}]
```

## Quick Start

### 1. Start an Interview

```bash
# Start with the API profile
gleam run -- interview --profile api

# Resume a previous session
gleam run -- interview --resume <session-id>
```

### 2. Generate Beads

```bash
# List all sessions
gleam run -- history

# Generate beads from a session
gleam run -- beads --session <session-id>
```

### 3. Emit to br

```bash
# Preview beads before emitting
gleam run -- plan-emit-beads <session-id>

# Actually create beads in br
gleam run -- plan-emit-beads <session-id> --execute
```

### 4. Analyze Effects

```bash
# Analyze second-order effects
gleam run -- effects examples/spec.cue

# Analyze specific behavior
gleam run -- effects examples/spec.cue --behavior <name>
```

## Commands

```bash
# Interview
gleam run -- interview --profile <profile>           # Start interview
gleam run -- interview --resume <session-id>         # Resume session
gleam run -- history                                 # List sessions
gleam run -- diff --session <session-id>             # Show session diff
gleam run -- sessions --profile <profile>            # Filter sessions by profile

# Beads
gleam run -- beads --session <session-id>            # Generate beads
gleam run -- beads-regenerate --session <session-id> # Regenerate beads
gleam run -- bead-status --bead-id <id>              # Check bead status

# Planning
gleam run -- plan                                    # Generate plan
gleam run -- plan-next                               # Get next task suggestion
gleam run -- plan-approve <plan-id>                  # Approve a plan
gleam run -- plan-emit-beads <session-id>            # Emit beads to br
gleam run -- plan-emit-beads <session-id> --execute  # Actually create beads

# Analysis
gleam run -- effects <spec.cue>                      # Analyze effects
gleam run -- effects <spec.cue> --behavior <name>    # Analyze specific behavior

# Documentation
gleam run -- vision --out <dir>                      # Generate vision document
gleam run -- ready --out <dir>                       # Generate ready document
```

## Command Aliases and Flag Shortcuts

Intent CLI provides convenient aliases for common commands and shortcuts for frequently used flags to improve your workflow efficiency.

### Command Aliases

| Alias    | Full Command   | Description                        |
|----------|---------------|------------------------------------|
| `int`    | `interview`   | Start interactive interview        |
| `hist`   | `history`     | List all interview sessions        |
| `sess`   | `sessions`    | List sessions with filtering       |
| `eff`    | `effects`     | Analyze second-order effects       |
| `vis`    | `vision`      | Generate vision document           |

**Examples:**

```bash
# Using aliases
gleam run -- int -p api                    # Same as: gleam run -- interview --profile api
gleam run -- hist                          # Same as: gleam run -- history
gleam run -- sess -p api                   # Same as: gleam run -- sessions --profile api
gleam run -- eff examples/spec.cue         # Same as: gleam run -- effects examples/spec.cue
gleam run -- vis examples/spec.cue         # Same as: gleam run -- vision examples/spec.cue
```

### Flag Shortcuts

| Shortcut | Full Flag     | Description                        |
|----------|--------------|------------------------------------|
| `-p`     | `--profile`   | Profile type for interview         |
| `-s`     | `--session`   | Session ID                         |
| `-f`     | `--format`    | Output format                      |
| `-o`     | `--out`       | Output directory                   |
| `-j`     | `--json`      | JSON output                        |

**Examples:**

```bash
# Using flag shortcuts
gleam run -- interview -p api              # Same as: gleam run -- interview --profile api
gleam run -- beads -s abc123 -f json      # Same as: gleam run -- beads --session abc123 --format json
gleam run -- effects examples/spec.cue -j  # Same as: gleam run -- effects examples/spec.cue --json
gleam run -- vision spec.cue -o docs/      # Same as: gleam run -- vision spec.cue --out docs/
```

### Combining Aliases and Shortcuts

You can combine command aliases with flag shortcuts for maximum efficiency:

```bash
# Full command: gleam run -- interview --profile api --session abc123
gleam run -- int -p api -s abc123

# Full command: gleam run -- beads --session abc123 --format json --out output/
gleam run -- beads -s abc123 -f json -o output/

# Full command: gleam run -- effects examples/spec.cue --json
gleam run -- eff examples/spec.cue -j
```

### Shell Completion

The bash and zsh completion scripts include support for all aliases and shortcuts. Tab completion works seamlessly with both full commands/flags and their aliases/shortcuts.

## Installation

```bash
# Clone the repository
git clone https://github.com/your-org/intent-cli.git
cd intent-cli

# Build from source
gleam build

# Run directly
gleam run -- interview --profile api

# Or add to PATH
export PATH="$PATH:$(pwd)/build/dev/erlang/<app>/bin"
```

## Shell Completion

Intent CLI provides shell completion for bash and zsh to improve usability with tab completion.

### Bash Completion

1. Install the completion script:

```bash
# Copy the completion script to your completions directory
mkdir -p ~/.bash/completions
cp completions/intent.bash ~/.bash/completions/intent

# Add to your ~/.bashrc or ~/.bash_profile
echo 'source ~/.bash/completions/intent' >> ~/.bashrc

# Reload your shell
source ~/.bashrc
```

### Zsh Completion

1. Install the completion script:

```bash
# Copy the completion script to your fpath
mkdir -p ~/.zsh/completions
cp completions/intent.zsh ~/.zsh/completions/_intent

# Add to your ~/.zshrc
echo 'fpath=(~/.zsh/completions $fpath)' >> ~/.zshrc
echo 'autoload -U compinit && compinit' >> ~/.zshrc

# Reload your shell
source ~/.zshrc
```

### Features

Completion supports:
- **Commands**: All Intent CLI commands and aliases (int/interview, hist/history, sess/sessions, eff/effects, vis/vision)
- **Subcommands**: Plan subcommands (plan, plan-next, plan-approve, plan-emit-beads)
- **Flags**: All available CLI flags with descriptions and shortcuts (-p/--profile, -s/--session, -f/--format, -o/--out, -j/--json)
- **Profile types**: api, cli, event, data, workflow, ui
- **Output formats**: json, jsonl, markdown
- **Session IDs**: Automatically extracts from `.interview/sessions.jsonl`
- **Strategies**: page_rank, critical_path, shortest, risk_first
- **File completion**: CUE files for commands that take spec files

### Manual Installation

If you prefer manual installation, you can also:

```bash
# For bash
eval "$(_INTENT_COMPLETE=bash source completions/intent.bash)"

# For zsh
eval "$(_INTENT_COMPLETE=zsh source completions/intent.zsh)"
```

## Profiles

Available interview profiles:

- `api` - REST API design and behavior specification
- `cli` - Command-line interface design
- `event` - Event-driven architecture
- `data` - Data pipelines and ETL
- `workflow` - Business workflow automation
- `ui` - User interface design

## Project Structure

```
src/intent/
├── interview.gleam        # Interview engine
├── interview_storage.gleam # Session management
├── bead_templates.gleam   # Bead generation
├── plan_mode.gleam        # Plan generation
├── plan_emit_beads.gleam  # Emit beads to br
├── effects_analyzer.gleam # Second-order effects
├── quality_analyzer.gleam # Spec quality analysis
├── semantic_validator.gleam # Semantic validation
└── types.gleam            # Core type definitions

schema/
├── intent.cue             # Core spec schema
├── questions.cue          # Interview questions
└── ai_protocol.cue        # AI planning directives

docs/
├── SPEC_FORMAT.md         # Specification format reference
├── USER_GUIDE.md          # Comprehensive user guide
├── schema-spec-type.md    # Spec type documentation
├── schema-behavior-type.md # Behavior type documentation
├── schema-invariant-type.md # Invariant type documentation
└── schema-verification-type.md # Verification type documentation
```

## Project Structure

```
src/intent/
├── interview.gleam        # Interview engine (722 lines)
├── bead_templates.gleam   # Bead generation
├── kirk/
│   ├── ears_parser.gleam      # EARS → behaviors
│   ├── quality_analyzer.gleam # 5-dimension scoring
│   ├── inversion_checker.gleam # What could fail?
│   └── coverage_analyzer.gleam # Test coverage
└── ...

schema/
├── questions.cue          # Interview questions database
├── ai_protocol.cue        # AI planning directive schemas
├── kirk.cue              # KIRK contract types
└── intent.cue            # Core spec schema

docs/
├── MENTAL_LATTICE_FRAMEWORK.md   # Theory
├── EARS_KIRK_WORKFLOW.md         # Workflow
└── INTERACTIVE_QUESTIONING.md    # Question system
```

## The Goal

> By the time a bead reaches implementation, every possible question has been answered, every edge case has been enumerated, and the work is clearly defined.

**This is systematic planning with AI assistance.**

## Documentation

- [User Guide](docs/USER_GUIDE.md) - Comprehensive usage guide
- [Spec Format](docs/SPEC_FORMAT.md) - Specification format reference
- [Migration Guide](MIGRATION.md) - Migrating from v2.0 to v3.0
- [CHANGELOG](CHANGELOG.md) - Version history and changes

## Examples

See the `examples/` directory for complete specifications:

**v3.0 Declarative Format (Current)**:
- `declarative-spec.cue` - User authentication with declarative format
- `interview-workflow.cue` - Interview workflow examples

**v2.0 HTTP Format (Legacy - Reference Only)**:
- `user-api.cue` - User management API (legacy format)
- `pokemon-api.cue` - Pokemon API (legacy format)
- `meal-planner-api.cue` - Meal planning API (legacy format)

**Note**: Most examples use the v2.0 HTTP format and are kept for reference. Use `declarative-spec.cue` as the template for new specifications. See [MIGRATION.md](MIGRATION.md) for details on the differences.

## License

MIT
