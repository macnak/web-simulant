// Web Simulant - Main Entry Point
//
// Starts both the engine (port 8080) and control plane (port 8081)

use anyhow::Result;
use tracing::info;
use std::sync::Arc;

mod config;
mod distributions;
mod engine;
mod control_plane;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    info!("Web Simulant starting...");
    info!("Version: {}", env!("CARGO_PKG_VERSION"));

    let registry = Arc::new(engine::EndpointRegistry::new());
    let control_state = control_plane::default_state(registry.clone());

    if let Ok(Some(config)) = control_plane::load_config(&control_state.config_path) {
        registry.set_endpoints(config.endpoints.clone());
        let mut guard = control_state.config.write().expect("config write lock");
        *guard = Some(config);
    }

    let (engine_shutdown_tx, engine_shutdown_rx) = tokio::sync::oneshot::channel::<()>();
    let (control_shutdown_tx, control_shutdown_rx) = tokio::sync::oneshot::channel::<()>();

    let engine_handle = tokio::spawn(engine::run_engine(registry, async {
        let _ = engine_shutdown_rx.await;
    }));

    let control_handle = tokio::spawn(control_plane::run_control_plane(control_state, async {
        let _ = control_shutdown_rx.await;
    }));

    info!("Web Simulant ready");
    info!("Engine: http://localhost:8080");
    info!("Control Plane: http://localhost:8081");

    // Keep running until interrupted
    tokio::signal::ctrl_c().await?;
    info!("Shutting down...");
    let _ = engine_shutdown_tx.send(());
    let _ = control_shutdown_tx.send(());
    let _ = engine_handle.await;
    let _ = control_handle.await;

    Ok(())
}
