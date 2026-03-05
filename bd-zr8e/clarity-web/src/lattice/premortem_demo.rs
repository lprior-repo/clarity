#![allow(missing_docs)]

#[cfg(test)]
mod demo_tests {
    use super::super::*;

    #[test]
    fn demo_acceptance_criteria() {
        let solution = "Build a scalable web application with user authentication";
        let constraints = vec
![
            "Must handle 10k concurrent users",
            "Must be GDPR compliant", 
            "Budget constraints apply"
        ];
        
        let output = generate_premortem(solution, &constraints);
        
        println!("\n=== PREMORTEM ANALYSIS DEMO ===\n");
        println!("Solution: {}", output.solution);
        println!("Total scenarios: {}\n", output.scenario_count());
        
        println!("Categories covered:");
        for (category, scenarios) in output.scenarios_by_category() {
            println!("  {:?}: {} scenarios", category, scenarios.len());
        }
        
        println!("\nHigh-risk scenarios (>= 70% likelihood): {}", output.high_risk_count());
        for scenario in &output.high_risk_scenarios {
            println!("  • [{:?}] {}", scenario.category, scenario.trigger);
        }
        
        // Verify acceptance criteria
        assert!(output.scenario_count() >= 3, "Should generate at least 3 scenarios");
        
        let categories: std::collections::HashSet<_> = 
            output.scenarios.iter().map(|s| s.category).collect();
        assert_eq!(categories.len(), 4, "Should cover all 4 categories");
        
        for scenario in &output.scenarios {
            assert!(!scenario.mitigation.is_empty(), "Mitigations should be actionable");
            assert!(scenario.mitigation.len() >= 3, "Should have multiple mitigations");
        }
        
        assert!(output.high_risk_count() > 0, "Should flag high-risk items");
        
        println!("\n✓ All acceptance criteria met!");
        println!("  - Generates 3+ scenarios: {} generated", output.scenario_count());
        println!("  - Covers all 4 categories");
        println!("  - Mitigations actionable: {}+ per scenario", 
                 output.scenarios.iter().map(|s| s.mitigation.len()).min().unwrap());
        println!("  - High-risk items flagged: {} identified", output.high_risk_count());
    }
}
