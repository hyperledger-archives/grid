use rustls;
use std::net::{TcpListener, TcpStream};
use url::Url;

use libsplinter::{
    create_client_config, create_client_session, create_server_config, create_server_session,
    load_cert, load_key, Connection, ConnectionType, Shared, SplinterError,
};

use std::net::{Ipv4Addr, Ipv6Addr};
use std::sync::{Arc, Mutex};
use std::thread;

pub struct SplinterDaemon {
    client_config: rustls::ClientConfig,
    server_config: rustls::ServerConfig,
    state: Arc<Mutex<Shared>>,
    service_endpoint: Url,
    network_endpoint: Url,
    initial_peers: Vec<Url>,
}

impl SplinterDaemon {
    pub fn new(
        ca_files: Vec<String>,
        client_cert: &str,
        server_cert: &str,
        server_key_file: &str,
        client_key_file: &str,
        network_endpoint: Url,
        service_endpoint: Url,
        initial_peers: Vec<Url>,
    ) -> Result<SplinterDaemon, SplinterError> {
        let mut ca_certs = Vec::new();
        for ca_file in ca_files {
            let ca_cert = load_cert(&ca_file)?;
            ca_certs.extend(ca_cert);
        }
        let server_key = load_key(server_key_file)?;
        let client_key = load_key(client_key_file)?;

        let client_certs = load_cert(client_cert)?;

        // This should be updated to not just be all the suites
        let cipher_suites = rustls::ALL_CIPHERSUITES.to_vec();

        let client_config =
            create_client_config(ca_certs.clone(), client_certs, client_key, cipher_suites)?;
        // create server config
        let server_certs = load_cert(server_cert)?;
        let server_config = create_server_config(ca_certs, server_certs, server_key)?;

        // create splinterD node
        let state = Arc::new(Mutex::new(Shared::new()));

        Ok(SplinterDaemon {
            client_config,
            server_config,
            state,
            service_endpoint,
            network_endpoint,
            initial_peers,
        })
    }

    pub fn start(&mut self) -> Result<(), SplinterError> {
        // create peers and pass to threads
        for peer in self.initial_peers.iter() {
            let mut socket = connect(peer)?;
            socket.set_nonblocking(true)?;

            let session = match peer.domain() {
                Some(d) if d.parse::<Ipv4Addr>().is_ok() => {
                    create_client_session(self.client_config.clone(), "localhost".into())?
                }
                Some(d) if d.parse::<Ipv6Addr>().is_ok() => {
                    create_client_session(self.client_config.clone(), "localhost".into())?
                }
                Some(d) => create_client_session(self.client_config.clone(), d.to_string())?,
                None => create_client_session(self.client_config.clone(), "localhost".into())?,
            };

            let mut connection =
                Connection::new(socket, session, self.state.clone(), ConnectionType::Network)?;
            let _ = thread::spawn(move || connection.handle_msg());
        }

        let network_endpoint = self.network_endpoint.clone();
        let network_server_config = self.server_config.clone();
        let network_state = self.state.clone();
        thread::spawn(move || {
            // start up a listener and accept incoming connections
            let listener = create_listener(&network_endpoint)?;
            for socket in listener.incoming() {
                match socket {
                    Ok(mut socket) => {
                        socket.set_nonblocking(true)?;

                        // update to use correct dns_name
                        let mut connection = Connection::new(
                            socket,
                            create_server_session(network_server_config.clone()),
                            network_state.clone(),
                            ConnectionType::Network,
                        )?;
                        let _ = thread::spawn(move || connection.handle_msg());
                    }
                    Err(e) => return Err(SplinterError::from(e)),
                }
            }

            Ok(())
        });

        // start up a listener and accept incoming connections
        let listener = create_listener(&self.service_endpoint)?;

        for socket in listener.incoming() {
            match socket {
                Ok(mut socket) => {
                    socket.set_nonblocking(true)?;

                    // update to use correct dns_name
                    let mut connection = Connection::new(
                        socket,
                        create_server_session(self.server_config.clone()),
                        self.state.clone(),
                        ConnectionType::Service,
                    )?;
                    let _ = thread::spawn(move || connection.handle_msg());
                }
                Err(e) => return Err(SplinterError::from(e)),
            }
        }

        Ok(())
    }
}

fn create_listener(url: &Url) -> Result<TcpListener, SplinterError> {
    let host = if let Some(h) = url.host_str() {
        h
    } else {
        return Err(SplinterError::HostNameNotFound);
    };

    let port = if let Some(p) = url.port() {
        p
    } else {
        return Err(SplinterError::PortNotIdentified);
    };

    TcpListener::bind(&format!("{}:{}", host, port)).map_err(SplinterError::from)
}

fn connect(url: &Url) -> Result<TcpStream, SplinterError> {
    let host = if let Some(h) = url.host_str() {
        h
    } else {
        return Err(SplinterError::HostNameNotFound);
    };

    let port = if let Some(p) = url.port() {
        p
    } else {
        return Err(SplinterError::PortNotIdentified);
    };

    println!("{}:{}", host, port);

    TcpStream::connect(&format!("{}:{}", host, port)).map_err(SplinterError::from)
}
