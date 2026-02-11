//! Tests for the PlanningCoach component

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use super::{build_thread, get_commands_for_step, is_phase_complete, TerminalCommand};
use crate::planner::types_coach::{CoachAnswer, CoachStep};
use crate::planner::types::DiamondPhase;

#[test]
fn test_get_commands_for_step_problem() {
    let commands = get_commands_for_step("problem", "Developers forget to rotate API tokens");

    assert_eq!(commands.len(), 2);

    assert_eq!(commands[0].agent, "planner");
    assert_eq!(commands[0].cmd, "bd init --project beads-plan");
    assert_eq!(commands[0].output, "Initialized .beads/ in current directory");

    assert_eq!(commands[1].agent, "planner");
    assert!(commands[1].cmd.contains("Problem: Developers forget to rotate API tokens"));
    assert_eq!(commands[1].output, "Created bd-a1f0  Problem Statement");
}

#[test]
fn test_get_commands_for_step_antithesis() {
    let commands = get_commands_for_step("antithesis", "Existing solutions work fine");

    assert_eq!(commands.len(), 1);

    assert_eq!(commands[0].agent, "planner");
    assert!(commands[0].cmd.contains("antithesis --note"));
    assert!(commands[0].cmd.contains("Existing solutions work fine"));
    assert_eq!(commands[0].output, "Updated bd-a1f0  +label:antithesis");
}

#[test]
fn test_get_commands_for_step_solution() {
    let commands = get_commands_for_step("solution", "Automatically rotate API tokens at deploy time");

    assert_eq!(commands.len(), 2);

    assert_eq!(commands[0].agent, "planner");
    assert!(commands[0].cmd.contains("Solution: Automatically rotate API tokens"));
    assert_eq!(commands[0].output, "Created bd-b2e1  Solution");

    assert_eq!(commands[1].agent, "planner");
    assert_eq!(commands[1].cmd, "bd dep add bd-b2e1 --blocks bd-a1f0 --type discovered-from");
    assert_eq!(commands[1].output, "Linked bd-b2e1 -> bd-a1f0 (discovered-from)");
}

#[test]
fn test_get_commands_for_step_unknown() {
    let commands = get_commands_for_step("unknown-step", "some value");

    assert_eq!(commands.len(), 0);
}

#[test]
fn test_is_phase_complete_no_required() {
    let steps = vec![
        CoachStep::new("step1", DiamondPhase::Top, "Title", "Q", "H", false, None),
        CoachStep::new("step2", DiamondPhase::Top, "Title", "Q", "H", false, None),
    ];
    let answers = vec![];

    assert!(is_phase_complete(&steps, &answers));
}

#[test]
fn test_is_phase_complete_with_required() {
    let steps = vec![
        CoachStep::new("step1", DiamondPhase::Top, "Title", "Q", "H", true, None),
        CoachStep::new("step2", DiamondPhase::Top, "Title", "Q", "H", true, None),
    ];
    let answers = vec![
        CoachAnswer::new("step1".into(), "answer1".into()),
        CoachAnswer::new("step2".into(), "answer2".into()),
    ];

    assert!(is_phase_complete(&steps, &answers));
}

#[test]
fn test_is_phase_complete_incomplete() {
    let steps = vec![
        CoachStep::new("step1", DiamondPhase::Top, "Title", "Q", "H", true, None),
        CoachStep::new("step2", DiamondPhase::Top, "Title", "Q", "H", true, None),
    ];
    let answers = vec![
        CoachAnswer::new("step1".into(), "answer1".into()),
    ];

    assert!(!is_phase_complete(&steps, &answers));
}

#[test]
fn test_build_thread() {
    let steps = vec![
        CoachStep::new("step1", DiamondPhase::Top, "Title1", "Question1", "Hint1", true, Some("Follow1")),
        CoachStep::new("step2", DiamondPhase::Top, "Title2", "Question2", "Hint2", false, None),
    ];
    let answers = vec![
        CoachAnswer::new("step1".into(), "Answer1".into()),
    ];

    let thread = build_thread(&steps, &answers);

    assert_eq!(thread.len(), 3);

    match &thread[0] {
        super::ThreadEntry::Coach { content, step_title } => {
            assert_eq!(content, "Question1");
            assert_eq!(step_title, Some("Title1"));
        },
        _ => panic!("Expected Coach entry"),
    }

    match &thread[1] {
        super::ThreadEntry::User { content } => {
            assert_eq!(content, "Answer1");
        },
        _ => panic!("Expected User entry"),
    }

    match &thread[2] {
        super::ThreadEntry::Coach { content, step_title } => {
            assert_eq!(content, "Follow1");
            assert_eq!(step_title, None);
        },
        _ => panic!("Expected Coach entry"),
    }
}