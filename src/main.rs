extern crate core;

mod commands_jobs;
mod util;

use std::env;
use clap::{Parser, Subcommand};
use log::{debug, warn};
use dotenv::dotenv;
use reqwest::{Client, header};
use crate::commands_jobs::command_job_purge_all;


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
	#[clap(short, long)]
	/// Either provide the token (dangerous!) or create an environment variable `CDN77_API_TOKEN` (preferred)
	api_token: Option<String>,
	#[clap(subcommand)]
	command: RootCommand,
}

#[derive(Subcommand)]
enum RootCommand {
	#[clap(subcommand)]
	/// Information about credit balance
	Billing(BillingCommand),
	#[clap(subcommand)]
	/// Status and commands for/of puring and prefetching
	Job(JobCommand),
	#[clap(subcommand)]
	/// CRUD operations for origins,
	Origin(OriginCommand),
	#[clap(subcommand)]
	/// Changing settings and getting raw logs
	RawLog(RawLogCommand),
	#[clap(subcommand)]
	/// CRUD operations for CDN resources
	Resource(ResourceCommand),
	#[clap(subcommand)]
	/// Get statistics
	Statistic(StatisticCommand),
	#[clap(subcommand)]
	/// Infos about storage locations
	StorageLocation(StorageLocationCommand),
}

#[derive(Debug, Subcommand)]
enum BillingCommand {
	/// List the current credit balance
	CreditBalance,
}

#[derive(Debug, Subcommand)]
enum JobCommand {
	/// Purge a list of files/paths from a resource
	Purge {
		#[clap(short, long)]
		/// The ID of the resource which you'd like to purge files from
		resource_id: String,
		#[clap(short, long)]
		/// A comma seperated list of paths you'd like to clear.
		/// Can contain wildcards (*)
		paths: String,
	},
	/// Purge all files from a specific CDN resource
	PurgeAll {
		#[clap(short, long)]
		/// The ID of the resource which you'd like to purge all files from
		resource_id: String,
	}
}

#[derive(Debug, Subcommand)]
enum OriginCommand {

}

#[derive(Debug, Subcommand)]
enum RawLogCommand {

}

#[derive(Debug, Subcommand)]
enum ResourceCommand {
	/// List all CDN resources
	List,
}

#[derive(Debug, Subcommand)]
enum StatisticCommand {

}

#[derive(Debug, Subcommand)]
enum StorageLocationCommand {

}

#[tokio::main]
async fn main() {
	dotenv().ok();
	env_logger::init();
	let cli_opts = CliOpts::parse();

	let client = create_cdn77_client(&cli_opts);

	match &cli_opts.command {
		RootCommand::Billing(command) => {
			// TODO Implement https://client.cdn77.com/support/api-reference/v3/billing
			panic!("Billing isn't implemented yet! {:?}", command);
		}
		RootCommand::Job(command) => {
			match &command {
				JobCommand::Purge { resource_id, paths } => {
					debug!("Purging resourceIds={} for resourceId={}", paths, resource_id);
				}
				JobCommand::PurgeAll { resource_id } => {
					command_job_purge_all(client, resource_id).await;
				}
			}
		}
		RootCommand::Origin(command) => {
			// TODO Implement https://client.cdn77.com/support/api-reference/v3/origin
			panic!("Origin isn't implemented yet! {:?}", command);
		},
		RootCommand::RawLog(command) => {
			// TODO Implement https://client.cdn77.com/support/api-reference/v3/raw-logs
			panic!("RawLog isn't implemented yet! {:?}", command);
		},
		RootCommand::Resource(command) => {
			// TODO Implement https://client.cdn77.com/support/api-reference/v3/cdn-resources
			panic!("Origin isn't implemented yet! {:?}", command);
		},
		RootCommand::Statistic(command) => {
			// TODO Implement https://client.cdn77.com/support/api-reference/v3/statistics
			panic!("Statistic isn't implemented yet! {:?}", command);
		},
		RootCommand::StorageLocation(command) => {
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
