#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Database repository layer for CRUD operations
//!
//! Provides type-safe database access using SQLx for Users and Beads entities.
//!
//! Uses runtime query checking instead of compile-time checking to avoid database
//! connectivity requirements during compilation.

use crate::db::error::{DbError, DbResult};
use crate::db::models::{
  Bead, BeadId, BeadPriority, BeadStatus, BeadType, Email, NewBead, NewUser, User, UserId, UserRole,
};
use chrono::{DateTime, Utc};
use sqlx::postgres::PgPool;
use sqlx::Row;
use uuid::Uuid;

type DbDateTime = DateTime<Utc>;

/// Create a new user in the database
///
/// # Errors
/// - Returns `DbError::Duplicate` if email already exists
/// - Returns `DbError::Validation` if email is invalid
/// - Returns `DbError::DatabaseError` if database operation fails
pub async fn create_user(pool: &PgPool, new_user: &NewUser) -> DbResult<User> {
  let user_id = UserId::new();
  let email = new_user.email.clone();
  let created_at = chrono::Utc::now().naive_utc();
  let updated_at = created_at;

  let result = sqlx::query(
    r#"
    INSERT INTO users (id, email, password_hash, role, created_at, updated_at)
    VALUES ($1, $2, $3, $4, $5, $6)
    RETURNING id, email, role, created_at, updated_at
    "#,
  )
  .bind(user_id.0)
  .bind(email.as_str())
  .bind(new_user.password_hash.clone())
  .bind(new_user.role.clone())
  .bind(created_at)
  .bind(updated_at)
  .fetch_one(pool)
  .await
  .map_err(|e: sqlx::Error| match e {
    sqlx::Error::Database(db_err) if db_err.code() == Some(std::borrow::Cow::Borrowed("23505")) => {
      DbError::Duplicate("Email already exists".into())
    }
    sqlx::Error::Database(_) => DbError::Connection(e),
    sqlx::Error::Io(_) => DbError::Connection(sqlx::Error::Io(std::io::Error::new(
      std::io::ErrorKind::Other,
      "Database connection error",
    ))),
    _ => DbError::Connection(e),
  })?;

  Ok(User {
    id: UserId(result.try_get::<Uuid, _>("id")?),
    email: Email(result.try_get::<String, _>("email")?),
    password_hash: result.try_get::<String, _>("password_hash")?,
    role: UserRole::from_str(result.try_get::<String, _>("role")?.as_str())
      .map_err(|e| DbError::Validation(e.to_string()))?,
    created_at: DateTime::from_naive_utc_and_offset(
      result.try_get::<chrono::NaiveDateTime, _>("created_at")?,
      chrono::Utc,
    ),
    updated_at: DateTime::from_naive_utc_and_offset(
      result.try_get::<chrono::NaiveDateTime, _>("updated_at")?,
      chrono::Utc,
    ),
  })
}

/// Get a user by ID from the database
///
/// # Errors
/// - Returns `DbError::NotFound` if user doesn't exist
/// - Returns `DbError::DatabaseError` if database operation fails
pub async fn get_user(pool: &PgPool, user_id: &UserId) -> DbResult<User> {
  let result = sqlx::query(
    r#"
    SELECT id, email, password_hash, role, created_at, updated_at
    FROM users
    WHERE id = $1
    "#,
  )
  .bind(user_id.0)
  .fetch_optional(pool)
  .await
  .map_err(|e| DbError::Connection(e))?;

  match result {
    Some(row) => Ok(User {
      id: UserId(row.try_get::<Uuid, _>("id")?),
      email: Email(row.try_get::<String, _>("email")?),
      password_hash: row.try_get::<String, _>("password_hash")?,
      role: UserRole::from_str(row.try_get::<String, _>("role")?.as_str())
        .map_err(|e| DbError::Validation(e.to_string()))?,
      created_at: DateTime::from_naive_utc_and_offset(
        row.try_get::<chrono::NaiveDateTime, _>("created_at")?,
        chrono::Utc,
      ),
      updated_at: DateTime::from_naive_utc_and_offset(
        row.try_get::<chrono::NaiveDateTime, _>("updated_at")?,
        chrono::Utc,
      ),
    }),
    None => Err(DbError::NotFound {
      entity: "User".into(),
      id: user_id.to_string(),
    }),
  }
}

