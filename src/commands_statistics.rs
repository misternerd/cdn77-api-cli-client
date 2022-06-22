use std::fmt::{Display, Formatter};
use std::process;
use std::str::FromStr;

use reqwest::{Client, StatusCode};
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

use crate::{CDN77_API_BASE, EXIT_CODE_API_EXPECTED_ERROR, EXIT_CODE_API_UNEXPECTED_ERROR, EXIT_CODE_INVALID_INPUT, ResourceId};
use crate::util::{handle_default_response_status_codes, parse_date_time_or_exit, parse_resource_ids_optional, read_body_or_return_default_error_text, send_http_request_return_response_or_exit};

pub async fn command_stats_get_stats(client: Client, stat_type: &GetStatsType, from: &str, to: &str, resource_ids: &Option<String>,
									 location_ids: &Option<String>, aggregation: &Option<String>) {
	let from = parse_date_time_or_exit(from, "Start date/time is not in a correct format");
	let to = parse_date_time_or_exit(to, "End date/time is not in a correct format");
	let resource_ids = parse_resource_ids_optional(resource_ids);
	let location_ids = parse_optional_location_ids(location_ids);

	let request_url = format!("{}/stats/{}", CDN77_API_BASE, stat_type);
	let request = GetStatsRequest {
		from: from.timestamp(),
		to: to.timestamp(),
		cdn_ids: resource_ids,
		location_ids,
		aggregation: aggregation.clone(),
	};
	let response = send_http_request_return_response_or_exit(client.post(request_url).json(&request)).await;

	match response.status() {
		StatusCode::OK => {
			// JSON parsing is just here to validate valid JSON was returned
			match response.json::<Value>().await {
				Ok(r) => {
					println!("{}", serde_json::to_string_pretty(&r).unwrap());
				}
				Err(err) => {
					eprintln!("Failed to deserialize response, e={:?}", err);
					process::exit(EXIT_CODE_API_UNEXPECTED_ERROR);
				}
			}
		}
		StatusCode::NOT_FOUND => {
			eprintln!("Could not get stats for this type without grouping: {}", read_body_or_return_default_error_text(response).await);
			process::exit(EXIT_CODE_API_EXPECTED_ERROR);
		}
		_ => {
			handle_default_response_status_codes(response).await;
		}
	}
}

fn parse_optional_location_ids(location_ids: &Option<String>) -> Option<Vec<String>> {
	match location_ids {
		Some(r) => Some(r.split(',').map(|r| r.trim()).filter(|r| !r.is_empty()).map(|s| s.to_string()).collect()),
		None => None,
	}
}

// TODO This is a tad of an overkill, we're converting string => enum => string. Maybe just validate string instead?
#[derive(Debug)]
pub enum GetStatsType {
	Bandwidth,
	Costs,
	Headers,
	HeadersDetail,
	HitMiss,
	HitMissDetail,
	Traffic,
	TrafficDetail,
}

impl Display for GetStatsType {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		let value = match self {
			GetStatsType::Bandwidth => "bandwidth",
			GetStatsType::Costs => "costs",
			GetStatsType::Headers => "headers",
			GetStatsType::HeadersDetail => "headers-details",
			GetStatsType::HitMiss => "hit-miss",
			GetStatsType::HitMissDetail => "hit-miss-detail",
			GetStatsType::Traffic => "traffic",
			GetStatsType::TrafficDetail => "traffic-detail",
		};
		write!(f, "{}", value)
	}
}

impl FromStr for GetStatsType {
	type Err = &'static str;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"bandwidth" => Ok(GetStatsType::Bandwidth),
			"costs" => Ok(GetStatsType::Costs),
			"headers" => Ok(GetStatsType::Headers),
			"headers-detail" => Ok(GetStatsType::HeadersDetail),
			"hit-miss" => Ok(GetStatsType::HitMiss),
			"hit-miss-detail" => Ok(GetStatsType::HitMissDetail),
			"traffic" => Ok(GetStatsType::Traffic),
			"traffic-detail" => Ok(GetStatsType::TrafficDetail),
			_ => Err("Invalid stat type"),
		}
	}
}

