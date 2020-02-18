// Copyright 2018-2020 Cargill Incorporated
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::collections::HashMap;
use std::thread::sleep;
use std::time::{Duration, Instant};

use actix_web::{client::Client, dev::Body, http::StatusCode, web, Error, HttpResponse};
use splinter::node_registry::Node;
use splinter::protocol;
use splinter::service::scabbard::{BatchInfo, BatchStatus};

use super::{ErrorResponse, SuccessResponse};

use crate::rest_api::RestApiResponseError;

const DEFAULT_WAIT: u64 = 30; // default wait time in seconds for batch to be commited

pub async fn submit_signed_payload(
    client: web::Data<Client>,
    splinterd_url: web::Data<String>,
    signed_payload: web::Bytes,
) -> Result<HttpResponse, Error> {
    let mut response = client
        .post(format!("{}/admin/submit", *splinterd_url))
        .header(
            "SplinterProtocolVersion",
            protocol::ADMIN_PROTOCOL_VERSION.to_string(),
        )
        .send_body(Body::Bytes(signed_payload))
        .await
        .map_err(Error::from)?;

    let status = response.status();
    let body = response.body().await?;

    match status {
        StatusCode::ACCEPTED => Ok(HttpResponse::Accepted().json(SuccessResponse::new(
            "The payload was submitted successfully",
        ))),
        StatusCode::BAD_REQUEST => {
            let body_value: serde_json::Value = serde_json::from_slice(&body)?;
            let message = match body_value.get("message") {
                Some(value) => value.as_str().unwrap_or("Request malformed."),
                None => "Request malformed.",
            };
            Ok(HttpResponse::BadRequest().json(ErrorResponse::bad_request(&message)))
        }
        _ => {
            debug!(
                "Internal Server Error. Splinterd responded with error {}",
                response.status(),
            );

            Ok(HttpResponse::InternalServerError().json(ErrorResponse::internal_error()))
        }
    }
}

pub async fn submit_scabbard_payload(
    client: web::Data<Client>,
    splinterd_url: web::Data<String>,
    circuit_id: web::Path<String>,
    node_info: web::Data<Node>,
    signed_payload: web::Bytes,
    query: web::Query<HashMap<String, String>>,
) -> Result<HttpResponse, Error> {
    let service_id = format!("gameroom_{}", node_info.identity);
    let wait = query
        .get("wait")
        .map(|val| match val.as_ref() {
            "false" => 0,
            _ => val.parse().unwrap_or(DEFAULT_WAIT),
        })
        .unwrap_or_else(|| DEFAULT_WAIT);

    let mut response = client
        .post(format!(
            "{}/scabbard/{}/{}/batches",
            *splinterd_url, &circuit_id, &service_id
        ))
        .header(
            "SplinterProtocolVersion",
            protocol::SCABBARD_PROTOCOL_VERSION.to_string(),
        )
        .send_body(Body::Bytes(signed_payload))
        .await?;

    let status = response.status();
    let body = response.body().await?;

    let link = match status {
        StatusCode::ACCEPTED => match parse_link(&body) {
            Ok(value) => value,
            Err(err) => {
                debug!(
                    "Internal Server Error. Error parsing splinter daemon response {}",
                    err
                );

                return Ok(
                    HttpResponse::InternalServerError().json(ErrorResponse::internal_error())
                );
            }
        },
        StatusCode::BAD_REQUEST => {
            let body_value: serde_json::Value = serde_json::from_slice(&body)?;
            let message = match body_value.get("message") {
                Some(value) => value.as_str().unwrap_or("Request malformed."),
                None => "Request malformed.",
            };

            return Ok(HttpResponse::BadRequest().json(ErrorResponse::bad_request(&message)));
        }
        _ => {
            let body_value: serde_json::Value = serde_json::from_slice(&body)?;

            let message = match body_value.get("message") {
                Some(value) => value.as_str().unwrap_or("Unknown cause"),
                None => "Unknown cause",
            };
            debug!(
                        "Internal Server Error. Gameroom service responded with an error {} with message {}",
                        response.status(),
                        message
                    );
            return Ok(HttpResponse::InternalServerError().json(ErrorResponse::internal_error()));
        }
    };
    let start = Instant::now();
    match check_batch_status(client, &splinterd_url, &link, start, wait).await {
        Ok(batches_info) => {
            let invalid_batches = batches_info
                .iter()
                .filter(|batch| {
                    if let BatchStatus::Invalid(_) = batch.status {
                        return true;
                    }
                    false
                })
                .collect::<Vec<&BatchInfo>>();
            if !invalid_batches.is_empty() {
                let error_message = process_failed_baches(&invalid_batches);
                return Ok(
                    HttpResponse::BadRequest().json(ErrorResponse::bad_request_with_data(
                        &error_message,
                        batches_info,
                    )),
                );
            }

            if batches_info
                .iter()
                .any(|batch| batch.status == BatchStatus::Pending)
            {
                return Ok(HttpResponse::Accepted().json(SuccessResponse::new(batches_info)));
            }

            Ok(HttpResponse::Ok().json(SuccessResponse::new(batches_info)))
        }
        Err(err) => match err {
            RestApiResponseError::BadRequest(message) => {
                Ok(HttpResponse::BadRequest().json(ErrorResponse::bad_request(&message)))
            }
            _ => Ok(HttpResponse::InternalServerError().json(ErrorResponse::internal_error())),
        },
    }
}