/// Update a user's role in the database
///
/// # Errors
/// - Returns `DbError::NotFound` if user doesn't exist
/// - Returns `DbError::DatabaseError` if database operation fails
pub async fn update_user_role(
  pool: &PgPool,
  user_id: &UserId,
  new_role: UserRole,
) -> DbResult<User> {
  let result = sqlx::query(
    r#"
    UPDATE users
    SET role = $1, updated_at = NOW()
    WHERE id = $2
    RETURNING id, email, password_hash, role, created_at, updated_at
    "#,
  )
  .bind(new_role)
  .bind(user_id.0)
  .fetch_optional(pool)
  .await
  .map_err(|e: sqlx::Error| DbError::Connection(e))?;

  match result {
    Some(row) => Ok(User {
      id: UserId(row.try_get::<Uuid, _>("id")?),
      email: Email(row.try_get::<String, _>("email")?),
      password_hash: row.try_get::<String, _>("password_hash")?,
      role: UserRole::from_str(row.try_get::<String, _>("role")?.as_str())
        .map_err(|e| DbError::Validation(e.to_string()))?,
      created_at: DateTime::from_naive_utc_and_offset(
        row.try_get::<chrono::NaiveDateTime, _>("created_at")?,
        chrono::Utc,
      ),
      updated_at: DateTime::from_naive_utc_and_offset(
        row.try_get::<chrono::NaiveDateTime, _>("updated_at")?,
        chrono::Utc,
      ),
    }),
    None => Err(DbError::NotFound {
      entity: "User".into(),
      id: user_id.to_string(),
    }),
  }
}

/// Delete a user from the database
///
/// # Errors
/// - Returns `DbError::NotFound` if user doesn't exist
/// - Returns `DbError::Connection` if database operation fails
pub async fn delete_user(pool: &PgPool, user_id: &UserId) -> DbResult<()> {
  let result = sqlx::query(
    r#"
    DELETE FROM users
    WHERE id = $1
    "#,
  )
  .bind(user_id.0)
  .execute(pool)
  .await
  .map_err(|e| DbError::Connection(e))?;

  if result.rows_affected() == 0 {
    return Err(DbError::NotFound {
      entity: "User".into(),
      id: user_id.to_string(),
    });
  }

  Ok(())
}

/// Count total users in the database
///
/// # Errors
/// - Returns `DbError::Connection` if database operation fails
pub async fn count_users(pool: &PgPool) -> DbResult<usize> {
  let result: Option<i64> = sqlx::query_scalar(
    r#"
    SELECT COUNT(*) FROM users
    "#,
  )
  .fetch_one(pool)
  .await
  .map_err(|e| DbError::Connection(e))?;

  Ok(result.unwrap_or(0) as usize)
}

/// Create a new bead in the database
///
/// # Errors
/// - Returns `DbError::Connection` if database operation fails
pub async fn create_bead(pool: &PgPool, new_bead: &NewBead) -> DbResult<Bead> {
  let bead_id = BeadId::new();
  let created_at = chrono::Utc::now().naive_utc();
  let updated_at = created_at;

  let result = sqlx::query(
    r#"
    INSERT INTO beads (id, title, description, status, priority, bead_type, created_by, created_at, updated_at)
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
    RETURNING id, title, description, status, priority, bead_type, created_by, created_at, updated_at
    "#,
  )
  .bind(bead_id.0)
  .bind(new_bead.title.clone())
  .bind(new_bead.description.clone())
  .bind(new_bead.status.clone())
  .bind(new_bead.priority.0)
  .bind(new_bead.bead_type)
  .bind(new_bead.created_by.unwrap().0)
  .bind(created_at)
  .bind(updated_at)
  .fetch_one(pool)
  .await
  .map_err(|e: sqlx::Error| DbError::Connection(e))?;

  Ok(Bead {
    id: BeadId(result.try_get::<Uuid, _>("id")?),
    title: result.try_get::<String, _>("title")?,
    description: Some(result.try_get::<String, _>("description")?),
    status: result.try_get::<BeadStatus, _>("status")?,
    priority: result
      .try_get::<Option<i16>, _>("priority")?
      .map(|opt| opt.map(BeadPriority))?,
    bead_type: result.try_get::<BeadType, _>("bead_type")?,
    created_by: result
      .try_get::<Option<Uuid>, _>("created_by")
      .map(|opt| opt.map(UserId))?,
    created_at: DateTime::from_naive_utc_and_offset(
      result.try_get::<chrono::NaiveDateTime, _>("created_at")?,
      chrono::Utc,
    ),
    updated_at: DateTime::from_naive_utc_and_offset(
      result.try_get::<chrono::NaiveDateTime, _>("updated_at")?,
      chrono::Utc,
    ),
  })
}