#[derive(Serialize)]
struct GetStatsRequest {
	from: i64,
	to: i64,
	cdn_ids: Option<Vec<ResourceId>>,
	location_ids: Option<Vec<String>>,
	aggregation: Option<String>,
}


pub async fn command_stats_bandwidth_95th_percentile(client: Client, from: &str, to: &str, resource_ids: &Option<String>, location_ids: &Option<String>) {
	let from = parse_date_time_or_exit(from, "Start date/time is not in a correct format");
	let to = parse_date_time_or_exit(to, "End date/time is not in a correct format");
	let resource_ids = parse_resource_ids_optional(resource_ids);
	let location_ids = parse_optional_location_ids(location_ids);

	let request_url = format!("{}/stats/bandwidth/percentile", CDN77_API_BASE);
	let request = Bandwidth95PercentileRequest {
		from: from.timestamp(),
		to: to.timestamp(),
		cdn_ids: resource_ids,
		location_ids,
	};
	let response = send_http_request_return_response_or_exit(client.post(request_url).json(&request)).await;

	match response.status() {
		StatusCode::OK => {
			match response.json::<Bandwidth95PercentileResponse>().await {
				Ok(r) => {
					println!("Percentile: {}", r.percentile);
				}
				Err(err) => {
					eprintln!("Failed to deserialize response, e={:?}", err);
					process::exit(EXIT_CODE_API_UNEXPECTED_ERROR);
				}
			}
		}
		StatusCode::NOT_FOUND => {
			eprintln!("Could not get stats for this type without grouping: {}", read_body_or_return_default_error_text(response).await);
			process::exit(EXIT_CODE_API_EXPECTED_ERROR);
		}
		_ => {
			handle_default_response_status_codes(response).await;
		}
	}
}

#[derive(Serialize)]
struct Bandwidth95PercentileRequest {
	from: i64,
	to: i64,
	cdn_ids: Option<Vec<ResourceId>>,
	location_ids: Option<Vec<String>>,
}

#[derive(Deserialize)]
struct Bandwidth95PercentileResponse {
	percentile: i64,
}


pub async fn command_stats_by_resource(client: Client, stat_type: &GetStatsType, from: &str, to: &str, resource_ids: &Option<String>,
									   location_ids: &Option<String>, aggregation: &Option<String>) {
	let from = parse_date_time_or_exit(from, "Start date/time is not in a correct format");
	let to = parse_date_time_or_exit(to, "End date/time is not in a correct format");
	let resource_ids = parse_resource_ids_optional(resource_ids);
	let location_ids = parse_optional_location_ids(location_ids);

	let request_url = format!("{}/stats/cdns/{}", CDN77_API_BASE, stat_type);
	let request = StatsByResourceRequest {
		from: from.timestamp(),
		to: to.timestamp(),
		cdn_ids: resource_ids,
		location_ids,
		aggregation: aggregation.clone(),
	};
	let response = send_http_request_return_response_or_exit(client.post(request_url).json(&request)).await;

	match response.status() {
		StatusCode::OK => {
			match response.json::<Value>().await {
				Ok(r) => {
					println!("{}", serde_json::to_string_pretty(&r).unwrap());
				}
				Err(err) => {
					eprintln!("Failed to deserialize response, e={:?}", err);
					process::exit(EXIT_CODE_API_UNEXPECTED_ERROR);
				}
			}
		}
		StatusCode::NOT_FOUND => {
			eprintln!("Couldn't get stat type grouped by resource: {}", read_body_or_return_default_error_text(response).await);
			process::exit(EXIT_CODE_API_EXPECTED_ERROR);
		}
		_ => {
			handle_default_response_status_codes(response).await;
		}
	}
}

#[derive(Serialize)]
struct StatsByResourceRequest {
	from: i64,
	to: i64,
	cdn_ids: Option<Vec<ResourceId>>,
	location_ids: Option<Vec<String>>,
	aggregation: Option<String>,
}


