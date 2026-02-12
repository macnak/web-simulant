// Configuration validator
//
// Validates configuration against schema rules

use super::error::ConfigError;
use super::{
    BandwidthCap, BehaviorSchedule, BehaviorScope, BehaviorWindow, BodyMatchType, BurstEvent,
    Configuration, DistributionParams, DistributionType, Endpoint, EndpointGroup, ErrorMix,
    ErrorProfile, HttpMethod, LatencyConfig, MixtureComponent, RampConfig,
    RateLimit, RequestMatch, Response, ScheduleMode, ValidationError,
};
use std::collections::HashSet;

/// Validate a configuration
pub fn validate(config: &Configuration) -> Result<(), ConfigError> {
    let mut errors = Vec::new();

    if config.version != "1.0" {
        push_error(&mut errors, "version", "must be '1.0'", None);
    }

    if config.endpoints.is_empty() {
        push_error(&mut errors, "endpoints", "must contain at least one endpoint", None);
    }

    let mut ids = HashSet::new();
    let mut routes = HashSet::new();

    for endpoint in &config.endpoints {
        validate_endpoint(endpoint, &mut errors);

        if !ids.insert(endpoint.id.clone()) {
            push_error(
                &mut errors,
                "endpoints.id",
                "duplicate endpoint id",
                Some(endpoint.id.clone()),
            );
        }

        let route_key = format!("{} {}", method_to_str(&endpoint.method), endpoint.path);
        if !routes.insert(route_key.clone()) {
            push_error(
                &mut errors,
                "endpoints.method+path",
                "duplicate method and path combination",
                Some(route_key),
            );
        }
    }

    validate_endpoint_groups(&config.endpoint_groups, &ids, &mut errors);
    validate_behavior_windows(&config.behavior_windows, &config.endpoint_groups, &ids, &mut errors);
    validate_burst_events(&config.burst_events, &config.endpoint_groups, &ids, &mut errors);

    if !errors.is_empty() {
        let count = errors.len();
        return Err(ConfigError::ValidationError(count, errors));
    }

    Ok(())
}

fn validate_endpoint(endpoint: &Endpoint, errors: &mut Vec<ValidationError>) {
    let location = Some(endpoint.id.clone());

    if endpoint.id.trim().is_empty() {
        push_error(errors, "endpoints.id", "id must not be empty", location.clone());
    }

    if !endpoint.path.starts_with('/') {
        push_error(
            errors,
            "endpoints.path",
            "path must start with '/'",
            location.clone(),
        );
    }

    validate_latency(&endpoint.latency, errors, location.clone());
    validate_response(&endpoint.response, errors, location.clone());
    validate_error_profile(&endpoint.error_profile, errors, location.clone());
    validate_rate_limit(endpoint.rate_limit.as_ref(), errors, location.clone());
    validate_bandwidth_cap(endpoint.bandwidth_cap.as_ref(), errors, location.clone());
    validate_request_match(endpoint.request.as_ref(), errors, location);
}

fn validate_endpoint_groups(
    groups: &[EndpointGroup],
    endpoint_ids: &HashSet<String>,
    errors: &mut Vec<ValidationError>,
) {
    let mut group_ids = HashSet::new();

    for group in groups {
        if group.id.trim().is_empty() {
            push_error(errors, "endpoint_groups.id", "id must not be empty", Some(group.id.clone()));
        }

        if !group_ids.insert(group.id.clone()) {
            push_error(errors, "endpoint_groups.id", "duplicate group id", Some(group.id.clone()));
        }

        if group.endpoint_ids.is_empty() {
            push_error(errors, "endpoint_groups.endpoint_ids", "must include at least one endpoint id", Some(group.id.clone()));
        }

        for endpoint_id in &group.endpoint_ids {
            if !endpoint_ids.contains(endpoint_id) {
                push_error(
                    errors,
                    "endpoint_groups.endpoint_ids",
                    "endpoint id does not exist",
                    Some(endpoint_id.clone()),
                );
            }
        }
    }
}

