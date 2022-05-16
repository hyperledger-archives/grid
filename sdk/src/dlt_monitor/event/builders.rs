// Copyright 2022 Cargill Incorporated
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

use crate::dlt_monitor::event::error::DltEventMonitorError;
use crate::dlt_monitor::event::socket::{Client, Handler};

pub struct ScabbardEventClientBuilder {
    port: Option<String>,
    host: Option<String>,
    circuit_id: Option<String>,
    service_id: Option<String>,
    last_event_id: Option<String>,
    authorization_token: Option<String>,
    is_secure: bool,
}

impl ScabbardEventClientBuilder {
    pub fn new() -> ScabbardEventClientBuilder {
        ScabbardEventClientBuilder {
            port: None,
            host: None,
            circuit_id: None,
            service_id: None,
            last_event_id: None,
            authorization_token: None,
            is_secure: true,
        }
    }

    pub fn with_port(mut self, port: String) -> ScabbardEventClientBuilder {
        self.port = Some(port);
        self
    }

    pub fn with_host(mut self, host: String) -> ScabbardEventClientBuilder {
        self.host = Some(host);
        self
    }

    pub fn with_circuit_id(mut self, circuit_id: String) -> ScabbardEventClientBuilder {
        self.circuit_id = Some(circuit_id);
        self
    }

    pub fn with_service_id(mut self, service_id: String) -> ScabbardEventClientBuilder {
        self.service_id = Some(service_id);
        self
    }

    pub fn with_last_event_id(
        mut self,
        last_event_id: Option<String>,
    ) -> ScabbardEventClientBuilder {
        self.last_event_id = last_event_id;
        self
    }

    pub fn with_authorization_token(
        mut self,
        authorization_token: String,
    ) -> ScabbardEventClientBuilder {
        self.authorization_token = Some(authorization_token);
        self
    }

    pub fn with_is_secure(mut self, is_secure: bool) -> ScabbardEventClientBuilder {
        self.is_secure = is_secure;
        self
    }

    fn url(&self) -> Result<String, DltEventMonitorError> {
        let host = self
            .host
            .as_ref()
            .ok_or_else(|| DltEventMonitorError::MissingArgument("host".to_string()))?;

        let port = self
            .port
            .as_ref()
            .ok_or_else(|| DltEventMonitorError::MissingArgument("port".to_string()))?;

        let circuit_id = self
            .circuit_id
            .as_ref()
            .ok_or_else(|| DltEventMonitorError::MissingArgument("circuit_id".to_string()))?;

        let service_id = self
            .service_id
            .as_ref()
            .ok_or_else(|| DltEventMonitorError::MissingArgument("service_id".to_string()))?;

        let protocol = if self.is_secure { "wss" } else { "ws" };

        let params = if let Some(last_event_id) = &self.last_event_id {
            format!("?last_event_id={last_event_id}")
        } else {
            "".to_string()
        };

        Ok(format!(
            "{protocol}://{host}:{port}/scabbard/{circuit_id}/{service_id}/ws/subscribe{params}"
        ))
    }

    pub fn build<T: Handler>(&self, handler: T) -> Result<Client<T>, DltEventMonitorError> {
        Client::new(
            self.is_secure,
            self.url()?,
            handler,
            self.authorization_token.clone(),
        )
    }
}

impl Default for ScabbardEventClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct SawtoothEventClientBuilder {
    port: Option<String>,
    host: Option<String>,
    is_secure: bool,
}

impl SawtoothEventClientBuilder {
    pub fn new() -> SawtoothEventClientBuilder {
        SawtoothEventClientBuilder {
            port: None,
            host: None,
            is_secure: true,
        }
    }

    pub fn with_port(mut self, port: String) -> SawtoothEventClientBuilder {
        self.port = Some(port);
        self
    }

    pub fn with_host(mut self, host: String) -> SawtoothEventClientBuilder {
        self.host = Some(host);
        self
    }

    pub fn with_is_secure(mut self, is_secure: bool) -> SawtoothEventClientBuilder {
        self.is_secure = is_secure;
        self
    }

    fn url(&self) -> Result<String, DltEventMonitorError> {
        let host = self
            .host
            .as_ref()
            .ok_or_else(|| DltEventMonitorError::MissingArgument("host".to_string()))?;

        let port = self
            .port
            .as_ref()
            .ok_or_else(|| DltEventMonitorError::MissingArgument("port".to_string()))?;

        let protocol = if self.is_secure { "wss" } else { "ws" };

        Ok(format!("{protocol}://{host}:{port}/ws/subscribe"))
    }

    pub fn build<T: Handler>(&self, handler: T) -> Result<Client<T>, DltEventMonitorError> {
        Client::new(self.is_secure, self.url()?, handler, None)
    }
}

