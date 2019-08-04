/*
 * Copyright 2019 Cargill Incorporated
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 * -----------------------------------------------------------------------------
 */

mod error;
pub use error::AppAuthHandlerError;

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;

use awc::ws::Codec;
use futures::{future, stream::Stream};
use hyper::{
    header,
    rt::{self, Future},
    Body, Client, Request, StatusCode,
};
use tokio::codec::Decoder;

pub struct AppAuthHandlerShutdownHandle {
    do_shutdown: Box<dyn Fn() -> Result<(), AppAuthHandlerError> + Send>,
}

impl AppAuthHandlerShutdownHandle {
    pub fn shutdown(&self) -> Result<(), AppAuthHandlerError> {
        (*self.do_shutdown)()
    }
}

pub fn run(
    splinterd_url: &str,
) -> Result<
    (
        AppAuthHandlerShutdownHandle,
        thread::JoinHandle<Result<(), AppAuthHandlerError>>,
    ),
    AppAuthHandlerError,
> {
    let splinterd_url = splinterd_url.to_owned();
    let client = Client::new();
    let shutdown_signaler = Arc::new(AtomicBool::new(true));
    let running = shutdown_signaler.clone();
    let join_handle = thread::Builder::new()
        .name("GameroomDAppAuthHandler".into())
        .spawn(move || {
            let req = Request::builder()
                .uri(format!("{}/ws/admin/register/gameroom", splinterd_url))
                .header(header::UPGRADE, "websocket")
                .header(header::CONNECTION, "Upgrade")
                .header(header::SEC_WEBSOCKET_VERSION, "13")
                .header(header::SEC_WEBSOCKET_KEY, "13")
                .body(Body::empty())
                .map_err(|err| AppAuthHandlerError::RequestError(format!("{}", err)))?;

            rt::run(
                client
                    .request(req)
                    .and_then(|res| {
                        if res.status() != StatusCode::SWITCHING_PROTOCOLS {
                            error!("The server didn't upgrade: {}", res.status());
                        }
                        res.into_body().on_upgrade()
                    })
                    .map_err(|e| error!("The client returned an error: {}", e))
                    .and_then(move |upgraded| {
                        let codec = Codec::new().client_mode();
                        let framed = codec.framed(upgraded);

                        // Read stream until shutdown signal is received
                        framed
                            .take_while(move |message| {
                                info!("Received Message: {:?}", message);
                                future::ok(running.load(Ordering::SeqCst))
                            })
                            // Transform stream into a future
                            .for_each(|_| future::ok(()))
                            .map_err(|e| error!("The client returned an error: {}", e))
                    }),
            );

            Ok(())
        })?;

    let do_shutdown = Box::new(move || {
        debug!("Shutting down application authentication handler");
        shutdown_signaler.store(false, Ordering::SeqCst);
        Ok(())
    });

    Ok((AppAuthHandlerShutdownHandle { do_shutdown }, join_handle))
}
