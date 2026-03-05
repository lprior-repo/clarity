// Example usage of the Compact module
//
// This demonstrates how to use the compact_artifacts function to summarize
// all phase answers into a concise, agent-ready format.

use clarity_web::lattice::compact::{compact_artifacts, CompactAnswer};

fn main() {
    // Example: Collecting answers from all phases
    let all_answers = vec![
        // Discover phase
        CompactAnswer {
            step_id: "discover-problem".to_string(),
            value: "Users report the system is slow during peak hours".to_string(),
            timestamp: "2024-01-15T10:00:00Z".to_string(),
        },
        CompactAnswer {
            step_id: "discover-solution".to_string(),
            value: "Implement distributed caching with Redis".to_string(),
            timestamp: "2024-01-15T10:05:00Z".to_string(),
        },
        CompactAnswer {
            step_id: "discover-user-needs".to_string(),
            value: "Users need sub-100ms response times".to_string(),
            timestamp: "2024-01-15T10:10:00Z".to_string(),
        },
        // Define phase
        CompactAnswer {
            step_id: "define-requirements".to_string(),
            value: "System must support 10,000 concurrent users".to_string(),
            timestamp: "2024-01-15T11:00:00Z".to_string(),
        },
        CompactAnswer {
            step_id: "define-constraints".to_string(),
            value: "Budget limited to $50,000".to_string(),
            timestamp: "2024-01-15T11:05:00Z".to_string(),
        },
        CompactAnswer {
            step_id: "define-scope".to_string(),
            value: "Phase 1: Caching layer only".to_string(),
            timestamp: "2024-01-15T11:10:00Z".to_string(),
        },
        // Develop phase
        CompactAnswer {
            step_id: "develop-tasks".to_string(),
            value: "Set up Redis cluster".to_string(),
            timestamp: "2024-01-15T12:00:00Z".to_string(),
        },
        CompactAnswer {
            step_id: "develop-implement".to_string(),
            value: "Implement cache invalidation strategy".to_string(),
            timestamp: "2024-01-15T12:05:00Z".to_string(),
        },
        CompactAnswer {
            step_id: "develop-design".to_string(),
            value: "Use cache-aside pattern with TTL".to_string(),
            timestamp: "2024-01-15T12:10:00Z".to_string(),
        },
        // Deliver phase
        CompactAnswer {
            step_id: "deliver-deploy".to_string(),
            value: "Deploy to production with blue-green deployment".to_string(),
            timestamp: "2024-01-15T13:00:00Z".to_string(),
        },
        CompactAnswer {
            step_id: "deliver-maintain".to_string(),
            value: "Set up monitoring and alerting".to_string(),
            timestamp: "2024-01-15T13:05:00Z".to_string(),
        },
    ];

    // Compact all answers into a summary
    match compact_artifacts(all_answers) {
        Ok(output) => {
            println!("=== Compact Summary ===\n");
            println!("Total artifacts processed: {}\n", output.artifact_count);
            println!("{}\n", output.to_agent_format());

            // Direct access to summary fields
            println!("=== Direct Field Access ===\n");
            println!("Problems ({}):", output.summary.problem.len());
            for (i, problem) in output.summary.problem.iter().enumerate() {
                println!("  {}. {}", i + 1, problem);
            }

            println!("\nSolutions ({}):", output.summary.solution.len());
            for (i, solution) in output.summary.solution.iter().enumerate() {
                println!("  {}. {}", i + 1, solution);
            }

            println!("\nRequirements ({}):", output.summary.requirements.len());
            for (i, req) in output.summary.requirements.iter().enumerate() {
                println!("  {}. {}", i + 1, req);
            }

            println!("\nConstraints ({}):", output.summary.constraints.len());
            for (i, constraint) in output.summary.constraints.iter().enumerate() {
                println!("  {}. {}", i + 1, constraint);
            }

            println!("\nTasks ({}):", output.summary.tasks.len());
            for (i, task) in output.summary.tasks.iter().enumerate() {
                println!("  {}. {}", i + 1, task);
            }
        }
        Err(e) => {
            eprintln!("Error: {e}");
        }
    }
}
