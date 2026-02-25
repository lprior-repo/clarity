//! Server functions for the Clarity Planner backend
//!
//! These functions run on the server and can be called from the client
//! using Dioxus fullstack server functions.

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

/// A planning bead (atomic work unit)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Bead {
    pub id: String,
    pub title: String,
    pub description: String,
    pub phase: Phase,
    pub status: BeadStatus,
    pub created_at: String,
    pub updated_at: String,
}

/// Planning phases (Double Diamond)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Phase {
    Discover,
    Define,
    Develop,
    Deliver,
}

/// Bead status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BeadStatus {
    Todo,
    InProgress,
    Review,
    Done,
}

/// Save a bead to the database
#[server]
pub async fn save_bead(bead: Bead) -> Result<Bead, ServerFnError> {
    // In a real app, this would save to a database
    // For now, we just return the bead with an updated timestamp
    let updated_bead = Bead {
        updated_at: chrono::Utc::now().to_rfc3339(),
        ..bead
    };
    Ok(updated_bead)
}

/// Get all beads for a project
#[server]
pub async fn get_beads(project_id: String) -> Result<Vec<Bead>, ServerFnError> {
    // In a real app, this would fetch from a database
    // For now, return sample data
    let beads = vec![
        Bead {
            id: "1".to_string(),
            title: "User Research".to_string(),
            description: "Conduct user interviews and surveys".to_string(),
            phase: Phase::Discover,
            status: BeadStatus::Done,
            created_at: "2025-01-01T00:00:00Z".to_string(),
            updated_at: "2025-01-15T00:00:00Z".to_string(),
        },
        Bead {
            id: "2".to_string(),
            title: "Define Problem Statement".to_string(),
            description: "Synthesize research into a clear problem definition".to_string(),
            phase: Phase::Define,
            status: BeadStatus::InProgress,
            created_at: "2025-01-16T00:00:00Z".to_string(),
            updated_at: "2025-01-20T00:00:00Z".to_string(),
        },
        Bead {
            id: "3".to_string(),
            title: "Design Prototype".to_string(),
            description: "Create interactive prototype for testing".to_string(),
            phase: Phase::Develop,
            status: BeadStatus::Todo,
            created_at: "2025-01-21T00:00:00Z".to_string(),
            updated_at: "2025-01-21T00:00:00Z".to_string(),
        },
    ];
    Ok(beads)
}

/// Delete a bead
#[server]
pub async fn delete_bead(bead_id: String) -> Result<(), ServerFnError> {
    // In a real app, this would delete from a database
    println!("Deleting bead: {}", bead_id);
    Ok(())
}

/// AI coaching prompt response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoachResponse {
    pub phase: Phase,
    pub guidance: String,
    pub questions: Vec<String>,
}

/// Get AI coaching guidance for a phase
#[server]
pub async fn get_coach_guidance(phase: Phase, context: String) -> Result<CoachResponse, ServerFnError> {
    // In a real app, this would call an AI API
    let guidance = match phase {
        Phase::Discover => format!(
            "In the Discover phase, focus on understanding users deeply. Context: {}",
            context
        ),
        Phase::Define => format!(
            "In the Define phase, synthesize your findings into a clear problem. Context: {}",
            context
        ),
        Phase::Develop => format!(
            "In the Develop phase, ideate and prototype solutions. Context: {}",
            context
        ),
        Phase::Deliver => format!(
            "In the Deliver phase, test and refine your solution. Context: {}",
            context
        ),
    };

    let questions = match phase {
        Phase::Discover => vec![
            "Who are your target users?".to_string(),
            "What problems do they face?".to_string(),
            "How do they currently solve these problems?".to_string(),
        ],
        Phase::Define => vec![
            "What is the core problem you're solving?".to_string(),
            "What insights emerged from research?".to_string(),
            "What constraints must you consider?".to_string(),
        ],
        Phase::Develop => vec![
            "What solutions have the highest impact?".to_string(),
            "How can you validate ideas quickly?".to_string(),
            "What resources do you need?".to_string(),
        ],
        Phase::Deliver => vec![
            "How will you measure success?".to_string(),
            "What feedback have you received?".to_string(),
            "What needs iteration?".to_string(),
        ],
    };

    Ok(CoachResponse {
        phase,
        guidance,
        questions,
    })
}
