// Configuration parser
//
// Parses YAML and JSON configuration files

use super::{Configuration, ConfigError};

/// Parse configuration from YAML string
pub fn parse_yaml(content: &str) -> Result<Configuration, ConfigError> {
    serde_yaml::from_str(content).map_err(|e| ConfigError::ParseError(e.to_string()))
}

/// Parse configuration from JSON string
pub fn parse_json(content: &str) -> Result<Configuration, ConfigError> {
    serde_json::from_str(content).map_err(|e| ConfigError::ParseError(e.to_string()))
}

/// Auto-detect format and parse
pub fn parse_auto(content: &str) -> Result<Configuration, ConfigError> {
    let trimmed = content.trim_start();

    if trimmed.starts_with('{') || trimmed.starts_with('[') {
        match parse_json(content) {
            Ok(config) => Ok(config),
            Err(json_err) => match parse_yaml(content) {
                Ok(config) => Ok(config),
                Err(yaml_err) => Err(ConfigError::ParseError(format!(
                    "JSON parse error: {}; YAML parse error: {}",
                    json_err, yaml_err
                ))),
            },
        }
    } else {
        match parse_yaml(content) {
            Ok(config) => Ok(config),
            Err(yaml_err) => match parse_json(content) {
                Ok(config) => Ok(config),
                Err(json_err) => Err(ConfigError::ParseError(format!(
                    "YAML parse error: {}; JSON parse error: {}",
                    yaml_err, json_err
                ))),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SIMPLE_YAML: &str = include_str!("../../examples/01-simple-health-check.yaml");

    #[test]
    fn test_parse_yaml() {
        let config = parse_yaml(SIMPLE_YAML).expect("YAML should parse");
        assert_eq!(config.version, "1.0");
        assert_eq!(config.endpoints.len(), 1);
    }

    #[test]
    fn test_parse_json() {
                let content = r#"{
    "version": "1.0",
    "metadata": { "name": "json-example" },
    "endpoints": [
        {
            "id": "health",
            "method": "GET",
            "path": "/health",
            "latency": {
                "distribution": "fixed",
                "params": { "delay_ms": 5 }
            },
            "response": {
                "status": 200,
                "headers": { "Content-Type": "application/json" },
                "body": "{}"
            },
            "error_profile": { "rate": 0.0 }
        }
    ],
    "workflows": []
}"#;

                let config = parse_json(content).expect("JSON should parse");
                assert_eq!(config.metadata.name.as_deref(), Some("json-example"));
                assert_eq!(config.endpoints.len(), 1);
        }

        #[test]
        fn test_parse_auto_json() {
                let content = r#"{ "version": "1.0", "endpoints": [], "workflows": [] }"#;
                let config = parse_auto(content).expect("Auto parse JSON");
                assert_eq!(config.version, "1.0");
        }

        #[test]
        fn test_parse_auto_yaml() {
                let content = "version: \"1.0\"\nendpoints: []\nworkflows: []\n";
                let config = parse_auto(content).expect("Auto parse YAML");
                assert_eq!(config.version, "1.0");
        }

        #[test]
        fn test_parse_yaml_error() {
                let content = "version: [";
                let err = parse_yaml(content).expect_err("Should fail to parse YAML");
                match err {
                        ConfigError::ParseError(_) => {}
                        _ => panic!("Expected parse error"),
                }
    }
}
