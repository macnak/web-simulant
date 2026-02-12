// Control plane API handlers

use crate::config::{
	parse_auto, parse_json, parse_yaml, validate, Configuration, ConfigError, Endpoint, ValidationError,
};
use crate::control_plane::persistence::save_config;
use crate::engine::EndpointRegistry;
use axum::body::Bytes;
use axum::extract::{Multipart, Path, Query, State};
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct ControlPlaneState {
	pub registry: Arc<EndpointRegistry>,
	pub config: Arc<RwLock<Option<Configuration>>>,
	pub config_path: PathBuf,
}

#[derive(Serialize)]
struct EndpointSummary {
	id: String,
	method: String,
	path: String,
	latency: serde_json::Value,
	error_rate: f64,
	response_status: u16,
}

#[derive(Serialize)]
struct StatusResponse {
	status: &'static str,
	version: String,
	config_loaded: bool,
	endpoints_count: usize,
}


pub async fn health() -> impl IntoResponse {
	axum::Json(json!({"status": "ok"}))
}

pub async fn status(State(state): State<ControlPlaneState>) -> impl IntoResponse {
	let config = state.config.read().expect("config read lock");
	let endpoints_count = config.as_ref().map(|c| c.endpoints.len()).unwrap_or(0);
	let response = StatusResponse {
		status: "ok",
		version: env!("CARGO_PKG_VERSION").to_string(),
		config_loaded: config.is_some(),
		endpoints_count,
	};
	axum::Json(response)
}

pub async fn list_endpoints(State(state): State<ControlPlaneState>) -> impl IntoResponse {
	let config = state.config.read().expect("config read lock");
	if let Some(config) = config.as_ref() {
		let endpoints = config.endpoints.iter().map(to_summary).collect::<Vec<_>>();
		axum::Json(json!({
			"status": "success",
			"endpoints_count": endpoints.len(),
			"endpoints": endpoints
		}))
	} else {
		axum::Json(json!({
			"status": "success",
			"endpoints_count": 0,
			"endpoints": []
		}))
	}
}

pub async fn get_endpoint(
	State(state): State<ControlPlaneState>,
	Path(endpoint_id): Path<String>,
) -> impl IntoResponse {
	let config = state.config.read().expect("config read lock");
	if let Some(config) = config.as_ref() {
		if let Some(endpoint) = config.endpoints.iter().find(|e| e.id == endpoint_id) {
			return axum::Json(json!({"status": "success", "endpoint": endpoint})).into_response();
		}
	}

	(axum::http::StatusCode::NOT_FOUND, axum::Json(json!({
		"status": "error",
		"message": "Endpoint not found"
	}))
	).into_response()
}

pub async fn create_endpoint(
	State(state): State<ControlPlaneState>,
	Json(endpoint): Json<Endpoint>,
) -> Response {
	let mut config = match current_config(&state) {
		Ok(config) => config,
		Err(response) => return response,
	};

	if config.endpoints.iter().any(|e| e.id == endpoint.id) {
		return conflict_response("Endpoint id already exists");
	}

	config.endpoints.push(endpoint.clone());
	apply_config_update(&state, config, "Endpoint created", Some(&endpoint))
}

pub async fn update_endpoint(
	State(state): State<ControlPlaneState>,
	Path(endpoint_id): Path<String>,
	Json(endpoint): Json<Endpoint>,
) -> Response {
	if endpoint.id != endpoint_id {
		return bad_request_response("Endpoint id in path does not match payload");
	}

	let mut config = match current_config(&state) {
		Ok(config) => config,
		Err(response) => return response,
	};

	let Some(existing) = config.endpoints.iter().position(|e| e.id == endpoint_id) else {
		return not_found_response("Endpoint not found");
	};

	config.endpoints[existing] = endpoint.clone();
	apply_config_update(&state, config, "Endpoint updated", Some(&endpoint))
}

