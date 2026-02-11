//! Simple example of PlanningCoach command generation

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

#[derive(Clone, Debug)]
struct TerminalCommand {
  pub agent: String,
  pub cmd: String,
  pub output: String,
}

fn get_commands_for_step(step_id: &str, value: &str) -> Vec<TerminalCommand> {
  let v = value.chars().take(60).collect::<String>();

  match step_id {
        "problem" => vec![
            TerminalCommand {
                agent: "planner".to_string(),
                cmd: format!("bd init --project beads-plan"),
                output: "Initialized .beads/ in current directory".to_string(),
            },
            TerminalCommand {
                agent: "planner".to_string(),
                cmd: format!("bd create --type epic --title \"Problem: {}...\"", v),
                output: "Created bd-a1f0  Problem Statement".to_string(),
            },
        ],
        "antithesis" => vec![
            TerminalCommand {
                agent: "planner".to_string(),
                cmd: format!("bd update bd-a1f0 --label antithesis --note \"{}...\"", v),
                output: "Updated bd-a1f0  +label:antithesis".to_string(),
            },
        ],
        "solution" => vec![
            TerminalCommand {
                agent: "planner".to_string(),
                cmd: format!("bd create --type epic --title \"Solution: {}...\"", v),
                output: "Created bd-b2e1  Solution".to_string(),
            },
            TerminalCommand {
                agent: "planner".to_string(),
                cmd: "bd dep add bd-b2e1 --blocks bd-a1f0 --type discovered-from".to_string(),
                output: "Linked bd-b2e1 -> bd-a1f0 (discovered-from)".to_string(),
            },
        ],
        "use-cases" => {
            let lines = value
                .lines()
                .map(|l| l.trim())
                .filter(|l| !l.is_empty())
                .collect::<Vec<_>>();

            lines
                .iter()
                .enumerate()
                .map(|(i, &uc)| TerminalCommand {
                    agent: "planner".to_string(),
                    cmd: format!("bd create --type feature --title \"{}...\"", uc.chars().take(55).collect::<String>()),
                    output: format!("Created bd-c{i}d{i}  {}...", uc.chars().take(30).collect::<String>()),
                })
                .collect()
        },
        "bd-show" => vec![
            TerminalCommand {
                agent: "planner".to_string(),
                cmd: "bd show --json".to_string(),
                output: r#"[{"id":"bd-a1f0","title":"Problem Statement","type":"epic","status":"completed","labels":["problem","core"]},{"id":"bd-b2e1","title":"Solution","type":"epic","status":"in-progress","labels":["solution","core"]},{"id":"bd-b2e1.1","title":"Persona definition","type":"task","status":"completed"},{"id":"bd-b2e1.2","title":"North Star Scenario","type":"task","status":"completed"}]"#.to_string(),
            },
            TerminalCommand {
                agent: "planner".to_string(),
                cmd: "bd show --tree".to_string(),
                output: "bd-a1f0 [epic] └─┬ bd-b2e1 [epic]\n                            ├─ bd-b2e1.1 [task]\n                            └─ bd-b2e1.2 [task]".to_string(),
            },
            TerminalCommand {
                agent: "planner".to_string(),
                cmd: "bd show --status".to_string(),
                output: "📊 Bead Status: 2 completed, 1 in-progress, 0 blocked".to_string(),
            },
        ],
        "tasks" => {
            let lines = value
                .lines()
                .map(|l| l.trim())
                .filter(|l| !l.is_empty())
                .collect::<Vec<_>>();

            let mut cmds = Vec::new();

            for (i, t) in lines.iter().enumerate() {
                let parts = t.split(':').collect::<Vec<_>>();
                let (mod_name, title) = if parts.len() > 1 {
                    (parts[0].trim().to_string(), parts[1..].join(":").trim().to_string())
                } else {
                    ("core".to_string(), t.to_string())
                };

                cmds.push(TerminalCommand {
                    agent: "claude-code".to_string(),
                    cmd: format!("bd create --type task --title \"{}\" --label \"{}\" --priority P2",
                                title.chars().take(50).collect::<String>(), mod_name),
                    output: format!("Created bd-d{i}e{i}  [{}] {}", mod_name, title.chars().take(25).collect::<String>()),
                });
            }

            cmds.push(TerminalCommand {
                agent: "claude-code".to_string(),
                cmd: "bd ready --json".to_string(),
                output: format!("[{} task(s) ready for execution]", lines.len()),
            });

            cmds
        },
        _ => Vec::new(),
    }
}

fn main() {
  println!("🧪 Testing PlanningCoach command generation...\n");

  // Test different step types
  let test_cases = vec![
    ("problem", "Developers forget to rotate API tokens"),
    ("antithesis", "Existing solutions work fine"),
    ("solution", "Automatically rotate API tokens at deploy time"),
    (
      "use-cases",
      "User can login\nUser can reset password\nUser can view profile",
    ),
    ("bd-show", ""),
    (
      "tasks",
      "auth: implement login endpoint\nui: build login form",
    ),
  ];

  for (step_id, input) in test_cases {
    println!("📋 Testing step: {} with input: '{}'", step_id, input);
    let commands = get_commands_for_step(step_id, input);

    println!("  Generated {} commands:", commands.len());
    for (i, cmd) in commands.iter().enumerate() {
      println!("    {}. {} $ {}", i + 1, cmd.agent, cmd.cmd);
      println!("       → {}", cmd.output);
    }
    println!();
  }

  println!("✅ All command generation tests completed!");
}
