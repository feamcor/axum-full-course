//! # Module 06: Middleware & Layers
//!
//! Tower middleware integration in Axum:
//! - Built-in middleware (CORS, Compression, Timeout)
//! - Custom middleware with `from_fn`
//! - Route-specific layers

use axum::{
    Router,
    extract::Request,
    http::{HeaderValue, Method, StatusCode, header},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
};
use std::time::{Duration, Instant};
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing::Level;

// ============================================================================
// LESSON 1: Custom Middleware with from_fn
// ============================================================================

/// Logging middleware - logs every request
async fn logging_middleware(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let start = Instant::now();

    let response = next.run(request).await;

    tracing::info!(
        method = %method,
        uri = %uri,
        status = %response.status().as_u16(),
        duration_ms = %start.elapsed().as_millis(),
        "Request completed"
    );
    response
}

/// Timing middleware - adds X-Response-Time header
async fn timing_middleware(request: Request, next: Next) -> Response {
    let start = Instant::now();
    let mut response = next.run(request).await;

    response.headers_mut().insert(
        "X-Response-Time",
        HeaderValue::from_str(&format!("{}ms", start.elapsed().as_millis())).unwrap(),
    );
    response
}

/// Authentication middleware
async fn auth_middleware(request: Request, next: Next) -> Result<Response, StatusCode> {
    let auth_header = request
        .headers()
        .get("X-API-Key")
        .and_then(|v| v.to_str().ok());

    match auth_header {
        Some("secret-key") => Ok(next.run(request).await),
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}

// ============================================================================
// LESSON 2: Built-in Tower-HTTP Middleware
// ============================================================================

fn cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION])
}

// ============================================================================
// Handlers
// ============================================================================

async fn index() -> &'static str {
    "Welcome to Axum Middleware Module!"
}

async fn public_data() -> impl IntoResponse {
    axum::Json(serde_json::json!({"message": "Public data", "accessible": true}))
}

async fn protected_data() -> impl IntoResponse {
    axum::Json(serde_json::json!({"message": "Secret data", "authorized": true}))
}

async fn slow_endpoint() -> &'static str {
    tokio::time::sleep(Duration::from_secs(1)).await;
    "Slow operation done!"
}

// ============================================================================
// MAIN
// ============================================================================

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    // Protected routes (require auth)
    let protected = Router::new()
        .route("/data", get(protected_data))
        .route_layer(middleware::from_fn(auth_middleware));

    // Main app with layered middleware
    let app = Router::new()
        .route("/", get(index))
        .route("/public", get(public_data))
        .route("/slow", get(slow_endpoint))
        .nest("/protected", protected)
        .layer(middleware::from_fn(timing_middleware))
        .layer(middleware::from_fn(logging_middleware))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(cors_layer())
                .layer(CompressionLayer::new()),
        );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("🚀 Module 06: Middleware & Layers");
    println!("   Server: http://localhost:3000");
    println!("\n📝 Endpoints:");
    println!("   GET /              - Welcome");
    println!("   GET /public        - Public data");
    println!("   GET /slow          - Slow endpoint");
    println!("   GET /protected/data - Auth required (X-API-Key: secret-key)");

    axum::serve(listener, app).await.unwrap();
}
