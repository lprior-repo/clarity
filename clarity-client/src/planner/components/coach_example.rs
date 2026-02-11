//! Example usage of the PlanningCoach component

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use super::{build_thread, get_commands_for_step, is_phase_complete, TerminalCommand};

#[derive(Clone)]
struct MockCoachStep {
    id: &'static str,
    title: &'static str,
    question: &'static str,
    hint: &'static str,
    required: bool,
    follow_up: Option<&'static str>,
}

#[derive(Clone)]
struct MockCoachAnswer {
    step_id: String,
    value: String,
    timestamp: i64,
}

impl MockCoachStep {
    fn new(
        id: &'static str,
        title: &'static str,
        question: &'static str,
        hint: &'static str,
        required: bool,
        follow_up: Option<&'static str>,
    ) -> Self {
        Self {
            id,
            title,
            question,
            hint,
            required,
            follow_up,
        }
    }
}

impl MockCoachAnswer {
    fn new(step_id: String, value: String) -> Self {
        Self {
            step_id,
            value,
            timestamp: chrono::Utc::now().timestamp(),
        }
    }
}

#[test]
fn test_example_workflow() {
    // Simulate the Discovery phase workflow
    let steps = vec![
        MockCoachStep::new(
            "problem",
            "The Problem",
            "What specific problem are your users facing right now?",
            "Be concrete. Not 'auth is hard' but 'developers manually copy API tokens and manage expiry, causing production auth errors'.",
            true,
            Some("Good start. Now let's stress-test that -- why might this NOT actually be a problem worth solving?"),
        ),
        MockCoachStep::new(
            "antithesis",
            "The Antithesis",
            "Why might the current situation actually be fine? What's the strongest argument against building this?",
            "This is the null hypothesis. If you can't argue against yourself, you haven't thought hard enough.",
            true,
            Some("That tension is exactly what shapes a sharp product. Now -- what are you building to resolve it?"),
        ),
    ];

    let answers = vec![
        MockCoachAnswer::new("problem".to_string(), "Developers manually copy API tokens and forget to rotate them, causing production auth errors".to_string()),
        MockCoachAnswer::new("antithesis".to_string(), "Existing solutions like Cloudflare API tokens work well enough for most use cases".to_string()),
    ];

    // Test command generation
    let problem_commands = get_commands_for_step("problem", "Developers manually copy API tokens");
    assert_eq!(problem_commands.len(), 2);

    let antithesis_commands = get_commands_for_step("antithesis", "Existing solutions work well");
    assert_eq!(antithesis_commands.len(), 1);

    // Test phase completion
    assert!(is_phase_complete(&steps, &answers));

    println!("✅ Example workflow test passed");
}

#[test]
fn test_command_generation_patterns() {
    // Test use cases with multiple lines
    let use_case_commands = get_commands_for_step("use-cases", "User can login\nUser can reset password\nUser can view profile");

    assert_eq!(use_case_commands.len(), 3);

    // Check each command contains the right agent
    for cmd in &use_case_commands {
        assert_eq!(cmd.agent, "planner");
        assert!(cmd.cmd.contains("bd create --type feature"));
    }

    // Test tasks with module prefixes
    let task_commands = get_commands_for_step("tasks", "auth: implement login\nui: build dashboard");

    assert_eq!(task_commands.len(), 3); // 2 tasks + 1 ready command

    // Check that the commands include the module labels
    assert!(task_commands.iter().any(|cmd| cmd.cmd.contains("--label \"auth\"")));
    assert!(task_commands.iter().any(|cmd| cmd.cmd.contains("--label \"ui\"")));

    // Check the ready command
    assert!(task_commands.iter().any(|cmd| cmd.cmd == "bd ready --json"));

    println!("✅ Command generation patterns test passed");
}

#[test]
fn test_terminal_animation_timing() {
    // Test the timing logic that would be used for terminal animation
    let commands = vec![
        TerminalCommand {
            agent: "planner".to_string(),
            cmd: "bd init".to_string(),
            output: "Initialized".to_string(),
        },
        TerminalCommand {
            agent: "claude-code".to_string(),
            cmd: "bd create".to_string(),
            output: "Created".to_string(),
        },
    ];

    // Simulate the animation sequence
    let mut visible_count = 0;
    let mut sequence = Vec::new();

    while visible_count < commands.len() * 2 {
        let delay = if visible_count % 2 == 0 { 300 } else { 150 };
        sequence.push((visible_count, delay));
        visible_count += 1;
    }

    // Verify the sequence
    assert_eq!(sequence.len(), 4);

    // Check alternating delays
    assert_eq!(sequence[0].1, 300); // First command shows
    assert_eq!(sequence[1].1, 150); // First output shows
    assert_eq!(sequence[2].1, 300); // Second command shows
    assert_eq!(sequence[3].1, 150); // Second output shows

    println!("✅ Terminal animation timing test passed");
}

fn main() {
    test_example_workflow();
    test_command_generation_patterns();
    test_terminal_animation_timing();
    println!("🎉 All coach example tests passed!");
}