fn validate_rate_limit(
    rate_limit: Option<&RateLimit>,
    errors: &mut Vec<ValidationError>,
    location: Option<String>,
) {
    let Some(rate_limit) = rate_limit else {
        return;
    };

    if !rate_limit.requests_per_second.is_finite() || rate_limit.requests_per_second <= 0.0 {
        push_error(
            errors,
            "rate_limit.requests_per_second",
            "must be > 0",
            location.clone(),
        );
    }

    if let Some(burst) = rate_limit.burst {
        if !burst.is_finite() || burst <= 0.0 {
            push_error(errors, "rate_limit.burst", "must be > 0", location);
        }
    }
}

fn validate_bandwidth_cap(
    bandwidth: Option<&BandwidthCap>,
    errors: &mut Vec<ValidationError>,
    location: Option<String>,
) {
    let Some(bandwidth) = bandwidth else {
        return;
    };

    if !bandwidth.bytes_per_second.is_finite() || bandwidth.bytes_per_second <= 0.0 {
        push_error(
            errors,
            "bandwidth_cap.bytes_per_second",
            "must be > 0",
            location,
        );
    }
}


fn validate_behavior_windows(
    windows: &[BehaviorWindow],
    groups: &[EndpointGroup],
    endpoint_ids: &HashSet<String>,
    errors: &mut Vec<ValidationError>,
) {
    let mut scope_fixed: Vec<(String, f64, f64)> = Vec::new();
    let mut scope_recurring = HashSet::new();
    let group_ids: HashSet<String> = groups.iter().map(|group| group.id.clone()).collect();

    for (index, window) in windows.iter().enumerate() {
        let scope_key = validate_scope(&window.scope, &group_ids, endpoint_ids, errors);

        if !validate_schedule(&window.schedule, errors, scope_key.as_deref()) {
            continue;
        }

        if let Some(latency) = &window.latency_override {
            validate_latency(latency, errors, scope_key.clone());
        }

        if let Some(profile) = &window.error_profile_override {
            validate_error_profile(profile, errors, scope_key.clone());
        }

        if window.error_mix == ErrorMix::Additive && window.error_profile_override.is_none() {
            push_error(
                errors,
                "behavior_windows.error_profile_override",
                "error_profile_override required for additive mix",
                scope_key.clone(),
            );
        }

        match window.schedule.mode {
            ScheduleMode::Fixed => {
                let start_ms = window.schedule.start_offset_ms.unwrap_or(0.0);
                let end_ms = start_ms + window.schedule.duration_ms;
                if let Some(scope_key) = scope_key.as_ref() {
                    scope_fixed.push((scope_key.clone(), start_ms, end_ms));
                }
            }
            ScheduleMode::Recurring => {
                if let Some(scope_key) = scope_key.as_ref() {
                    if !scope_recurring.insert(scope_key.clone()) {
                        push_error(
                            errors,
                            "behavior_windows.schedule",
                            "multiple recurring windows for the same scope",
                            Some(scope_key.clone()),
                        );
                    }
                }
            }
        }

        if window.ramp.is_some() {
            validate_ramp(
                window.ramp.as_ref().unwrap(),
                "behavior_windows.ramp",
                errors,
                scope_key.clone(),
            );
        }

        if scope_key.is_none() {
            push_error(
                errors,
                "behavior_windows.scope",
                &format!("window {} scope is invalid", index),
                None,
            );
        }
    }

    scope_fixed.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal)));
    for window in scope_fixed.windows(2) {
        let (scope_a, _start_a, end_a) = (&window[0].0, window[0].1, window[0].2);
        let (scope_b, start_b, _end_b) = (&window[1].0, window[1].1, window[1].2);
        if scope_a == scope_b && start_b < end_a {
            push_error(
                errors,
                "behavior_windows.schedule",
                "fixed windows overlap for the same scope",
                Some(scope_a.clone()),
            );
        }
    }
}

