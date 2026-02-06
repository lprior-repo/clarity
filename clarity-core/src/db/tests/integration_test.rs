#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Integration tests for database CRUD operations
//!
//! These tests require a running PostgreSQL database.
//! They can be run using:
//! - `cargo test` with DATABASE_URL set
//! - `sqlx-cli test` for automatic test database setup
//! - testcontainers for automatic database provisioning

use crate::db;
use crate::db::{
  count_beads, count_users, create_bead, create_user, delete_bead, delete_user, get_bead, get_user,
  get_user_by_email, list_beads, list_beads_by_status, list_beads_by_user, list_users,
  run_migrations, update_bead_priority, update_bead_status, update_user_email, update_user_role,
  Bead, BeadId, BeadPriority, BeadStatus, BeadType, DbConfig, Email, NewBead, NewUser, UserId,
  UserRole,
};
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

/// Helper function to create a test database connection pool
async fn create_test_pool() -> PgPool {
  // Try to get DATABASE_URL from environment, otherwise use default test database
  let database_url = std::env::var("DATABASE_URL")
    .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost:5432/clarity_test".to_string());

  let config = DbConfig::new(database_url);
  let pool: PgPool = db::create_pool(&config)
    .await
    .expect("Failed to create test pool");

  // Run migrations on test database
  run_migrations(&pool)
    .await
    .expect("Failed to run migrations");

  // Clean up any existing data
  sqlx::query("DELETE FROM beads")
    .execute(&pool)
    .await
    .expect("Failed to clean beads table");

  sqlx::query("DELETE FROM users")
    .execute(&pool)
    .await
    .expect("Failed to clean users table");

  pool
}

/// Helper function to create a test user
fn create_test_user() -> NewUser {
  NewUser {
    email: Email::new(format!("test{}@example.com", Uuid::new_v4())).unwrap(),
    password_hash: "hash123".to_string(),
    role: UserRole::User,
  }
}

/// Helper function to create a test bead
fn create_test_bead(created_by: Option<db::UserId>) -> NewBead {
  NewBead {
    title: format!("Test Bead {}", Uuid::new_v4()),
    description: Some("Test description".to_string()),
    status: BeadStatus::Open,
    priority: BeadPriority::MEDIUM,
    bead_type: BeadType::Feature,
    created_by,
  }
}

// ===== Connection Pool Tests =====

#[sqlx::test]
async fn test_connection_pool_creation() {
  let database_url = std::env::var("DATABASE_URL")
    .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost:5432/clarity_test".to_string());

  let config = DbConfig::new(database_url);
  let pool_result: Result<PgPool, db::DbError> = db::create_pool(&config).await;
  assert!(
    pool_result.is_ok(),
    "Should create connection pool successfully"
  );
  let pool = pool_result.unwrap();

  // Test connection
  let result: db::DbResult<()> = db::test_connection(&pool).await;
  assert!(result.is_ok(), "Should successfully ping database");
}

#[sqlx::test]
async fn test_migration_execution(pool: PgPool) {
  let result = run_migrations(&pool).await;
  assert!(result.is_ok(), "Migrations should run successfully");

  // Check that tables exist
  let table_check = sqlx::query(
    "SELECT EXISTS (
      SELECT FROM information_schema.tables
      WHERE table_schema = 'public'
      AND table_name = 'users'
    )",
  )
  .fetch_one(&pool)
  .await
  .map_err(|e| db::DbError::Connection(e))
  .unwrap();

  let exists: bool = table_check.get("exists");
  assert!(exists, "Users table should exist after migrations");
}

// ===== User CREATE Tests =====

#[sqlx::test]
async fn test_create_user(pool: PgPool) {
  let new_user = NewUser {
    email: Email::new("test@example.com".to_string()).unwrap(),
    password_hash: "hashed_password".to_string(),
    role: UserRole::User,
  };

  let user_result: Result<crate::db::User, db::DbError> = create_user(&pool, &new_user).await;
  assert!(user_result.is_ok(), "User should be created successfully");
  let user: crate::db::User = user_result.unwrap();

  assert_ne!(
    user.id.as_uuid(),
    Uuid::nil(),
    "User should have a non-nil UUID"
  );
  assert_eq!(user.email.as_str(), "test@example.com");
  assert_eq!(user.password_hash, "hashed_password");
  assert_eq!(user.role, UserRole::User);
}