pub async fn delete_endpoint(
	State(state): State<ControlPlaneState>,
	Path(endpoint_id): Path<String>,
) -> Response {
	let mut config = match current_config(&state) {
		Ok(config) => config,
		Err(response) => return response,
	};

	let before = config.endpoints.len();
	config.endpoints.retain(|e| e.id != endpoint_id);
	if config.endpoints.len() == before {
		return not_found_response("Endpoint not found");
	}

	apply_config_update(&state, config, "Endpoint deleted", None)
}

pub async fn validate_config(headers: HeaderMap, body: Bytes) -> Response {
	match parse_config_from_body(headers, body) {
		Ok(config) => match validate(&config) {
			Ok(_) => axum::Json(json!({
				"status": "valid",
				"message": "Configuration is valid",
				"summary": {
					"endpoints_count": config.endpoints.len(),
					"endpoints": config.endpoints.iter().map(to_summary).collect::<Vec<_>>()
				},
				"warnings": []
			})).into_response(),
			Err(err) => validation_error_response(err),
		},
		Err(err) => parse_error_response(err),
	}
}

pub async fn import_config(
	State(state): State<ControlPlaneState>,
	headers: HeaderMap,
	body: Bytes,
) -> Response {
	let config = match parse_config_from_body(headers, body) {
		Ok(config) => config,
		Err(err) => return parse_error_response(err),
	};

	if let Err(err) = validate(&config) {
		return validation_error_response(err);
	}

	if let Err(err) = save_config(&state.config_path, &config) {
		return parse_error_response(err);
	}

	state.registry.set_config(config.clone());
	let mut guard = state.config.write().expect("config write lock");
	*guard = Some(config.clone());

	let endpoints = config.endpoints.iter().map(to_summary).collect::<Vec<_>>();

	axum::Json(json!({
		"status": "success",
		"message": "Configuration loaded successfully",
		"summary": {
			"endpoints_loaded": endpoints.len(),
			"endpoints": endpoints
		},
		"metadata": config.metadata
	}))
	.into_response()
}

pub async fn import_config_multipart(
	State(state): State<ControlPlaneState>,
	mut multipart: Multipart,
) -> Response {
	let mut content_type = None;
	let mut content = None;

	while let Ok(Some(field)) = multipart.next_field().await {
		if content.is_none() {
			content_type = field.content_type().map(|ct| ct.to_string());
			if let Ok(bytes) = field.bytes().await {
				content = Some(bytes);
				break;
			}
		}
	}

	let Some(content) = content else {
		return (
			axum::http::StatusCode::BAD_REQUEST,
			axum::Json(json!({
				"status": "error",
				"message": "No configuration file provided",
				"previous_config_retained": true
			})),
		)
			.into_response();
	};

	let headers = content_type
		.map(|ct| {
			let mut headers = HeaderMap::new();
			headers.insert(axum::http::header::CONTENT_TYPE, ct.parse().unwrap());
			headers
		})
		.unwrap_or_default();

	import_config(State(state), headers, content).await
}

pub async fn export_config(
	State(state): State<ControlPlaneState>,
	Query(query): Query<HashMap<String, String>>,
) -> Response {
	let config = state.config.read().expect("config read lock");
	let Some(config) = config.as_ref() else {
		return (
			axum::http::StatusCode::NOT_FOUND,
			axum::Json(json!({"status": "error", "message": "No configuration currently loaded"})),
		)
			.into_response();
	};

	let format = query.get("format").map(String::as_str).unwrap_or("yaml");
	let (body, content_type, extension) = if format == "json" {
		let body = serde_json::to_string_pretty(config).unwrap_or_else(|_| "{}".to_string());
		(body, "application/json", "json")
	} else {
		let body = serde_yaml::to_string(config).unwrap_or_else(|_| "".to_string());
		(body, "application/x-yaml", "yaml")
	};

	let filename = format!("simulation-config-{}.{}", chrono::Utc::now().format("%Y-%m-%d"), extension);
	let mut response = axum::response::Response::new(axum::body::Body::from(body));
	response.headers_mut().insert(
		axum::http::header::CONTENT_TYPE,
		content_type.parse().unwrap(),
	);
	response.headers_mut().insert(
		axum::http::header::CONTENT_DISPOSITION,
		format!("attachment; filename=\"{}\"", filename).parse().unwrap(),
	);
	response
}