fn parse_link(response_bytes: &[u8]) -> Result<String, RestApiResponseError> {
    let mut response_value: HashMap<String, String> = serde_json::from_slice(&response_bytes)
        .map_err(|err| {
            RestApiResponseError::InternalError(format!(
                "Failed to parse batches_ids from splinterd response {}",
                err
            ))
        })?;

    if let Some(link) = response_value.remove("link") {
        Ok(link)
    } else {
        Err(RestApiResponseError::InternalError(
            "The splinter daemon did not return a link for batch status".to_string(),
        ))
    }
}

fn process_failed_baches(invalid_batches: &[&BatchInfo]) -> String {
    if invalid_batches.is_empty() {
        "".to_string()
    } else if invalid_batches.len() == 1 {
        if let BatchStatus::Invalid(invalid_transactions) = &invalid_batches[0].status {
            if invalid_transactions.len() <= 1 {
                "A transaction failed. Please try again. If it continues to fail contact your administrator for help.".to_string()
            } else {
                "Several transactions failed. Please try again. If it continues to fail contact your administrator for help.".to_string()
            }
        } else {
            "".to_string()
        }
    } else {
        "Several transactions failed. Please try again. If it continues to fail please contact your administrator.".to_string()
    }
}

async fn check_batch_status(
    client: web::Data<Client>,
    splinterd_url: &str,
    link: &str,
    start_time: Instant,
    wait: u64,
) -> Result<Vec<BatchInfo>, RestApiResponseError> {
    let splinterd_url = splinterd_url.to_owned();
    let link = link.to_owned();

    loop {
        debug!("Checking batch status {}", link);
        let mut response = match client
            .get(format!("{}{}", splinterd_url, link))
            .header(
                "SplinterProtocolVersion",
                protocol::SCABBARD_PROTOCOL_VERSION.to_string(),
            )
            .send()
            .await
            .map_err(|err| {
                RestApiResponseError::InternalError(format!("Failed to send request {}", err))
            }) {
            Ok(r) => r,
            Err(err) => {
                return Err(RestApiResponseError::InternalError(format!(
                    "Failed to retrieve state: {}",
                    err
                )));
            }
        };

        let body = match response.body().await {
            Ok(b) => b,
            Err(err) => {
                return Err(RestApiResponseError::InternalError(format!(
                    "Failed to receive response body {}",
                    err
                )));
            }
        };
        match response.status() {
            StatusCode::OK => {
                let batches_info: Vec<BatchInfo> = match serde_json::from_slice(&body) {
                    Ok(b) => b,
                    Err(err) => {
                        return Err(RestApiResponseError::InternalError(format!(
                            "Failed to parse response body {}",
                            err
                        )));
                    }
                };

                // If batch status is still pending and the wait time has not yet passed,
                // send request again to re-check the batch status
                if batches_info
                    .iter()
                    .any(|batch_info| match batch_info.status {
                        BatchStatus::Pending => true,
                        BatchStatus::Valid(_) => true,
                        _ => false,
                    })
                    && Instant::now().duration_since(start_time) < Duration::from_secs(wait)
                {
                    // wait one second before sending request again
                    sleep(Duration::from_secs(1));
                    continue;
                } else {
                    return Ok(batches_info);
                }
            }
            StatusCode::BAD_REQUEST => {
                let body_value: serde_json::Value = match serde_json::from_slice(&body) {
                    Ok(b) => b,
                    Err(err) => {
                        return Err(RestApiResponseError::InternalError(format!(
                            "Failed to parse response body {}",
                            err
                        )));
                    }
                };

                let message = match body_value.get("message") {
                    Some(value) => value.as_str().unwrap_or("Request malformed."),
                    None => "Request malformed.",
                };

                return Err(RestApiResponseError::BadRequest(message.to_string()));
            }
            _ => {
                let body_value: serde_json::Value = match serde_json::from_slice(&body) {
                    Ok(b) => b,
                    Err(err) => {
                        return Err(RestApiResponseError::InternalError(format!(
                            "Failed to parse response body {}",
                            err
                        )));
                    }
                };

                let message = match body_value.get("message") {
                    Some(value) => value.as_str().unwrap_or("Unknown cause"),
                    None => "Unknown cause",
                };

                return Err(RestApiResponseError::InternalError(message.to_string()));
            }
        };
    }
}