pub async fn command_stats_sum_by_resource(client: Client, stat_type: &String, from: &str, to: &str, resource_ids: &Option<String>,
										   location_ids: &Option<String>) {
	if !SUM_BY_RESOURCE_TYPE.contains(&&stat_type[..]) {
		eprintln!("Invalid stat type: {}", stat_type);
		process::exit(EXIT_CODE_INVALID_INPUT);
	}

	let from = parse_date_time_or_exit(from, "Start date/time is not in a correct format");
	let to = parse_date_time_or_exit(to, "End date/time is not in a correct format");
	let resource_ids = parse_resource_ids_optional(resource_ids);
	let location_ids = parse_optional_location_ids(location_ids);

	let request_url = format!("{}/stats/cdns/sum/{}", CDN77_API_BASE, stat_type);
	let request = SumByResourceRequest {
		from: from.timestamp(),
		to: to.timestamp(),
		cdn_ids: resource_ids,
		location_ids,
	};
	let response = send_http_request_return_response_or_exit(client.post(request_url).json(&request)).await;

	match response.status() {
		StatusCode::OK => {
			match response.json::<Value>().await {
				Ok(r) => {
					println!("{}", serde_json::to_string_pretty(&r).unwrap());
				}
				Err(err) => {
					eprintln!("Failed to deserialize response, e={:?}", err);
					process::exit(EXIT_CODE_API_UNEXPECTED_ERROR);
				}
			}
		}
		StatusCode::NOT_FOUND => {
			eprintln!("Couldn't get stat sum by resource: {}", read_body_or_return_default_error_text(response).await);
			process::exit(EXIT_CODE_API_EXPECTED_ERROR);
		}
		_ => {
			handle_default_response_status_codes(response).await;
		}
	}
}

