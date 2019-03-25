// Copyright 2018 Cargill Incorporated
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

use std::ops::Deref;
use rocket::request::{self, FromRequest};
use rocket::{Request, State, Outcome};
use sawtooth_sdk::messaging::zmq_stream::{ZmqMessageSender, ZmqMessageConnection};
use sawtooth_sdk::messaging::stream::MessageConnection;

pub struct ValidatorConn(pub ZmqMessageSender);

impl<'a, 'r> FromRequest<'a, 'r> for ValidatorConn {

    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<ValidatorConn, ()> {
        let connection = request.guard::<State<ZmqMessageConnection>>()?;
        let (sender, _) = connection.create();
        Outcome::Success(ValidatorConn(sender))
    }
}

impl Deref for ValidatorConn {
    type Target = ZmqMessageSender;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
