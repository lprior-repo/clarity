//! Integration tests for br_show module
//!
//! These tests verify that the br integration works correctly.

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;

    #[tokio::test]
    async fn test_fetch_br_issue_bd_1bf() {
        // Test the specific case mentioned in the requirement
        let result = fetch_br_issue("bd-1bf").await;

        assert!(result.is_ok(), "Should be able to fetch bd-1bf");

        let issue = result.unwrap();
        assert_eq!(issue.id, "bd-1bf");
        assert_eq!(issue.title, "qa: Run adversarial testing on all components");
        assert_eq!(issue.status, "open");
        assert_eq!(issue.priority, 1);
        assert_eq!(issue.issue_type, "chore");
    }

    #[tokio::test]
    async fn test_fetch_br_issue_nonexistent() {
        // Test with a non-existent ID
        let result = fetch_br_issue("nonexistent-123").await;

        assert!(result.is_err(), "Should fail for non-existent issue");
        match result {
            Err(BrShowError::IssueNotFound(id)) => {
                assert_eq!(id, "nonexistent-123");
            }
            _ => panic!("Expected IssueNotFound error"),
        }
    }

    #[tokio::test]
    async fn test_issue_exists_bd_1bf() {
        // Test existence check for bd-1bf
        let result = issue_exists("bd-1bf").await;

        assert!(result.is_ok(), "Should be able to check existence");
        assert!(result.unwrap(), "bd-1bf should exist");
    }

    #[tokio::test]
    async fn test_issue_exists_nonexistent() {
        // Test existence check for non-existent
        let result = issue_exists("nonexistent-123").await;

        assert!(result.is_ok(), "Should be able to check existence");
        assert!(!result.unwrap(), "nonexistent-123 should not exist");
    }

    #[tokio::test]
    async fn test_get_issue_ids() {
        // Test getting all issue IDs
        let result = get_issue_ids().await;

        assert!(result.is_ok(), "Should be able to get issue IDs");
        let ids = result.unwrap();
        assert!(!ids.is_empty(), "Should have some issue IDs");
        assert!(ids.contains(&"bd-1bf".to_string()), "Should contain bd-1bf");
    }
}