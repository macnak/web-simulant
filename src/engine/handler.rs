// Endpoint request handler

use crate::config::{
	BandwidthCap, BehaviorWindow, BodyMatchType, DistributionParams, DistributionType, Endpoint,
	ErrorProfile, LatencyConfig, MixtureComponent, RequestMatch,
};
use crate::distributions::{
	Distribution, ExponentialDistribution, FixedDistribution, LogNormalDistribution, NormalDistribution,
	UniformDistribution,
};
use crate::engine::response::{build_plain_text, build_response};
use axum::response::Response;
use rand::Rng;
use std::time::Duration;

pub async fn handle_request(endpoint: &Endpoint, request_body: &str) -> Response {
	if !request_matches(request_body, endpoint.request.as_ref()) {
		return build_plain_text(400, "Request body did not match");
	}

	if !check_rate_limit(endpoint) {
		return build_plain_text(429, "Rate limit exceeded");
	}

	let (latency_config, error_profile) = effective_behavior(endpoint);
	let delay = sample_latency(latency_config);
	if delay > Duration::from_millis(0) {
		tokio::time::sleep(delay).await;
	}

	let mut status = endpoint.response.status;
	let mut body = endpoint.response.body.as_str();

	if should_error(error_profile) {
		if error_profile.error_in_payload {
			if !error_profile.body.is_empty() {
				body = error_profile.body.as_str();
			}
		} else {
			status = pick_error_status(error_profile);
			body = error_profile.body.as_str();
		}
	}

	let final_body = apply_payload_corruption(body, error_profile);

	let bandwidth_delay = compute_bandwidth_delay(&final_body, endpoint.bandwidth_cap.as_ref());
	if bandwidth_delay > Duration::from_millis(0) {
		tokio::time::sleep(bandwidth_delay).await;
	}

	build_response(status, &endpoint.response.headers, &final_body)
}

fn effective_behavior(endpoint: &Endpoint) -> (&LatencyConfig, &ErrorProfile) {
	if let Some(window) = active_window(endpoint) {
		let latency = window.latency_override.as_ref().unwrap_or(&endpoint.latency);
		let error_profile = window
			.error_profile_override
			.as_ref()
			.unwrap_or(&endpoint.error_profile);
		return (latency, error_profile);
	}

	(&endpoint.latency, &endpoint.error_profile)
}

fn active_window(endpoint: &Endpoint) -> Option<&BehaviorWindow> {
	let loaded_at = endpoint.loaded_at?;
	let elapsed_ms = loaded_at.elapsed().as_millis() as f64;

	for window in &endpoint.behavior_windows {
		if elapsed_ms >= window.start_offset_ms && elapsed_ms < window.end_offset_ms {
			return Some(window);
		}
	}

	None
}

fn check_rate_limit(endpoint: &Endpoint) -> bool {
	let Some(limiter) = endpoint.rate_limiter.as_ref() else {
		return true;
	};

	let mut limiter = limiter.lock().expect("rate limiter lock");
	limiter.try_take()
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
	sample_latency_with(&latency.distribution, &latency.params)
}

fn sample_latency_with(distribution: &DistributionType, params: &DistributionParams) -> Duration {
	match (distribution, params) {
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
		(DistributionType::LogNormal, DistributionParams::LogNormal { mean_ms, stddev_ms }) => {
			LogNormalDistribution::new(*mean_ms, *stddev_ms).sample()
		}
		(DistributionType::Mixture, DistributionParams::Mixture { components }) => {
			sample_mixture(components)
		}
		_ => Duration::from_millis(0),
	}
}

fn sample_mixture(components: &[MixtureComponent]) -> Duration {
	let mut total_weight = 0.0;
	for component in components {
		if component.weight.is_finite() && component.weight > 0.0 {
			total_weight += component.weight;
		}
	}

	if total_weight <= 0.0 {
		return Duration::from_millis(0);
	}

	let mut roll: f64 = rand::thread_rng().gen::<f64>() * total_weight;
	for component in components {
		if !component.weight.is_finite() || component.weight <= 0.0 {
			continue;
		}
		if component.distribution == DistributionType::Mixture {
			continue;
		}
		if roll < component.weight {
			return sample_latency_with(&component.distribution, component.params.as_ref());
		}
		roll -= component.weight;
	}

	Duration::from_millis(0)
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

fn apply_payload_corruption(body: &str, profile: &ErrorProfile) -> String {
	let Some(corruption) = &profile.payload_corruption else {
		return body.to_string();
	};

	if corruption.rate <= 0.0 {
		return body.to_string();
	}

	let roll: f64 = rand::thread_rng().gen();
	if roll >= corruption.rate {
		return body.to_string();
	}

	match corruption.mode {
		crate::config::CorruptionMode::Truncate => {
			let ratio = corruption.truncate_ratio.unwrap_or(0.5);
			let ratio = ratio.clamp(0.0, 1.0);
			let len = (body.len() as f64 * ratio).floor() as usize;
			body[..len.min(body.len())].to_string()
		}
		crate::config::CorruptionMode::Replace => {
			corruption
				.replacement
				.as_deref()
				.unwrap_or("")
				.to_string()
		}
	}
}

fn compute_bandwidth_delay(body: &str, cap: Option<&BandwidthCap>) -> Duration {
	let Some(cap) = cap else {
		return Duration::from_millis(0);
	};

	if cap.bytes_per_second <= 0.0 || !cap.bytes_per_second.is_finite() {
		return Duration::from_millis(0);
	}

	let bytes = body.as_bytes().len() as f64;
	let seconds = bytes / cap.bytes_per_second;
	Duration::from_secs_f64(seconds)
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
			rate_limit: None,
			bandwidth_cap: None,
			behavior_windows: vec![],
			loaded_at: None,
			rate_limiter: None,
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
