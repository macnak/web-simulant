// Endpoint registry

use crate::config::{
	BehaviorWindow, BurstEvent, Configuration, Endpoint, EndpointGroup, HttpMethod, TokenBucket,
};
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

#[derive(Debug, Clone)]
pub struct EndpointBehaviors {
	pub windows: Vec<BehaviorWindow>,
	pub bursts: Vec<BurstEvent>,
}

#[derive(Debug, Clone)]
pub struct ResolvedEndpoint {
	pub endpoint: Endpoint,
	pub behaviors: EndpointBehaviors,
}

#[derive(Debug, Default)]
struct RegistryInner {
	endpoints: Vec<ResolvedEndpoint>,
	by_route: HashMap<RouteKey, ResolvedEndpoint>,
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

	pub fn set_config(&self, config: Configuration) {
		let loaded_at = Instant::now();
		let group_index = build_group_index(&config.endpoint_groups);
		let mut resolved = Vec::new();

		for mut endpoint in config.endpoints {
			endpoint.loaded_at = Some(loaded_at);
			endpoint.rate_limiter = endpoint.rate_limit.as_ref().map(|limit| {
				Arc::new(Mutex::new(TokenBucket::new(
					limit.requests_per_second,
					limit.burst,
				)))
			});

			let behaviors = resolve_behaviors(
				&endpoint,
				&config.behavior_windows,
				&config.burst_events,
				&group_index,
			);

			resolved.push(ResolvedEndpoint {
				endpoint,
				behaviors,
			});
		}

		let mut by_route = HashMap::new();
		for item in &resolved {
			let key = RouteKey::new(&item.endpoint.method, &item.endpoint.path);
			by_route.insert(key, item.clone());
		}

		let mut inner = self.inner.write().expect("registry write lock");
		inner.endpoints = resolved;
		inner.by_route = by_route;
	}

	#[allow(dead_code)]
	pub fn set_endpoints(&self, endpoints: Vec<Endpoint>) {
		let loaded_at = Instant::now();
		let mut resolved = Vec::new();
		for mut endpoint in endpoints {
			endpoint.loaded_at = Some(loaded_at);
			endpoint.rate_limiter = endpoint.rate_limit.as_ref().map(|limit| {
				Arc::new(Mutex::new(TokenBucket::new(
					limit.requests_per_second,
					limit.burst,
				)))
			});

			resolved.push(ResolvedEndpoint {
				endpoint,
				behaviors: EndpointBehaviors {
					windows: vec![],
					bursts: vec![],
				},
			});
		}

		let mut by_route = HashMap::new();
		for item in &resolved {
			let key = RouteKey::new(&item.endpoint.method, &item.endpoint.path);
			by_route.insert(key, item.clone());
		}

		let mut inner = self.inner.write().expect("registry write lock");
		inner.endpoints = resolved;
		inner.by_route = by_route;
	}

	pub fn get(&self, method: &HttpMethod, path: &str) -> Option<ResolvedEndpoint> {
		let inner = self.inner.read().expect("registry read lock");
		let key = RouteKey::new(method, path);
		inner.by_route.get(&key).cloned()
	}

	#[allow(dead_code)]
	pub fn list(&self) -> Vec<ResolvedEndpoint> {
		let inner = self.inner.read().expect("registry read lock");
		inner.endpoints.clone()
	}
}

fn build_group_index(groups: &[EndpointGroup]) -> HashMap<String, Vec<String>> {
	let mut index = HashMap::new();
	for group in groups {
		index.insert(group.id.clone(), group.endpoint_ids.clone());
	}
	index
}

fn resolve_behaviors(
	endpoint: &Endpoint,
	behavior_windows: &[BehaviorWindow],
	burst_events: &[BurstEvent],
	group_index: &HashMap<String, Vec<String>>,
) -> EndpointBehaviors {
	let mut windows = Vec::new();
	let mut bursts = Vec::new();

	for window in behavior_windows {
		if scope_matches(endpoint, &window.scope, group_index) {
			windows.push(window.clone());
		}
	}

	for burst in burst_events {
		if scope_matches(endpoint, &burst.scope, group_index) {
			bursts.push(burst.clone());
		}
	}

	EndpointBehaviors { windows, bursts }
}

fn scope_matches(
	endpoint: &Endpoint,
	scope: &crate::config::BehaviorScope,
	group_index: &HashMap<String, Vec<String>>,
) -> bool {
	if scope.global {
		return true;
	}
	if let Some(endpoint_id) = &scope.endpoint_id {
		return endpoint_id == &endpoint.id;
	}
	if let Some(group_id) = &scope.group_id {
		if let Some(ids) = group_index.get(group_id) {
			return ids.iter().any(|id| id == &endpoint.id);
		}
	}
	false
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
		assert_eq!(list[0].endpoint.id, "health");
	}
}