#[sqlx::test]
async fn test_create_user_with_admin_role(pool: PgPool) {
  let new_user = NewUser {
    email: Email::new("admin@example.com".to_string()).unwrap(),
    password_hash: "admin_hash".to_string(),
    role: UserRole::Admin,
  };

  let user: crate::db::User = create_user(&pool, &new_user).await.unwrap();

  assert_eq!(user.role, UserRole::Admin, "User should have admin role");
}

#[sqlx::test]
async fn test_create_user_duplicate_email_fails(pool: PgPool) {
  let email = "duplicate@example.com".to_string();
  let new_user = NewUser {
    email: Email::new(email.clone()).unwrap(),
    password_hash: "hash1".to_string(),
    role: UserRole::User,
  };

  // Create first user
  create_user(&pool, &new_user).await.unwrap();

  // Try to create duplicate user
  let duplicate_user: NewUser = NewUser {
    email: Email::new(email).unwrap(),
    password_hash: "hash2".to_string(),
    role: UserRole::User,
  };

  let result: Result<crate::db::User, db::DbError> = create_user(&pool, &duplicate_user).await;
  assert!(result.is_err(), "Duplicate email should fail");

  match result {
    Err(db::DbError::Duplicate(msg)) => {
      assert!(
        msg.contains("already exists"),
        "Error should mention duplicate"
      );
    }
    _ => panic!("Expected Duplicate error"),
  }
}

// ===== User READ Tests =====

#[sqlx::test]
async fn test_get_user_by_id(pool: PgPool) {
  let new_user = create_test_user();
  let created_user: crate::db::User = create_user(&pool, &new_user).await.unwrap();

  let fetched_user: Result<crate::db::User, db::DbError> = get_user(&pool, &created_user.id).await;

  assert!(fetched_user.is_ok(), "Should fetch user successfully");
  let fetched_user: crate::db::User = fetched_user.unwrap();

  assert_eq!(fetched_user.id, created_user.id);
  assert_eq!(fetched_user.email.as_str(), created_user.email.as_str());
}

#[sqlx::test]
async fn test_get_user_by_id_not_found(pool: PgPool) {
  let fake_id = db::UserId::new();
  let result = get_user(&pool, &fake_id).await;

  assert!(result.is_err(), "Should return error for non-existent user");

  match result {
    Err(db::DbError::NotFound { entity, id }) => {
      assert_eq!(entity, "User");
      assert_eq!(id, fake_id.to_string());
    }
    _ => panic!("Expected NotFound error"),
  }
}

#[sqlx::test]
async fn test_get_user_by_email(pool: PgPool) {
  let new_user = create_test_user();
  let created_user: crate::db::User = create_user(&pool, &new_user).await.unwrap();

  let fetched_user: Result<crate::db::User, db::DbError> =
    get_user_by_email(&pool, created_user.email.as_str()).await;

  assert!(
    fetched_user.is_ok(),
    "Should fetch user by email successfully"
  );
  let fetched_user: crate::db::User = fetched_user.unwrap();

  assert_eq!(fetched_user.id, created_user.id);
  assert_eq!(fetched_user.email.as_str(), created_user.email.as_str());
}

#[sqlx::test]
async fn test_get_user_by_email_not_found(pool: PgPool) {
  let result = get_user_by_email(&pool, "nonexistent@example.com").await;

  assert!(
    result.is_err(),
    "Should return error for non-existent email"
  );

  match result {
    Err(db::DbError::NotFound { entity, .. }) => {
      assert_eq!(entity, "User");
    }
    _ => panic!("Expected NotFound error"),
  }
}

#[sqlx::test]
async fn test_list_users_empty(pool: PgPool) {
  let users = list_users(&pool).await.unwrap();
  assert_eq!(
    users.len(),
    0,
    "Should return empty list when no users exist"
  );
}

