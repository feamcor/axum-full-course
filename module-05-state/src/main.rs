//! # Module 05: State Management
//!
//! Learn how to manage application state in Axum:
//! - Immutable shared state with State<T>
//! - Mutable shared state with Arc<Mutex<T>>
//! - Database connection pools
//! - Multiple state types

use axum::Json;
use axum::Router;
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::get;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;
use uuid::Uuid;

// ============================================================================
// LESSON 1: Immutable Shared State
// ============================================================================

/// Configuration that doesn't change at runtime
#[derive(Clone)]
struct AppConfig {
    app_name: String,
    version: String,
    max_items_per_page: usize,
}

async fn get_config(State(config): State<Arc<AppConfig>>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "app_name": config.app_name,
        "version": config.version,
        "max_items": config.max_items_per_page
    }))
}

// ============================================================================
// LESSON 2: Mutable Shared State
// ============================================================================

/// A simple in-memory "database" of todos
/// Using `RwLock` for better read performance (multiple readers, single writer)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Todo {
    id: String,
    title: String,
    completed: bool,
}

#[derive(Debug, Deserialize)]
struct CreateTodo {
    title: String,
}

#[derive(Debug, Deserialize)]
struct UpdateTodo {
    title: Option<String>,
    completed: Option<bool>,
}

/// Our mutable state - a thread-safe `HashMap`
type TodoStore = Arc<RwLock<HashMap<String, Todo>>>;

// List all todos
async fn list_todos(State(store): State<TodoStore>) -> Json<Vec<Todo>> {
    let todos = store.read().unwrap();
    let todos_vec: Vec<Todo> = todos.values().cloned().collect();
    Json(todos_vec)
}

// Create a new todo
async fn create_todo(State(store): State<TodoStore>, Json(input): Json<CreateTodo>) -> (StatusCode, Json<Todo>) {
    let todo = Todo {
        id: Uuid::new_v4().to_string(),
        title: input.title,
        completed: false,
    };

    store.write().unwrap().insert(todo.id.clone(), todo.clone());

    (StatusCode::CREATED, Json(todo))
}

