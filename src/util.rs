use std::process;
use chrono::NaiveDateTime;

use reqwest::{RequestBuilder, Response, StatusCode};

use crate::{EXIT_CODE_API_EXPECTED_ERROR, EXIT_CODE_API_UNEXPECTED_ERROR, EXIT_CODE_INVALID_INPUT};

/// An alias for the resource ID type
pub type ResourceId = u64;

/// These are the default status codes as defined here: https://client.cdn77.com/support/api-reference/v3/introduction
/// Unfortunately, some codes have a duplicate meaning for some API operations
/// For example, 403 might signify "bad credentials" or "purge-all not allowed on resource"
/// So this handler is only invoked after the expected API operation specific codes have been handled.
pub async fn handle_default_response_status_codes(response: Response) {
	match response.status() {
		StatusCode::UNAUTHORIZED => {
			eprintln!("Got 401/unauthorized. Please check your credentials.");
			process::exit(EXIT_CODE_API_EXPECTED_ERROR);
		}
		StatusCode::FORBIDDEN => {
			eprintln!("Got 403/forbidden. Please check your credentials or the API operation args.");
			process::exit(EXIT_CODE_API_EXPECTED_ERROR);
		}
		StatusCode::NOT_FOUND => {
			println!("The requested resource was not found. Please validate your args.");
			process::exit(EXIT_CODE_API_EXPECTED_ERROR);
		}
		StatusCode::METHOD_NOT_ALLOWED => {
			eprintln!("Received 405/MethodNotAllowed. This might be an issue with an outdated client due to API changes.");
			process::exit(EXIT_CODE_API_UNEXPECTED_ERROR);
		}
		StatusCode::UNPROCESSABLE_ENTITY => {
			eprintln!("Received 422/UnprocessableEntity. This might be an issue with this client, please check for an update.");
			process::exit(EXIT_CODE_API_UNEXPECTED_ERROR);
		}
		code => {
			let body: String = response.text().await.unwrap_or_else(|_| "FAILED TO READ RESPONSE, EMPTY?".to_string());
			eprintln!("Received unexpected/unknown status code={}, please check the response for an explanation: {}", code, body);
			process::exit(EXIT_CODE_API_UNEXPECTED_ERROR);
		}
	};
}

pub fn parse_date_time_or_exit(input: &str, error_msg: &str) -> NaiveDateTime {
	NaiveDateTime::parse_from_str(input, "%Y-%m-%d %H:%M")
		.unwrap_or_else(|_| {
			println!("{}", error_msg);
			process::exit(EXIT_CODE_INVALID_INPUT)
		})
}

pub fn parse_resource_ids_optional(input: &Option<String>) -> Option<Vec<ResourceId>> {
	match input {
		Some(r) => {
			let resource_ids = r.split(',')
				.map(|r| r.trim())
				.filter(|r| !r.is_empty())
				.map(|s| s.parse::<ResourceId>().expect("At least one resource id is malformed"))
				.collect();
			Some(resource_ids)
		}
		None => None,
	}
}

pub async fn send_http_request_return_response_or_exit(request: RequestBuilder) -> Response {
	let response = request.send().await;

	match response {
		Ok(r) => r,
		Err(err) => {
			eprintln!("Failed to get response HTTP request, e={:?}", err);
			process::exit(EXIT_CODE_API_UNEXPECTED_ERROR);
		}
	}
}