fn validate_burst_events(
    bursts: &[BurstEvent],
    groups: &[EndpointGroup],
    endpoint_ids: &HashSet<String>,
    errors: &mut Vec<ValidationError>,
) {
    let group_ids: HashSet<String> = groups.iter().map(|group| group.id.clone()).collect();

    for burst in bursts {
        let scope_key = validate_scope(&burst.scope, &group_ids, endpoint_ids, errors);

        if !burst.duration_ms.is_finite() || burst.duration_ms <= 0.0 {
            push_error(
                errors,
                "burst_events.duration_ms",
                "duration_ms must be > 0",
                scope_key.clone(),
            );
        }

        if !burst.frequency.every_ms.is_finite() || burst.frequency.every_ms <= 0.0 {
            push_error(
                errors,
                "burst_events.frequency.every_ms",
                "every_ms must be > 0",
                scope_key.clone(),
            );
        }

        if let Some(jitter) = burst.frequency.jitter_ms {
            if !jitter.is_finite() || jitter < 0.0 {
                push_error(
                    errors,
                    "burst_events.frequency.jitter_ms",
                    "jitter_ms must be >= 0",
                    scope_key.clone(),
                );
            }
            if jitter > 0.0 && jitter > (burst.frequency.every_ms - burst.duration_ms) {
                push_error(
                    errors,
                    "burst_events.frequency.jitter_ms",
                    "jitter_ms must be <= every_ms - duration_ms",
                    scope_key.clone(),
                );
            }
        }

        if burst.duration_ms > burst.frequency.every_ms {
            push_error(
                errors,
                "burst_events.duration_ms",
                "duration_ms must be <= every_ms",
                scope_key.clone(),
            );
        }

        if let Some(latency) = &burst.latency_spike {
            validate_latency(latency, errors, scope_key.clone());
        }

        if let Some(error_spike) = &burst.error_spike {
            validate_error_profile(&error_spike.error_profile, errors, scope_key.clone());
        }

        if let Some(ramp) = &burst.ramp {
            validate_ramp(ramp, "burst_events.ramp", errors, scope_key.clone());
        }

        if scope_key.is_none() {
            push_error(errors, "burst_events.scope", "scope is invalid", None);
        }
    }
}

fn validate_scope(
    scope: &BehaviorScope,
    group_ids: &HashSet<String>,
    endpoint_ids: &HashSet<String>,
    errors: &mut Vec<ValidationError>,
) -> Option<String> {
    let mut count = 0;
    if scope.endpoint_id.as_ref().map_or(false, |v| !v.trim().is_empty()) {
        count += 1;
        if let Some(endpoint_id) = &scope.endpoint_id {
            if !endpoint_ids.contains(endpoint_id) {
                push_error(errors, "scope.endpoint_id", "endpoint id does not exist", Some(endpoint_id.clone()));
            }
        }
    }
    if scope.group_id.as_ref().map_or(false, |v| !v.trim().is_empty()) {
        count += 1;
        if let Some(group_id) = &scope.group_id {
            if !group_ids.contains(group_id) {
                push_error(errors, "scope.group_id", "group id does not exist", Some(group_id.clone()));
            }
        }
    }
    if scope.global {
        count += 1;
    }

    if count != 1 {
        push_error(errors, "scope", "must define exactly one of endpoint_id, group_id, or global", None);
        return None;
    }

    if scope.global {
        return Some("global".to_string());
    }
    if let Some(endpoint_id) = &scope.endpoint_id {
        return Some(format!("endpoint: {}", endpoint_id));
    }
    scope.group_id.as_ref().map(|group_id| format!("group: {}", group_id))
}

