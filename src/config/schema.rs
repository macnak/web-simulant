// Configuration schema definitions
//
// Rust structs that match the YAML/JSON schema

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Top-level configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Configuration {
    pub version: String,
    #[serde(default)]
    pub metadata: Metadata,
    pub endpoints: Vec<Endpoint>,
    #[serde(default)]
    pub endpoint_groups: Vec<EndpointGroup>,
    #[serde(default)]
    pub behavior_windows: Vec<BehaviorWindow>,
    #[serde(default)]
    pub burst_events: Vec<BurstEvent>,
    #[serde(default)]
    pub workflows: Vec<Workflow>,
}

/// Configuration metadata
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Metadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
}

/// Endpoint definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Endpoint {
    pub id: String,
    pub method: HttpMethod,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request: Option<RequestMatch>,
    pub latency: LatencyConfig,
    pub response: Response,
    #[serde(default)]
    pub error_profile: ErrorProfile,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate_limit: Option<RateLimit>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bandwidth_cap: Option<BandwidthCap>,
    #[serde(skip)]
    pub loaded_at: Option<Instant>,
    #[serde(skip)]
    pub rate_limiter: Option<Arc<Mutex<TokenBucket>>>,
}

/// Endpoint group definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointGroup {
    pub id: String,
    pub endpoint_ids: Vec<String>,
}

/// Scope for behavior rules.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorScope {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_id: Option<String>,
    #[serde(default)]
    pub global: bool,
}

/// Schedule for behavior windows.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorSchedule {
    pub mode: ScheduleMode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_offset_ms: Option<f64>,
    pub duration_ms: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub every_ms: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jitter_ms: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_occurrences: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_delay_ms: Option<f64>,
}

/// Schedule modes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScheduleMode {
    Fixed,
    Recurring,
}

/// Ramp configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RampConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub up_ms: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub down_ms: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub curve: Option<RampCurve>,
}

/// Ramp curves.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RampCurve {
    Linear,
    SCurve,
}

/// Error mix strategies.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ErrorMix {
    Override,
    Additive,
    Blend,
}

fn default_error_mix() -> ErrorMix {
    ErrorMix::Override
}

/// Time-window behavior overrides.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorWindow {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub scope: BehaviorScope,
    pub schedule: BehaviorSchedule,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ramp: Option<RampConfig>,
    #[serde(default = "default_error_mix")]
    pub error_mix: ErrorMix,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_override: Option<LatencyConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_profile_override: Option<ErrorProfile>,
}

/// Burst events for clustered spikes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BurstEvent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub scope: BehaviorScope,
    pub frequency: BurstFrequency,
    pub duration_ms: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ramp: Option<RampConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_spike: Option<LatencyConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_spike: Option<ErrorSpike>,
}

/// Burst frequency settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BurstFrequency {
    pub every_ms: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jitter_ms: Option<f64>,
}

/// Error spike settings for bursts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorSpike {
    #[serde(default = "default_error_mix")]
    pub error_mix: ErrorMix,
    pub error_profile: ErrorProfile,
}

/// Request rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimit {
    pub requests_per_second: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub burst: Option<f64>,
}

/// Bandwidth cap configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BandwidthCap {
    pub bytes_per_second: f64,
}

/// HTTP methods
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Head,
    Options,
}

/// Request matching configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestMatch {
    #[serde(default = "default_body_match")]
    pub body_match: BodyMatchType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
}

fn default_body_match() -> BodyMatchType {
    BodyMatchType::Any
}

/// Body matching types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BodyMatchType {
    Any,
    Exact,
    Contains,
    Ignore,
}

/// Latency configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyConfig {
    pub distribution: DistributionType,
    pub params: DistributionParams,
}

/// Distribution types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DistributionType {
    Fixed,
    Normal,
    Exponential,
    Uniform,
    LogNormal,
    Mixture,
}

/// Distribution parameters (variant based on type)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DistributionParams {
    Fixed {
        delay_ms: f64,
    },
    Normal {
        mean_ms: f64,
        stddev_ms: f64,
    },
    Exponential {
        rate: f64,
    },
    Uniform {
        min_ms: f64,
        max_ms: f64,
    },
    LogNormal {
        mean_ms: f64,
        stddev_ms: f64,
    },
    Mixture {
        components: Vec<MixtureComponent>,
    },
}

/// Mixture distribution component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MixtureComponent {
    pub weight: f64,
    pub distribution: DistributionType,
    pub params: Box<DistributionParams>,
}

/// Response configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub status: u16,
    #[serde(default = "default_headers")]
    pub headers: HashMap<String, String>,
    pub body: String,
}

fn default_headers() -> HashMap<String, String> {
    let mut headers = HashMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());
    headers
}

/// Error profile configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorProfile {
    #[serde(default)]
    pub rate: f64,
    #[serde(default)]
    pub codes: Vec<u16>,
    #[serde(default)]
    pub body: String,
    #[serde(default)]
    pub error_in_payload: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payload_corruption: Option<PayloadCorruption>,
}

/// Payload corruption settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayloadCorruption {
    pub rate: f64,
    pub mode: CorruptionMode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub truncate_ratio: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replacement: Option<String>,
}

/// Payload corruption mode
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CorruptionMode {
    Truncate,
    Replace,
}

impl Default for ErrorProfile {
    fn default() -> Self {
        Self {
            rate: 0.0,
            codes: vec![],
            body: String::new(),
            error_in_payload: false,
            payload_corruption: None,
        }
    }
}

/// Runtime token bucket for rate limiting (not serialized).
#[derive(Debug)]
pub struct TokenBucket {
    capacity: f64,
    tokens: f64,
    refill_rate: f64,
    last_refill: Instant,
}

impl TokenBucket {
    pub fn new(requests_per_second: f64, burst: Option<f64>) -> Self {
        let capacity = burst.unwrap_or(requests_per_second).max(0.0);
        let refill_rate = requests_per_second.max(0.0);
        Self {
            capacity,
            tokens: capacity,
            refill_rate,
            last_refill: Instant::now(),
        }
    }

    pub fn try_take(&mut self) -> bool {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        self.last_refill = now;

        if elapsed > 0.0 && self.refill_rate > 0.0 {
            let refill = elapsed * self.refill_rate;
            self.tokens = (self.tokens + refill).min(self.capacity);
        }

        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }
}

/// Workflow definition (Phase 2)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: String,
    // TODO: Phase 2 - Define workflow structure
}
