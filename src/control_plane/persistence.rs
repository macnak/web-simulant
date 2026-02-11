// Configuration persistence

use crate::config::{parse_yaml, validate, Configuration, ConfigError};
use std::fs;
use std::path::Path;

pub fn save_config(path: &Path, config: &Configuration) -> Result<(), ConfigError> {
	let content = serde_yaml::to_string(config).map_err(|e| ConfigError::ParseError(e.to_string()))?;
	atomic_write(path, content.as_bytes())?;
	Ok(())
}

pub fn load_config(path: &Path) -> Result<Option<Configuration>, ConfigError> {
	if !path.exists() {
		return Ok(None);
	}

	let content = fs::read_to_string(path)?;
	let config = parse_yaml(&content)?;
	validate(&config)?;
	Ok(Some(config))
}

fn atomic_write(path: &Path, content: &[u8]) -> Result<(), std::io::Error> {
	let dir = path.parent().unwrap_or_else(|| Path::new("."));
	fs::create_dir_all(dir)?;

	let tmp_path = path.with_extension("tmp");
	fs::write(&tmp_path, content)?;
	fs::rename(tmp_path, path)?;
	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::config::{DistributionParams, DistributionType, Endpoint, ErrorProfile, HttpMethod, LatencyConfig, Metadata, Response};
	use std::collections::HashMap;
	use tempfile::tempdir;

	fn sample_config() -> Configuration {
		Configuration {
			version: "1.0".to_string(),
			metadata: Metadata::default(),
			endpoints: vec![Endpoint {
				id: "health".to_string(),
				method: HttpMethod::Get,
				path: "/health".to_string(),
				request: None,
				latency: LatencyConfig {
					distribution: DistributionType::Fixed,
					params: DistributionParams::Fixed { delay_ms: 1.0 },
				},
				response: Response {
					status: 200,
					headers: HashMap::new(),
					body: "ok".to_string(),
				},
				error_profile: ErrorProfile::default(),
			}],
			workflows: vec![],
		}
	}

	#[test]
	fn test_save_and_load() {
		let dir = tempdir().expect("tempdir");
		let path = dir.path().join("config.yaml");

		let config = sample_config();
		save_config(&path, &config).expect("save");
		let loaded = load_config(&path).expect("load").expect("config");

		assert_eq!(loaded.endpoints.len(), 1);
		assert_eq!(loaded.endpoints[0].id, "health");
	}
}
