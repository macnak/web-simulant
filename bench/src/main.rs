use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug, Deserialize)]
struct SimulateRequest {
    latency_ms: u64,
    #[serde(default)]
    error_rate: f64,
}

#[derive(Debug, Serialize)]
struct SimulateResponse {
    message: String,
    latency_ms: u64,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
    latency_ms: u64,
}

#[derive(Clone)]
struct AppState {
    request_count: Arc<std::sync::atomic::AtomicU64>,
}

impl AppState {
    fn new() -> Self {
        Self {
            request_count: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        }
    }

    fn increment_requests(&self) -> u64 {
        self.request_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    }
}

async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

async fn simulate(
    State(state): State<AppState>,
    Json(payload): Json<SimulateRequest>,
) -> impl IntoResponse {
    let count = state.increment_requests();
    
    // Apply configured latency
    if payload.latency_ms > 0 {
        sleep(Duration::from_millis(payload.latency_ms)).await;
    }

    // Determine if this request should error based on error_rate
    let should_error = if payload.error_rate > 0.0 {
        // Use request count for deterministic error pattern
        let error_threshold = (payload.error_rate * 100.0) as u64;
        (count % 100) < error_threshold
    } else {
        false
    };

    if should_error {
        let error_response = ErrorResponse {
            error: "Simulated error".to_string(),
            latency_ms: payload.latency_ms,
        };
        (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)).into_response()
    } else {
        let response = SimulateResponse {
            message: "Success".to_string(),
            latency_ms: payload.latency_ms,
        };
        (StatusCode::OK, Json(response)).into_response()
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let state = AppState::new();

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/simulate", post(simulate))
        .with_state(state);

    let addr = "0.0.0.0:8080";
    log::info!("Starting benchmark server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address");

    axum::serve(listener, app)
        .await
        .expect("Server failed");
}