/// Get a bead by ID from the database
///
/// # Errors
/// - Returns `DbError::NotFound` if bead doesn't exist
/// - Returns `DbError::Connection` if database operation fails
pub async fn get_bead(pool: &PgPool, bead_id: &BeadId) -> DbResult<Bead> {
  let result = sqlx::query(
    r#"
    SELECT id, title, description, status, priority, bead_type, created_by, created_at, updated_at
    FROM beads
    WHERE id = $1
    "#,
  )
  .bind(bead_id.0)
  .fetch_optional(pool)
  .await
  .map_err(|e: sqlx::Error| DbError::Connection(e))?;

  match result {
    Some(row) => Ok(Bead {
      id: BeadId(row.try_get::<Uuid, _>("id")?),
      title: row.try_get::<String, _>("title")?,
      description: Some(row.try_get::<String, _>("description")?),
      status: row.try_get::<BeadStatus, _>("status")?,
      priority: row
        .try_get::<Option<i16>, _>("priority")?
        .map(|opt| opt.map(BeadPriority))?,
      bead_type: row.try_get::<BeadType, _>("bead_type")?,
      created_by: result
        .expect("Bead should exist")
        .try_get::<Option<Uuid>, _>("created_by")?
        .map(|opt| opt.map(UserId))?,
      created_at: DateTime::from_naive_utc_and_offset(
        row.try_get::<chrono::NaiveDateTime, _>("created_at")?,
        chrono::Utc,
      ),
      updated_at: DateTime::from_naive_utc_and_offset(
        row.try_get::<chrono::NaiveDateTime, _>("updated_at")?,
        chrono::Utc,
      ),
    }),
    None => Err(DbError::NotFound {
      entity: "Bead".into(),
      id: bead_id.to_string(),
    }),
  }
}

/// List all beads from the database
///
/// # Errors
/// - Returns `DbError::Connection` if database operation fails
pub async fn list_beads(pool: &PgPool) -> DbResult<Vec<Bead>> {
  let results = sqlx::query(
    r#"
    SELECT id, title, description, status, priority, bead_type, created_by, created_at, updated_at
    FROM beads
    ORDER BY created_at DESC
    "#,
  )
  .fetch_all(pool)
  .await
  .map_err(|e: sqlx::Error| DbError::Connection(e))?;

  Ok(
    results
      .into_iter()
      .map(|r: sqlx::postgres::PgRow| {
        Ok::<_, DbError>(Bead {
          id: BeadId(r.try_get::<Uuid, _>("id")?),
          title: r.try_get::<String, _>("title")?,
          description: Some(r.try_get::<String, _>("description")?),
          status: r.try_get::<BeadStatus, _>("status")?,
          priority: r
            .try_get::<Option<i16>, _>("priority")?
            .map(|opt| opt.map(BeadPriority))?
          bead_type: r.try_get::<BeadType, _>("bead_type")?,
          created_by: r
            .try_get::<Option<Uuid>, _>("created_by")?
            .map(|opt| opt.map(UserId))?
          created_at: DateTime::from_naive_utc_and_offset(
            r.try_get::<chrono::NaiveDateTime, _>("created_at")?,
            chrono::Utc,
          ),
          updated_at: DateTime::from_naive_utc_and_offset(
            r.try_get::<chrono::NaiveDateTime, _>("updated_at")?,
            chrono::Utc,
          ),
        })
      })
      .collect::<DbResult<Vec<Bead>>>()?,
  )
}

