// Engine HTTP server (port 8080)

use crate::config::HttpMethod;
use crate::engine::{handle_request, EndpointRegistry};
use crate::engine::response::build_plain_text;
use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::http::Method;
use axum::routing::any;
use axum::Router;
use std::future::Future;
use std::net::SocketAddr;
use std::sync::Arc;

#[derive(Clone)]
struct EngineState {
	registry: Arc<EndpointRegistry>,
}

pub fn build_router(registry: Arc<EndpointRegistry>) -> Router {
	let state = EngineState { registry };

	Router::new()
		.route("/*path", any(handle_all))
		.with_state(state)
}

pub async fn run_engine<F>(registry: Arc<EndpointRegistry>, shutdown: F) -> anyhow::Result<()>
where
	F: Future<Output = ()> + Send + 'static,
{
	let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
	let listener = tokio::net::TcpListener::bind(addr).await?;
	axum::serve(listener, build_router(registry))
		.with_graceful_shutdown(shutdown)
		.await?;
	Ok(())
}

async fn handle_all(
	State(state): State<EngineState>,
	method: Method,
	Path(path): Path<String>,
	body: Bytes,
) -> axum::response::Response {
	let method = match to_http_method(&method) {
		Some(method) => method,
		None => return build_plain_text(405, "Method not supported"),
	};

	let full_path = if path.is_empty() {
		"/".to_string()
	} else {
		format!("/{}", path)
	};

	let endpoint = state.registry.get(&method, &full_path);
	match endpoint {
		Some(endpoint) => {
			let request_body = String::from_utf8_lossy(&body).to_string();
			handle_request(&endpoint, &request_body).await
		}
		None => build_plain_text(404, "Not Found"),
	}
}

fn to_http_method(method: &Method) -> Option<HttpMethod> {
	match *method {
		Method::GET => Some(HttpMethod::Get),
		Method::POST => Some(HttpMethod::Post),
		Method::PUT => Some(HttpMethod::Put),
		Method::DELETE => Some(HttpMethod::Delete),
		Method::PATCH => Some(HttpMethod::Patch),
		Method::HEAD => Some(HttpMethod::Head),
		Method::OPTIONS => Some(HttpMethod::Options),
		_ => None,
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::config::{DistributionParams, DistributionType, Endpoint, ErrorProfile, LatencyConfig, Response};
	use std::collections::HashMap;
	use tower::util::ServiceExt;
	use axum::http::{Request, StatusCode};

	fn endpoint(id: &str, method: HttpMethod, path: &str) -> Endpoint {
		Endpoint {
			id: id.to_string(),
			method,
			path: path.to_string(),
			request: None,
			latency: LatencyConfig {
				distribution: DistributionType::Fixed,
				params: DistributionParams::Fixed { delay_ms: 0.0 },
			},
			response: Response {
				status: 200,
				headers: HashMap::new(),
				body: "ok".to_string(),
			},
			error_profile: ErrorProfile::default(),
			rate_limit: None,
			bandwidth_cap: None,
			loaded_at: None,
			rate_limiter: None,
		}
	}

	#[tokio::test]
	async fn test_router_match() {
		let registry = Arc::new(EndpointRegistry::new());
		registry.set_endpoints(vec![endpoint("health", HttpMethod::Get, "/health")]);
		let app = build_router(registry);

		let response = app
			.oneshot(Request::builder().method("GET").uri("/health").body(axum::body::Body::empty()).unwrap())
			.await
			.unwrap();

		assert_eq!(response.status(), StatusCode::OK);
	}

	#[tokio::test]
	async fn test_router_not_found() {
		let registry = Arc::new(EndpointRegistry::new());
		let app = build_router(registry);

		let response = app
			.oneshot(Request::builder().method("GET").uri("/missing").body(axum::body::Body::empty()).unwrap())
			.await
			.unwrap();

		assert_eq!(response.status(), StatusCode::NOT_FOUND);
	}

	#[tokio::test]
	async fn test_router_method_not_supported() {
		let registry = Arc::new(EndpointRegistry::new());
		let app = build_router(registry);

		let response = app
			.oneshot(Request::builder().method("TRACE").uri("/health").body(axum::body::Body::empty()).unwrap())
			.await
			.unwrap();

		assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
	}
}