const SUM_BY_RESOURCE_TYPE: [&'static str; 4] = ["headers", "traffic", "hit-miss", "costs"];

#[derive(Serialize)]
struct SumByResourceRequest {
	from: i64,
	to: i64,
	cdn_ids: Option<Vec<ResourceId>>,
	location_ids: Option<Vec<String>>,
}


pub async fn command_stats_by_data_center(client: Client, stat_type: &GetStatsType, from: &str, to: &str, resource_ids: &Option<String>,
										  location_ids: &Option<String>, aggregation: &Option<String>) {
	let from = parse_date_time_or_exit(from, "Start date/time is not in a correct format");
	let to = parse_date_time_or_exit(to, "End date/time is not in a correct format");
	let resource_ids = parse_resource_ids_optional(resource_ids);
	let location_ids = parse_optional_location_ids(location_ids);

	let request_url = format!("{}/stats/datacenters/{}", CDN77_API_BASE, stat_type);
	let request = StatsByResourceRequest {
		from: from.timestamp(),
		to: to.timestamp(),
		cdn_ids: resource_ids,
		location_ids,
		aggregation: aggregation.clone(),
	};
	let response = send_http_request_return_response_or_exit(client.post(request_url).json(&request)).await;

	match response.status() {
		StatusCode::OK => {
			match response.json::<Value>().await {
				Ok(r) => {
					println!("{}", serde_json::to_string_pretty(&r).unwrap());
				}
				Err(err) => {
					eprintln!("Failed to deserialize response, e={:?}", err);
					process::exit(EXIT_CODE_API_UNEXPECTED_ERROR);
				}
			}
		}
		StatusCode::NOT_FOUND => {
			eprintln!("Couldn't get stat type grouped by datacenter: {}", read_body_or_return_default_error_text(response).await);
			process::exit(EXIT_CODE_API_EXPECTED_ERROR);
		}
		_ => {
			handle_default_response_status_codes(response).await;
		}
	}
}

#[derive(Serialize)]
struct StatsByDataCenterRequest {
	from: i64,
	to: i64,
	cdn_ids: Option<Vec<ResourceId>>,
	location_ids: Option<Vec<String>>,
	aggregation: Option<String>,
}


pub async fn command_stats_sum_by_data_center(client: Client, stat_type: &String, from: &str, to: &str, resource_ids: &Option<String>,
											  location_ids: &Option<String>) {
	if !SUM_BY_DATA_CENTER_TYPE.contains(&&stat_type[..]) {
		eprintln!("Invalid stat type: {}", stat_type);
		process::exit(EXIT_CODE_INVALID_INPUT);
	}

	let from = parse_date_time_or_exit(from, "Start date/time is not in a correct format");
	let to = parse_date_time_or_exit(to, "End date/time is not in a correct format");
	let resource_ids = parse_resource_ids_optional(resource_ids);
	let location_ids = parse_optional_location_ids(location_ids);

	let request_url = format!("{}/stats/datacenters/sum/{}", CDN77_API_BASE, stat_type);
	let request = SumByDataCenterRequest {
		from: from.timestamp(),
		to: to.timestamp(),
		cdn_ids: resource_ids,
		location_ids,
	};
	let response = send_http_request_return_response_or_exit(client.post(request_url).json(&request)).await;

	match response.status() {
		StatusCode::OK => {
			match response.json::<Value>().await {
				Ok(r) => {
					println!("{}", serde_json::to_string_pretty(&r).unwrap());
				}
				Err(err) => {
					eprintln!("Failed to deserialize response, e={:?}", err);
					process::exit(EXIT_CODE_API_UNEXPECTED_ERROR);
				}
			}
		}
		StatusCode::NOT_FOUND => {
			eprintln!("Couldn't get stat sum by data center: {}", read_body_or_return_default_error_text(response).await);
			process::exit(EXIT_CODE_API_EXPECTED_ERROR);
		}
		_ => {
			handle_default_response_status_codes(response).await;
		}
	}
}

const SUM_BY_DATA_CENTER_TYPE: [&'static str; 4] = ["headers", "traffic", "hit-miss", "costs"];

#[derive(Serialize)]
struct SumByDataCenterRequest {
	from: i64,
	to: i64,
	cdn_ids: Option<Vec<ResourceId>>,
	location_ids: Option<Vec<String>>,
}


pub async fn command_stats_sum(client: Client, stat_type: &String, from: &str, to: &str, resource_ids: &Option<String>, location_ids: &Option<String>) {
	if !SUM_TYPE.contains(&&stat_type[..]) {
		eprintln!("Invalid stat type: {}", stat_type);
		process::exit(EXIT_CODE_INVALID_INPUT);
	}

	let from = parse_date_time_or_exit(from, "Start date/time is not in a correct format");
	let to = parse_date_time_or_exit(to, "End date/time is not in a correct format");
	let resource_ids = parse_resource_ids_optional(resource_ids);
	let location_ids = parse_optional_location_ids(location_ids);

	let request_url = format!("{}/stats/sum/{}", CDN77_API_BASE, stat_type);
	let request = SumRequest {
		from: from.timestamp(),
		to: to.timestamp(),
		cdn_ids: resource_ids,
		location_ids,
	};
	let response = send_http_request_return_response_or_exit(client.post(request_url).json(&request)).await;

	match response.status() {
		StatusCode::OK => {
			match response.json::<SumResponse>().await {
				Ok(r) => {
					println!("Sum: {}", r.sum);
				}
				Err(err) => {
					eprintln!("Failed to deserialize response, e={:?}", err);
					process::exit(EXIT_CODE_API_UNEXPECTED_ERROR);
				}
			}
		}
		StatusCode::NOT_FOUND => {
			eprintln!("Couldn't get stats sum: {}", read_body_or_return_default_error_text(response).await);
			process::exit(EXIT_CODE_API_EXPECTED_ERROR);
		}
		_ => {
			handle_default_response_status_codes(response).await;
		}
	}
}

const SUM_TYPE: [&'static str; 4] = ["headers", "traffic", "hit-miss", "costs"];

#[derive(Serialize)]
struct SumRequest {
	from: i64,
	to: i64,
	cdn_ids: Option<Vec<ResourceId>>,
	location_ids: Option<Vec<String>>,
}

#[derive(Deserialize)]
struct SumResponse {
	sum: f64,
}
