//! # Module 01: Introduction to Axum
//!
//! This module covers the basics of setting up an Axum web server.
//! We'll learn about:
//! - Creating a basic server with `axum::serve`
//! - Defining route handlers
//! - Understanding the Router type
//! - Basic request/response flow

use axum::{
    Router,
    routing::{get, post},
};

// ============================================================================
// LESSON 1: Your First Handler
// ============================================================================

/// The simplest possible handler - just returns a string
///
/// In Axum, a handler is any async function that returns something
/// implementing `IntoResponse`. Strings automatically implement this!
async fn hello_world() -> &'static str {
    "Hello, World! 🦀"
}

/// Handlers can return different types
/// Here we return a String (owned) instead of &str (borrowed)
async fn hello_axum() -> String {
    format!("Welcome to Axum {}!", env!("CARGO_PKG_VERSION"))
}

// ============================================================================
// LESSON 2: The Health Check Pattern
// ============================================================================

/// A common pattern - health check endpoint for monitoring
///
/// This is essential for:
/// - Load balancers to check if your service is alive
/// - Kubernetes liveness/readiness probes
/// - Monitoring systems
async fn health_check() -> &'static str {
    "OK"
}

// ============================================================================
// LESSON 3: Multiple Response Types
// ============================================================================

/// You can return tuples for more control
/// (`StatusCode`, headers, body) or (`StatusCode`, body)
use axum::http::StatusCode;

async fn with_status() -> (StatusCode, &'static str) {
    (StatusCode::CREATED, "Resource created!")
}

/// Return different status codes based on conditions
async fn conditional_response() -> (StatusCode, &'static str) {
    let is_working = true;

    if is_working {
        (StatusCode::OK, "Everything is working!")
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, "Service is down")
    }
}

// ============================================================================
// LESSON 4: POST Handlers (Preview of Extractors)
// ============================================================================

/// A simple POST handler that echoes back the body
/// We'll learn more about extractors in Module 03
async fn echo(body: String) -> String {
    format!("You sent: {body}")
}

// ============================================================================
// MAIN: Building the Router and Serving
// ============================================================================

#[tokio::main]
async fn main() {
    // Build our application router
    //
    // The Router is the core of Axum - it maps paths to handlers
    // You can chain multiple routes together using the builder pattern
    let app = Router::new()
        // Basic GET routes
        .route("/", get(hello_world))
        .route("/hello", get(hello_axum))
        .route("/health", get(health_check))
        // Routes with different status codes
        .route("/created", get(with_status))
        .route("/status", get(conditional_response))
        // POST route (we'll explore this more later)
        .route("/echo", post(echo));

    // Create a TCP listener
    //
    // In Axum 0.8+, we use `axum::serve` instead of hyper::Server
    // This is the new, simplified way to run an Axum application
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind to port 3000");

    println!("🚀 Module 01: Introduction to Axum");
    println!("   Server running on http://localhost:3000");
    println!();
    println!("📝 Try these endpoints:");
    println!("   GET  /        - Hello World");
    println!("   GET  /hello   - Welcome message");
    println!("   GET  /health  - Health check");
    println!("   GET  /created - Status code example");
    println!("   POST /echo    - Echo back your message");
    println!();
    println!("💡 Example: curl http://localhost:3000/hello");
    println!("💡 Example: curl -X POST -d 'Hello!' http://localhost:3000/echo");

    // Start serving requests
    //
    // `axum::serve` is the new function in Axum 0.8+
    // It replaces the old hyper::Server approach
    axum::serve(listener, app)
        .await
        .expect("Server failed to start");
}
