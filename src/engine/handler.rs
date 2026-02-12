// Endpoint request handler

use crate::config::{
	BandwidthCap, BehaviorSchedule, BehaviorWindow, BodyMatchType, BurstEvent, DistributionParams,
	DistributionType, Endpoint, ErrorMix, ErrorProfile, LatencyConfig, MixtureComponent,
	RampConfig, RampCurve, RequestMatch, ScheduleMode,
};
use crate::distributions::{
	Distribution, ExponentialDistribution, FixedDistribution, LogNormalDistribution, NormalDistribution,
	UniformDistribution,
};
use crate::engine::response::{build_plain_text, build_response};
use crate::engine::{EndpointBehaviors, ResolvedEndpoint};
use axum::response::Response;
use rand::Rng;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::Duration;

pub async fn handle_request(resolved: &ResolvedEndpoint, request_body: &str) -> Response {
	let endpoint = &resolved.endpoint;
	let behaviors = &resolved.behaviors;
	if !request_matches(request_body, endpoint.request.as_ref()) {
		return build_plain_text(400, "Request body did not match");
	}

	if !check_rate_limit(endpoint) {
		return build_plain_text(429, "Rate limit exceeded");
	}

	let elapsed_ms = elapsed_ms(endpoint);
	let delay = sample_latency_with_behaviors(endpoint, behaviors, elapsed_ms);
	if delay > Duration::from_millis(0) {
		tokio::time::sleep(delay).await;
	}

	let mut status = endpoint.response.status;
	let mut body = endpoint.response.body.as_str();

	let error_profile = effective_error_profile(endpoint, behaviors, elapsed_ms);
	if should_error(&error_profile) {
		if error_profile.error_in_payload {
			if !error_profile.body.is_empty() {
				body = error_profile.body.as_str();
			}
		} else {
			status = pick_error_status(&error_profile);
			body = error_profile.body.as_str();
		}
	}

	let final_body = apply_payload_corruption(body, &error_profile);

	let bandwidth_delay = compute_bandwidth_delay(&final_body, endpoint.bandwidth_cap.as_ref());
	if bandwidth_delay > Duration::from_millis(0) {
		tokio::time::sleep(bandwidth_delay).await;
	}

	build_response(status, &endpoint.response.headers, &final_body)
}

fn elapsed_ms(endpoint: &Endpoint) -> f64 {
	endpoint
		.loaded_at
		.map(|loaded_at| loaded_at.elapsed().as_millis() as f64)
		.unwrap_or(0.0)
}

fn sample_latency_with_behaviors(
	endpoint: &Endpoint,
	behaviors: &EndpointBehaviors,
	elapsed_ms: f64,
) -> Duration {
	let mut base_sample = sample_latency(&endpoint.latency);

	if let Some((window, factor)) = active_window(behaviors, elapsed_ms) {
		if let Some(latency_override) = window.latency_override.as_ref() {
			base_sample = blend_latency(&endpoint.latency, latency_override, factor);
		}
	}

	if let Some((burst, factor)) = active_burst(behaviors, elapsed_ms) {
		if let Some(latency_override) = burst.latency_spike.as_ref() {
			let roll: f64 = rand::thread_rng().gen();
			if factor >= 1.0 || roll < factor {
				return sample_latency(latency_override);
			}
		}
	}

	base_sample
}

fn blend_latency(base: &LatencyConfig, override_latency: &LatencyConfig, factor: f64) -> Duration {
	if factor <= 0.0 {
		return sample_latency(base);
	}
	if factor >= 1.0 {
		return sample_latency(override_latency);
	}
	let roll: f64 = rand::thread_rng().gen();
	if roll < factor {
		sample_latency(override_latency)
	} else {
		sample_latency(base)
	}
}

fn effective_error_profile(
	endpoint: &Endpoint,
	behaviors: &EndpointBehaviors,
	elapsed_ms: f64,
) -> ErrorProfile {
	let mut profile = endpoint.error_profile.clone();

	if let Some((window, factor)) = active_window(behaviors, elapsed_ms) {
		if let Some(override_profile) = window.error_profile_override.as_ref() {
			profile = merge_error_profiles(&profile, override_profile, window.error_mix.clone(), factor);
		}
	}

	if let Some((burst, factor)) = active_burst(behaviors, elapsed_ms) {
		if let Some(error_spike) = burst.error_spike.as_ref() {
			profile = merge_error_profiles(
				&profile,
				&error_spike.error_profile,
				error_spike.error_mix.clone(),
				factor,
			);
		}
	}

	profile
}

