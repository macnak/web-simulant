// Configuration schema definitions
//
// Rust structs that match the YAML/JSON schema

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Top-level configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Configuration {
    pub version: String,
    #[serde(default)]
    pub metadata: Metadata,
    pub endpoints: Vec<Endpoint>,
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
}

impl Default for ErrorProfile {
    fn default() -> Self {
        Self {
            rate: 0.0,
            codes: vec![],
            body: String::new(),
        }
    }
}

/// Workflow definition (Phase 2)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: String,
    // TODO: Phase 2 - Define workflow structure
}