fn validate_schedule(
    schedule: &BehaviorSchedule,
    errors: &mut Vec<ValidationError>,
    location: Option<&str>,
) -> bool {
    if !schedule.duration_ms.is_finite() || schedule.duration_ms <= 0.0 {
        push_error(
            errors,
            "behavior_windows.schedule.duration_ms",
            "duration_ms must be > 0",
            location.map(|value| value.to_string()),
        );
        return false;
    }

    match schedule.mode {
        ScheduleMode::Fixed => {
            let Some(start_ms) = schedule.start_offset_ms else {
                push_error(
                    errors,
                    "behavior_windows.schedule.start_offset_ms",
                    "start_offset_ms is required for fixed schedules",
                    location.map(|value| value.to_string()),
                );
                return false;
            };
            if !start_ms.is_finite() || start_ms < 0.0 {
                push_error(
                    errors,
                    "behavior_windows.schedule.start_offset_ms",
                    "start_offset_ms must be >= 0",
                    location.map(|value| value.to_string()),
                );
                return false;
            }
        }
        ScheduleMode::Recurring => {
            let Some(every_ms) = schedule.every_ms else {
                push_error(
                    errors,
                    "behavior_windows.schedule.every_ms",
                    "every_ms is required for recurring schedules",
                    location.map(|value| value.to_string()),
                );
                return false;
            };

            if !every_ms.is_finite() || every_ms <= 0.0 {
                push_error(
                    errors,
                    "behavior_windows.schedule.every_ms",
                    "every_ms must be > 0",
                    location.map(|value| value.to_string()),
                );
                return false;
            }

            if schedule.duration_ms > every_ms {
                push_error(
                    errors,
                    "behavior_windows.schedule.duration_ms",
                    "duration_ms must be <= every_ms",
                    location.map(|value| value.to_string()),
                );
                return false;
            }

            if let Some(jitter_ms) = schedule.jitter_ms {
                if !jitter_ms.is_finite() || jitter_ms < 0.0 {
                    push_error(
                        errors,
                        "behavior_windows.schedule.jitter_ms",
                        "jitter_ms must be >= 0",
                        location.map(|value| value.to_string()),
                    );
                    return false;
                }
                if jitter_ms > (every_ms - schedule.duration_ms) {
                    push_error(
                        errors,
                        "behavior_windows.schedule.jitter_ms",
                        "jitter_ms must be <= every_ms - duration_ms",
                        location.map(|value| value.to_string()),
                    );
                    return false;
                }
            }

            if let Some(min_delay_ms) = schedule.min_delay_ms {
                if !min_delay_ms.is_finite() || min_delay_ms < 0.0 {
                    push_error(
                        errors,
                        "behavior_windows.schedule.min_delay_ms",
                        "min_delay_ms must be >= 0",
                        location.map(|value| value.to_string()),
                    );
                    return false;
                }
            }

            if let Some(max_occurrences) = schedule.max_occurrences {
                if max_occurrences == 0 {
                    push_error(
                        errors,
                        "behavior_windows.schedule.max_occurrences",
                        "max_occurrences must be > 0",
                        location.map(|value| value.to_string()),
                    );
                    return false;
                }
            }
        }
    }

    true
}

fn validate_ramp(
    ramp: &RampConfig,
    prefix: &str,
    errors: &mut Vec<ValidationError>,
    location: Option<String>,
) {
    if let Some(up_ms) = ramp.up_ms {
        if !up_ms.is_finite() || up_ms < 0.0 {
            push_error(errors, &format!("{}.up_ms", prefix), "up_ms must be >= 0", location.clone());
        }
    }
    if let Some(down_ms) = ramp.down_ms {
        if !down_ms.is_finite() || down_ms < 0.0 {
            push_error(errors, &format!("{}.down_ms", prefix), "down_ms must be >= 0", location.clone());
        }
    }
}

fn validate_latency(latency: &LatencyConfig, errors: &mut Vec<ValidationError>, location: Option<String>) {
    validate_distribution(&latency.distribution, &latency.params, errors, location);
}

fn validate_distribution(
    distribution: &DistributionType,
    params: &DistributionParams,
    errors: &mut Vec<ValidationError>,
    location: Option<String>,
) {
    match (distribution, params) {
        (DistributionType::Fixed, DistributionParams::Fixed { delay_ms }) => {
            if !delay_ms.is_finite() || *delay_ms < 0.0 {
                push_error(errors, "latency.params.delay_ms", "must be >= 0", location);
            }
        }
        (DistributionType::Normal, DistributionParams::Normal { mean_ms, stddev_ms }) => {
            if !mean_ms.is_finite() || *mean_ms < 0.0 {
                push_error(errors, "latency.params.mean_ms", "must be >= 0", location.clone());
            }
            if !stddev_ms.is_finite() || *stddev_ms <= 0.0 {
                push_error(errors, "latency.params.stddev_ms", "must be > 0", location);
            }
        }
        (DistributionType::Exponential, DistributionParams::Exponential { rate }) => {
            if !rate.is_finite() || *rate <= 0.0 {
                push_error(errors, "latency.params.rate", "must be > 0", location);
            }
        }
        (DistributionType::Uniform, DistributionParams::Uniform { min_ms, max_ms }) => {
            if !min_ms.is_finite() || *min_ms < 0.0 {
                push_error(errors, "latency.params.min_ms", "must be >= 0", location.clone());
            }
            if !max_ms.is_finite() || *max_ms <= *min_ms {
                push_error(errors, "latency.params.max_ms", "must be > min_ms", location);
            }
        }
        (DistributionType::LogNormal, DistributionParams::LogNormal { mean_ms, stddev_ms }) => {
            if !mean_ms.is_finite() || *mean_ms <= 0.0 {
                push_error(errors, "latency.params.mean_ms", "must be > 0", location.clone());
            }
            if !stddev_ms.is_finite() || *stddev_ms < 0.0 {
                push_error(errors, "latency.params.stddev_ms", "must be >= 0", location);
            }
        }
        (DistributionType::Mixture, DistributionParams::Mixture { components }) => {
            validate_mixture_components(components, errors, location);
        }
        _ => {
            push_error(
                errors,
                "latency",
                "distribution type and params do not match",
                location,
            );
        }
    }
}

