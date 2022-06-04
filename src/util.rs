use log::{debug, error, info, warn};
use reqwest::{Response, StatusCode};

/// These are the default status codes as defined here: https://client.cdn77.com/support/api-reference/v3/introduction
/// Unfortunately, some codes have a duplicate meaning for some API operations
/// For example, 403 might signify "bad credentials" or "purge-all not allowed on resource"
/// So this handler is only invoked after the expected API operation specific codes have been handled.
pub async fn handle_default_response_status_codes(response: Response) {
	match response.status() {
		StatusCode::UNAUTHORIZED => {
			warn!("Got 401/unauthorized. Please check your credentials.");
		},
		StatusCode::FORBIDDEN => {
			warn!("Got 403/forbidden. Please check your credentials or the API operation args.");
		},
		StatusCode::NOT_FOUND => {
			info!("The requested resource was not found. Please validate your args.");
		},
		StatusCode::METHOD_NOT_ALLOWED => {
			error!("Received 405/MethodNotAllowed. This might be an issue with an outdated client due to API changes.");
		},
		StatusCode::UNPROCESSABLE_ENTITY => {
			error!("Received 422/UnprocessableEntity. This might be an issue with this client, please check for an update.");
		},
		code => {
			let body: String = response.text().await.unwrap_or_else(|_| "FAILED TO READ RESPONSE, EMPTY?".to_string());
			warn!("Received unexpected/unknown status code={}, please check the response for an explanation: {}", code, body);
		}
	};
}