fn parse_config_from_body(headers: HeaderMap, body: Bytes) -> Result<Configuration, ConfigError> {
	let content = String::from_utf8_lossy(&body).to_string();
	let content_type = headers
		.get(axum::http::header::CONTENT_TYPE)
		.and_then(|value| value.to_str().ok())
		.unwrap_or("");

	if content_type.contains("json") {
		parse_json(&content)
	} else if content_type.contains("yaml") || content_type.contains("yml") {
		parse_yaml(&content)
	} else {
		parse_auto(&content)
	}
}

fn validation_error_response(err: ConfigError) -> Response {
	match err {
		ConfigError::ValidationError(_, errors) => {
			let errors = errors.into_iter().map(map_validation_error).collect::<Vec<_>>();
			(
				axum::http::StatusCode::BAD_REQUEST,
				axum::Json(json!({
					"status": "error",
					"message": "Configuration validation failed",
					"errors": errors,
					"previous_config_retained": true
				})),
			)
				.into_response()
		}
		_ => parse_error_response(err),
	}
}

fn parse_error_response(err: ConfigError) -> Response {
	(
		axum::http::StatusCode::BAD_REQUEST,
		axum::Json(json!({
			"status": "error",
			"message": "Failed to parse configuration file",
			"errors": [{"error": err.to_string()}],
			"previous_config_retained": true
		})),
	)
		.into_response()
}

fn current_config(state: &ControlPlaneState) -> Result<Configuration, Response> {
	let config = state.config.read().expect("config read lock");
	if let Some(config) = config.as_ref() {
		Ok(config.clone())
	} else {
		Err(not_found_response("No configuration currently loaded"))
	}
}

fn apply_config_update(
	state: &ControlPlaneState,
	config: Configuration,
	message: &str,
	endpoint: Option<&Endpoint>,
) -> Response {
	if let Err(err) = validate(&config) {
		return validation_error_response(err);
	}

	if let Err(err) = save_config(&state.config_path, &config) {
		return parse_error_response(err);
	}

		state.registry.set_config(config.clone());
	let mut guard = state.config.write().expect("config write lock");
	*guard = Some(config.clone());

	let summary = endpoint.map(to_summary);
	let response = json!({
		"status": "success",
		"message": message,
		"endpoint": summary,
		"endpoints_count": config.endpoints.len()
	});
	axum::Json(response).into_response()
}

fn not_found_response(message: &str) -> Response {
	(
		axum::http::StatusCode::NOT_FOUND,
		axum::Json(json!({
			"status": "error",
			"message": message
		})),
	)
		.into_response()
}

fn bad_request_response(message: &str) -> Response {
	(
		axum::http::StatusCode::BAD_REQUEST,
		axum::Json(json!({
			"status": "error",
			"message": message
		})),
	)
		.into_response()
}

fn conflict_response(message: &str) -> Response {
	(
		axum::http::StatusCode::CONFLICT,
		axum::Json(json!({
			"status": "error",
			"message": message
		})),
	)
		.into_response()
}

fn map_validation_error(error: ValidationError) -> serde_json::Value {
	let mut value = json!({
		"field": error.field,
		"error": error.error
	});
	if let Some(location) = error.location {
		value["location"] = json!(location);
	}
	value
}


