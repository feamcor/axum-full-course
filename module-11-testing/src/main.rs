//! # Module 11: Testing
//!
//! Testing Axum applications:
//! - Unit testing handlers
//! - Integration testing with `TestClient`
//! - Testing with mock state

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::get,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

// ============================================================================
// MAIN
// ============================================================================

#[tokio::main]
async fn main() {
    let store = Arc::new(RwLock::new(HashMap::new()));
    let app = create_app(store);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("🚀 Module 11: Testing");
    println!("   Server: http://localhost:3000\n");
    println!("📝 Endpoints:");
    println!("   GET  /health    - Health check");
    println!("   GET  /users     - List users");
    println!("   POST /users     - Create user");
    println!("   GET  /users/:id - Get user\n");
    println!("🧪 Run tests: cargo test");

    axum::serve(listener, app).await.unwrap();
}

// ============================================================================
// APPLICATION CODE
// ============================================================================

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
struct User {
    id: u64,
    name: String,
}

#[derive(Deserialize)]
struct CreateUser {
    name: String,
}

type UserStore = Arc<RwLock<HashMap<u64, User>>>;

async fn list_users(State(store): State<UserStore>) -> Json<Vec<User>> {
    let users = store.read().unwrap();
    Json(users.values().cloned().collect())
}

async fn get_user(
    State(store): State<UserStore>,
    Path(id): Path<u64>,
) -> Result<Json<User>, StatusCode> {
    let users = store.read().unwrap();
    users
        .get(&id)
        .cloned()
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

async fn create_user(
    State(store): State<UserStore>,
    Json(input): Json<CreateUser>,
) -> (StatusCode, Json<User>) {
    let mut users = store.write().unwrap();
    let id = users.len() as u64 + 1;
    let user = User {
        id,
        name: input.name,
    };
    users.insert(id, user.clone());
    (StatusCode::CREATED, Json(user))
}

async fn health() -> &'static str {
    "OK"
}

fn create_app(store: UserStore) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/users", get(list_users).post(create_user))
        .route("/users/{id}", get(get_user))
        .with_state(store)
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    fn test_store() -> UserStore {
        Arc::new(RwLock::new(HashMap::new()))
    }

    #[tokio::test]
    async fn test_health_check() {
        let app = create_app(test_store());

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(&body[..], b"OK");
    }

    #[tokio::test]
    async fn test_create_user() {
        let app = create_app(test_store());

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/users")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"name":"Alice"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let user: User = serde_json::from_slice(&body).unwrap();
        assert_eq!(user.name, "Alice");
    }

    #[tokio::test]
    async fn test_get_user_not_found() {
        let app = create_app(test_store());

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/users/999")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_list_users() {
        let store = test_store();
        store.write().unwrap().insert(
            1,
            User {
                id: 1,
                name: "Bob".to_string(),
            },
        );

        let app = create_app(store);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/users")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let users: Vec<User> = serde_json::from_slice(&body).unwrap();
        assert_eq!(users.len(), 1);
    }
}