fn merge_error_profiles(
	base: &ErrorProfile,
	override_profile: &ErrorProfile,
	mix: ErrorMix,
	factor: f64,
) -> ErrorProfile {
	let factor = factor.clamp(0.0, 1.0);
	let (base_weight, override_weight) = match mix {
		ErrorMix::Override => (1.0 - factor, factor),
		ErrorMix::Additive => (1.0, factor),
		ErrorMix::Blend => (1.0 - factor, factor),
	};

	let base_rate = base.rate.max(0.0) * base_weight;
	let override_rate = override_profile.rate.max(0.0) * override_weight;
	let combined_rate = (base_rate + override_rate).min(1.0);

	let mut codes = base.codes.clone();
	for code in &override_profile.codes {
		if !codes.contains(code) {
			codes.push(*code);
		}
	}

	let body = if override_weight > 0.5 && !override_profile.body.is_empty() {
		override_profile.body.clone()
	} else {
		base.body.clone()
	};

	let error_in_payload = if override_weight > 0.5 {
		override_profile.error_in_payload
	} else {
		base.error_in_payload
	};

	let payload_corruption = if override_weight > 0.5 {
		override_profile.payload_corruption.clone().or_else(|| base.payload_corruption.clone())
	} else {
		base.payload_corruption.clone().or_else(|| override_profile.payload_corruption.clone())
	};

	ErrorProfile {
		rate: combined_rate,
		codes,
		body,
		error_in_payload,
		payload_corruption,
	}
}

fn active_window(
	behaviors: &EndpointBehaviors,
	elapsed_ms: f64,
) -> Option<(&BehaviorWindow, f64)> {
	for window in &behaviors.windows {
		if let Some((start_ms, end_ms)) = schedule_range(&window.schedule, elapsed_ms, window) {
			if elapsed_ms >= start_ms && elapsed_ms < end_ms {
				let factor = ramp_factor(elapsed_ms, start_ms, end_ms, window.ramp.as_ref());
				return Some((window, factor));
			}
		}
	}

	None
}

fn active_burst(
	behaviors: &EndpointBehaviors,
	elapsed_ms: f64,
) -> Option<(&BurstEvent, f64)> {
	for burst in &behaviors.bursts {
		let frequency = &burst.frequency;
		if let Some((start_ms, end_ms)) = burst_range(frequency, burst.duration_ms, elapsed_ms, burst) {
			if elapsed_ms >= start_ms && elapsed_ms < end_ms {
				let factor = ramp_factor(elapsed_ms, start_ms, end_ms, burst.ramp.as_ref());
				return Some((burst, factor));
			}
		}
	}

	None
}

fn schedule_range(
	schedule: &BehaviorSchedule,
	elapsed_ms: f64,
	window: &BehaviorWindow,
) -> Option<(f64, f64)> {
	match schedule.mode {
		ScheduleMode::Fixed => {
			let start_ms = schedule.start_offset_ms.unwrap_or(0.0);
			let end_ms = start_ms + schedule.duration_ms;
			Some((start_ms, end_ms))
		}
		ScheduleMode::Recurring => {
			let every_ms = schedule.every_ms.unwrap_or(0.0);
			if every_ms <= 0.0 {
				return None;
			}
			let min_delay_ms = schedule.min_delay_ms.unwrap_or(0.0);
			if elapsed_ms < min_delay_ms {
				return None;
			}
			let elapsed_since_start = elapsed_ms - min_delay_ms;
			let occurrence = (elapsed_since_start / every_ms).floor() as u32;
			if let Some(max_occurrences) = schedule.max_occurrences {
				if occurrence >= max_occurrences {
					return None;
				}
			}
			let base_start = min_delay_ms + (occurrence as f64 * every_ms);
			let jitter_ms = schedule.jitter_ms.unwrap_or(0.0);
			let jitter = jitter_for_window(window, occurrence, jitter_ms);
			let start_ms = (base_start + jitter).max(min_delay_ms).max(0.0);
			let end_ms = start_ms + schedule.duration_ms;
			Some((start_ms, end_ms))
		}
	}
}