fn distribution_name(distribution: &crate::config::DistributionType) -> &'static str {
	match distribution {
		crate::config::DistributionType::Fixed => "fixed",
		crate::config::DistributionType::Normal => "normal",
		crate::config::DistributionType::Exponential => "exponential",
		crate::config::DistributionType::Uniform => "uniform",
		crate::config::DistributionType::LogNormal => "log_normal",
		crate::config::DistributionType::Mixture => "mixture",
	}
}

fn distribution_params(params: &crate::config::DistributionParams) -> serde_json::Value {
	match params {
		crate::config::DistributionParams::Fixed { delay_ms } => json!({
			"delay_ms": delay_ms
		}),
		crate::config::DistributionParams::Normal { mean_ms, stddev_ms } => json!({
			"mean_ms": mean_ms,
			"stddev_ms": stddev_ms
		}),
		crate::config::DistributionParams::Exponential { rate } => json!({
			"rate": rate
		}),
		crate::config::DistributionParams::Uniform { min_ms, max_ms } => json!({
			"min_ms": min_ms,
			"max_ms": max_ms
		}),
		crate::config::DistributionParams::LogNormal { mean_ms, stddev_ms } => json!({
			"mean_ms": mean_ms,
			"stddev_ms": stddev_ms
		}),
		crate::config::DistributionParams::Mixture { components } => {
			let components_json: Vec<serde_json::Value> = components
				.iter()
				.map(|component| {
					json!({
						"weight": component.weight,
						"distribution": distribution_name(&component.distribution),
						"params": distribution_params(component.params.as_ref())
					})
				})
				.collect();
			json!({"components": components_json})
		}
	}
}

fn to_summary(endpoint: &crate::config::Endpoint) -> EndpointSummary {
	let latency = json!({
		"distribution": distribution_name(&endpoint.latency.distribution),
		"params": distribution_params(&endpoint.latency.params)
	});

	EndpointSummary {
		id: endpoint.id.clone(),
		method: format!("{:?}", endpoint.method).to_uppercase(),
		path: endpoint.path.clone(),
		latency,
		error_rate: endpoint.error_profile.rate,
		response_status: endpoint.response.status,
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use axum::http::StatusCode;
	use tower::util::ServiceExt;

	fn state() -> ControlPlaneState {
		ControlPlaneState {
			registry: Arc::new(EndpointRegistry::new()),
			config: Arc::new(RwLock::new(None)),
			config_path: PathBuf::from("config/test.yaml"),
		}
	}

	#[tokio::test]
	async fn test_health() {
		let router = crate::control_plane::server::build_router(state());
		let response = router
			.oneshot(
				axum::http::Request::builder()
					.uri("/api/health")
					.body(axum::body::Body::empty())
					.unwrap(),
			)
			.await
			.unwrap();

		assert_eq!(response.status(), StatusCode::OK);
	}

	#[tokio::test]
	async fn test_status() {
		let router = crate::control_plane::server::build_router(state());
		let response = router
			.oneshot(
				axum::http::Request::builder()
					.uri("/api/status")
					.body(axum::body::Body::empty())
					.unwrap(),
			)
			.await
			.unwrap();

		assert_eq!(response.status(), StatusCode::OK);
	}

	#[tokio::test]
	async fn test_create_endpoint_without_config() {
		let router = crate::control_plane::server::build_router(state());
		let payload = serde_json::json!({
			"id": "new-endpoint",
			"method": "GET",
			"path": "/new",
			"latency": {"distribution": "fixed", "params": {"delay_ms": 1}},
			"response": {"status": 200, "headers": {"Content-Type": "application/json"}, "body": "{}"},
			"error_profile": {"rate": 0.0}
		});
		let response = router
			.oneshot(
				axum::http::Request::builder()
					.method("POST")
					.uri("/api/endpoints")
					.header("Content-Type", "application/json")
					.body(axum::body::Body::from(payload.to_string()))
					.unwrap(),
			)
			.await
			.unwrap();

		assert_eq!(response.status(), StatusCode::NOT_FOUND);
	}
}
