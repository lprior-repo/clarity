# Database Integration Tests

This directory contains comprehensive integration tests for the clarity-core database layer.

## Overview

The integration tests cover:
- Connection pool creation and management
- Database migration execution
- Full CRUD operations for Users
- Full CRUD operations for Beads
- Relationship tests between Users and Beads
- Edge cases and error handling

## Test Statistics

- **Total Integration Tests**: 40+
- **Coverage**: CREATE, READ, UPDATE, DELETE for all entities
- **Test Types**: Unit tests, workflow tests, relationship tests

## Setting Up Test Database

### Option 1: Docker Compose (Recommended)

1. Start the test database:
   ```bash
   docker-compose -f docker-compose.test.yml up -d
   ```

2. Run tests:
   ```bash
   DATABASE_URL="postgresql://postgres:postgres@localhost:5433/clarity_test" cargo test --package clarity-core --features integration-tests
   ```

3. Stop the database when done:
   ```bash
   docker-compose -f docker-compose.test.yml down
   ```

### Option 2: Local PostgreSQL

1. Create a test database:
   ```bash
   createdb clarity_test
   ```

2. Run tests:
   ```bash
   DATABASE_URL="postgresql://postgres@localhost:5432/clarity_test" cargo test --package clarity-core --features integration-tests
   ```

### Option 3: Environment Variable

1. Copy the example environment file:
   ```bash
   cp .env.example.test .env.test
   ```

2. Edit `.env.test` with your database credentials

3. Source and run:
   ```bash
   source .env.test
   cargo test --package clarity-core --features integration-tests
   ```

## Running Tests

### Run All Integration Tests

```bash
cargo test --package clarity-core --features integration-tests --test integration_test
```

### Run Specific Test

```bash
cargo test --package clarity-core --features integration-tests test_create_user
```

### Run with Output

```bash
cargo test --package clarity-core --features integration-tests -- --nocapture
```

## Test Categories

### Connection Tests
- `test_connection_pool_creation` - Verifies pool can be created
- `test_migration_execution` - Runs and verifies migrations

### User CRUD Tests
- **CREATE**: `test_create_user`, `test_create_user_with_admin_role`, `test_create_user_duplicate_email_fails`
- **READ**: `test_get_user_by_id`, `test_get_user_by_email`, `test_list_users_*`
- **UPDATE**: `test_update_user_email`, `test_update_user_role`
- **DELETE**: `test_delete_user`, `test_delete_user_not_found`
- **COUNT**: `test_count_users`

### Bead CRUD Tests
- **CREATE**: `test_create_bead`, `test_create_bead_with_user`, `test_create_bead_all_types`, `test_create_bead_all_statuses`
- **READ**: `test_get_bead_by_id`, `test_list_beads_*`, `test_list_beads_by_status`, `test_list_beads_by_user`
- **UPDATE**: `test_update_bead_status`, `test_update_bead_priority`, `test_update_bead_status_workflow`
- **DELETE**: `test_delete_bead`, `test_delete_bead_not_found`
- **COUNT**: `test_count_beads`

### Complex Integration Tests
- `test_user_bead_relationship` - Tests foreign key relationships
- `test_full_crud_workflow` - End-to-end CRUD workflow
- `test_count_across_tables` - Aggregation tests

## Test Data

Each test:
- Creates isolated test data using UUID generation
- Cleans up after itself
- Uses transactions where appropriate
- Verifies database state changes

## CI/CD Integration

For CI/CD pipelines, use Docker Compose:

```yaml
test:
  script:
    - docker-compose -f clarity-core/docker-compose.test.yml up -d
    - sleep 5  # Wait for PostgreSQL to be ready
    - DATABASE_URL="postgresql://postgres:postgres@localhost:5433/clarity_test" cargo test --package clarity-core --features integration-tests
    - docker-compose -f clarity-core/docker-compose.test.yml down
```

## Troubleshooting

### Connection Refused
Ensure PostgreSQL is running and accessible:
```bash
docker ps | grep clarity-test-db
```

### Migration Failures
Drop and recreate the test database:
```bash
dropdb clarity_test && createdb clarity_test
```

### Permission Errors
Ensure the database user has proper permissions:
```sql
GRANT ALL PRIVILEGES ON DATABASE clarity_test TO postgres;
```

## Adding New Tests

1. Add test function to `src/db/tests/integration_test.rs`
2. Use `#[sqlx::test]` attribute for automatic pool injection
3. Follow naming convention: `test_<entity>_<operation>`
4. Include assertions for success and error cases

Example:
```rust
#[sqlx::test]
async fn test_my_new_feature(pool: PgPool) {
  // Arrange
  let test_data = create_test_data();

  // Act
  let result = my_operation(&pool, &test_data).await;

  // Assert
  assert!(result.is_ok());
}
```

## Repository Functions Tested

All repository functions in `src/db/repository.rs` have corresponding integration tests:

- `create_user`, `get_user`, `get_user_by_email`, `list_users`
- `update_user_email`, `update_user_role`, `delete_user`
- `create_bead`, `get_bead`, `list_beads`, `list_beads_by_status`, `list_beads_by_user`
- `update_bead_status`, `update_bead_priority`, `delete_bead`
- `count_users`, `count_beads`
