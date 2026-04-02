# AGENTS.md - Axum Full Course Codebase Guidelines

This is a Rust/Axum 0.8 workspace containing 12 educational modules. Each module is a standalone crate demonstrating a specific aspect of Axum web development.

## Important Notes

- **Rust Edition**: 2024 (use `rustup default stable` to ensure you have the latest Rust)
- **Unsafe Code**: Forbidden (`unsafe_code = "forbid"` in workspace lints)
- **Clippy**: All code should pass `clippy --pedantic`

## Project Structure

```
axum-full-course/
├── Cargo.toml              # Workspace manifest (shared dependencies in [workspace.dependencies])
├── module-01-intro/         # Each module: Cargo.toml + src/main.rs + README.md
├── module-02-routing/
├── module-03-extractors/
├── module-04-responses/
├── module-05-state/
├── module-06-middleware/
├── module-07-errors/
├── module-08-database/
├── module-09-auth/
├── module-10-advanced/
├── module-11-testing/
└── module-12-production/
```

## Build & Test Commands

```bash
# Build all modules
cargo build --workspace

# Build single module
cargo build -p module-01-intro

# Run a module (all modules run on port 3000)
cargo run -p module-08-database

# Run all tests
cargo test --workspace

# Run single module tests
cargo test -p module-11-testing

# Run a specific test by name
cargo test -p module-11-testing test_health_check

# Run with output visible
cargo test --workspace -- --nocapture

# Run doc tests
cargo test --doc

# Check formatting
cargo fmt --check

# Auto-format
cargo fmt

# Lint with clippy (basic)
cargo clippy --workspace --all-targets

# Lint with clippy pedantic (strict mode)
cargo clippy --workspace --all-targets -- -W clippy::pedantic

# Clippy for specific module
cargo clippy -p module-03-extractors

# Release build
cargo build --workspace --release
```

## Code Style Guidelines

### General

- 4-space indentation, no tabs
- No trailing whitespace
- Single blank line between function definitions
- Use `// ============================================================================` section separators for major code blocks
- Module-level doc comments: `//! # Module XX: Title`

### Naming Conventions

| Element | Convention | Example |
|---------|------------|---------|
| Functions | snake_case | `list_users`, `get_user` |
| Variables | snake_case | `user_store`, `max_connections` |
| Types/Structs | PascalCase | `UserStore`, `AppConfig` |
| Enums | PascalCase | `AppError`, `ValidationError` |
| Enum variants | PascalCase | `UserNotFound`, `InvalidInput` |
| Constants | SCREAMING_SNAKE | `MAX_RETRY_COUNT` |
| Module files | snake_case | `main.rs` (special), `auth.rs` |

### Imports

Organize imports in this order with blank lines between groups:

```rust
// Standard library
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

// External crates
use axum::{
    extract::State,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
```

### Error Handling

- Use `thiserror` for custom error types with `#[derive(Error, Debug)]`
- Implement `IntoResponse` for errors to convert to HTTP responses
- Use `Result<T, AppError>` for fallible handlers
- Prefer `.unwrap()` for known-ok cases in initialization; use `?` in handlers

```rust
#[derive(Error, Debug)]
enum AppError {
    #[error("User not found: {0}")]
    UserNotFound(u64),
    #[error("Internal server error")]
    Internal,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::UserNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::Internal => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };
        (status, Json(ErrorResponse { error: message, code: status.as_u16() })).into_response()
    }
}
```

### State Management

- Use `Arc<T>` for shared immutable state
- Use `Arc<RwLock<T>>` for shared mutable state (better read performance than Mutex)
- Define type aliases for state: `type UserStore = Arc<RwLock<HashMap<u64, User>>>;`
- Use `State<T>` extractor for handlers that need state
- Use `Extension<T>` for request-scoped data (e.g., current user from middleware)

```rust
type TodoStore = Arc<RwLock<HashMap<String, Todo>>>;

async fn list_todos(State(store): State<TodoStore>) -> Json<Vec<Todo>> {
    let todos = store.read().unwrap();
    Json(todos.values().cloned().collect())
}
```

### Handlers

- Async functions that return `impl IntoResponse`
- Return `Result<T, Error>` when fallible
- Extractors come before other parameters
- Body-consuming extractors (Json, String, Bytes) must be LAST
- Axum 0.8: No `#[async_trait]` needed for custom extractors

```rust
// Good: extractors first, body last
async fn create_user(
    Path(id): Path<u64>,
    Query(params): Query<ListParams>,
    Json(body): Json<CreateUser>,
) -> Result<Json<User>, AppError> { ... }
```

### Routing

- Axum 0.8 path syntax: `/{id}` (not `/:id`)
- Chain routes: `Router::new().route("/", get(handler)).route("/path", post(handler))`
- Use `.with_state(state)` to attach application state
- Use `.layer()` for middleware

```rust
let app = Router::new()
    .route("/users", get(list_users).post(create_user))
    .route("/users/{id}", get(get_user))
    .with_state(store)
    .layer(Extension(current_user));
```

### Testing

- Tests are inline using `#[cfg(test)] mod tests { ... }`
- Use `tower::ServiceExt::oneshot()` for testing handlers
- Create test fixtures with helper functions: `fn test_store() -> UserStore { ... }`
- Test module dependencies: `use axum::{body::Body, http::Request}; use http_body_util::BodyExt;`

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_health_check() {
        let app = create_app(test_store());
        let response = app
            .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
```

### Async/Await

- Use `#[tokio::main]` for main entry point
- All handlers are `async fn`
- No `#[async_trait]` needed (Axum 0.8 supports native async traits)
- Avoid blocking in async context; use async alternatives from tokio

### Serialization

- Use `#[derive(Serialize, Deserialize)]` from serde
- Use `#[serde(rename = "snake_case")]` for JSON field mapping when needed
- Use `serde_json::json!()` macro for constructing JSON in tests/responses

### Attributes

```rust
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[allow(dead_code)]  // For intentionally unused variants/fields in examples
#[serde(rename_all = "snake_case")]
```

## Environment Setup

For modules 08-09 (database) and 12 (production):
```bash
# Start PostgreSQL
docker-compose up -d postgres

# Copy environment template
cp .env.example .env
```

## Common Patterns

### Creating a New Module

1. Create `module-XX-name/Cargo.toml` with module name
2. Add to workspace members in root `Cargo.toml`
3. Create `src/main.rs` with doc comment `//! # Module XX: Title`
4. Create `README.md` with module documentation

### Adding a Handler

1. Define input/output types with serde derives
2. Create async handler function
3. Register route: `.route("/path", get(handler))`
4. Add startup message with curl examples
