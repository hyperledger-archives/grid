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

//! Implementation for the running of the REST API, the `GriddleRestApi` handles the startup and
//! shutdown of Griddle's REST API.

use std::{future::Future, net::SocketAddr, pin::Pin, thread::JoinHandle};

use actix_web::dev::ServerHandle;

/// Contains information about the ports to which the REST API is bound.
#[derive(Debug)]
pub struct GriddleBindAddress {
    /// The SocketAddr which defines the bound port.
    pub addr: SocketAddr,

    /// The scheme (such as http) that is running on this port.
    pub scheme: PortScheme,
}

/// Indicates the scheme used by a port
#[derive(Debug)]
pub enum PortScheme {
    Http,
}

/// A running instance of the Griddle REST API.
pub struct GriddleRestApi {
    bind_addresses: Vec<GriddleBindAddress>,
    join_handle: Option<JoinHandle<()>>,
    server_handle: ServerHandle,
    shutdown_future: Option<Pin<Box<dyn Future<Output = ()>>>>,
}
