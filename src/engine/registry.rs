// Endpoint registry

use crate::config::{Endpoint, HttpMethod, TokenBucket};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Instant;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
struct RouteKey {
	method: String,
	path: String,
}

impl RouteKey {
	fn new(method: &HttpMethod, path: &str) -> Self {
		Self {
			method: method_to_string(method).to_string(),
			path: path.to_string(),
		}
	}
}

#[derive(Debug, Default)]
struct RegistryInner {
	endpoints: Vec<Endpoint>,
	by_route: HashMap<RouteKey, Endpoint>,
}

#[derive(Debug, Default)]
pub struct EndpointRegistry {
	inner: RwLock<RegistryInner>,
}

impl EndpointRegistry {
	pub fn new() -> Self {
		Self {
			inner: RwLock::new(RegistryInner::default()),
		}
	}

	pub fn set_endpoints(&self, endpoints: Vec<Endpoint>) {
		let loaded_at = Instant::now();
		let mut endpoints = endpoints;
		for endpoint in &mut endpoints {
			endpoint.loaded_at = Some(loaded_at);
			endpoint.rate_limiter = endpoint.rate_limit.as_ref().map(|limit| {
				Arc::new(Mutex::new(TokenBucket::new(
					limit.requests_per_second,
					limit.burst,
				)))
			});
		}

		let mut by_route = HashMap::new();
		for endpoint in &endpoints {
			let key = RouteKey::new(&endpoint.method, &endpoint.path);
			by_route.insert(key, endpoint.clone());
		}

		let mut inner = self.inner.write().expect("registry write lock");
		inner.endpoints = endpoints;
		inner.by_route = by_route;
	}

	pub fn get(&self, method: &HttpMethod, path: &str) -> Option<Endpoint> {
		let inner = self.inner.read().expect("registry read lock");
		let key = RouteKey::new(method, path);
		inner.by_route.get(&key).cloned()
	}

	pub fn list(&self) -> Vec<Endpoint> {
		let inner = self.inner.read().expect("registry read lock");
		inner.endpoints.clone()
	}
}

fn method_to_string(method: &HttpMethod) -> &'static str {
	match method {
		HttpMethod::Get => "GET",
		HttpMethod::Post => "POST",
		HttpMethod::Put => "PUT",
		HttpMethod::Delete => "DELETE",
		HttpMethod::Patch => "PATCH",
		HttpMethod::Head => "HEAD",
		HttpMethod::Options => "OPTIONS",
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::config::{DistributionParams, DistributionType, ErrorProfile, LatencyConfig, Response};
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
			behavior_windows: vec![],
			loaded_at: None,
			rate_limiter: None,
		}
	}

	#[test]
	fn test_registry_lookup() {
		let registry = EndpointRegistry::new();
		registry.set_endpoints(vec![
			endpoint("health", HttpMethod::Get, "/health"),
			endpoint("create", HttpMethod::Post, "/items"),
		]);

		let found = registry.get(&HttpMethod::Get, "/health");
		assert!(found.is_some());

		let missing = registry.get(&HttpMethod::Get, "/missing");
		assert!(missing.is_none());
	}

	#[test]
	fn test_registry_list() {
		let registry = EndpointRegistry::new();
		registry.set_endpoints(vec![endpoint("health", HttpMethod::Get, "/health")]);
		let list = registry.list();
		assert_eq!(list.len(), 1);
		assert_eq!(list[0].id, "health");
	}
}
