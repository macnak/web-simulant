// Endpoint request handler

use crate::config::{
	BodyMatchType, DistributionParams, DistributionType, Endpoint, ErrorProfile, LatencyConfig,
	RequestMatch,
};
use crate::distributions::{
	Distribution, ExponentialDistribution, FixedDistribution, NormalDistribution, UniformDistribution,
};
use crate::engine::response::{build_plain_text, build_response};
use axum::response::Response;
use rand::Rng;
use std::time::Duration;

pub async fn handle_request(endpoint: &Endpoint, request_body: &str) -> Response {
	if !request_matches(request_body, endpoint.request.as_ref()) {
		return build_plain_text(400, "Request body did not match");
	}

	let delay = sample_latency(&endpoint.latency);
	if delay > Duration::from_millis(0) {
		tokio::time::sleep(delay).await;
	}

	if should_error(&endpoint.error_profile) {
		let status = pick_error_status(&endpoint.error_profile);
		let body = endpoint.error_profile.body.as_str();
		return build_response(status, &endpoint.response.headers, body);
	}

	build_response(
		endpoint.response.status,
		&endpoint.response.headers,
		&endpoint.response.body,
	)
}

fn request_matches(body: &str, request: Option<&RequestMatch>) -> bool {
	let Some(request) = request else {
		return true;
	};

	match request.body_match {
		BodyMatchType::Any | BodyMatchType::Ignore => true,
		BodyMatchType::Exact => request.body.as_deref().map_or(false, |expected| expected == body),
		BodyMatchType::Contains => request.body.as_deref().map_or(false, |expected| body.contains(expected)),
	}
}

fn sample_latency(latency: &LatencyConfig) -> Duration {
	match (&latency.distribution, &latency.params) {
		(DistributionType::Fixed, DistributionParams::Fixed { delay_ms }) => {
			FixedDistribution::new(*delay_ms).sample()
		}
		(DistributionType::Normal, DistributionParams::Normal { mean_ms, stddev_ms }) => {
			NormalDistribution::new(*mean_ms, *stddev_ms).sample()
		}
		(DistributionType::Exponential, DistributionParams::Exponential { rate }) => {
			ExponentialDistribution::new(*rate).sample()
		}
		(DistributionType::Uniform, DistributionParams::Uniform { min_ms, max_ms }) => {
			UniformDistribution::new(*min_ms, *max_ms).sample()
		}
		_ => Duration::from_millis(0),
	}
}

fn should_error(profile: &ErrorProfile) -> bool {
	if profile.rate <= 0.0 {
		return false;
	}
	if profile.rate >= 1.0 {
		return true;
	}

	let roll: f64 = rand::thread_rng().gen();
	roll < profile.rate
}

fn pick_error_status(profile: &ErrorProfile) -> u16 {
	if profile.codes.is_empty() {
		return 500;
	}

	let idx = rand::thread_rng().gen_range(0..profile.codes.len());
	profile.codes[idx]
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::config::{DistributionParams, DistributionType, Response as ConfigResponse};
	use std::collections::HashMap;

	fn base_endpoint() -> Endpoint {
		Endpoint {
			id: "health".to_string(),
			method: crate::config::HttpMethod::Get,
			path: "/health".to_string(),
			request: None,
			latency: LatencyConfig {
				distribution: DistributionType::Fixed,
				params: DistributionParams::Fixed { delay_ms: 0.0 },
			},
			response: ConfigResponse {
				status: 200,
				headers: HashMap::new(),
				body: "ok".to_string(),
			},
			error_profile: ErrorProfile::default(),
		}
	}

	#[tokio::test]
	async fn test_handle_request_success() {
		let endpoint = base_endpoint();
		let response = handle_request(&endpoint, "").await;
		assert_eq!(response.status(), axum::http::StatusCode::OK);
	}

	#[tokio::test]
	async fn test_handle_request_error_rate_one() {
		let mut endpoint = base_endpoint();
		endpoint.error_profile.rate = 1.0;
		endpoint.error_profile.codes = vec![503];
		endpoint.error_profile.body = "error".to_string();

		let response = handle_request(&endpoint, "").await;
		assert_eq!(response.status(), axum::http::StatusCode::SERVICE_UNAVAILABLE);
	}

	#[tokio::test]
	async fn test_handle_request_body_match_exact() {
		let mut endpoint = base_endpoint();
		endpoint.request = Some(RequestMatch {
			body_match: BodyMatchType::Exact,
			body: Some("ping".to_string()),
		});

		let response = handle_request(&endpoint, "pong").await;
		assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST);
	}
}
