// Copyright 2019 Cargill Incorporated
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

mod error;
mod routes;

use std::sync::mpsc;
use std::thread;

use actix_web::{client::Client, web, App, HttpServer, Result};
use futures::{
    future::{self, Either},
    Future, Stream,
};
use gameroom_database::ConnectionPool;
use hyper::{Client as HyperClient, StatusCode, Uri};
use libsplinter::node_registry::Node;
use serde_json::Value;
use tokio::runtime::Runtime;

pub use error::{RestApiResponseError, RestApiServerError};

pub struct RestApiShutdownHandle {
    do_shutdown: Box<dyn Fn() -> Result<(), RestApiServerError> + Send>,
}

impl RestApiShutdownHandle {
    pub fn shutdown(&self) -> Result<(), RestApiServerError> {
        (*self.do_shutdown)()
    }
}

pub fn run(
    bind_url: &str,
    splinterd_url: &str,
    database_connection: ConnectionPool,
) -> Result<
    (
        RestApiShutdownHandle,
        thread::JoinHandle<Result<(), RestApiServerError>>,
    ),
    RestApiServerError,
> {
    let bind_url = bind_url.to_owned();
    let splinterd_url = splinterd_url.to_owned();
    let (tx, rx) = mpsc::channel();
    let join_handle = thread::Builder::new()
        .name("GameroomdRestApi".into())
        .spawn(move || {
            let sys = actix::System::new("Gameroomd-Rest-API");

            // get splinter node information from splinterd
            let node = get_node(&splinterd_url)?;

            let addr = HttpServer::new(move || {
                App::new()
                    .data(database_connection.clone())
                    .data(Client::new())
                    .data(splinterd_url.to_owned())
                    .data(node.clone())
                    .service(
                        web::resource("/nodes/{identity}")
                            .route(web::get().to_async(routes::fetch_node)),
                    )
                    .service(web::resource("/nodes").route(web::get().to_async(routes::list_nodes)))
                    .service(
                        web::resource("/gamerooms/propose")
                            .route(web::post().to_async(routes::propose_gameroom)),
                    )
                    .service(
                        web::scope("/users")
                            .service(
                                web::resource("").route(web::post().to_async(routes::register)),
                            )
                            .service(
                                web::resource("/authenticate")
                                    .route(web::post().to_async(routes::login)),
                            ),
                    )
                    .service(
                        web::scope("/proposals")
                            .service(
                                web::resource("/{proposal_id}/vote")
                                    .route(web::post().to_async(routes::proposal_vote)),
                            )
                            .service(
                                web::resource("/{proposal_id}")
                                    .route(web::get().to_async(routes::fetch_proposal)),
                            )
                            .service(
                                web::resource("")
                                    .route(web::get().to_async(routes::list_proposals)),
                            ),
                    )
                    .service(
                        web::scope("/notifications")
                            .service(
                                web::scope("/{notification_id}")
                                    .service(
                                        web::resource("")
                                            .route(web::get().to_async(routes::fetch_notificaiton)),
                                    )
                                    .service(
                                        web::resource("/read").route(
                                            web::patch().to_async(routes::read_notification),
                                        ),
                                    ),
                            )
                            .service(
                                web::resource("")
                                    .route(web::get().to_async(routes::list_unread_notifications)),
                            ),
                    )
                    .service(
                        web::resource("/submit")
                            .route(web::post().to_async(routes::submit_signed_payload)),
                    )
                    .service(
                        web::scope("/gamerooms")
                            .service(
                                web::resource("")
                                    .route(web::get().to_async(routes::list_gamerooms)),
                            )
                            .service(
                                web::resource("/{circuit_id}")
                                    .route(web::get().to_async(routes::fetch_gameroom)),
                            ),
                    )
                    .service(
                        web::resource("/subscribe").route(web::get().to(routes::connect_socket)),
                    )
            })
            .bind(bind_url)?
            .disable_signals()
            .system_exit()
            .start();

            tx.send(addr).map_err(|err| {
                RestApiServerError::StartUpError(format!("Unable to send Server Addr: {}", err))
            })?;
            sys.run()?;

            info!("Rest API terminating");

            Ok(())
        })?;

    let addr = rx.recv().map_err(|err| {
        RestApiServerError::StartUpError(format!("Unable to receive Server Addr: {}", err))
    })?;

    let do_shutdown = Box::new(move || {
        debug!("Shutting down Rest API");
        addr.stop(true);
        debug!("Graceful signal sent to Rest API");

        Ok(())
    });

    Ok((RestApiShutdownHandle { do_shutdown }, join_handle))
}

fn get_node(splinterd_url: &str) -> Result<Node, RestApiServerError> {
    let mut runtime = Runtime::new()?;
    let client = HyperClient::new();
    let splinterd_url = splinterd_url.to_owned();
    let uri = format!("{}/status", splinterd_url)
        .parse::<Uri>()
        .map_err(|err| {
            RestApiServerError::StartUpError(format!("Failed to get set up request : {}", err))
        })?;

    runtime.block_on(
        client
            .get(uri)
            .map_err(|err| {
                RestApiServerError::StartUpError(format!(
                    "Failed to get splinter node metadata: {}",
                    err
                ))
            })
            .and_then(|resp| {
                if resp.status() != StatusCode::OK {
                    return Err(RestApiServerError::StartUpError(format!(
                        "Failed to get splinter node metadata. Splinterd responded with status {}",
                        resp.status()
                    )));
                }
                let body = resp
                    .into_body()
                    .concat2()
                    .wait()
                    .map_err(|err| {
                        RestApiServerError::StartUpError(format!(
                            "Failed to get splinter node metadata: {}",
                            err
                        ))
                    })?
                    .to_vec();

                let node_status: Value = serde_json::from_slice(&body).map_err(|err| {
                    RestApiServerError::StartUpError(format!(
                        "Failed to get splinter node metadata: {}",
                        err
                    ))
                })?;

                let node_id = match node_status.get("node_id") {
                    Some(node_id_val) => node_id_val.as_str().unwrap_or("").to_string(),
                    None => "".to_string(),
                };

                Ok(node_id)
            })
            .and_then(move |node_id| {
                let uri = match format!("{}/nodes/{}", splinterd_url, node_id).parse::<Uri>() {
                        Ok(uri) => uri,
                        Err(err) => return
                            Either::A(
                                future::err(RestApiServerError::StartUpError(format!(
                                    "Failed to get set up request : {}",
                                    err
                                ))))
                };

                Either::B(client
                    .get(uri)
                    .map_err(|err| {
                        RestApiServerError::StartUpError(format!(
                            "Failed to get splinter node: {}",
                            err
                        ))
                    })
                    .then(|resp| {
                        let response = resp?;
                        let status = response.status();
                        let body = response
                            .into_body()
                            .concat2()
                            .wait()
                            .map_err(|err| {
                                RestApiServerError::StartUpError(format!(
                                    "Failed to get splinter node metadata: {}",
                                    err
                                ))
                            })?
                            .to_vec();

                        match status {
                            StatusCode::OK => {
                                let node: Node = serde_json::from_slice(&body).map_err(|err| {
                                    RestApiServerError::StartUpError(format!(
                                        "Failed to get splinter node: {}",
                                        err
                                    ))
                                })?;

                                Ok(node)
                            }
                            _ => Err(RestApiServerError::StartUpError(format!(
                                "Failed to get splinter node data. Splinterd responded with status {}",
                                status
                            ))),
                        }
                    }))
            }),
    )
}