impl Default for SawtoothEventClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PORT: &str = "8080";
    const HOST: &str = "127.0.0.1";
    const CIRCUIT_ID: &str = "CIRCUIT";
    const SERVICE_ID: &str = "SERVICE";
    const LAST_EVENT_ID: &str = "123";

    #[derive(Debug)]
    struct StubHandler {}

    /// A handler that does nothing with the handle callback information
    impl StubHandler {
        fn new() -> Self {
            StubHandler {}
        }
    }

    impl Handler for StubHandler {
        fn handle(&mut self, _: Vec<u8>) {}
    }

    #[test]
    fn test_sawtooth_url_insecure() {
        let builder = SawtoothEventClientBuilder::new()
            .with_host(HOST.to_string())
            .with_port(PORT.to_string())
            .with_is_secure(false);

        assert_eq!(
            builder.url().expect("unable to construct url"),
            "ws://127.0.0.1:8080/ws/subscribe"
        );
    }

    #[test]
    fn test_sawtooth_url_secure() {
        let builder = SawtoothEventClientBuilder::new()
            .with_port(PORT.to_string())
            .with_host(HOST.to_string())
            .with_is_secure(true);

        assert_eq!(
            builder.url().expect("unable to construct url"),
            "wss://127.0.0.1:8080/ws/subscribe"
        );
    }

    #[test]
    fn test_scabbard_url_insecure_with_event_id() {
        let builder = ScabbardEventClientBuilder::new()
            .with_host(HOST.to_string())
            .with_port(PORT.to_string())
            .with_circuit_id(CIRCUIT_ID.to_string())
            .with_service_id(SERVICE_ID.to_string())
            .with_last_event_id(Some(LAST_EVENT_ID.to_string()))
            .with_is_secure(false);

        assert_eq!(
            builder.url().expect("unable to construct url"),
            "ws://127.0.0.1:8080/scabbard/CIRCUIT/SERVICE/ws/subscribe?last_event_id=123"
        );
    }

    #[test]
    fn test_scabbard_url_secure_without_event_id() {
        let builder = ScabbardEventClientBuilder::new()
            .with_host(HOST.to_string())
            .with_port(PORT.to_string())
            .with_circuit_id(CIRCUIT_ID.to_string())
            .with_service_id(SERVICE_ID.to_string())
            .with_is_secure(true);

        assert_eq!(
            builder.url().expect("unable to construct url"),
            "wss://127.0.0.1:8080/scabbard/CIRCUIT/SERVICE/ws/subscribe"
        );
    }

    #[test]
    fn test_sawtooth_build_fails_without_host() {
        let result = SawtoothEventClientBuilder::new()
            .with_port(PORT.to_string())
            .with_is_secure(false)
            .build(StubHandler::new());

        assert_eq!(format!("{:?}", result), "Err(MissingArgument(\"host\"))");
    }

    #[test]
    fn test_sawtooth_build_fails_without_port() {
        let result = SawtoothEventClientBuilder::new()
            .with_host(HOST.to_string())
            .with_is_secure(false)
            .build(StubHandler::new());

        assert_eq!(format!("{:?}", result), "Err(MissingArgument(\"port\"))");
    }

    #[test]
    fn test_scabbard_build_fails_without_host() {
        let result = ScabbardEventClientBuilder::new()
            .with_port(PORT.to_string())
            .with_circuit_id(CIRCUIT_ID.to_string())
            .with_service_id(SERVICE_ID.to_string())
            .with_last_event_id(Some(LAST_EVENT_ID.to_string()))
            .with_is_secure(false)
            .build(StubHandler::new());

        assert_eq!(format!("{:?}", result), "Err(MissingArgument(\"host\"))");
    }

    #[test]
    fn test_scabbard_build_fails_without_port() {
        let result = ScabbardEventClientBuilder::new()
            .with_host(HOST.to_string())
            .with_circuit_id(CIRCUIT_ID.to_string())
            .with_service_id(SERVICE_ID.to_string())
            .with_last_event_id(Some(LAST_EVENT_ID.to_string()))
            .with_is_secure(false)
            .build(StubHandler::new());

        assert_eq!(format!("{:?}", result), "Err(MissingArgument(\"port\"))");
    }

    #[test]
    fn test_scabbard_build_fails_without_circuit_id() {
        let result = ScabbardEventClientBuilder::new()
            .with_host(HOST.to_string())
            .with_port(PORT.to_string())
            .with_service_id(SERVICE_ID.to_string())
            .with_last_event_id(Some(LAST_EVENT_ID.to_string()))
            .with_is_secure(false)
            .build(StubHandler::new());

        assert_eq!(
            format!("{:?}", result),
            "Err(MissingArgument(\"circuit_id\"))"
        );
    }

    #[test]
    fn test_scabbard_build_fails_without_service_id() {
        let result = ScabbardEventClientBuilder::new()
            .with_host(HOST.to_string())
            .with_port(PORT.to_string())
            .with_circuit_id(CIRCUIT_ID.to_string())
            .with_last_event_id(Some(LAST_EVENT_ID.to_string()))
            .with_is_secure(false)
            .build(StubHandler::new());

        assert_eq!(
            format!("{:?}", result),
            "Err(MissingArgument(\"service_id\"))"
        );
    }
}
