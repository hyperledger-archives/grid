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
use std::str::FromStr;

use clap::ArgMatches;
use libsplinter::protos::network::{NetworkEcho, NetworkMessage, NetworkMessageType};
use protobuf::Message;
use splinter_client::SplinterClient;

use crate::cert::get_certs;
use crate::error::CliError;

use super::Action;

pub struct EchoAction;

impl Action for EchoAction {
    fn run<'a>(&mut self, arg_matches: Option<&ArgMatches<'a>>) -> Result<(), CliError> {
        let matches = arg_matches.ok_or_else(|| CliError::RequiresArgs)?;
        let url = matches.value_of("url").unwrap_or("tcp://localhost:8045");

        let recipient = matches
            .value_of("recipient")
            .ok_or_else(|| CliError::MissingArg("recipient".into()))?;
        let ttl = i32::from_str(
            matches
                .value_of("ttl")
                .ok_or_else(|| CliError::MissingArg("ttl".into()))?,
        )
        .map_err(|_| CliError::InvalidArg("ttl must be a valid integer".into()))?;

        let msg = {
            let mut echo = NetworkEcho::new();
            echo.set_payload(b"HelloWorld".to_vec());
            echo.set_recipient(recipient.into());
            echo.set_time_to_live(ttl);
            let echo_bytes = echo.write_to_bytes().map_err(|err| {
                CliError::ActionError(format!("unable to write echo message to bytes: {}", err))
            })?;

            let mut network_msg = NetworkMessage::new();
            network_msg.set_message_type(NetworkMessageType::NETWORK_ECHO);
            network_msg.set_payload(echo_bytes);

            network_msg
        };

        let mut conn = SplinterClient::connect(url, get_certs())?;

        conn.send(&msg).map(|_| ()).map_err(CliError::from)
    }
}
