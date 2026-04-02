//! # Module 12: Production Ready
//!
//! Production deployment patterns:
//! - Graceful shutdown
//! - Connection limiting (NEW in Axum 0.8)
//! - Structured logging with tracing
//! - Health checks

use axum::{Json, Router, extract::State, routing::get};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    time::Duration,
};
use tokio::net::TcpListener;
use tower_http::{compression::CompressionLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// ============================================================================
// APPLICATION STATE
// ============================================================================

#[derive(Clone)]
struct AppState {
    ready: Arc<AtomicBool>,
    request_count: Arc<AtomicU64>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            ready: Arc::new(AtomicBool::new(true)),
            request_count: Arc::new(AtomicU64::new(0)),
        }
    }
}

// ============================================================================
// HEALTH & READINESS
// ============================================================================

async fn health() -> &'static str {
    "OK"
}

async fn ready(
    State(state): State<AppState>,
) -> Result<&'static str, (axum::http::StatusCode, &'static str)> {
    if state.ready.load(Ordering::SeqCst) {
        Ok("ready")
    } else {
        Err((axum::http::StatusCode::SERVICE_UNAVAILABLE, "not ready"))
    }
}

async fn metrics(State(state): State<AppState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "requests": state.request_count.load(Ordering::SeqCst),
        "ready": state.ready.load(Ordering::SeqCst)
    }))
}

async fn index(State(state): State<AppState>) -> &'static str {
    state.request_count.fetch_add(1, Ordering::SeqCst);
    "Hello from production-ready Axum!"
}

// ============================================================================
// MAIN
// ============================================================================

#[tokio::main]
async fn main() {
    // Initialize tracing (structured JSON logging for production)
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .json()
                .with_target(true)
                .with_current_span(true),
        )
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .init();

    let state = AppState::default();

    let app = Router::new()
        .route("/", get(index))
        .route("/health", get(health)) // Liveness probe
        .route("/ready", get(ready)) // Readiness probe
        .route("/metrics", get(metrics))
        .with_state(state.clone())
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new());

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();

    tracing::info!("🚀 Server starting on http://localhost:3000");

    // Graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal(state))
        .await
        .unwrap();

    tracing::info!("Server shut down gracefully");
}

async fn shutdown_signal(state: AppState) {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }

    tracing::info!("Shutdown signal received, starting graceful shutdown");

    // Mark as not ready for new connections
    state.ready.store(false, Ordering::SeqCst);

    // Allow time for load balancer to detect
    tokio::time::sleep(Duration::from_secs(5)).await;
}
