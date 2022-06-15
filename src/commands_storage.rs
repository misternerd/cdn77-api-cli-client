use std::process;

use reqwest::{Client, StatusCode};
use serde::Deserialize;

use crate::{CDN77_API_BASE, EXIT_CODE_API_UNEXPECTED_ERROR};
use crate::util::{handle_default_response_status_codes, send_http_request_return_response_or_exit};

pub async fn command_storage_list(client: Client) {
	let request_url = format!("{}/storage-location", CDN77_API_BASE);
	let response = send_http_request_return_response_or_exit(client.get(request_url)).await;

	match response.status() {
		StatusCode::OK => {
			match response.json::<Vec<StorageListEntry>>().await {
				Ok(r) => {
					println!("Found {} storage locations", &r.len());

					for (i, location) in r.into_iter().enumerate() {
						println!("\nLocation #{}\nID={}\nLocation={}",
								 i, location.id, location.location);
					}
				}
				Err(err) => {
					eprintln!("Failed to deserialize response, e={:?}", err);
					process::exit(EXIT_CODE_API_UNEXPECTED_ERROR);
				}
			}
		}
		StatusCode::NOT_FOUND => {
			println!("You do not have a PAYG tariff nor Monthly Plan active")
		}
		_ => {
			handle_default_response_status_codes(response).await;
		}
	}
}

#[derive(Deserialize)]
struct StorageListEntry {
	id: String,
	location: String,
}


pub async fn command_storage_detail(client: Client, storage_id: &str) {
	let request_url = format!("{}/storage-location/{}", CDN77_API_BASE, storage_id);
	let response = send_http_request_return_response_or_exit(client.get(request_url)).await;

	match response.status() {
		StatusCode::OK => {
			match response.json::<StorageDetailResponse>().await {
				Ok(r) => {
					println!("ID={}\nLocation={}", r.id, r.location);
				}
				Err(err) => {
					eprintln!("Failed to deserialize response, e={:?}", err);
					process::exit(EXIT_CODE_API_UNEXPECTED_ERROR);
				}
			}
		}
		StatusCode::NOT_FOUND => {
			println!("You do not have a PAYG tariff nor Monthly Plan active")
		}
		_ => {
			handle_default_response_status_codes(response).await;
		}
	}
}

#[derive(Deserialize)]
struct StorageDetailResponse {
	id: String,
	location: String,
}
