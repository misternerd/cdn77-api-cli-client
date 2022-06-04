use std::collections::HashMap;
use reqwest::{Client, StatusCode};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use crate::CDN77_API_BASE;
use crate::util::handle_default_response_status_codes;

// Docs: https://client.cdn77.com/support/api-reference/v3/jobs


pub async fn command_job_purge_all(client: Client, resource_id: &str) {
	let resource_id: u64 = resource_id.parse().expect("Please provide a valid resource ID");
	debug!("Purging all data in resource_id={}", &resource_id);

	let request_url = format!("{}/cdn/{}/job/purge-all", CDN77_API_BASE, &resource_id);
	let response = client.post(request_url)
		.send()
		.await;

	let response = match response {
		Ok(r) => r,
		Err(err) => {
			warn!("Failed to get purge-all, e={:?}", err);
			return;
		},
	};

	match response.status() {
		StatusCode::ACCEPTED => {
			match response.json::<PurgeAllResponse>().await {
				Ok(r) => {
					debug!("Successfully executed {} of resource IDs {:?} => job ID={}", r.resource_type, r.cdn, r.id);
				},
				Err(err) => {
					warn!("Failed to deserialize purge-all response, e={:?}", err);
				}
			}
		},
		StatusCode::FORBIDDEN => {
			info!("Purging all files is disabled for resource={}: {:?}", resource_id, response);
		},
		StatusCode::NOT_FOUND => {
			warn!("Didn't find resource={}", resource_id);
		},
		_ => {
			handle_default_response_status_codes(response);
		},
	}
}


#[derive(Deserialize, Debug, Serialize)]
struct PurgeAllResponse {
	id: String,
	#[serde(rename = "type")]
	resource_type: String,
	cdn: HashMap<String, u64>,
	state: String,
	queued_at: String,
	done_at: String,
}
