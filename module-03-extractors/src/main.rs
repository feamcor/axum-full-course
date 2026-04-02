//! # Module 03: Extractors Deep Dive
//!
//! Extractors are one of Axum's most powerful features. They allow you to
//! extract data from incoming requests in a type-safe way.
//!
//! Key topics:
//! - Built-in extractors (Path, Query, Json, Headers)
//! - NEW: `OptionalFromRequestParts` trait (Axum 0.8)
//! - NEW: No more `#[async_trait]` needed!
//! - Custom extractors
//! - Extractor ordering (important!)

use axum::Json;
use axum::Router;
use axum::body::Bytes;
use axum::extract::FromRequest;
use axum::extract::FromRequestParts;
use axum::extract::Path;
use axum::extract::Query;
use axum::extract::Request;
use axum::extract::State;
use axum::http::StatusCode;
use axum::http::header::HeaderMap;
use axum::http::request::Parts;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::routing::get;
use axum::routing::post;
use serde::Deserialize;
use serde::Serialize;
use std::sync::Arc;

// ============================================================================
// LESSON 1: Built-in Extractors
// ============================================================================

/// Path extractor - extracts path parameters
/// Route: GET /users/{id}
async fn get_user(Path(id): Path<u64>) -> String {
    format!("User ID: {id}")
}

/// Query extractor - extracts query string parameters
#[derive(Debug, Deserialize)]
struct ListParams {
    page: Option<u32>,
    limit: Option<u32>,
    sort: Option<String>,
}

async fn list_users(Query(params): Query<ListParams>) -> String {
    format!(
        "Page: {}, Limit: {}, Sort: {}",
        params.page.unwrap_or(1),
        params.limit.unwrap_or(10),
        params.sort.unwrap_or_else(|| "id".to_string())
    )
}

/// Json extractor - extracts and deserializes JSON body
#[derive(Debug, Deserialize)]
struct CreateUserRequest {
    name: String,
    email: String,
}

#[derive(Debug, Serialize)]
struct CreateUserResponse {
    id: u64,
    name: String,
    email: String,
}

async fn create_user(Json(payload): Json<CreateUserRequest>) -> Json<CreateUserResponse> {
    Json(CreateUserResponse {
        id: 1,
        name: payload.name,
        email: payload.email,
    })
}

/// Headers extractor - access request headers
async fn show_headers(headers: HeaderMap) -> String {
    let user_agent = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("Unknown");

    let content_type = headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("Not specified");

    format!("User-Agent: {user_agent}\nContent-Type: {content_type}")
}

/// Raw body extractor
async fn raw_body(body: Bytes) -> String {
    format!("Received {} bytes", body.len())
}

// ============================================================================
// LESSON 2: Multiple Extractors
// ============================================================================

/// You can use multiple extractors in a single handler
/// ORDER MATTERS! Extractors are applied in parameter order
/// Body-consuming extractors (Json, String, Bytes) must come LAST
async fn combined_extractors(
    Path(id): Path<u64>,
    Query(params): Query<ListParams>,
    headers: HeaderMap,
    Json(body): Json<CreateUserRequest>, // Must be last!
) -> String {
    format!(
        "ID: {}\nPage: {:?}\nUser-Agent: {:?}\nName: {}",
        id,
        params.page,
        headers.get("user-agent"),
        body.name
    )
}

// ============================================================================
// LESSON 3: Optional Extractors - NEW IN AXUM 0.8!
// ============================================================================

/// In Axum 0.8, Option<T> as an extractor requires T to implement
/// `OptionalFromRequestParts` or `OptionalFromRequest`
///
/// This allows better error handling - rejections can be turned into
/// error responses instead of being silently ignored.
///
/// Built-in types like Query and `HeaderMap` already implement this.
async fn optional_query(Query(params): Query<Option<ListParams>>) -> String {
    match params {
        Some(p) => format!("Got params: page={:?}", p.page),
        None => "No query params provided".to_string(),
    }
}

// ============================================================================
// LESSON 4: Custom Extractor - NO MORE #[async_trait]!
// ============================================================================

/// In Axum 0.8, you don't need #[`async_trait`] anymore!
/// Rust now supports `impl Future<Output = _>` in traits natively.
/// A custom extractor for API keys
struct ApiKey(String);

/// Custom error type for our extractor
#[derive(Debug)]
struct ApiKeyError;

impl IntoResponse for ApiKeyError {
    fn into_response(self) -> Response {
        (
            StatusCode::UNAUTHORIZED,
            "Missing or invalid API key. Provide X-API-Key header.",
        )
            .into_response()
    }
}