// Get a single todo
async fn get_todo(
    State(store): State<TodoStore>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<Todo>, StatusCode> {
    let todos = store.read().unwrap();
    todos.get(&id).cloned().map(Json).ok_or(StatusCode::NOT_FOUND)
}

// Update a todo
async fn update_todo(
    State(store): State<TodoStore>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(input): Json<UpdateTodo>,
) -> Result<Json<Todo>, StatusCode> {
    let mut todos = store.write().unwrap();

    if let Some(todo) = todos.get_mut(&id) {
        if let Some(title) = input.title {
            todo.title = title;
        }
        if let Some(completed) = input.completed {
            todo.completed = completed;
        }
        Ok(Json(todo.clone()))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

// Delete a todo
async fn delete_todo(
    State(store): State<TodoStore>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> StatusCode {
    let mut todos = store.write().unwrap();
    if todos.remove(&id).is_some() {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}

// ============================================================================
// LESSON 3: Multiple State Types
// ============================================================================

/// When you need multiple independent state types, combine them
#[derive(Clone)]
#[allow(dead_code)] // Fields shown for demonstration
struct CombinedState {
    config: Arc<AppConfig>,
    todos: TodoStore,
    metrics: Arc<RwLock<Metrics>>,
}

#[derive(Debug, Default)]
struct Metrics {
    request_count: u64,
    error_count: u64,
}

// You can extract the whole state or use From traits for convenience
async fn get_metrics(State(state): State<CombinedState>) -> Json<serde_json::Value> {
    let metrics = state.metrics.read().unwrap();
    Json(serde_json::json!({
        "requests": metrics.request_count,
        "errors": metrics.error_count,
        "app_version": state.config.version
    }))
}

async fn increment_request_count(State(state): State<CombinedState>) -> &'static str {
    let mut metrics = state.metrics.write().unwrap();
    metrics.request_count += 1;
    "Request counted!"
}

// ============================================================================
// LESSON 4: Database Connection Pool Pattern
// ============================================================================

/// Simulating a database connection pool
/// In a real app, this would be `sqlx::PgPool` or similar
#[derive(Clone)]
#[allow(dead_code)] // Fields shown for demonstration
struct DbPool {
    connection_string: String,
    max_connections: u32,
}

impl DbPool {
    fn new(connection_string: &str) -> Self {
        Self {
            connection_string: connection_string.to_string(),
            max_connections: 10,
        }
    }

    // Simulated query
    #[allow(clippy::unused_async)]
    async fn query(&self, _sql: &str) -> Result<Vec<String>, String> {
        // In real app: sqlx::query!(...).fetch_all(&self.pool).await
        Ok(vec!["result1".to_string(), "result2".to_string()])
    }
}

async fn db_query(State(pool): State<DbPool>) -> Json<Vec<String>> {
    match pool.query("SELECT * FROM users").await {
        Ok(results) => Json(results),
        Err(_) => Json(vec![]),
    }
}

// ============================================================================
// LESSON 5: State with Extension Pattern
// ============================================================================

/// Sometimes you want to add state dynamically (e.g., from middleware)
use axum::Extension;

#[derive(Clone)]
struct CurrentUser {
    id: String,
    name: String,
}

async fn get_current_user(Extension(user): Extension<CurrentUser>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "id": user.id,
        "name": user.name
    }))
}

// ============================================================================
// MAIN
// ============================================================================

#[tokio::main]
async fn main() {
    // Initialize immutable config
    let config = Arc::new(AppConfig {
        app_name: "Axum Todo API".to_string(),
        version: "1.0.0".to_string(),
        max_items_per_page: 100,
    });

    // Initialize mutable todo store
    let todo_store: TodoStore = Arc::new(RwLock::new(HashMap::new()));

    // Pre-populate with some todos
    {
        let mut store = todo_store.write().unwrap();
        let todo = Todo {
            id: Uuid::new_v4().to_string(),
            title: "Learn Axum".to_string(),
            completed: false,
        };
        store.insert(todo.id.clone(), todo);
    }

    // Combined state for complex apps
    let combined_state = CombinedState {
        config: config.clone(),
        todos: todo_store.clone(),
        metrics: Arc::new(RwLock::new(Metrics::default())),
    };

    // Simulated DB pool
    let db_pool = DbPool::new("postgres://localhost/myapp");

    // Current user (normally set by auth middleware)
    let current_user = CurrentUser {
        id: "user-123".to_string(),
        name: "Demo User".to_string(),
    };

    // Build routes for todo CRUD
    let todo_routes = Router::new()
        .route("/", get(list_todos).post(create_todo))
        .route("/{id}", get(get_todo).put(update_todo).delete(delete_todo))
        .with_state(todo_store);

    // Build main app
    let app = Router::new()
        // Config endpoint
        .route("/config", get(get_config))
        .with_state(config)
        // Merge todo routes
        .merge(Router::new().nest("/todos", todo_routes))
        // Metrics endpoints
        .route("/metrics", get(get_metrics))
        .route("/track", get(increment_request_count))
        .with_state(combined_state)
        // Database endpoint
        .route("/db/users", get(db_query))
        .with_state(db_pool)
        // Extension-based state
        .route("/me", get(get_current_user))
        .layer(Extension(current_user));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind");

    println!("🚀 Module 05: State Management");
    println!("   Server running on http://localhost:3000");
    println!();
    println!("📝 Todo CRUD Endpoints:");
    println!("   GET    /todos      - List all todos");
    println!("   POST   /todos      - Create todo");
    println!("   GET    /todos/:id  - Get single todo");
    println!("   PUT    /todos/:id  - Update todo");
    println!("   DELETE /todos/:id  - Delete todo");
    println!();
    println!("📝 Other Endpoints:");
    println!("   GET /config   - App configuration");
    println!("   GET /metrics  - Request metrics");
    println!("   GET /me       - Current user (Extension)");
    println!();
    println!("💡 Try: curl -X POST -H 'Content-Type: application/json' \\");
    println!("        -d '{{\"title\":\"New Todo\"}}' http://localhost:3000/todos");

    axum::serve(listener, app).await.expect("Server failed");
}
