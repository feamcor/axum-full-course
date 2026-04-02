//! # Module 02: Routing Deep Dive
//!
//! This module covers everything about routing in Axum 0.8:
//! - Path parameters with NEW `/{param}` syntax
//! - Wildcard routes with `/{*rest}`
//! - Query parameters
//! - Router nesting and merging
//! - Method routing (GET, POST, PUT, DELETE, etc.)

use axum::{
    Router,
    extract::{Path, Query},
    routing::{delete, get, patch, post, put},
};
use serde::Deserialize;

// ============================================================================
// LESSON 1: Path Parameters - NEW SYNTAX IN AXUM 0.8!
// ============================================================================

/// IMPORTANT: Axum 0.8 introduced new path syntax!
///
/// OLD (pre-0.8): `/:id` and `/*rest`
/// NEW (0.8+):    `/{id}` and `/{*rest}`
///
/// This change allows routes like `/api/:colon` that start with `:` or `*`
/// Single path parameter
async fn get_user(Path(id): Path<u64>) -> String {
    format!("Getting user with ID: {id}")
}

/// Multiple path parameters - use a tuple
async fn get_user_post(Path((user_id, post_id)): Path<(u64, u64)>) -> String {
    format!("User {user_id} - Post {post_id}")
}

/// You can also deserialize into a struct
#[derive(Deserialize)]
struct PostPath {
    user: u64,
    post: u64,
    comment: u64,
}

async fn get_comment(Path(params): Path<PostPath>) -> String {
    format!(
        "User {} - Post {} - Comment {}",
        params.user, params.post, params.comment
    )
}

// ============================================================================
// LESSON 2: Wildcard Routes - NEW SYNTAX!
// ============================================================================

/// Wildcard captures the rest of the path
/// OLD: `/*rest`
/// NEW: `/{*rest}`
async fn files(Path(path): Path<String>) -> String {
    format!("Accessing file: {path}")
}

// ============================================================================
// LESSON 3: Query Parameters
// ============================================================================

#[derive(Deserialize)]
struct Pagination {
    page: Option<u32>,
    limit: Option<u32>,
}

async fn list_items(Query(pagination): Query<Pagination>) -> String {
    let page = pagination.page.unwrap_or(1);
    let limit = pagination.limit.unwrap_or(10);
    format!("Listing items - Page: {page}, Limit: {limit}")
}

/// Query parameters with validation
#[derive(Deserialize)]
struct SearchParams {
    q: String,
    category: Option<String>,
    sort: Option<String>,
}

async fn search(Query(params): Query<SearchParams>) -> String {
    format!(
        "Searching for '{}' in category '{}', sorted by '{}'",
        params.q,
        params.category.unwrap_or_else(|| "all".to_string()),
        params.sort.unwrap_or_else(|| "relevance".to_string())
    )
}

// ============================================================================
// LESSON 4: HTTP Methods
// ============================================================================

async fn create_user() -> &'static str {
    "Creating new user (POST)"
}

async fn update_user(Path(id): Path<u64>) -> String {
    format!("Updating user {id} (PUT - full update)")
}

async fn patch_user(Path(id): Path<u64>) -> String {
    format!("Patching user {id} (PATCH - partial update)")
}

async fn delete_user(Path(id): Path<u64>) -> String {
    format!("Deleting user {id} (DELETE)")
}

// ============================================================================
// LESSON 5: Router Nesting
// ============================================================================

/// Create a sub-router for user-related routes
fn user_routes() -> Router {
    Router::new()
        .route("/", get(list_users).post(create_user))
        .route(
            "/{id}",
            get(get_user)
                .put(update_user)
                .patch(patch_user)
                .delete(delete_user),
        )
}

async fn list_users() -> &'static str {
    "Listing all users"
}

/// Create a sub-router for post-related routes
fn post_routes() -> Router {
    Router::new()
        .route("/", get(list_posts))
        .route("/{id}", get(get_post))
}

async fn list_posts() -> &'static str {
    "Listing all posts"
}

async fn get_post(Path(id): Path<u64>) -> String {
    format!("Getting post {id}")
}

// ============================================================================
// LESSON 6: Router Merging
// ============================================================================

/// Create an API v1 router by merging multiple routers
fn api_v1_routes() -> Router {
    Router::new()
        .nest("/users", user_routes())
        .nest("/posts", post_routes())
}

/// You can also have multiple API versions
fn api_v2_routes() -> Router {
    Router::new()
        .route("/users", get(|| async { "API v2 - Users endpoint" }))
        .route("/posts", get(|| async { "API v2 - Posts endpoint" }))
}

// ============================================================================
// LESSON 7: Fallback Routes
// ============================================================================

async fn not_found() -> (axum::http::StatusCode, &'static str) {
    (axum::http::StatusCode::NOT_FOUND, "404 - Route not found")
}

// ============================================================================
// MAIN: Putting It All Together
// ============================================================================

#[tokio::main]
async fn main() {
    let app =
        Router::new()
            // Basic routes
            .route("/", get(|| async { "Welcome to the Routing Module!" }))
            // ===== HTTP METHODS DEMO =====
            // Each method demonstrated with a standalone route
            .route("/resource", get(|| async { "GET - Read resource" }))
            .route("/resource", post(|| async { "POST - Create resource" }))
            .route(
                "/resource/{id}",
                get(|Path(id): Path<u64>| async move { format!("GET - Read resource {id}") }),
            )
            .route(
                "/resource/{id}",
                put(
                    |Path(id): Path<u64>| async move { format!("PUT - Full update resource {id}") },
                ),
            )
            .route(
                "/resource/{id}",
                patch(|Path(id): Path<u64>| async move {
                    format!("PATCH - Partial update resource {id}")
                }),
            )
            .route(
                "/resource/{id}",
                delete(
                    |Path(id): Path<u64>| async move { format!("DELETE - Remove resource {id}") },
                ),
            )
            // Path parameters (new syntax!)
            .route("/users/{id}/posts/{post_id}", get(get_user_post))
            .route(
                "/users/{user_id}/posts/{post_id}/comments/{comment_id}",
                get(get_comment),
            )
            // Wildcard route (must come after specific routes)
            .route("/files/{*path}", get(files))
            // Query parameters
            .route("/items", get(list_items))
            .route("/search", get(search))
            // Nested routers - creates /api/v1/users, /api/v1/posts, etc.
            .nest("/api/v1", api_v1_routes())
            .nest("/api/v2", api_v2_routes())
            // Fallback for unmatched routes
            .fallback(not_found);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind to port 3000");

    println!("🚀 Module 02: Routing Deep Dive");
    println!("   Server running on http://localhost:3000");
    println!();
    println!("📝 HTTP Methods Demo:");
    println!("   GET    /resource      - Read all");
    println!("   POST   /resource      - Create new");
    println!("   GET    /resource/123  - Read one");
    println!("   PUT    /resource/123  - Full update");
    println!("   PATCH  /resource/123  - Partial update");
    println!("   DELETE /resource/123  - Remove");
    println!();
    println!("📝 Path Parameters (NEW SYNTAX!):");
    println!("   GET /users/123/posts/456");
    println!("   GET /files/docs/readme.md");
    println!();
    println!("📝 Nested Routes:");
    println!("   GET  /api/v1/users");
    println!("   POST /api/v1/users");
    println!("   PUT  /api/v1/users/123");

    axum::serve(listener, app).await.expect("Server failed");
}