/// List beads filtered by status
///
/// # Errors
/// - Returns `DbError::Connection` if database operation fails
pub async fn list_beads_by_status(pool: &PgPool, status: BeadStatus) -> DbResult<Vec<Bead>> {
  let results = sqlx::query(
    r#"
    SELECT id, title, description, status, priority, bead_type, created_by, created_at, updated_at
    FROM beads
    WHERE status = $1
    ORDER BY created_at DESC
    "#,
  )
  .bind(status)
  .fetch_all(pool)
  .await
  .map_err(|e: sqlx::Error| DbError::Connection(e))?;

  Ok(
    results
      .into_iter()
      .map(|r: sqlx::postgres::PgRow| {
        Ok::<_, DbError>(Bead {
          id: BeadId(r.try_get::<Uuid, _>("id")?),
          title: r.try_get::<String, _>("title")?,
          description: Some(r.try_get::<String, _>("description")?),
          status: r.try_get::<BeadStatus, _>("status")?,
          priority: r
            .try_get::<Option<i16>, _>("priority")?
            .map(|opt| opt.map(BeadPriority))?
          bead_type: r.try_get::<BeadType, _>("bead_type")?,
          created_by: r
            .try_get::<Option<Uuid>, _>("created_by")?
            .map(|opt| opt.map(UserId))?
          created_at: DateTime::from_naive_utc_and_offset(
            r.try_get::<chrono::NaiveDateTime, _>("created_at")?,
            chrono::Utc,
          ),
          updated_at: DateTime::from_naive_utc_and_offset(
            r.try_get::<chrono::NaiveDateTime, _>("updated_at")?,
            chrono::Utc,
          ),
        })
      })
      .collect::<DbResult<Vec<Bead>>>()?,
  )
}

/// List beads filtered by creator user ID
///
/// # Errors
/// - Returns `DbError::Connection` if database operation fails
pub async fn list_beads_by_user(pool: &PgPool, user_id: &UserId) -> DbResult<Vec<Bead>> {
  let results = sqlx::query(
    r#"
    SELECT id, title, description, status, priority, bead_type, created_by, created_at, updated_at
    FROM beads
    WHERE created_by = $1
    ORDER BY created_at DESC
    "#,
  )
  .bind(user_id.0)
  .fetch_all(pool)
  .await
  .map_err(|e: sqlx::Error| DbError::Connection(e))?;

  Ok(
    results
      .into_iter()
      .map(|r: sqlx::postgres::PgRow| {
        Ok::<_, DbError>(Bead {
          id: BeadId(r.try_get::<Uuid, _>("id")?),
          title: r.try_get::<String, _>("title")?,
          description: Some(r.try_get::<String, _>("description")?),
          status: r.try_get::<BeadStatus, _>("status")?,
          priority: r
            .try_get::<Option<i16>, _>("priority")?
            .map(|opt| opt.map(BeadPriority))?
          bead_type: r.try_get::<BeadType, _>("bead_type")?,
          created_by: r
            .try_get::<Option<Uuid>, _>("created_by")?
            .map(|opt| opt.map(UserId))?
          created_at: DateTime::from_naive_utc_and_offset(
            r.try_get::<chrono::NaiveDateTime, _>("created_at")?,
            chrono::Utc,
          ),
          updated_at: DateTime::from_naive_utc_and_offset(
            r.try_get::<chrono::NaiveDateTime, _>("updated_at")?,
            chrono::Utc,
          ),
        })
      })
      .collect::<DbResult<Vec<Bead>>>()?,
  )
}

