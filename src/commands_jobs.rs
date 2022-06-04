use std::collections::HashMap;
use std::process;
use std::str::FromStr;

use log::{debug, info, warn};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};

use crate::{CDN77_API_BASE, ResourceId};
use crate::util::{handle_default_response_status_codes};

// Docs: https://client.cdn77.com/support/api-reference/v3/jobs

#[derive(Debug)]
pub enum JobType {
	Prefetch,
	Purge,
	PurgeAll,
}

pub async fn command_jobs_list(client: Client, resource_id: &ResourceId, job_type: &JobType) {
	let job_type = match job_type {
		JobType::Prefetch => "prefetch",
		JobType::Purge => "purge",
		JobType::PurgeAll => "purge-all",
	};
	debug!("Listing jobs of type={} for resource_id={}", job_type, &resource_id);
	let request_url = format!("{}/cdn/{}/job-log/{}", CDN77_API_BASE, &resource_id, job_type);
	let response = client.get(request_url)
		.send()
		.await;

	let response = match response {
		Ok(r) => r,
		Err(err) => {
			warn!("Failed to list jobs, e={:?}", err);
			return;
		}
	};

	match response.status() {
		StatusCode::OK => {
			match response.json::<Vec<ListJobDetail>>().await {
				Ok(r) => {
					debug!("Found {} jobs", &r.len());

					for (i, job) in r.into_iter().enumerate() {
						debug!("Job #{}\nID={}\nType={}\nCDN={:?}\nPathsCount={}\nState={}\nQueuedAt={}\nDoneAt={}",
							i, job.id, job.resource_type, job.cdn, job.paths_count, job.state, job.queued_at, job.done_at);
					}
				}
				Err(err) => {
					warn!("Failed to deserialize list-jobs response, e={:?}", err);
				}
			}
		}
		_ => {
			handle_default_response_status_codes(response).await;
		}
	}
}

impl FromStr for JobType {
	type Err = &'static str;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"prefetch" => Ok(JobType::Prefetch),
			"purge" => Ok(JobType::Purge),
			"purge-all" => Ok(JobType::PurgeAll),
			_ => Err("Invalid job type"),
		}
	}
}

#[derive(Debug, Deserialize)]
struct ListJobDetail {
	id: String,
	#[serde(rename = "type")]
	resource_type: String,
	cdn: HashMap<String, ResourceId>,
	paths_count: u64,
	state: String,
	queued_at: String,
	done_at: String,
}

pub async fn command_jobs_detail(client: Client, resource_id: &ResourceId, job_id: &str) {
	debug!("Getting job details for job_id={} in resource_id={}", job_id, resource_id);
	let request_url = format!("{}/cdn/{}/job/{}", CDN77_API_BASE, resource_id, job_id);
	let response = client.get(request_url)
		.send()
		.await;

	let response = match response {
		Ok(r) => r,
		Err(err) => {
			warn!("Failed to get job_id={}, e={:?}", job_id, err);
			return;
		}
	};

	match response.status() {
		StatusCode::OK => {
			match response.json::<GetJobDetailsResponse>().await {
				Ok(r) => {
					debug!("Found Job\nID={}\nType={}\nCDN={:?}\nPaths={:?}\nPathsCount={}\nState={}\nQueuedAt={}\nDoneAt={}",
						r.id, r.resource_type, r.cdn, r.paths, r.paths_count, r.state, r.queued_at, r.done_at);
				}
				Err(err) => {
					warn!("Failed to deserialize job-details response, e={:?}", err);
				}
			}
		}
		StatusCode::NOT_FOUND => {
			info!("Didn't find job_id={} for resource_id={}", job_id, resource_id);
		}
		_ => {
			handle_default_response_status_codes(response).await;
		}
	}
}

#[derive(Debug, Deserialize)]
struct GetJobDetailsResponse {
	id: String,
	#[serde(rename = "type")]
	resource_type: String,
	cdn: HashMap<String, ResourceId>,
	paths: Vec<String>,
	paths_count: u64,
	state: String,
	queued_at: String,
	done_at: String,
}

pub async fn command_jobs_prefetch(client: Client, resource_id: &ResourceId, paths: &str, upstream_host: &Option<String>) {
	let paths: Vec<String> = paths.split(',')
		.filter(|s| !s.is_empty())
		.map(|s| s.to_string()).collect();

	if paths.is_empty() {
		warn!("Please specify at least one path");
		process::exit(1);
	}

	debug!("Prefetching paths={:?} from resource_id={}", &paths, resource_id);
	let request_url = format!("{}/cdn/{}/job/prefetch", CDN77_API_BASE, resource_id);
	let request = PrefetchRequest{
		paths,
		upstream_host: upstream_host.clone(),
	};
	let response = client.post(request_url)
		.json(&request)
		.send()
		.await;

	let response = match response {
		Ok(r) => r,
		Err(err) => {
			warn!("Failed to execute purge, e={:?}", err);
			return;
		}
	};

	match response.status() {
		StatusCode::ACCEPTED => {
			match response.json::<PrefetchResponse>().await {
				Ok(r) => {
					debug!("Successfully executed {} of resource_ids={:?}\nJobID={}\nPaths={}/{:?}\nState={}\nQueuedAt={}",
						r.resource_type, r.cdn, r.id, r.paths_count, r.paths, r.state, r.queued_at);
				}
				Err(err) => {
					warn!("Failed to deserialize prefetch response, e={:?}", err);
				}
			}
		}
		StatusCode::NOT_FOUND => {
			warn!("Cannot prefetch paths, didn't find resource_id={}", resource_id);
		}
		_ => {
			handle_default_response_status_codes(response).await;
		}
	}
}

#[derive(Serialize)]
struct PrefetchRequest {
	paths: Vec<String>,
	upstream_host: Option<String>,
}

#[derive(Deserialize)]
struct PrefetchResponse {
	id: String,
	#[serde(rename = "type")]
	resource_type: String,
	cdn: HashMap<String, ResourceId>,
	paths: Vec<String>,
	paths_count: u64,
	state: String,
	queued_at: String,
}

pub async fn command_jobs_purge_all(client: Client, resource_id: &ResourceId) {
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
		}
	};

	match response.status() {
		StatusCode::ACCEPTED => {
			match response.json::<PurgeAllResponse>().await {
				Ok(r) => {
					debug!("Successfully executed {} of resource IDs {:?}\nJobID={}\nType={}\nState={}\nQueuedAt={}\nDoneAt={}",
						r.resource_type, r.cdn, r.id, r.resource_type, r.state, r.queued_at, r.done_at);
				}
				Err(err) => {
					warn!("Failed to deserialize purge-all response, e={:?}", err);
				}
			}
		}
		StatusCode::FORBIDDEN => {
			info!("Purging all files is disabled for resource={}: {:?}", resource_id, response);
		}
		StatusCode::NOT_FOUND => {
			warn!("Didn't find resource_id={}", resource_id);
		}
		_ => {
			handle_default_response_status_codes(response).await;
		}
	}
}


#[derive(Deserialize)]
struct PurgeAllResponse {
	id: String,
	#[serde(rename = "type")]
	resource_type: String,
	cdn: HashMap<String, ResourceId>,
	state: String,
	queued_at: String,
	done_at: String,
}