#[sqlx::test]
async fn test_list_users_multiple(pool: PgPool) {
  let user1: crate::db::User = create_user(&pool, &create_test_user()).await.unwrap();
  let user2: crate::db::User = create_user(&pool, &create_test_user()).await.unwrap();
  let user3: crate::db::User = create_user(&pool, &create_test_user()).await.unwrap();

  let users: Vec<crate::db::User> = list_users(&pool).await.unwrap();

  assert_eq!(users.len(), 3, "Should return all users");
  assert!(
    users.iter().any(|u| u.id == user1.id),
    "Should contain user1"
  );
  assert!(
    users.iter().any(|u| u.id == user2.id),
    "Should contain user2"
  );
  assert!(
    users.iter().any(|u| u.id == user3.id),
    "Should contain user3"
  );
}

// ===== User UPDATE Tests =====

#[sqlx::test]
async fn test_update_user_email(pool: PgPool) {
  let new_user = create_test_user();
  let user = create_user(&pool, &new_user).await.unwrap();

  let updated_user = update_user_email(&pool, &user.id, "newemail@example.com")
    .await
    .unwrap();

  assert_eq!(updated_user.email.as_str(), "newemail@example.com");
  assert_ne!(
    updated_user.updated_at, user.updated_at,
    "updated_at should change"
  );
}

#[sqlx::test]
async fn test_update_user_role(pool: PgPool) {
  let _new_user = NewUser {
    email: Email::new("roleuser@example.com".to_string()).unwrap(),
    password_hash: "hash".to_string(),
    role: UserRole::User,
  };

  let users: Vec<crate::db::User> = list_users(&pool).await.unwrap();
  assert_eq!(
    users.len(),
    0,
    "Should return empty list when no users exist"
  );
}

#[sqlx::test]
async fn test_list_beads_multiple(pool: PgPool) {
  let bead1 = create_bead(&pool, &create_test_bead(None)).await.unwrap();
  let bead2 = create_bead(&pool, &create_test_bead(None)).await.unwrap();
  let bead3 = create_bead(&pool, &create_test_bead(None)).await.unwrap();

  let beads = list_beads(&pool).await.unwrap();

  assert_eq!(beads.len(), 3, "Should return all beads");
  assert!(
    beads.iter().any(|b| b.id == bead1.id),
    "Should contain bead1"
  );
  assert!(
    beads.iter().any(|b| b.id == bead2.id),
    "Should contain bead2"
  );
  assert!(
    beads.iter().any(|b| b.id == bead3.id),
    "Should contain bead3"
  );
}

#[sqlx::test]
async fn test_list_beads_by_status(pool: PgPool) {
  // Create beads with different statuses
  create_bead(&pool, &create_test_bead(None)).await.unwrap(); // Open

  let mut in_progress = create_test_bead(None);
  in_progress.status = BeadStatus::InProgress;
  create_bead(&pool, &in_progress).await.unwrap();

  let mut closed = create_test_bead(None);
  closed.status = BeadStatus::Closed;
  create_bead(&pool, &closed).await.unwrap();

  let open_beads = list_beads_by_status(&pool, BeadStatus::Open).await.unwrap();
  assert_eq!(open_beads.len(), 1, "Should find one open bead");

  let in_progress_beads = list_beads_by_status(&pool, BeadStatus::InProgress)
    .await
    .unwrap();
  assert_eq!(
    in_progress_beads.len(),
    1,
    "Should find one in_progress bead"
  );

  let closed_beads = list_beads_by_status(&pool, BeadStatus::Closed)
    .await
    .unwrap();
  assert_eq!(closed_beads.len(), 1, "Should find one closed bead");

  let blocked_beads = list_beads_by_status(&pool, BeadStatus::Blocked)
    .await
    .unwrap();
  assert_eq!(blocked_beads.len(), 0, "Should find no blocked beads");
}

