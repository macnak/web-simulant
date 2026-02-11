// Response building utilities

use axum::body::Body;
use axum::http::{HeaderName, HeaderValue, Response, StatusCode};
use std::collections::HashMap;

pub fn build_response(status: u16, headers: &HashMap<String, String>, body: &str) -> Response<Body> {
	let status = StatusCode::from_u16(status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
	let mut response = Response::new(Body::from(body.to_string()));
	*response.status_mut() = status;

	for (key, value) in headers {
		if let (Ok(name), Ok(val)) = (HeaderName::from_bytes(key.as_bytes()), HeaderValue::from_str(value)) {
			response.headers_mut().insert(name, val);
		}
	}

	response
}

pub fn build_plain_text(status: u16, body: &str) -> Response<Body> {
	let mut headers = HashMap::new();
	headers.insert("Content-Type".to_string(), "text/plain".to_string());
	build_response(status, &headers, body)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_build_response_status_and_headers() {
		let mut headers = HashMap::new();
		headers.insert("Content-Type".to_string(), "application/json".to_string());
		let response = build_response(201, &headers, "{}");
		assert_eq!(response.status(), StatusCode::CREATED);
		assert_eq!(response.headers().get("Content-Type").unwrap(), "application/json");
	}
}
