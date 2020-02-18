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

mod error;
mod routes;

use std::sync::mpsc;
use std::thread;

use actix_web::{
    client::Client, dev, error as ActixError, web, App, FromRequest, HttpResponse, HttpServer,
    Result,
};
use gameroom_database::ConnectionPool;
use splinter::node_registry::Node;

pub use error::{RestApiResponseError, RestApiServerError};
use routes::ErrorResponse;

#[derive(Clone)]
pub struct GameroomdData {
    pub public_key: String,
}

pub struct RestApiShutdownHandle {
    server: dev::Server,
}

impl RestApiShutdownHandle {
    pub async fn shutdown(&self) {
        self.server.stop(true).await;
    }
}

pub fn run(
    bind_url: &str,
    splinterd_url: &str,
    node: Node,
    database_connection: ConnectionPool,
    public_key: String,
) -> Result<
    (
        RestApiShutdownHandle,
        thread::JoinHandle<Result<(), RestApiServerError>>,
    ),
    RestApiServerError,
> {
    let bind_url = bind_url.to_owned();
    let splinterd_url = splinterd_url.to_owned();
    let gameroomd_data = GameroomdData { public_key };
    let (tx, rx) = mpsc::channel();
    let join_handle = thread::Builder::new()
        .name("GameroomdRestApi".into())
        .spawn(move || {
            let sys = actix::System::new("Gameroomd-Rest-API");

            let server = HttpServer::new(move || {
                App::new()
                    .data(database_connection.clone())
                    .data(Client::new())
                    .data(splinterd_url.to_owned())
                    .data(node.clone())
                    .data(gameroomd_data.clone())
                    .data(
                        // change path extractor configuration
                        web::Path::<String>::configure(|cfg| {
                            // <- create custom error response
                            cfg.error_handler(|err, _| handle_error(Box::new(err)))
                        }),
                    )
                    .data(
                        // change json extractor configuration
                        web::Json::<String>::configure(|cfg| {
                            // <- create custom error response
                            cfg.error_handler(|err, _| handle_error(Box::new(err)))
                        }),
                    )
                    .service(
                        web::resource("/nodes/{identity}").route(web::get().to(routes::fetch_node)),
                    )
                    .service(web::resource("/nodes").route(web::get().to(routes::list_nodes)))
                    .service(
                        web::resource("/gamerooms/propose")
                            .route(web::post().to(routes::propose_gameroom)),
                    )
                    .service(
                        web::scope("/users")
                            .service(web::resource("").route(web::post().to(routes::register)))
                            .service(
                                web::resource("/authenticate").route(web::post().to(routes::login)),
                            ),
                    )
                    .service(
                        web::scope("/proposals")
                            .service(
                                web::resource("/{proposal_id}/vote")
                                    .route(web::post().to(routes::proposal_vote)),
                            )
                            .service(
                                web::resource("/{proposal_id}")
                                    .route(web::get().to(routes::fetch_proposal)),
                            )
                            .service(
                                web::resource("").route(web::get().to(routes::list_proposals)),
                            ),
                    )
                    .service(
                        web::scope("/notifications")
                            .service(
                                web::scope("/{notification_id}")
                                    .service(
                                        web::resource("")
                                            .route(web::get().to(routes::fetch_notificaiton)),
                                    )
                                    .service(
                                        web::resource("/read")
                                            .route(web::patch().to(routes::read_notification)),
                                    ),
                            )
                            .service(
                                web::resource("")
                                    .route(web::get().to(routes::list_unread_notifications)),
                            ),
                    )
                    .service(
                        web::resource("/submit")
                            .route(web::post().to(routes::submit_signed_payload)),
                    )
                    .service(
                        web::scope("/gamerooms")
                            .service(web::resource("").route(web::get().to(routes::list_gamerooms)))
                            .service(
                                web::scope("/{circuit_id}")
                                    .service(
                                        web::resource("")
                                            .route(web::get().to(routes::fetch_gameroom)),
                                    )
                                    .service(
                                        web::resource("/batches")
                                            .route(web::post().to(routes::submit_scabbard_payload)),
                                    ),
                            ),
                    )
                    .service(
                        web::resource("/subscribe").route(web::get().to(routes::connect_socket)),
                    )
                    .service(
                        web::scope("/xo/{circuit_id}").service(
                            web::scope("/games")
                                .service(
                                    web::resource("/{game_id}")
                                        .route(web::get().to(routes::fetch_xo)),
                                )
                                .service(web::resource("").route(web::get().to(routes::list_xo))),
                        ),
                    )
                    .service(web::scope("/keys").service(
                        web::resource("/{public_key}").route(web::get().to(routes::fetch_key_info)),
                    ))
            })
            .bind(bind_url)?
            .disable_signals()
            .system_exit()
            .run();

            tx.send(server).map_err(|err| {
                RestApiServerError::StartUpError(format!("Unable to send Server Addr: {}", err))
            })?;
            sys.run()?;

            info!("Rest API terminating");

            Ok(())
        })?;

    let server = rx.recv().map_err(|err| {
        RestApiServerError::StartUpError(format!("Unable to receive Server Addr: {}", err))
    })?;

    Ok((RestApiShutdownHandle { server }, join_handle))
}

fn handle_error(err: Box<dyn ActixError::ResponseError>) -> ActixError::Error {
    let message = err.to_string();
    ActixError::InternalError::from_response(
        err,
        HttpResponse::BadRequest().json(ErrorResponse::bad_request(&message)),
    )
    .into()
}