fn validate_mixture_components(
    components: &[MixtureComponent],
    errors: &mut Vec<ValidationError>,
    location: Option<String>,
) {
    if components.is_empty() {
        push_error(errors, "latency.params.components", "must include at least one component", location);
        return;
    }

    let mut total_weight = 0.0;

    for (index, component) in components.iter().enumerate() {
        if !component.weight.is_finite() || component.weight <= 0.0 {
            push_error(
                errors,
                "latency.params.components.weight",
                &format!("component {} weight must be > 0", index),
                location.clone(),
            );
        } else {
            total_weight += component.weight;
        }

        if component.distribution == DistributionType::Mixture {
            push_error(
                errors,
                "latency.params.components.distribution",
                &format!("component {} distribution cannot be mixture", index),
                location.clone(),
            );
            continue;
        }

        validate_distribution(
            &component.distribution,
            component.params.as_ref(),
            errors,
            location.clone(),
        );
    }

    if total_weight <= 0.0 {
        push_error(
            errors,
            "latency.params.components.weight",
            "total weight must be > 0",
            location,
        );
    }
}

fn validate_response(response: &Response, errors: &mut Vec<ValidationError>, location: Option<String>) {
    if !is_valid_status(response.status) {
        push_error(errors, "response.status", "invalid HTTP status code", location);
    }
}

fn validate_error_profile(profile: &ErrorProfile, errors: &mut Vec<ValidationError>, location: Option<String>) {
    if !profile.rate.is_finite() || profile.rate < 0.0 || profile.rate > 1.0 {
        push_error(errors, "error_profile.rate", "must be between 0.0 and 1.0", location.clone());
    }

    if profile.rate > 0.0 && profile.codes.is_empty() && !profile.error_in_payload {
        push_error(errors, "error_profile.codes", "must include at least one status code when rate > 0", location.clone());
    }

    for code in &profile.codes {
        if !is_valid_status(*code) {
            push_error(errors, "error_profile.codes", "invalid HTTP status code", location.clone());
            break;
        }
    }

    if let Some(corruption) = &profile.payload_corruption {
        if !corruption.rate.is_finite() || corruption.rate < 0.0 || corruption.rate > 1.0 {
            push_error(errors, "error_profile.payload_corruption.rate", "must be between 0.0 and 1.0", location.clone());
        }

    match corruption.mode {
            crate::config::CorruptionMode::Truncate => {
                if let Some(ratio) = corruption.truncate_ratio {
                    if !ratio.is_finite() || ratio <= 0.0 || ratio > 1.0 {
                        push_error(errors, "error_profile.payload_corruption.truncate_ratio", "must be > 0 and <= 1", location.clone());
                    }
                }
            }
            crate::config::CorruptionMode::Replace => {
                if corruption.replacement.as_deref().unwrap_or("").is_empty() {
                    push_error(errors, "error_profile.payload_corruption.replacement", "must be provided for replace mode", location.clone());
                }
            }
        }
    }
}

fn validate_request_match(request: Option<&RequestMatch>, errors: &mut Vec<ValidationError>, location: Option<String>) {
    if let Some(request) = request {
        match request.body_match {
            BodyMatchType::Exact | BodyMatchType::Contains => {
                if request.body.as_deref().unwrap_or("").is_empty() {
                    push_error(errors, "request.body", "body required for exact or contains match", location);
                }
            }
            BodyMatchType::Any | BodyMatchType::Ignore => {}
        }
    }
}

fn is_valid_status(status: u16) -> bool {
    (100..=599).contains(&status)
}

fn method_to_str(method: &HttpMethod) -> &'static str {
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

