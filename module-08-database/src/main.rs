//! # Module 08: Database Integration
//!
//! `SQLx` with `PostgreSQL` in Axum:
//! - Connection pooling
//! - CRUD operations
//! - Query macros
//! - Migrations

use axum::Json;
use axum::Router;
use axum::extract::Path;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use serde::Deserialize;
use serde::Serialize;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;

// ============================================================================
// MODELS
// ============================================================================

#[derive(Debug, Serialize, sqlx::FromRow)]
struct User {
    id: Uuid,
    name: String,
    email: String,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
struct CreateUser {
    name: String,
    email: String,
}

#[derive(Debug, Deserialize)]
struct UpdateUser {
    name: Option<String>,
    email: Option<String>,
}

// ============================================================================
// ERROR HANDLING
// ============================================================================

#[derive(Debug, thiserror::Error)]
enum DbError {
    #[error("User not found")]
    NotFound,
    #[error("Database error: {0}")]
    Sqlx(#[from] sqlx::Error),
}

impl IntoResponse for DbError {
    fn into_response(self) -> axum::response::Response {
        let (status, msg) = match self {
            DbError::NotFound => (StatusCode::NOT_FOUND, "User not found"),
            DbError::Sqlx(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Database error"),
        };
        (status, msg).into_response()
    }
}

// ============================================================================
// HANDLERS
// ============================================================================

async fn list_users(State(pool): State<PgPool>) -> Result<Json<Vec<User>>, DbError> {
    let users = sqlx::query_as::<_, User>("SELECT * FROM users ORDER BY created_at DESC")
        .fetch_all(&pool)
        .await?;
    Ok(Json(users))
}

async fn get_user(State(pool): State<PgPool>, Path(id): Path<Uuid>) -> Result<Json<User>, DbError> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(id)
        .fetch_optional(&pool)
        .await?
        .ok_or(DbError::NotFound)?;
    Ok(Json(user))
}

async fn create_user(
    State(pool): State<PgPool>,
    Json(input): Json<CreateUser>,
) -> Result<(StatusCode, Json<User>), DbError> {
    let user = sqlx::query_as::<_, User>(
        "INSERT INTO users (id, name, email, created_at) VALUES ($1, $2, $3, NOW()) RETURNING *",
    )
    .bind(Uuid::new_v4())
    .bind(&input.name)
    .bind(&input.email)
    .fetch_one(&pool)
    .await?;
    Ok((StatusCode::CREATED, Json(user)))
}

async fn update_user(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateUser>,
) -> Result<Json<User>, DbError> {
    let user = sqlx::query_as::<_, User>(
        "UPDATE users SET name = COALESCE($2, name), email = COALESCE($3, email) WHERE id = $1 RETURNING *",
    )
    .bind(id)
    .bind(&input.name)
    .bind(&input.email)
    .fetch_optional(&pool)
    .await?
    .ok_or(DbError::NotFound)?;
    Ok(Json(user))
}

async fn delete_user(State(pool): State<PgPool>, Path(id): Path<Uuid>) -> Result<StatusCode, DbError> {
    let result = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await?;

    if result.rows_affected() == 0 {
        Err(DbError::NotFound)
    } else {
        Ok(StatusCode::NO_CONTENT)
    }
}

// ============================================================================
// MAIN
// ============================================================================

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost/axum_course".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    // Run migrations
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS users (
            id UUID PRIMARY KEY,
            name TEXT NOT NULL,
            email TEXT NOT NULL UNIQUE,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
    )
    .execute(&pool)
    .await
    .expect("Failed to create table");

    let app = Router::new()
        .route("/users", get(list_users).post(create_user))
        .route("/users/{id}", get(get_user).put(update_user).delete(delete_user))
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("🚀 Module 08: Database Integration");
    println!("   Server: http://localhost:3000\n");
    println!("📝 CRUD Endpoints:");
    println!("   GET    /users     - List all users");
    println!("   POST   /users     - Create user");
    println!("   GET    /users/:id - Get user");
    println!("   PUT    /users/:id - Update user");
    println!("   DELETE /users/:id - Delete user");
    println!("\n⚠️  Requires PostgreSQL running!");

    axum::serve(listener, app).await.unwrap();
}