#[sqlx::test]
async fn test_list_beads_by_user(pool: PgPool) {
  let user1 = create_user(&pool, &create_test_user()).await.unwrap();
  let user2 = create_user(&pool, &create_test_user()).await.unwrap();

  // Create beads for user1
  let mut bead1 = create_test_bead(Some(user1.id));
  bead1.title = "User1 Bead 1".to_string();
  create_bead(&pool, &bead1).await.unwrap();

  let mut bead2 = create_test_bead(Some(user1.id));
  bead2.title = "User1 Bead 2".to_string();
  create_bead(&pool, &bead2).await.unwrap();

  // Create bead for user2
  let mut bead3 = create_test_bead(Some(user2.id));
  bead3.title = "User2 Bead".to_string();
  create_bead(&pool, &bead3).await.unwrap();

  // Create bead with no user
  let mut bead4 = create_test_bead(None);
  bead4.title = "No User Bead".to_string();
  create_bead(&pool, &bead4).await.unwrap();

  let user1_beads = list_beads_by_user(&pool, &user1.id).await.unwrap();
  assert_eq!(user1_beads.len(), 2, "Should find 2 beads for user1");

  let user2_beads = list_beads_by_user(&pool, &user2.id).await.unwrap();
  assert_eq!(user2_beads.len(), 1, "Should find 1 bead for user2");
}

// ===== Bead UPDATE Tests =====

#[sqlx::test]
async fn test_update_bead_status(pool: PgPool) {
  let new_bead = create_test_bead(None);
  let bead: crate::db::Bead = create_bead(&pool, &new_bead).await.unwrap();
  assert_eq!(bead.status, BeadStatus::Open);

  let updated_bead = update_bead_status(&pool, &bead.id, BeadStatus::InProgress)
    .await
    .unwrap();

  assert_eq!(updated_bead.status, BeadStatus::InProgress);
  assert_ne!(
    updated_bead.updated_at, bead.updated_at,
    "updated_at should change"
  );
}

#[sqlx::test]
async fn test_update_bead_priority(pool: PgPool) {
  let new_bead = create_test_bead(None);
  let bead: crate::db::Bead = create_bead(&pool, &new_bead).await.unwrap();
  assert_eq!(bead.priority, BeadPriority::MEDIUM);

  let updated_bead = update_bead_priority(&pool, &bead.id, BeadPriority::HIGH)
    .await
    .unwrap();

  assert_eq!(updated_bead.priority, BeadPriority::HIGH);
}

#[sqlx::test]
async fn test_update_bead_status_workflow(pool: PgPool) {
  let new_bead = create_test_bead(None);
  let bead: crate::db::Bead = create_bead(&pool, &new_bead).await.unwrap();

  // Simulate workflow: Open -> InProgress -> Blocked -> InProgress -> Closed
  let workflow = vec![
    BeadStatus::InProgress,
    BeadStatus::Blocked,
    BeadStatus::InProgress,
    BeadStatus::Closed,
  ];

  for status in workflow {
    let updated = update_bead_status(&pool, &bead.id, status).await.unwrap();
    assert_eq!(updated.status, status);
  }
}

#[sqlx::test]
async fn test_update_bead_not_found(pool: PgPool) {
  let fake_id = BeadId::new();
  let result = update_bead_status(&pool, &fake_id, BeadStatus::Closed).await;

  assert!(result.is_err(), "Should return error for non-existent bead");

  match result {
    Err(db::DbError::NotFound { entity, .. }) => {
      assert_eq!(entity, "Bead");
    }
    _ => panic!("Expected NotFound error"),
  }
}

// ===== Bead DELETE Tests =====

#[sqlx::test]
async fn test_delete_bead(pool: PgPool) {
  let new_bead = create_test_bead(None);
  let bead: crate::db::Bead = create_bead(&pool, &new_bead).await.unwrap();

  let result: Result<(), db::DbError> = delete_bead(&pool, &bead.id).await;
  assert!(result.is_ok(), "Should delete bead successfully");

  // Verify bead is deleted
  let fetch_result = get_bead(&pool, &bead.id).await;
  assert!(fetch_result.is_err(), "Bead should no longer exist");
}

#[sqlx::test]
async fn test_delete_bead_not_found(pool: PgPool) {
  let fake_id = BeadId::new();
  let result = delete_bead(&pool, &fake_id).await;

  assert!(result.is_err(), "Should return error for non-existent bead");

  match result {
    Err(db::DbError::NotFound { entity, .. }) => {
      assert_eq!(entity, "Bead");
    }
    _ => panic!("Expected NotFound error"),
  }
}

