// Request routing logic

use crate::config::HttpMethod;
use crate::engine::{EndpointRegistry, ResolvedEndpoint};
use std::sync::Arc;

/// Match method + path to a configured endpoint
#[allow(dead_code)]
pub fn match_route(
	registry: Arc<EndpointRegistry>,
	method: &HttpMethod,
	path: &str,
) -> Option<ResolvedEndpoint> {
	registry.get(method, path)
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::config::{
		DistributionParams, DistributionType, Endpoint, ErrorProfile, LatencyConfig, Response,
	};
	use std::collections::HashMap;

	fn endpoint(id: &str, method: HttpMethod, path: &str) -> Endpoint {
		Endpoint {
			id: id.to_string(),
			method,
			path: path.to_string(),
			request: None,
			latency: LatencyConfig {
				distribution: DistributionType::Fixed,
				params: DistributionParams::Fixed { delay_ms: 1.0 },
			},
			response: Response {
				status: 200,
				headers: HashMap::new(),
				body: "{}".to_string(),
			},
			error_profile: ErrorProfile::default(),
			rate_limit: None,
			bandwidth_cap: None,
			loaded_at: None,
			rate_limiter: None,
		}
	}

	#[test]
	fn test_match_route_hit() {
		let registry = Arc::new(EndpointRegistry::new());
		registry.set_endpoints(vec![endpoint("health", HttpMethod::Get, "/health")]);
		let matched = match_route(registry, &HttpMethod::Get, "/health");
		assert!(matched.is_some());
		let matched = matched.unwrap();
		assert_eq!(matched.endpoint.id, "health");
	}

	#[test]
	fn test_match_route_miss() {
		let registry = Arc::new(EndpointRegistry::new());
		registry.set_endpoints(vec![endpoint("health", HttpMethod::Get, "/health")]);
		let matched = match_route(registry, &HttpMethod::Post, "/health");
		assert!(matched.is_none());
	}

}
