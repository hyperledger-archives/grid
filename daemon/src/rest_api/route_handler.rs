// Copyright 2019 Bitwise IO, Inc.
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

use crate::rest_api::{error::RestApiResponseError, AppState};

use actix::{Actor, Context, Handler, Message};
use actix_web::{HttpMessage, HttpRequest, HttpResponse, State};
use futures::future;
use futures::future::Future;
use protobuf;
use sawtooth_sdk::messaging::stream::MessageSender;

pub struct SawtoothMessageSender {
    sender: Box<dyn MessageSender>,
}

impl Actor for SawtoothMessageSender {
    type Context = Context<Self>;
}

impl SawtoothMessageSender {
    pub fn new(sender: Box<dyn MessageSender>) -> SawtoothMessageSender {
        SawtoothMessageSender { sender }
    }
}
pub fn submit_batches(
    (_req, _state): (HttpRequest<AppState>, State<AppState>),
) -> Box<Future<Item = HttpResponse, Error = RestApiResponseError>> {
    unimplemented!()
}

pub fn get_batch_statuses(
    (_req, _state): (HttpRequest<AppState>, State<AppState>),
) -> Box<Future<Item = HttpResponse, Error = RestApiResponseError>> {
    unimplemented!()
}
