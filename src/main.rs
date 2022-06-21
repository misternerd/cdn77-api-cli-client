extern crate core;

use std::env;

use clap::{Parser, Subcommand};
use dotenv::dotenv;
use reqwest::{Client, header};

use crate::commands_billing::command_billing_get_credit_balance;
use crate::commands_jobs::{command_jobs_detail, command_jobs_list, command_jobs_prefetch, command_jobs_purge, command_jobs_purge_all, JobType};
use crate::commands_statistics::{command_stats_bandwidth_95th_percentile, command_stats_get_stats, GetStatsType};
use crate::commands_storage::{command_storage_detail, command_storage_list};
use crate::util::ResourceId;

mod commands_billing;
mod commands_jobs;
mod commands_storage;
mod commands_statistics;
mod util;

pub const CDN77_API_BASE: &str = "https://api.cdn77.com/v3";
const USER_AGENT: &'static str = "cdn77-api-cli-client (https://github.com/misternerd/cdn77-api-cli-client)";

/// The user provided some unexpected/invalid input
pub const EXIT_CODE_INVALID_INPUT: i32 = 2;
/// The API provided a non-success code, but it might be expected (like "not found")
pub const EXIT_CODE_API_EXPECTED_ERROR: i32 = 3;
/// The API provided a non-success code, but it is unexpected (like "invalid input" or "invalid HTTP method")
pub const EXIT_CODE_API_UNEXPECTED_ERROR: i32 = 4;


#[derive(Parser)]
#[clap(
name = "cdn77-client",
version = "0.1",
author, about,
long_about = "Command line client for the CDN77 API."
)]
#[clap(propagate_version = true)]
struct CliOpts {
	#[clap(short = 'a', long)]
	/// Either provide the token (dangerous!) or create an environment variable `CDN77_API_TOKEN` (preferred)
	api_token: Option<String>,
	#[clap(subcommand)]
	command: RootCommands,
}

#[derive(Subcommand)]
enum RootCommands {
	#[clap(subcommand)]
	/// Information about credit balance
	Billing(BillingCommands),
	#[clap(subcommand)]
	/// Status and commands for/of puring and prefetching
	Jobs(JobsCommands),
	#[clap(subcommand)]
	/// CRUD operations for origins,
	Origin(OriginCommands),
	#[clap(subcommand)]
	/// Changing settings and getting raw logs
	RawLogs(RawLogCommands),
	#[clap(subcommand)]
	/// CRUD operations for CDN resources
	Resources(ResourcesCommands),
	#[clap(subcommand)]
	/// Get statistics
	Statistics(StatisticsCommands),
	#[clap(subcommand)]
	/// Infos about storage locations
	Storage(StorageCommands),
}

#[derive(Debug, Subcommand)]
enum BillingCommands {
	/// List the current credit balance
	CreditBalance,
}

#[derive(Debug, Subcommand)]
enum JobsCommands {
	/// List all jobs of a certain type
	List {
		#[clap(short = 'i', long)]
		/// The ID of the resource which you'd like to purge files from
		resource_id: ResourceId,
		#[clap(short = 't', long)]
		/// Which jobs to list (prefetch, purge, purge-all)
		job_type: JobType,
	},
	/// Display details about a job
	Detail {
		#[clap(short = 'i', long)]
		/// The ID of the resource which you'd like to purge files from
		resource_id: ResourceId,
		#[clap(short = 'i', long)]
		/// The ID of the resource which you'd like to purge files from
		job_id: String,
	},
	/// Prefetch a list of files on a CDN resource
	Prefetch {
		#[clap(short = 'i', long)]
		/// The ID of the resource which you'd like to purge files from
		resource_id: ResourceId,
		#[clap(short = 'p', long)]
		/// A comma separated list of paths to prefetch
		paths: String,
		#[clap(short = 'u', long)]
		/// Use when host header forwarding is active on your CDN Resource
		upstream_host: Option<String>,
	},
	/// Purge a list of files/paths from a resource
	Purge {
		#[clap(short = 'i', long)]
		/// The ID of the resource which you'd like to purge files from
		resource_id: ResourceId,
		#[clap(short = 'p', long)]
		/// A comma seperated list of paths you'd like to clear.
		/// Can contain wildcards (*)
		paths: String,
	},
	/// Purge all files from a specific CDN resource
	PurgeAll {
		#[clap(short = 'i', long)]
		/// The ID of the resource which you'd like to purge all files from
		resource_id: ResourceId,
	},
}

#[derive(Debug, Subcommand)]
enum OriginCommands {}

#[derive(Debug, Subcommand)]
enum RawLogCommands {}

#[derive(Debug, Subcommand)]
enum ResourcesCommands {
	/// List all CDN resources
	List,
}

