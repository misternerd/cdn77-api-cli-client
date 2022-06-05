extern crate core;

mod commands_jobs;
mod util;

use std::env;
use clap::{Parser, Subcommand};
use log::{debug, warn};
use dotenv::dotenv;
use reqwest::{Client, header};
use crate::commands_jobs::{command_jobs_detail, command_jobs_purge_all, command_jobs_list, JobType, command_jobs_prefetch};
use crate::util::ResourceId;


pub const CDN77_API_BASE: &str = "https://api.cdn77.com/v3";


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
		#[clap(short = 'r', long)]
		/// The ID of the resource which you'd like to purge files from
		resource_id: ResourceId,
		#[clap(short = 't', long)]
		/// Which jobs to list (prefetch, purge, purge-all)
		job_type: JobType,
	},
	/// Display details about a job
	Detail {
		#[clap(short = 'r', long)]
		/// The ID of the resource which you'd like to purge files from
		resource_id: ResourceId,
		#[clap(short = 'i', long)]
		/// The ID of the resource which you'd like to purge files from
		job_id: String,
	},
	/// Prefetch a list of files on a CDN resource
	Prefetch {
		#[clap(short = 'r', long)]
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
		#[clap(short = 'r', long)]
		/// The ID of the resource which you'd like to purge files from
		resource_id: ResourceId,
		#[clap(short = 'p', long)]
		/// A comma seperated list of paths you'd like to clear.
		/// Can contain wildcards (*)
		paths: String,
	},
	/// Purge all files from a specific CDN resource
	PurgeAll {
		#[clap(short = 'r', long)]
		/// The ID of the resource which you'd like to purge all files from
		resource_id: ResourceId,
	}
}

#[derive(Debug, Subcommand)]
enum OriginCommands {

}

#[derive(Debug, Subcommand)]
enum RawLogCommands {

}

#[derive(Debug, Subcommand)]
enum ResourcesCommands {
	/// List all CDN resources
	List,
}

#[derive(Debug, Subcommand)]
enum StatisticsCommands {

}

#[derive(Debug, Subcommand)]
enum StorageCommands {

}

#[tokio::main]
async fn main() {
	dotenv().ok();
	env_logger::init();
	let cli_opts = CliOpts::parse();

	let client = create_cdn77_client(&cli_opts);

	match &cli_opts.command {
		RootCommands::Billing(command) => {
			// TODO Implement https://client.cdn77.com/support/api-reference/v3/billing
			panic!("Billing isn't implemented yet! {:?}", command);
		}
		RootCommands::Jobs(command) => {
			match &command {
				JobsCommands::List {resource_id, job_type} => {
					command_jobs_list(client, resource_id, job_type).await;
				},
				JobsCommands::Detail {resource_id, job_id} => {
					command_jobs_detail(client, resource_id, job_id).await;
				},
				JobsCommands::Prefetch {resource_id, paths, upstream_host} => {
					command_jobs_prefetch(client, resource_id, paths, upstream_host).await;
				},
				JobsCommands::Purge { resource_id, paths } => {
					debug!("Purging resourceIds={} for resourceId={}", paths, resource_id);
				}
				JobsCommands::PurgeAll { resource_id } => {
					command_jobs_purge_all(client, resource_id).await;
				}
			}
		}
		RootCommands::Origin(command) => {
			// TODO Implement https://client.cdn77.com/support/api-reference/v3/origin
			panic!("Origin isn't implemented yet! {:?}", command);
		},
		RootCommands::RawLogs(command) => {
			// TODO Implement https://client.cdn77.com/support/api-reference/v3/raw-logs
			panic!("RawLog isn't implemented yet! {:?}", command);
		},
		RootCommands::Resources(command) => {
			// TODO Implement https://client.cdn77.com/support/api-reference/v3/cdn-resources
			panic!("Origin isn't implemented yet! {:?}", command);
		},
		RootCommands::Statistics(command) => {
			// TODO Implement https://client.cdn77.com/support/api-reference/v3/statistics
			panic!("Statistic isn't implemented yet! {:?}", command);
		},
		RootCommands::Storage(command) => {
			// TODO Implement https://client.cdn77.com/support/api-reference/v3/storage-location
			panic!("StorageLocation isn't implemented yet! {:?}", command);
		},
	}
}

fn create_cdn77_client(cli_opts: &CliOpts) -> Client {
	let token = match &cli_opts.api_token {
		Some(t) => t.to_string(),
		_ => match env::var("CDN77_API_TOKEN") {
			Ok(t) => t,
			Err(_) => {
				warn!("No API token detected, please specify one either in the arguments or via env");
				std::process::exit(1);
			}
		},
	};

	let mut default_headers = header::HeaderMap::new();
	let token = format!("Bearer {}", &token);
	default_headers.insert(header::AUTHORIZATION, header::HeaderValue::from_str(token.as_str()).unwrap());

	Client::builder()
		.default_headers(default_headers)
		.build()
		.unwrap_or_else(|err| panic!("Failed to create Reqwuest client: {:?}", err))
}
