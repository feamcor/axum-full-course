//! # Module 04: Response Types
//!
//! Learn how to return different types of responses in Axum:
//! - Simple types (String, &str)
//! - JSON responses
//! - HTML responses
//! - Custom response types
//! - Status codes and headers
//! - The `IntoResponse` trait

use axum::Router;
use axum::body::Body;
use axum::http::HeaderMap;
use axum::http::HeaderValue;
use axum::http::StatusCode;
use axum::http::header;
use axum::response::Html;
use axum::response::IntoResponse;
use axum::response::Json;
use axum::response::Redirect;
use axum::response::Response;
use axum::routing::get;
use serde::Serialize;

// ============================================================================
// LESSON 1: Simple Response Types
// ============================================================================

/// Returning a &'static str
async fn static_string() -> &'static str {
    "Hello from static string!"
}

/// Returning an owned String
async fn owned_string() -> String {
    format!("Hello at timestamp: {}", chrono_lite())
}

fn chrono_lite() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// Returning a tuple with status code
async fn with_status() -> (StatusCode, &'static str) {
    (StatusCode::CREATED, "Resource created successfully!")
}

// ============================================================================
// LESSON 2: JSON Responses
// ============================================================================

#[derive(Serialize)]
struct User {
    id: u64,
    name: String,
    email: String,
    active: bool,
}

async fn json_user() -> Json<User> {
    Json(User {
        id: 1,
        name: "John Doe".to_string(),
        email: "john@example.com".to_string(),
        active: true,
    })
}

/// Returning a list of users
#[derive(Serialize)]
struct UsersResponse {
    users: Vec<User>,
    total: usize,
    page: u32,
}

async fn json_users() -> Json<UsersResponse> {
    let users = vec![
        User {
            id: 1,
            name: "John".to_string(),
            email: "john@example.com".to_string(),
            active: true,
        },
        User {
            id: 2,
            name: "Jane".to_string(),
            email: "jane@example.com".to_string(),
            active: true,
        },
    ];
    let total = users.len();
    Json(UsersResponse { users, total, page: 1 })
}

/// JSON with custom status code
async fn json_with_status() -> (StatusCode, Json<User>) {
    (
        StatusCode::CREATED,
        Json(User {
            id: 3,
            name: "New User".to_string(),
            email: "new@example.com".to_string(),
            active: true,
        }),
    )
}

// ============================================================================
// LESSON 3: HTML Responses
// ============================================================================

async fn html_page() -> Html<&'static str> {
    Html(
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Axum HTML Response</title>
            <style>
                body {
                    font-family: system-ui, sans-serif;
                    max-width: 800px;
                    margin: 50px auto;
                    padding: 20px;
                    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                    min-height: 100vh;
                }
                .card {
                    background: white;
                    border-radius: 12px;
                    padding: 30px;
                    box-shadow: 0 10px 40px rgba(0,0,0,0.2);
                }
                h1 { color: #333; }
                p { color: #666; line-height: 1.6; }
            </style>
        </head>
        <body>
            <div class="card">
                <h1>🦀 Welcome to Axum!</h1>
                <p>This is an HTML response from your Axum server.</p>
                <p>You can return full HTML pages, templates, or fragments.</p>
            </div>
        </body>
        </html>
        "#,
    )
}

/// Dynamic HTML
#[allow(clippy::format_collect, clippy::needless_raw_string_hashes)]
async fn dynamic_html() -> Html<String> {
    let items = ["Routing", "Extractors", "Responses", "Middleware"];
    let list_items: String = items.iter().map(|item| format!("<li>{item}</li>")).collect();

    Html(format!(
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Axum Course Modules</title>
            <style>
                body {{ font-family: system-ui; padding: 20px; }}
                ul {{ list-style-type: none; padding: 0; }}
                li {{
                    padding: 10px 15px;
                    margin: 5px 0;
                    background: #f0f0f0;
                    border-radius: 5px;
                }}
            </style>
        </head>
        <body>
            <h1>Course Topics</h1>
            <ul>{list_items}</ul>
        </body>
        </html>
        "#
    ))
}

// ============================================================================
// LESSON 4: Custom Response with Headers
// ============================================================================

async fn with_headers() -> (HeaderMap, &'static str) {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/plain; charset=utf-8"),
    );
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("max-age=3600"));
    headers.insert("X-Custom-Header", HeaderValue::from_static("Hello!"));

    (headers, "Response with custom headers")
}