// NEW IN AXUM 0.8: No #[async_trait] needed!
impl<S> FromRequestParts<S> for ApiKey
where
    S: Send + Sync,
{
    type Rejection = ApiKeyError;

    // Notice: we return `impl Future` instead of using #[async_trait]
    fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send {
        // We use async block to create the future
        let api_key = parts
            .headers
            .get("x-api-key")
            .and_then(|v| v.to_str().ok())
            .map(ToString::to_string);

        async move {
            match api_key {
                Some(key) if !key.is_empty() => Ok(ApiKey(key)),
                _ => Err(ApiKeyError),
            }
        }
    }
}

async fn protected_endpoint(ApiKey(key): ApiKey) -> String {
    format!("Access granted! Your API key: {key}")
}

// ============================================================================
// LESSON 5: Custom Extractor with Body
// ============================================================================

/// A custom extractor that validates JSON body
#[derive(Debug, Deserialize)]
struct ValidatedUser {
    name: String,
    email: String,
}

struct ValidatedJson<T>(T);

#[derive(Debug)]
enum ValidationError {
    InvalidJson(String),
    InvalidEmail,
    NameTooShort,
}

impl IntoResponse for ValidationError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ValidationError::InvalidJson(e) => (StatusCode::BAD_REQUEST, format!("Invalid JSON: {e}")),
            ValidationError::InvalidEmail => (StatusCode::BAD_REQUEST, "Invalid email format".to_string()),
            ValidationError::NameTooShort => (
                StatusCode::BAD_REQUEST,
                "Name must be at least 2 characters".to_string(),
            ),
        };
        (status, message).into_response()
    }
}

// Custom extractor that validates the request body
// Note: For body extractors, we implement FromRequest instead of FromRequestParts
impl<S> FromRequest<S> for ValidatedJson<ValidatedUser>
where
    S: Send + Sync,
{
    type Rejection = ValidationError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        // Extract JSON first
        let Json(user): Json<ValidatedUser> = Json::from_request(req, state)
            .await
            .map_err(|e| ValidationError::InvalidJson(e.to_string()))?;

        // Validate name length
        if user.name.len() < 2 {
            return Err(ValidationError::NameTooShort);
        }

        // Validate email (simple check)
        if !user.email.contains('@') {
            return Err(ValidationError::InvalidEmail);
        }

        Ok(ValidatedJson(user))
    }
}

async fn create_validated_user(ValidatedJson(user): ValidatedJson<ValidatedUser>) -> String {
    format!("Created user: {} <{}>", user.name, user.email)
}

// ============================================================================
// LESSON 6: State as Extractor
// ============================================================================

#[derive(Clone)]
struct AppState {
    db_pool: String, // In real app, this would be a database pool
    api_version: String,
}

async fn with_state(State(state): State<Arc<AppState>>) -> String {
    format!("API Version: {}, DB: {}", state.api_version, state.db_pool)
}

// ============================================================================
// MAIN: Putting It All Together
// ============================================================================

#[tokio::main]
async fn main() {
    // Create shared state
    let state = Arc::new(AppState {
        db_pool: "postgres://localhost/mydb".to_string(),
        api_version: "v1.0.0".to_string(),
    });

    let app = Router::new()
        // Built-in extractors
        .route("/users/{id}", get(get_user))
        .route("/users", get(list_users).post(create_user))
        .route("/headers", get(show_headers))
        .route("/raw", post(raw_body))
        // Multiple extractors
        .route("/users/{id}/update", post(combined_extractors))
        // Optional extractors
        .route("/optional", get(optional_query))
        // Custom extractors
        .route("/protected", get(protected_endpoint))
        .route("/validated", post(create_validated_user))
        // State extractor
        .route("/state", get(with_state))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind");

    println!("🚀 Module 03: Extractors Deep Dive");
    println!("   Server running on http://localhost:3000");
    println!();
    println!("📝 Built-in Extractors:");
    println!("   GET  /users/123          - Path extractor");
    println!("   GET  /users?page=2       - Query extractor");
    println!("   POST /users              - Json extractor");
    println!("   GET  /headers            - Headers extractor");
    println!();
    println!("📝 Custom Extractors:");
    println!("   GET  /protected          - API key (Header: X-API-Key)");
    println!("   POST /validated          - Validated JSON body");
    println!();
    println!("💡 Examples:");
    println!("   curl http://localhost:3000/users?page=2&limit=5");
    println!("   curl -H 'X-API-Key: secret123' http://localhost:3000/protected");
    println!("   curl -X POST -H 'Content-Type: application/json' \\");
    println!("        -d '{{\"name\":\"John\",\"email\":\"john@example.com\"}}' \\");
    println!("        http://localhost:3000/validated");

    axum::serve(listener, app).await.expect("Server failed");
}