fn burst_range(
	frequency: &crate::config::BurstFrequency,
	duration_ms: f64,
	elapsed_ms: f64,
	burst: &BurstEvent,
) -> Option<(f64, f64)> {
	let every_ms = frequency.every_ms;
	if every_ms <= 0.0 {
		return None;
	}
	let occurrence = (elapsed_ms / every_ms).floor() as u32;
	let base_start = occurrence as f64 * every_ms;
	let jitter_ms = frequency.jitter_ms.unwrap_or(0.0);
	let jitter = jitter_for_burst(burst, occurrence, jitter_ms);
	let start_ms = (base_start + jitter).max(0.0);
	let end_ms = start_ms + duration_ms;
	Some((start_ms, end_ms))
}

fn jitter_for_window(window: &BehaviorWindow, occurrence: u32, jitter_ms: f64) -> f64 {
	if jitter_ms <= 0.0 {
		return 0.0;
	}
	let mut hasher = DefaultHasher::new();
	window_key(window).hash(&mut hasher);
	occurrence.hash(&mut hasher);
	let hash = hasher.finish();
	let roll = (hash % 10_000) as f64 / 10_000.0;
	(roll * 2.0 - 1.0) * jitter_ms
}

fn jitter_for_burst(burst: &BurstEvent, occurrence: u32, jitter_ms: f64) -> f64 {
	if jitter_ms <= 0.0 {
		return 0.0;
	}
	let mut hasher = DefaultHasher::new();
	burst_key(burst).hash(&mut hasher);
	occurrence.hash(&mut hasher);
	let hash = hasher.finish();
	let roll = (hash % 10_000) as f64 / 10_000.0;
	(roll * 2.0 - 1.0) * jitter_ms
}

fn window_key(window: &BehaviorWindow) -> String {
	if let Some(id) = &window.id {
		return id.clone();
	}
	if window.scope.global {
		return "global".to_string();
	}
	if let Some(endpoint_id) = &window.scope.endpoint_id {
		return format!("endpoint:{}", endpoint_id);
	}
	if let Some(group_id) = &window.scope.group_id {
		return format!("group:{}", group_id);
	}
	"window".to_string()
}

fn burst_key(burst: &BurstEvent) -> String {
	if let Some(id) = &burst.id {
		return id.clone();
	}
	if burst.scope.global {
		return "global".to_string();
	}
	if let Some(endpoint_id) = &burst.scope.endpoint_id {
		return format!("endpoint:{}", endpoint_id);
	}
	if let Some(group_id) = &burst.scope.group_id {
		return format!("group:{}", group_id);
	}
	"burst".to_string()
}

fn ramp_factor(elapsed_ms: f64, start_ms: f64, end_ms: f64, ramp: Option<&RampConfig>) -> f64 {
	let Some(ramp) = ramp else {
		return 1.0;
	};
	let duration = end_ms - start_ms;
	if duration <= 0.0 {
		return 1.0;
	}
	let up_ms = ramp.up_ms.unwrap_or(0.0).max(0.0);
	let down_ms = ramp.down_ms.unwrap_or(0.0).max(0.0);
	let progress = elapsed_ms - start_ms;
	let mut factor = 1.0;
	if up_ms > 0.0 && progress < up_ms {
		factor = progress / up_ms;
	} else if down_ms > 0.0 && elapsed_ms > (end_ms - down_ms) {
		factor = (end_ms - elapsed_ms) / down_ms;
	}
	let factor = factor.clamp(0.0, 1.0);
	match ramp.curve {
		Some(RampCurve::SCurve) => smoothstep(factor),
		_ => factor,
	}
}

fn smoothstep(value: f64) -> f64 {
	let value = value.clamp(0.0, 1.0);
	value * value * (3.0 - 2.0 * value)
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
			loaded_at: None,
			rate_limiter: None,
		}
	}

	#[tokio::test]
	async fn test_handle_request_success() {
		let endpoint = base_endpoint();
		let resolved = ResolvedEndpoint {
			endpoint,
			behaviors: EndpointBehaviors {
				windows: vec![],
				bursts: vec![],
			},
		};
		let response = handle_request(&resolved, "").await;
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
