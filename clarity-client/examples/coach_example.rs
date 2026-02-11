//! Example usage of the PlanningCoach component

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use clarity_client::planner::components::coach::{
  build_thread, get_commands_for_step, is_phase_complete, TerminalCommand,
};

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

fn main() {
  println!("🧪 Testing PlanningCoach functionality...\n");

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
  println!("📋 Testing command generation...");
  let problem_commands = get_commands_for_step("problem", "Developers manually copy API tokens");
  assert_eq!(problem_commands.len(), 2);
  println!("✅ Problem generates {} commands", problem_commands.len());

  let antithesis_commands = get_commands_for_step("antithesis", "Existing solutions work well");
  assert_eq!(antithesis_commands.len(), 1);
  println!(
    "✅ Antithesis generates {} commands",
    antithesis_commands.len()
  );

  // Test phase completion
  println!("\n🎯 Testing phase completion...");
  assert!(is_phase_complete(&steps, &answers));
  println!("✅ Phase is complete");

  // Test use cases with multiple lines
  println!("\n📝 Testing use case command generation...");
  let use_case_commands = get_commands_for_step(
    "use-cases",
    "User can login\nUser can reset password\nUser can view profile",
  );
  assert_eq!(use_case_commands.len(), 3);
  println!("✅ Use cases generate {} commands", use_case_commands.len());

  // Test tasks with module prefixes
  println!("\n🛠️ Testing task command generation...");
  let task_commands = get_commands_for_step("tasks", "auth: implement login\nui: build dashboard");
  assert_eq!(task_commands.len(), 3);
  println!("✅ Tasks generate {} commands", task_commands.len());

  // Test bd-show command
  println!("\n📊 Testing bd-show command generation...");
  let bd_show_commands = get_commands_for_step("bd-show", "");
  assert_eq!(bd_show_commands.len(), 3);
  println!("✅ BD-show generates {} commands", bd_show_commands.len());

  // Test constraints command
  println!("\n⚙️ Testing constraints command generation...");
  let constraint_commands = get_commands_for_step(
    "constraints",
    "Next.js 16, TypeScript strict, Supabase for auth",
  );
  assert_eq!(constraint_commands.len(), 1);
  println!(
    "✅ Constraints generate {} commands",
    constraint_commands.len()
  );

  println!("\n🎉 All tests passed! The PlanningCoach component is working correctly.");

  // Print some sample commands for demonstration
  println!("\n📜 Sample commands generated:");
  for cmd in &problem_commands {
    println!("  {} $ {}", cmd.agent, cmd.cmd);
    println!("    → {}", cmd.output);
  }
}
