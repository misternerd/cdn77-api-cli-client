use std::process;

use chrono::{DateTime, NaiveDateTime, Utc};
use reqwest::{Client, StatusCode};
use serde::Deserialize;

use crate::{CDN77_API_BASE, EXIT_CODE_API_UNEXPECTED_ERROR};
use crate::util::{handle_default_response_status_codes, send_http_request_return_response_or_exit};

pub async fn command_billing_get_credit_balance(client: Client) {
	let request_url = format!("{}/credit-balance", CDN77_API_BASE);
	let response = send_http_request_return_response_or_exit(client.get(request_url)).await;

	match response.status() {
		StatusCode::OK => {
			match response.json::<GetCreditBalanceResponse>().await {
				Ok(r) => {
					let credits_expire = NaiveDateTime::from_timestamp(r.credit_expires_at, 0);
					let credits_expire = DateTime::<Utc>::from_utc(credits_expire, Utc);
					println!("Current balance:    {} $", r.current_credit);
					println!("Balance expires at: {}", credits_expire.format("%Y-%m-%d"));
					println!("Last 30 days spent: {} $", r.credit_spent_in_30_days);
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
struct GetCreditBalanceResponse {
	current_credit: f32,
	credit_expires_at: i64,
	credit_spent_in_30_days: f32,
}
