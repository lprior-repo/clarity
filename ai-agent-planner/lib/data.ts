import type { PlanSession } from "./types"

export const DEMO_SESSION: PlanSession = {
  id: "user-auth-20260210",
  status: "discover",
  createdAt: "2026-02-10T12:00:00Z",

  // ── Discover ─────────────────────────────
  thesis: {
    problem:
      "Developers using the CLI tool must manually copy-paste API tokens from a web dashboard and manage expiry themselves, leading to auth errors and wasted time.",
    solution:
      "Add a native login command that handles credentials, stores tokens in the system keyring, and automatically refreshes them -- so developers never think about auth again.",
    antithesis:
      "Developers may prefer managing tokens manually for security control, or they may use environment variables in CI/CD where interactive login is impossible.",
  },

  personas: [
    {
      name: "Solo Dev",
      description:
        "A full-stack developer working on personal projects. Uses the CLI daily from a single machine. Values speed and simplicity over configurability.",
      means: "Comfortable with terminal, has admin access to their machine, uses macOS or Linux.",
    },
    {
      name: "Platform Engineer",
      description:
        "Manages CI/CD pipelines for a team. Needs non-interactive auth for automation. Cares about audit trails and token rotation policies.",
      means: "Deep ops knowledge, manages secrets via Vault/SSM, runs headless environments.",
    },
    {
      name: "New Hire",
      description: "Just joined the team, unfamiliar with the tool. Needs clear error messages and guided setup.",
      means: "Basic terminal skills, follows documentation step-by-step, may use Windows.",
      isNonpersona: true,
    },
  ],

  scenarios: [
    {
      title: "First-time login flow",
      persona: "Solo Dev",
      story:
        "Alex opens terminal to deploy a new feature. They run `intent deploy` but get 'Not authenticated'. They run `intent login`, enter their email and password at the prompts, and see 'Logged in as alex@dev.io'. From now on, every command works without thinking about auth. Two weeks later the token expires silently, and the next command refreshes it automatically.",
    },
    {
      title: "CI pipeline authentication",
      persona: "Platform Engineer",
      story:
        "Jordan sets up a GitHub Actions workflow. They set INTENT_TOKEN as a repository secret. The CLI detects the env var and skips interactive login. When the token nears expiry, the pipeline uses a service account refresh token to rotate it. Jordan sees audit logs showing which pipeline used which token.",
    },
  ],

  // ── Define ──────────────────────────────
  useCases: [
    { id: "uc-1", persona: "Solo Dev", action: "log in with email and password", motivation: "they can use CLI commands without manual token management", northStar: "First-time login flow", priority: "must" },
    { id: "uc-2", persona: "Solo Dev", action: "have tokens refresh automatically", motivation: "they never encounter expired-token errors", northStar: "First-time login flow", priority: "must" },
    { id: "uc-3", persona: "Solo Dev", action: "run `intent whoami`", motivation: "they can verify which account is active", northStar: "First-time login flow", priority: "should" },
    { id: "uc-4", persona: "Solo Dev", action: "log out", motivation: "they can switch accounts or revoke access on a shared machine", northStar: "First-time login flow", priority: "should" },
    { id: "uc-5", persona: "Platform Engineer", action: "authenticate via environment variable", motivation: "CI/CD pipelines work without interactive prompts", northStar: "CI pipeline authentication", priority: "must" },
    { id: "uc-6", persona: "Platform Engineer", action: "see audit logs of token usage", motivation: "they can trace which pipeline used which credential", northStar: "CI pipeline authentication", priority: "could" },
  ],

  context:
    "Existing codebase uses Rust with clap for CLI. API is REST. Need to follow exit code conventions in AGENTS.md. Keyring crate available for secure storage.",

  // ── Develop ─────────────────────────────
  tasks: [
    {
      id: "task-001",
      title: "auth: Implement login command with JWT retrieval",
      type: "feature",
      priority: 1,
      effort: "2hr",
      description:
        "Implement CLI command that accepts username/password via interactive prompt, calls auth API, and stores JWT token in system keyring.",
      dependsOn: [],
      ears: {
        ubiquitous: [
          { text: "THE SYSTEM SHALL encrypt credentials in transit using TLS" },
          { text: "THE SYSTEM SHALL validate input before sending to API" },
          { text: "THE SYSTEM SHALL log authentication attempts without exposing secrets" },
        ],
        eventDriven: [
          { trigger: "WHEN user runs 'intent login' with valid credentials", response: "THE SYSTEM SHALL retrieve JWT and store in keyring" },
          { trigger: "WHEN user provides --help flag", response: "THE SYSTEM SHALL display usage and exit 0" },
        ],
        unwanted: [
          { condition: "IF password provided via CLI argument", shallNot: "THE SYSTEM SHALL NOT accept it", because: "args visible in process lists" },
          { condition: "IF API returns 500", shallNot: "THE SYSTEM SHALL NOT retry infinitely", because: "causes DOS and CLI hang" },
        ],
      },
      contracts: {
        preconditions: ["Network connectivity to API endpoint", "Config file exists at ~/.intent/config.toml"],
        postconditions: ["JWT token stored in keyring under key 'intent-token'", "Exit code 0 on success", "Exit code 3 on invalid credentials"],
        invariants: ["Password never appears in logs or stdout", "All timestamps use ISO8601 format"],
      },
      tests: {
        happy: ["Login with valid credentials stores token and exits 0", "Login prompts for password interactively"],
        error: ["Invalid credentials returns exit 3", "Network timeout returns exit 5", "Keyring access denied returns exit 4"],
        edge: ["Empty username prompts for input", "Very long password (>10k chars) rejected gracefully"],
      },
      research: {
        files: ["src/commands/mod.rs", "docs/AGENTS.md", "Cargo.toml"],
        patterns: ["How do other commands handle API errors?", "Standard structure for CLI subcommands"],
        questions: ["Should we reuse existing HTTP client?", "Token expiry policy?"],
      },
      implementation: {
        phase0: ["Read src/commands/mod.rs for patterns", "Review API auth docs"],
        phase1: ["Write test_login_valid_credentials", "Write test_login_invalid_returns_error"],
        phase2: ["Implement login_command in src/commands/login.rs", "Wire to CLI parser in src/main.rs"],
      },
    },
    {
      id: "task-002",
      title: "auth: Implement logout command",
      type: "feature",
      priority: 2,
      effort: "1hr",
      description: "Implement a logout command that clears stored JWT from the system keyring and confirms to the user.",
      dependsOn: ["task-001"],
      ears: {
        ubiquitous: [{ text: "THE SYSTEM SHALL confirm successful logout to the user" }],
        eventDriven: [{ trigger: "WHEN user runs 'intent logout'", response: "THE SYSTEM SHALL remove token from keyring and exit 0" }],
        unwanted: [{ condition: "IF no token exists in keyring", shallNot: "THE SYSTEM SHALL NOT return an error", because: "logout should be idempotent" }],
      },
      contracts: {
        preconditions: ["Binary installed at ~/.local/bin/intent"],
        postconditions: ["Token removed from keyring", "Exit code 0"],
        invariants: ["No sensitive data in stdout"],
      },
      tests: {
        happy: ["Logout removes token and prints confirmation"],
        error: ["Logout when not logged in exits 0 gracefully"],
        edge: [],
      },
      research: { files: ["src/commands/login.rs"], patterns: ["How does login store the token?"], questions: [] },
      implementation: {
        phase0: ["Read login command to understand keyring usage"],
        phase1: ["Write test_logout_removes_token"],
        phase2: ["Implement logout_command in src/commands/logout.rs"],
      },
    },
    {
      id: "task-003",
      title: "auth: Add token refresh logic",
      type: "feature",
      priority: 1,
      effort: "2hr",
      description: "Add automatic token refresh using refresh tokens. When an access token expires, silently obtain a new one.",
      dependsOn: ["task-001"],
      ears: {
        ubiquitous: [
          { text: "THE SYSTEM SHALL refresh tokens transparently to the user" },
          { text: "THE SYSTEM SHALL store new token pair after refresh" },
        ],
        eventDriven: [
          { trigger: "WHEN access token expires during API call", response: "THE SYSTEM SHALL use refresh token to obtain new pair" },
          { trigger: "WHEN refresh token is also expired", response: "THE SYSTEM SHALL prompt user to login again" },
        ],
        unwanted: [{ condition: "IF refresh fails due to revoked token", shallNot: "THE SYSTEM SHALL NOT loop retrying", because: "token is permanently invalid" }],
      },
      contracts: {
        preconditions: ["Valid refresh token exists in keyring", "Network connectivity"],
        postconditions: ["New access token stored in keyring", "Original request retried and completed"],
        invariants: ["At most one refresh attempt per request cycle", "Old token pair invalidated after refresh"],
      },
      tests: {
        happy: ["Expired access token triggers transparent refresh", "Refreshed token is used for retry"],
        error: ["Expired refresh token prompts re-login with exit 3", "Network error during refresh returns exit 5"],
        edge: ["Concurrent requests share a single refresh cycle"],
      },
      research: {
        files: ["src/commands/login.rs", "src/http/client.rs"],
        patterns: ["How is the HTTP client structured for middleware?"],
        questions: ["Should refresh be an HTTP middleware or per-command?"],
      },
      implementation: {
        phase0: ["Understand HTTP client architecture", "Review refresh token API endpoint"],
        phase1: ["Write test_refresh_on_expired_access", "Write test_relogin_on_expired_refresh"],
        phase2: ["Add refresh interceptor to HTTP client", "Update keyring to store refresh token"],
      },
    },
    {
      id: "task-004",
      title: "auth: Add whoami command",
      type: "feature",
      priority: 3,
      effort: "30min",
      description: "A simple command that decodes the stored JWT and displays the user's identity information.",
      dependsOn: ["task-001"],
      ears: {
        ubiquitous: [{ text: "THE SYSTEM SHALL decode JWT without verifying signature locally" }],
        eventDriven: [{ trigger: "WHEN user runs 'intent whoami'", response: "THE SYSTEM SHALL display username and token expiry" }],
        unwanted: [{ condition: "IF token is malformed", shallNot: "THE SYSTEM SHALL NOT crash", because: "show helpful error instead" }],
      },
      contracts: {
        preconditions: ["Token exists in keyring"],
        postconditions: ["Username and expiry printed to stdout", "Exit code 0"],
        invariants: ["Token value never printed to stdout"],
      },
      tests: {
        happy: ["Whoami shows username from valid token"],
        error: ["Whoami without login prints 'Not logged in' and exits 3"],
        edge: ["Whoami with corrupt token shows 'Invalid token'"],
      },
      research: { files: ["src/commands/login.rs"], patterns: ["JWT decoding without verification"], questions: [] },
      implementation: {
        phase0: [],
        phase1: ["Write test_whoami_shows_username"],
        phase2: ["Implement whoami_command in src/commands/whoami.rs"],
      },
    },
  ],
}