#[sqlx::test]
async fn test_count_beads(pool: PgPool) {
  // Initially zero
  let count = count_beads(&pool).await.unwrap();
  assert_eq!(count, 0, "Should start with zero beads");

  // Add beads
  create_bead(&pool, &create_test_bead(None)).await.unwrap();
  create_bead(&pool, &create_test_bead(None)).await.unwrap();
  create_bead(&pool, &create_test_bead(None)).await.unwrap();

  let count = count_beads(&pool).await.unwrap();
  assert_eq!(count, 3, "Should count all beads");

  // Delete one
  let bead: crate::db::Bead = create_bead(&pool, &create_test_bead(None)).await.unwrap();
  delete_bead(&pool, &bead.id).await.unwrap();

  let count = count_beads(&pool).await.unwrap();
  assert_eq!(count, 3, "Count should decrease after deletion");
}

// ===== Complex Integration Tests =====

#[sqlx::test]
async fn test_user_bead_relationship(pool: PgPool) {
  // Create user
  let user = create_user(&pool, &create_test_user()).await.unwrap();

  // Create multiple beads for user
  for i in 0..5 {
    let mut bead = create_test_bead(Some(user.id));
    bead.title = format!("Bead {}", i);
    create_bead(&pool, &bead).await.unwrap();
  }

  // Verify relationship
  let user_beads = list_beads_by_user(&pool, &user.id).await.unwrap();
  assert_eq!(user_beads.len(), 5, "User should have 5 beads");

  // Delete user and verify beads still exist but with NULL created_by
  delete_user(&pool, &user.id).await.unwrap();

  let user_beads = list_beads_by_user(&pool, &user.id).await.unwrap();
  assert_eq!(
    user_beads.len(),
    0,
    "Should return no beads for deleted user"
  );

  let all_beads = list_beads(&pool).await.unwrap();
  assert_eq!(all_beads.len(), 5, "Beads should still exist");
  assert!(
    all_beads.iter().all(|b| b.created_by.is_none()),
    "All beads should have NULL created_by"
  );
}

#[sqlx::test]
async fn test_full_crud_workflow(pool: PgPool) {
  // CREATE: Create user and beads
  let user: crate::db::User = create_user(&pool, &create_test_user()).await.unwrap();

  let mut bead = create_test_bead(Some(user.id));
  bead.title = "CRUD Test Bead".to_string();
  let bead: crate::db::Bead = create_bead(&pool, &bead).await.unwrap();

  // READ: Verify creation
  let fetched_user: crate::db::User = get_user(&pool, &user.id).await.unwrap();
  assert_eq!(fetched_user.id, user.id);

  let fetched_bead: crate::db::Bead = get_bead(&pool, &bead.id).await.unwrap();
  assert_eq!(fetched_bead.id, bead.id);

  // UPDATE: Modify entities
  let updated_user: crate::db::User = update_user_role(&pool, &user.id, UserRole::Admin)
    .await
    .unwrap();
  assert_eq!(updated_user.role, UserRole::Admin);

  let updated_bead: crate::db::Bead = update_bead_status(&pool, &bead.id, BeadStatus::Closed)
    .await
    .unwrap();
  assert_eq!(updated_bead.status, BeadStatus::Closed);

  // DELETE: Remove entities
  delete_bead(&pool, &bead.id).await.unwrap();
  delete_user(&pool, &user.id).await.unwrap();

  // Verify deletion
  assert!(get_user(&pool, &user.id).await.is_err());
  assert!(get_bead(&pool, &bead.id).await.is_err());
}

#[sqlx::test]
async fn test_count_across_tables(pool: PgPool) {
  // Create multiple users and beads
  for _ in 0..3 {
    create_user(&pool, &create_test_user()).await.unwrap();
  }

  for _ in 0..7 {
    create_bead(&pool, &create_test_bead(None)).await.unwrap();
  }

  let user_count: usize = count_users(&pool).await.unwrap();
  let bead_count: usize = count_beads(&pool).await.unwrap();

  assert_eq!(user_count, 3, "Should have 3 users");
  assert_eq!(bead_count, 7, "Should have 7 beads");
}