fn push_error(errors: &mut Vec<ValidationError>, field: &str, error: &str, location: Option<String>) {
    errors.push(ValidationError {
        field: field.to_string(),
        error: error.to_string(),
        location,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn base_response(status: u16) -> Response {
        Response {
            status,
            headers: HashMap::new(),
            body: "{}".to_string(),
        }
    }

    fn base_endpoint(id: &str, method: HttpMethod, path: &str, latency: LatencyConfig) -> Endpoint {
        Endpoint {
            id: id.to_string(),
            method,
            path: path.to_string(),
            request: None,
            latency,
            response: base_response(200),
            error_profile: ErrorProfile::default(),
            rate_limit: None,
            bandwidth_cap: None,
            loaded_at: None,
            rate_limiter: None,
        }
    }

    fn base_config() -> Configuration {
        Configuration {
            version: "1.0".to_string(),
            metadata: Default::default(),
            endpoints: vec![base_endpoint(
                "health",
                HttpMethod::Get,
                "/health",
                LatencyConfig {
                    distribution: DistributionType::Fixed,
                    params: DistributionParams::Fixed { delay_ms: 5.0 },
                },
            )],
            endpoint_groups: vec![],
            behavior_windows: vec![],
            burst_events: vec![],
            workflows: vec![],
        }
    }

    fn validation_errors(config: &Configuration) -> Vec<ValidationError> {
        match validate(config) {
            Ok(_) => vec![],
            Err(ConfigError::ValidationError(_, errors)) => errors,
            Err(_) => vec![],
        }
    }

    #[test]
    fn test_validate_valid_config() {
        let config = base_config();
        assert!(validate(&config).is_ok());
    }

    #[test]
    fn test_validate_duplicate_id() {
        let mut config = base_config();
        let latency = LatencyConfig {
            distribution: DistributionType::Fixed,
            params: DistributionParams::Fixed { delay_ms: 10.0 },
        };
        config.endpoints.push(base_endpoint("health", HttpMethod::Get, "/health2", latency));
        let errors = validation_errors(&config);
        assert!(errors.iter().any(|e| e.field == "endpoints.id"));
    }

    #[test]
    fn test_validate_duplicate_method_path() {
        let mut config = base_config();
        let latency = LatencyConfig {
            distribution: DistributionType::Fixed,
            params: DistributionParams::Fixed { delay_ms: 10.0 },
        };
        config.endpoints.push(base_endpoint("health2", HttpMethod::Get, "/health", latency));
        let errors = validation_errors(&config);
        assert!(errors.iter().any(|e| e.field == "endpoints.method+path"));
    }

    #[test]
    fn test_validate_version() {
        let mut config = base_config();
        config.version = "2.0".to_string();
        let errors = validation_errors(&config);
        assert!(errors.iter().any(|e| e.field == "version"));
    }

    #[test]
    fn test_validate_empty_endpoints() {
        let mut config = base_config();
        config.endpoints.clear();
        let errors = validation_errors(&config);
        assert!(errors.iter().any(|e| e.field == "endpoints"));
    }

    #[test]
    fn test_validate_path_prefix() {
        let mut config = base_config();
        config.endpoints[0].path = "health".to_string();
        let errors = validation_errors(&config);
        assert!(errors.iter().any(|e| e.field == "endpoints.path"));
    }

    #[test]
    fn test_validate_response_status() {
        let mut config = base_config();
        config.endpoints[0].response.status = 99;
        let errors = validation_errors(&config);
        assert!(errors.iter().any(|e| e.field == "response.status"));
    }

    #[test]
    fn test_validate_error_rate_range() {
        let mut config = base_config();
        config.endpoints[0].error_profile.rate = 1.5;
        let errors = validation_errors(&config);
        assert!(errors.iter().any(|e| e.field == "error_profile.rate"));
    }

    #[test]
    fn test_validate_error_rate_negative() {
        let mut config = base_config();
        config.endpoints[0].error_profile.rate = -0.1;
        let errors = validation_errors(&config);
        assert!(errors.iter().any(|e| e.field == "error_profile.rate"));
    }

    #[test]
    fn test_validate_error_codes_required() {
        let mut config = base_config();
        config.endpoints[0].error_profile.rate = 0.1;
        config.endpoints[0].error_profile.codes.clear();
        let errors = validation_errors(&config);
        assert!(errors.iter().any(|e| e.field == "error_profile.codes"));
    }

    #[test]
    fn test_validate_error_codes_not_required_for_payload_error() {
        let mut config = base_config();
        config.endpoints[0].error_profile.rate = 0.1;
        config.endpoints[0].error_profile.codes.clear();
        config.endpoints[0].error_profile.error_in_payload = true;
        assert!(validate(&config).is_ok());
    }

    #[test]
    fn test_validate_payload_corruption_rate() {
        let mut config = base_config();
        config.endpoints[0].error_profile.payload_corruption = Some(crate::config::PayloadCorruption {
            rate: 2.0,
            mode: crate::config::CorruptionMode::Truncate,
            truncate_ratio: Some(0.5),
            replacement: None,
        });
        let errors = validation_errors(&config);
        assert!(errors.iter().any(|e| e.field == "error_profile.payload_corruption.rate"));
    }

    #[test]
    fn test_validate_payload_corruption_replace_requires_value() {
        let mut config = base_config();
        config.endpoints[0].error_profile.payload_corruption = Some(crate::config::PayloadCorruption {
            rate: 0.5,
            mode: crate::config::CorruptionMode::Replace,
            truncate_ratio: None,
            replacement: None,
        });
        let errors = validation_errors(&config);
        assert!(errors.iter().any(|e| e.field == "error_profile.payload_corruption.replacement"));
    }

    #[test]
    fn test_validate_rate_limit_requests_per_second() {
        let mut config = base_config();
        config.endpoints[0].rate_limit = Some(crate::config::RateLimit {
            requests_per_second: 0.0,
            burst: Some(5.0),
        });
        let errors = validation_errors(&config);
        assert!(errors.iter().any(|e| e.field == "rate_limit.requests_per_second"));
    }

    #[test]
    fn test_validate_rate_limit_burst() {
        let mut config = base_config();
        config.endpoints[0].rate_limit = Some(crate::config::RateLimit {
            requests_per_second: 10.0,
            burst: Some(-1.0),
        });
        let errors = validation_errors(&config);
        assert!(errors.iter().any(|e| e.field == "rate_limit.burst"));
    }

    #[test]
    fn test_validate_bandwidth_cap() {
        let mut config = base_config();
        config.endpoints[0].bandwidth_cap = Some(crate::config::BandwidthCap {
            bytes_per_second: 0.0,
        });
        let errors = validation_errors(&config);
        assert!(errors.iter().any(|e| e.field == "bandwidth_cap.bytes_per_second"));
    }

    #[test]
    fn test_validate_error_codes_invalid() {
        let mut config = base_config();
        config.endpoints[0].error_profile.rate = 0.1;
        config.endpoints[0].error_profile.codes = vec![700];
        let errors = validation_errors(&config);
        assert!(errors.iter().any(|e| e.field == "error_profile.codes"));
    }

    #[test]
    fn test_validate_latency_fixed_negative() {
        let mut config = base_config();
        config.endpoints[0].latency.params = DistributionParams::Fixed { delay_ms: -1.0 };
        let errors = validation_errors(&config);
        assert!(errors.iter().any(|e| e.field == "latency.params.delay_ms"));
    }

    #[test]
    fn test_validate_latency_normal_stddev() {
        let mut config = base_config();
        config.endpoints[0].latency.distribution = DistributionType::Normal;
        config.endpoints[0].latency.params = DistributionParams::Normal {
            mean_ms: 50.0,
            stddev_ms: 0.0,
        };
        let errors = validation_errors(&config);
        assert!(errors.iter().any(|e| e.field == "latency.params.stddev_ms"));
    }

    #[test]
    fn test_validate_latency_normal_mean() {
        let mut config = base_config();
        config.endpoints[0].latency.distribution = DistributionType::Normal;
        config.endpoints[0].latency.params = DistributionParams::Normal {
            mean_ms: -1.0,
            stddev_ms: 5.0,
        };
        let errors = validation_errors(&config);
        assert!(errors.iter().any(|e| e.field == "latency.params.mean_ms"));
    }

    #[test]
    fn test_validate_latency_uniform_range() {
        let mut config = base_config();
        config.endpoints[0].latency.distribution = DistributionType::Uniform;
        config.endpoints[0].latency.params = DistributionParams::Uniform {
            min_ms: 50.0,
            max_ms: 25.0,
        };
        let errors = validation_errors(&config);
        assert!(errors.iter().any(|e| e.field == "latency.params.max_ms"));
    }

    #[test]
    fn test_validate_latency_uniform_min() {
        let mut config = base_config();
        config.endpoints[0].latency.distribution = DistributionType::Uniform;
        config.endpoints[0].latency.params = DistributionParams::Uniform {
            min_ms: -5.0,
            max_ms: 25.0,
        };
        let errors = validation_errors(&config);
        assert!(errors.iter().any(|e| e.field == "latency.params.min_ms"));
    }

    #[test]
    fn test_validate_latency_exponential_rate() {
        let mut config = base_config();
        config.endpoints[0].latency.distribution = DistributionType::Exponential;
        config.endpoints[0].latency.params = DistributionParams::Exponential { rate: 0.0 };
        let errors = validation_errors(&config);
        assert!(errors.iter().any(|e| e.field == "latency.params.rate"));
    }

    #[test]
    fn test_validate_latency_log_normal_mean() {
        let mut config = base_config();
        config.endpoints[0].latency.distribution = DistributionType::LogNormal;
        config.endpoints[0].latency.params = DistributionParams::LogNormal {
            mean_ms: 0.0,
            stddev_ms: 5.0,
        };
        let errors = validation_errors(&config);
        assert!(errors.iter().any(|e| e.field == "latency.params.mean_ms"));
    }

    #[test]
    fn test_validate_latency_log_normal_stddev() {
        let mut config = base_config();
        config.endpoints[0].latency.distribution = DistributionType::LogNormal;
        config.endpoints[0].latency.params = DistributionParams::LogNormal {
            mean_ms: 50.0,
            stddev_ms: -1.0,
        };
        let errors = validation_errors(&config);
        assert!(errors.iter().any(|e| e.field == "latency.params.stddev_ms"));
    }

    #[test]
    fn test_validate_latency_mixture_empty_components() {
        let mut config = base_config();
        config.endpoints[0].latency.distribution = DistributionType::Mixture;
        config.endpoints[0].latency.params = DistributionParams::Mixture { components: vec![] };
        let errors = validation_errors(&config);
        assert!(errors.iter().any(|e| e.field == "latency.params.components"));
    }

    #[test]
    fn test_validate_latency_mixture_weight() {
        let mut config = base_config();
        config.endpoints[0].latency.distribution = DistributionType::Mixture;
        config.endpoints[0].latency.params = DistributionParams::Mixture {
            components: vec![MixtureComponent {
                weight: 0.0,
                distribution: DistributionType::Fixed,
                params: Box::new(DistributionParams::Fixed { delay_ms: 5.0 }),
            }],
        };
        let errors = validation_errors(&config);
        assert!(errors.iter().any(|e| e.field == "latency.params.components.weight"));
    }

    #[test]
    fn test_validate_latency_mixture_valid() {
        let mut config = base_config();
        config.endpoints[0].latency.distribution = DistributionType::Mixture;
        config.endpoints[0].latency.params = DistributionParams::Mixture {
            components: vec![
                MixtureComponent {
                    weight: 0.7,
                    distribution: DistributionType::Fixed,
                    params: Box::new(DistributionParams::Fixed { delay_ms: 5.0 }),
                },
                MixtureComponent {
                    weight: 0.3,
                    distribution: DistributionType::Normal,
                    params: Box::new(DistributionParams::Normal {
                        mean_ms: 50.0,
                        stddev_ms: 10.0,
                    }),
                },
            ],
        };
        assert!(validate(&config).is_ok());
    }

    #[test]
    fn test_validate_latency_distribution_mismatch() {
        let mut config = base_config();
        config.endpoints[0].latency.distribution = DistributionType::Normal;
        config.endpoints[0].latency.params = DistributionParams::Fixed { delay_ms: 10.0 };
        let errors = validation_errors(&config);
        assert!(errors.iter().any(|e| e.field == "latency"));
    }

    #[test]
    fn test_validate_request_body_match_requires_body() {
        let mut config = base_config();
        config.endpoints[0].request = Some(RequestMatch {
            body_match: BodyMatchType::Exact,
            body: None,
        });
        let errors = validation_errors(&config);
        assert!(errors.iter().any(|e| e.field == "request.body"));
    }

}
