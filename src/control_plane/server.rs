// Control plane HTTP server (port 8081)

use crate::control_plane::handlers::{
	create_endpoint, delete_endpoint, export_config, get_endpoint, health, import_config,
	import_config_multipart, list_endpoints, status, update_endpoint, validate_config,
	ControlPlaneState,
};
use crate::engine::EndpointRegistry;
use axum::routing::{get, get_service, post};
use axum::Router;
use std::future::Future;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use tower_http::services::{ServeDir, ServeFile};

pub fn build_router(state: ControlPlaneState) -> Router {
	let static_service = ServeDir::new("static");

	Router::new()
		.route("/", get_service(ServeFile::new("static/index.html")))
		.route("/api/health", get(health))
		.route("/api/status", get(status))
		.route("/api/endpoints", get(list_endpoints).post(create_endpoint))
		.route(
			"/api/endpoints/:id",
			get(get_endpoint).put(update_endpoint).delete(delete_endpoint),
		)
		.route("/api/config/export", get(export_config))
		.route("/api/config/validate", post(validate_config))
		.route("/api/config/import", post(import_config))
		.route("/api/config/import/multipart", post(import_config_multipart))
		.nest_service("/static", static_service)
		.with_state(state)
}

pub fn default_state(registry: Arc<EndpointRegistry>) -> ControlPlaneState {
	ControlPlaneState {
		registry,
		config: Arc::new(RwLock::new(None)),
		config_path: PathBuf::from("config/active.yaml"),
	}
}

pub async fn run_control_plane<F>(state: ControlPlaneState, shutdown: F) -> anyhow::Result<()>
where
	F: Future<Output = ()> + Send + 'static,
{
	let addr = SocketAddr::from(([0, 0, 0, 0], 8081));
	let listener = tokio::net::TcpListener::bind(addr).await?;
	axum::serve(listener, build_router(state))
		.with_graceful_shutdown(shutdown)
		.await?;
	Ok(())
}