/// Update a bead's status in the database
///
/// # Errors
/// - Returns `DbError::NotFound` if bead doesn't exist
/// - Returns `DbError::DatabaseError` if database operation fails
pub async fn update_bead_status(
  pool: &PgPool,
  bead_id: &BeadId,
  new_status: BeadStatus,
) -> DbResult<Bead> {
  let result = sqlx::query(
    r#"
    UPDATE beads
    SET status = $1, updated_at = NOW()
    WHERE id = $2
    RETURNING id, title, description, status, priority, bead_type, created_by, created_at, updated_at
    "#,
  )
  .bind(new_status)
  .bind(bead_id.0)
  .fetch_optional(pool)
  .await
  .map_err(|e: sqlx::Error| DbError::Connection(e))?;

  match result {
    Some(row) => Ok(Bead {
      id: BeadId(row.try_get::<Uuid, _>("id")?),
      title: row.try_get::<String, _>("title")?,
      description: Some(row.try_get::<String, _>("description")?),
      status: row.try_get::<BeadStatus, _>("status")?,
      priority: row
        .try_get::<Option<i16>, _>("priority")?
        .map(|opt| opt.map(BeadPriority))?
      bead_type: row.try_get::<BeadType, _>("bead_type")?,
      created_by: result
        .expect("Bead should exist")
        .try_get::<Option<Uuid>, _>("created_by")?
        .map(|opt| opt.map(UserId))?
      created_at: DateTime::from_naive_utc_and_offset(
        row.try_get::<chrono::NaiveDateTime, _>("created_at")?,
        chrono::Utc,
      ),
      updated_at: DateTime::from_naive_utc_and_offset(
        row.try_get::<chrono::NaiveDateTime, _>("updated_at")?,
        chrono::Utc,
      ),
    }),
    None => Err(DbError::NotFound {
      entity: "Bead".into(),
      id: bead_id.to_string(),
    }),
  }
}

/// Update a bead's priority in the database
///
/// # Errors
/// - Returns `DbError::NotFound` if bead doesn't exist
/// - Returns `DbError::DatabaseError` if database operation fails
pub async fn update_bead_priority(
  pool: &PgPool,
  bead_id: &BeadId,
  new_priority: BeadPriority,
) -> DbResult<Bead> {
  let result = sqlx::query(
    r#"
    UPDATE beads
    SET priority = $1, updated_at = NOW()
    WHERE id = $2
    RETURNING id, title, description, status, priority, bead_type, created_by, created_at, updated_at
    "#,
  )
  .bind(new_priority.0)
  .bind(bead_id.0)
  .fetch_optional(pool)
  .await
  .map_err(|e: sqlx::Error| DbError::Connection(e))?;

  match result {
    Some(row) => Ok(Bead {
      id: BeadId(row.try_get::<Uuid, _>("id")?),
      title: row.try_get::<String, _>("title")?,
      description: Some(row.try_get::<String, _>("description")?),
      status: row.try_get::<BeadStatus, _>("status")?,
      priority: row
        .try_get::<Option<i16>, _>("priority")?
        .map(|opt| opt.map(BeadPriority))?
      bead_type: row.try_get::<BeadType, _>("bead_type")?,
      created_by: result
        .expect("Bead should exist")
        .try_get::<Option<Uuid>, _>("created_by")?
        .map(|opt| opt.map(UserId))?
      created_at: DateTime::from_naive_utc_and_offset(
        row.try_get::<chrono::NaiveDateTime, _>("created_at")?,
        chrono::Utc,
      ),
      updated_at: DateTime::from_naive_utc_and_offset(
        row.try_get::<chrono::NaiveDateTime, _>("updated_at")?,
        chrono::Utc,
      ),
    }),
    None => Err(DbError::NotFound {
      entity: "Bead".into(),
      id: bead_id.to_string(),
    }),
  }
}

/// Delete a bead from the database
///
/// # Errors
/// - Returns `DbError::NotFound` if bead doesn't exist
/// - Returns `DbError::DatabaseError` if database operation fails
pub async fn delete_bead(pool: &PgPool, bead_id: &BeadId) -> DbResult<()> {
  let result = sqlx::query(
    r#"
    DELETE FROM beads
    WHERE id = $1
    "#,
  )
  .bind(bead_id.0)
  .execute(pool)
  .await
  .map_err(|e| DbError::Connection(e))?;

  if result.rows_affected() == 0 {
    return Err(DbError::NotFound {
      entity: "Bead".into(),
      id: bead_id.to_string(),
    });
  }

  Ok(())
}

/// Count total beads in the database
///
/// # Errors
/// - Returns `DbError::Connection` if database operation fails
pub async fn count_beads(pool: &PgPool) -> DbResult<usize> {
  let result: Option<i64> = sqlx::query_scalar(
    r#"
    SELECT COUNT(*) FROM beads
    "#,
  )
  .fetch_one(pool)
  .await
  .map_err(|e| DbError::Connection(e))?;

  Ok(result.unwrap_or(0) as usize)
}