/// Status + headers + body
async fn full_response() -> (StatusCode, HeaderMap, &'static str) {
    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("text/plain"));
    headers.insert("X-Request-Id", HeaderValue::from_static("12345"));

    (StatusCode::OK, headers, "Full control over the response!")
}

// ============================================================================
// LESSON 5: Redirects
// ============================================================================

async fn redirect_permanent() -> Redirect {
    Redirect::permanent("/new-location")
}

async fn redirect_temporary() -> Redirect {
    Redirect::temporary("/temp-location")
}

async fn redirect_see_other() -> Redirect {
    // Commonly used after form submissions
    Redirect::to("/success")
}

async fn new_location() -> &'static str {
    "You've been redirected here!"
}

// ============================================================================
// LESSON 6: The IntoResponse Trait
// ============================================================================

/// Custom response type implementing `IntoResponse`
struct CustomResponse {
    message: String,
    status: StatusCode,
}

impl IntoResponse for CustomResponse {
    fn into_response(self) -> Response {
        let body = format!(
            r#"{{"message": "{}", "status": {}}}"#,
            self.message,
            self.status.as_u16()
        );

        Response::builder()
            .status(self.status)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body))
            .unwrap()
    }
}

async fn custom_response() -> CustomResponse {
    CustomResponse {
        message: "This is a custom response type!".to_string(),
        status: StatusCode::OK,
    }
}

/// API Response wrapper for consistent JSON responses
#[derive(Serialize)]
struct ApiResponse<T: Serialize> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
}

impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> Response {
        let status = if self.success {
            StatusCode::OK
        } else {
            StatusCode::BAD_REQUEST
        };

        (status, Json(self)).into_response()
    }
}

async fn api_success() -> ApiResponse<User> {
    ApiResponse {
        success: true,
        data: Some(User {
            id: 1,
            name: "John".to_string(),
            email: "john@example.com".to_string(),
            active: true,
        }),
        error: None,
    }
}

async fn api_error() -> ApiResponse<()> {
    ApiResponse {
        success: false,
        data: None,
        error: Some("Something went wrong".to_string()),
    }
}

// ============================================================================
// LESSON 7: Either/Result Response Types
// ============================================================================

/// Handlers can return Result for error handling
async fn maybe_error() -> Result<Json<User>, (StatusCode, String)> {
    let success = true; // Toggle this to see different responses

    if success {
        Ok(Json(User {
            id: 1,
            name: "Success User".to_string(),
            email: "success@example.com".to_string(),
            active: true,
        }))
    } else {
        Err((StatusCode::NOT_FOUND, "User not found".to_string()))
    }
}

// ============================================================================
// MAIN
// ============================================================================

#[tokio::main]
async fn main() {
    let app = Router::new()
        // Simple responses
        .route("/string", get(static_string))
        .route("/owned", get(owned_string))
        .route("/status", get(with_status))
        // JSON responses
        .route("/json/user", get(json_user))
        .route("/json/users", get(json_users))
        .route("/json/created", get(json_with_status))
        // HTML responses
        .route("/html", get(html_page))
        .route("/html/dynamic", get(dynamic_html))
        // Headers
        .route("/headers", get(with_headers))
        .route("/full", get(full_response))
        // Redirects
        .route("/redirect/permanent", get(redirect_permanent))
        .route("/redirect/temp", get(redirect_temporary))
        .route("/redirect/other", get(redirect_see_other))
        .route("/new-location", get(new_location))
        .route("/temp-location", get(new_location))
        .route("/success", get(|| async { "Form submitted successfully!" }))
        // Custom responses
        .route("/custom", get(custom_response))
        .route("/api/success", get(api_success))
        .route("/api/error", get(api_error))
        // Result type
        .route("/maybe-error", get(maybe_error));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind");

    println!("🚀 Module 04: Response Types");
    println!("   Server running on http://localhost:3000");
    println!();
    println!("📝 Endpoints:");
    println!("   GET /string            - Static string");
    println!("   GET /json/user         - JSON user object");
    println!("   GET /json/users        - JSON array");
    println!("   GET /html              - Beautiful HTML page");
    println!("   GET /headers           - Custom headers");
    println!("   GET /redirect/permanent - Redirect example");
    println!("   GET /custom            - Custom IntoResponse");
    println!("   GET /api/success       - API wrapper success");
    println!("   GET /api/error         - API wrapper error");

    axum::serve(listener, app).await.expect("Server failed");
}
