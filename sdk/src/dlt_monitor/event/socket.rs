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

use std::io::Read;
use std::io::Write;
use std::thread;

use websocket::client::sync::Client as WsClient;
use websocket::header::Headers;
use websocket::url::Url;
use websocket::ws::dataframe::DataFrame;
use websocket::ClientBuilder;

use crate::dlt_monitor::event::error::DltEventMonitorError;

pub trait Handler: Send {
    fn handle(&mut self, message: Vec<u8>);
}

#[derive(Debug)]
pub struct Client<T: 'static + Handler> {
    secure: bool,
    url: Url,
    handler: T,
    token: Option<String>,
}

impl<T: Handler> Client<T> {
    pub fn new(
        secure: bool,
        url: String,
        handler: T,
        token: Option<String>,
    ) -> Result<Client<T>, DltEventMonitorError> {
        Ok(Client {
            secure,
            url: Url::parse(&url)
                .map_err(|err| DltEventMonitorError::InternalError(Box::new(err)))?,
            handler,
            token,
        })
    }

    pub fn start(self) -> Result<(), DltEventMonitorError> {
        let mut ws = ClientBuilder::from_url(&self.url);

        if let Some(token) = &self.token {
            let mut headers = Headers::new();
            headers.append_raw(
                "Authorization",
                format!("Bearer Cylinder: {token}").as_bytes().to_vec(),
            );
            ws = ws.custom_headers(&headers);
        }

        if self.secure {
            let client = ws
                .connect_secure(None)
                .map_err(|err| DltEventMonitorError::InternalError(Box::new(err)))?;
            self.spawn(client);
        } else {
            let client = ws
                .connect_insecure()
                .map_err(|err| DltEventMonitorError::InternalError(Box::new(err)))?;
            self.spawn(client);
        }

        Ok(())
    }

    fn spawn<Y: 'static + Write + Read + Send>(mut self, mut client: WsClient<Y>) {
        thread::spawn(move || {
            while let Ok(message) = client.recv_message() {
                self.handler.handle(message.take_payload());
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::net::TcpListener;
    use std::sync::mpsc;
    use std::sync::mpsc::Sender;
    use std::thread;

    use websocket::sync::Server;
    use websocket::Message;

    /// Find a port that is not currently in use
    fn get_available_port(host: &str) -> Option<u16> {
        (8000..9000).find(|port| TcpListener::bind((host, *port)).is_ok())
    }

    struct ServerInfo {
        host: String,
        port: u16,
    }

    /// Create a faux DLT server
    fn create_server(messages: Vec<String>) -> ServerInfo {
        let host = "127.0.0.1";
        let port = get_available_port(host).expect("could not find available port");

        let bind = format!("{}:{}", host, port);
        let server_info = ServerInfo {
            host: host.to_string(),
            port,
        };

        thread::spawn(move || {
            let server = Server::bind(bind).unwrap();

            for connection in server.filter_map(Result::ok) {
                let mut client = connection.accept().unwrap();

                for message in messages.iter() {
                    let message = Message::text(message);
                    let _ = client.send_message(&message);
                }
            }
        });

        server_info
    }

    struct TestHandler {
        tx: Sender<Vec<u8>>,
    }

    impl TestHandler {
        fn new(tx: Sender<Vec<u8>>) -> Self {
            TestHandler { tx }
        }
    }

    impl Handler for TestHandler {
        fn handle(&mut self, data: Vec<u8>) {
            self.tx.send(data).expect("error sending message");
        }
    }

    #[test]
    fn test_create_client_success() {
        let (tx, rx) = mpsc::channel::<Vec<u8>>();
        let address = create_server(vec!["test message".to_string()]);
        let is_secure = false;
        let handler = TestHandler::new(tx);

        Client::new(
            is_secure,
            format!(
                "ws://{host}:{port}",
                host = address.host,
                port = address.port
            ),
            handler,
            None,
        )
        .expect("failed to start client")
        .start()
        .expect("failed to start client");

        assert_eq!(
            String::from_utf8_lossy(&rx.recv().expect("error with channel message")),
            "test message"
        );
    }
}
