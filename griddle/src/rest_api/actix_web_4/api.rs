// Copyright 2018-2022 Cargill Incorporated
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

use std::{
    future::Future,
    io::Error as IoError,
    net::SocketAddr,
    pin::Pin,
    sync::{mpsc, Arc, Mutex},
    thread::{self, JoinHandle},
};

use actix_web::{dev::ServerHandle, middleware, rt::System, web, App, HttpServer};
use cylinder::Signer;
use futures_0_3::executor::block_on;

use grid_sdk::{error::InternalError, threading::lifecycle::ShutdownHandle};
#[cfg(feature = "proxy")]
use grid_sdk::{proxy::ProxyClient, rest_api::actix_web_4::routes::proxy_get};

use crate::internals::DLTBackend;
use crate::rest_api::{actix_web_4::GriddleResourceProvider, error::GriddleRestApiServerError};

/// Contains information about the ports to which the REST API is bound.
#[derive(Debug)]
pub struct GriddleBindAddress {
    /// The SocketAddr which defines the bound port.
    pub addr: SocketAddr,

    /// The scheme (such as http) that is running on this port.
    pub scheme: String,
}

enum FromThreadMessage {
    IoError(IoError, String),
    Running(ServerHandle, Vec<GriddleBindAddress>),
}

/// A running instance of the REST API.
pub struct GriddleRestApi {
    bind_addresses: Vec<GriddleBindAddress>,
    join_handle: Option<JoinHandle<()>>,
    server_handle: ServerHandle,
    shutdown_future: Option<Pin<Box<dyn Future<Output = ()>>>>,
}

impl GriddleRestApi {
    pub(super) fn new(
        bind_url: String,
        resource_providers: Vec<Box<dyn GriddleResourceProvider>>,
        #[cfg(feature = "proxy")] proxy_client: Box<dyn ProxyClient>,
        signer: Box<dyn Signer>,
        dlt_backend: DLTBackend,
    ) -> Result<Self, GriddleRestApiServerError> {
        let providers: Arc<Mutex<Vec<_>>> = Arc::new(Mutex::new(resource_providers));
        let (sender, receiver) = mpsc::channel();
        let join_handle =
            thread::Builder::new()
                .name("GriddleRestApi".into())
                .spawn(move || {
                    let sys = System::new();
                    let mut http_server = HttpServer::new(move || {
                        let app = App::new();

                        let mut app = app
                            .wrap(middleware::Logger::default())
                            .app_data(web::Data::new(signer.clone()))
                            .app_data(web::Data::new(dlt_backend.clone()));

                        for provider in providers.lock().unwrap().iter() {
                            for resource in provider.resources() {
                                app = app.service(resource)
                            }
                        }

                        #[cfg(feature = "proxy")]
                        {
                            app = app
                                .app_data(web::Data::new(proxy_client.cloned_box()))
                                .default_service(web::get().to(proxy_get));
                        }

                        app
                    });

                    http_server = match http_server.bind(bind_url.clone()) {
                        Ok(http_server) => http_server,
                        Err(err1) => {
                            let error_msg = format!("Bind to \"{}\" failed", bind_url);
                            if let Err(err2) =
                                sender.send(FromThreadMessage::IoError(err1, error_msg.clone()))
                            {
                                error!("{}", error_msg);
                                error!("Failed to notify receiver of bind error: {}", err2);
                            }
                            return;
                        }
                    };

                    let bind_addresses = http_server
                        .addrs_with_scheme()
                        .iter()
                        .map(|(addr, scheme)| GriddleBindAddress {
                            addr: *addr,
                            scheme: scheme.to_string(),
                        })
                        .collect();

                    let server = http_server.disable_signals().system_exit().run();
                    let handle = server.handle();

                    // Send the server and bind addresses to the parent thread
                    if let Err(err) =
                        sender.send(FromThreadMessage::Running(handle, bind_addresses))
                    {
                        error!("Unable to send running message to parent thread: {}", err);
                        return;
                    }

                    match sys.block_on(server) {
                        Ok(()) => info!("Griddle Rest API terminating"),
                        Err(err) => error!("Griddle REST API unexpectedly exiting: {}", err),
                    };
                })?;

        let (server_handle, bind_addresses) = loop {
            match receiver.recv() {
                Ok(FromThreadMessage::Running(server_handle, bind_address)) => {
                    break (server_handle, bind_address);
                }
                Ok(FromThreadMessage::IoError(err, error_msg)) => {
                    Err(GriddleRestApiServerError::StartUpError(format!(
                        "Failed to start Griddle Rest API: {}: {}",
                        error_msg, err
                    )))
                }
                Err(err) => Err(GriddleRestApiServerError::StartUpError(format!(
                    "Error receiving message from Griddle Rest Api thread: {}",
                    err
                ))),
            }?;
        };

        Ok(GriddleRestApi {
            bind_addresses,
            join_handle: Some(join_handle),
            server_handle,
            shutdown_future: None,
        })
    }

    /// Returns the list of addresses to which this REST API is bound.
    pub fn bind_addresses(&self) -> &Vec<GriddleBindAddress> {
        &self.bind_addresses
    }
}

impl ShutdownHandle for GriddleRestApi {
    fn signal_shutdown(&mut self) {
        self.shutdown_future = Some(Box::pin(self.server_handle.stop(true)));
    }

    fn wait_for_shutdown(mut self) -> Result<(), InternalError> {
        match (self.shutdown_future.take(), self.join_handle.take()) {
            (Some(f), Some(join_handle)) => {
                block_on(f);
                join_handle.join().map_err(|_| {
                    InternalError::with_message(
                        "GriddleRestApi thread panicked, join() failed".to_string(),
                    )
                })?;
                Ok(())
            }
            (_, _) => Err(InternalError::with_message(
                "Called wait_for_shutdown() prior to signal_shutdown()".to_string(),
            )),
        }
    }
}