#[derive(Debug, Subcommand)]
enum StatisticsCommands {
	/// Retrieve various stats. This method outputs prettified JSON.
	Get {
		#[clap(short = 't', long)]
		/// Stat type: Costs, Headers, HeadersDetail, HitMiss, HitMissDetail, Traffic, TrafficDetail,
		stat_type: GetStatsType,
		#[clap(short = 'f', long)]
		/// Start date/time in format: YYYY-MM-DD hh:mm
		from: String,
		#[clap(short = 'e', long)]
		/// End date/time in format YYYY-MM-DD hh:mm
		to: String,
		#[clap(short = 'i', long)]
		/// (opt) IDs of CDN resources, defaults to all
		resource_ids: Option<String>,
		#[clap(short = 'l', long)]
		/// (opt) Location names (e.g. 'prague'), defaults to all
		location_ids: Option<String>,
		#[clap(short = 'a', long)]
		/// Aggregation, examples from docs: 5-m, 1-h, 1-d, 1-month (minutes must be divisible by five)
		aggregation: Option<String>
	},
	Bandwidth95Percentile {
		#[clap(short = 'f', long)]
		/// Start date/time in format: YYYY-MM-DD hh:mm
		from: String,
		#[clap(short = 'e', long)]
		/// End date/time in format YYYY-MM-DD hh:mm
		to: String,
		#[clap(short = 'i', long)]
		/// (opt) IDs of CDN resources, defaults to all
		resource_ids: Option<String>,
		#[clap(short = 'l', long)]
		/// (opt) Location names (e.g. 'prague'), defaults to all
		location_ids: Option<String>,
	},
}

#[derive(Debug, Subcommand)]
enum StorageCommands {
	/// List all storage locations
	List,
	/// Show details for a storage location
	Detail {
		#[clap(short = 'i', long)]
		/// The ID of the storage location to show
		storage_id: String,
	}
}

#[tokio::main]
async fn main() {
	dotenv().ok();
	let cli_opts = CliOpts::parse();
	let client = create_cdn77_client(&cli_opts);

	match &cli_opts.command {
		RootCommands::Billing(command) => {
			match &command {
				BillingCommands::CreditBalance {} => {
					command_billing_get_credit_balance(client).await;
				}
			}
		}
		RootCommands::Jobs(command) => {
			match &command {
				JobsCommands::List { resource_id, job_type } => {
					command_jobs_list(client, resource_id, job_type).await;
				}
				JobsCommands::Detail { resource_id, job_id } => {
					command_jobs_detail(client, resource_id, job_id).await;
				}
				JobsCommands::Prefetch { resource_id, paths, upstream_host } => {
					command_jobs_prefetch(client, resource_id, paths, upstream_host).await;
				}
				JobsCommands::Purge { resource_id, paths } => {
					command_jobs_purge(client, resource_id, paths).await;
				}
				JobsCommands::PurgeAll { resource_id } => {
					command_jobs_purge_all(client, resource_id).await;
				}
			}
		}
		RootCommands::Origin(command) => {
			// TODO Implement https://client.cdn77.com/support/api-reference/v3/origin
			panic!("Origin isn't implemented yet! {:?}", command);
		}
		RootCommands::RawLogs(command) => {
			// TODO Implement https://client.cdn77.com/support/api-reference/v3/raw-logs
			panic!("RawLog isn't implemented yet! {:?}", command);
		}
		RootCommands::Resources(command) => {
			// TODO Implement https://client.cdn77.com/support/api-reference/v3/cdn-resources
			panic!("Origin isn't implemented yet! {:?}", command);
		}
		RootCommands::Statistics(command) => {
			match &command {
				StatisticsCommands::Get {stat_type, from, to, resource_ids, location_ids, aggregation, } => {
					command_stats_get_stats(client, stat_type, from, to, resource_ids, location_ids, aggregation).await;
				},
				StatisticsCommands::Bandwidth95Percentile {from, to, resource_ids, location_ids} => {
					command_stats_bandwidth_95th_percentile(client, from, to, resource_ids, location_ids).await;
				}
			}

			// TODO Implement https://client.cdn77.com/support/api-reference/v3/statistics
			panic!("Statistic isn't implemented yet! {:?}", command);
		}
		RootCommands::Storage(command) => {
			match &command {
				StorageCommands::List => {
					command_storage_list(client).await;
				}
				StorageCommands::Detail { storage_id } => {
					command_storage_detail(client, storage_id).await;
				}
			}
		}
	}
}

fn create_cdn77_client(cli_opts: &CliOpts) -> Client {
	let token = match &cli_opts.api_token {
		Some(t) => t.to_string(),
		_ => match env::var("CDN77_API_TOKEN") {
			Ok(t) => t,
			Err(_) => {
				eprintln!("No API token detected, please specify one either in the arguments or via env");
				std::process::exit(EXIT_CODE_INVALID_INPUT);
			}
		},
	};

	let mut default_headers = header::HeaderMap::new();
	let token = format!("Bearer {}", &token);
	default_headers.insert(header::AUTHORIZATION, header::HeaderValue::from_str(token.as_str()).unwrap());
	default_headers.append(header::USER_AGENT,
						  header::HeaderValue::from_str(USER_AGENT).unwrap());

	Client::builder()
		.default_headers(default_headers)
		.build()
		.unwrap_or_else(|err| panic!("Failed to create Reqwuest client: {:?}", err))
}
