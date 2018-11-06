use rustls;
use std::net::{SocketAddr, TcpListener, TcpStream};
use url::Url;

use libsplinter::{
    create_client_config, create_client_session, create_server_config, create_server_session,
    load_cert, load_key, ConnectionDriver, ConnectionType, DaemonRequest, Shared, SplinterError,
};

use libsplinter::connection::tls_connection::TlsConnection;

use std::net::{Ipv4Addr, Ipv6Addr};
use std::sync::{mpsc, Arc, Mutex};
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
        let (tx, rx) = mpsc::channel();

        // create peers and pass to threads
        for peer in self.initial_peers.iter() {
            let mut connection = create_peer_connection(
                peer.clone(),
                to_socket_addr(&self.network_endpoint)?,
                tx.clone(),
                self.client_config.clone(),
                self.state.clone(),
            )?;
            let _ = thread::spawn(move || connection.run());
        }

        let network_endpoint = self.network_endpoint.clone();
        let network_addr = to_socket_addr(&self.network_endpoint)?;
        let network_server_config = self.server_config.clone();
        let network_state = self.state.clone();
        let network_sender = tx.clone();

        thread::spawn(move || {
            // start up a listener and accept incoming connections
            let listener = create_listener(&network_endpoint)?;
            for socket in listener.incoming() {
                match socket {
                    Ok(mut socket) => {
                        socket.set_nonblocking(true)?;

                        let session = create_server_session(network_server_config.clone());
                        let peer_addr = socket.peer_addr()?.clone();
                        let connection = TlsConnection::new(socket, session);

                        let mut connection_driver = ConnectionDriver::new(
                            connection,
                            network_addr,
                            peer_addr,
                            network_state.clone(),
                            ConnectionType::Network,
                            network_sender.clone(),
                        )?;
                        let _ = thread::spawn(move || connection_driver.run());
                    }
                    Err(e) => return Err(SplinterError::from(e)),
                }
            }

            Ok(())
        });

        let service_endpoint = self.service_endpoint.clone();
        let service_server_config = self.server_config.clone();
        let service_state = self.state.clone();
        let service_sender = tx.clone();
        thread::spawn(move || {
            // start up a listener and accept incoming connections
            let listener = create_listener(&service_endpoint)?;

            for socket in listener.incoming() {
                match socket {
                    Ok(mut socket) => {
                        socket.set_nonblocking(true)?;

                        let session = create_server_session(service_server_config.clone());
                        let peer_addr = socket.peer_addr()?.clone();
                        let connection = TlsConnection::new(socket, session);

                        let mut connection_driver = ConnectionDriver::new(
                            connection,
                            network_addr,
                            peer_addr,
                            service_state.clone(),
                            ConnectionType::Service,
                            service_sender.clone(),
                        )?;
                        let _ = thread::spawn(move || connection_driver.run());
                    }
                    Err(e) => return Err(SplinterError::from(e)),
                }
            }
            Ok(())
        });

        // Wait for thread requests to create
        //
        let new_connection_network_addr = to_socket_addr(&self.network_endpoint)?;
        loop {
            let request = rx.recv().unwrap();

            match request {
                DaemonRequest::CreateConnection { address } => {
                    let mut connection = match create_peer_connection(
                        Url::parse(&address)?,
                        new_connection_network_addr,
                        tx.clone(),
                        self.client_config.clone(),
                        self.state.clone(),
                    ) {
                        Ok(connection) => connection,
                        Err(err) => {
                            warn!("Unable to connect to {}: {:?}", address, err);
                            continue;
                        }
                    };

                    thread::spawn(move || connection.run());
                }
            }
        }
    }
}

fn create_peer_connection(
    peer: Url,
    network_addr: SocketAddr,
    sender: mpsc::Sender<DaemonRequest>,
    client_config: rustls::ClientConfig,
    state: Arc<Mutex<Shared>>,
) -> Result<ConnectionDriver<TlsConnection<rustls::ClientSession>>, SplinterError> {
    let socket = connect(&peer)?;
    socket.set_nonblocking(true)?;

    let session = match peer.domain() {
        Some(d) if d.parse::<Ipv4Addr>().is_ok() => {
            create_client_session(client_config.clone(), "localhost".into())?
        }
        Some(d) if d.parse::<Ipv6Addr>().is_ok() => {
            create_client_session(client_config.clone(), "localhost".into())?
        }
        Some(d) => create_client_session(client_config.clone(), d.to_string())?,
        None => create_client_session(client_config.clone(), "localhost".into())?,
    };

    let peer_addr = socket.peer_addr()?.clone();
    let connection = TlsConnection::new(socket, session);

    ConnectionDriver::new(
        connection,
        network_addr,
        peer_addr,
        state.clone(),
        ConnectionType::Network,
        sender,
    )
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

    debug!("{}:{}", host, port);

    TcpStream::connect(&format!("{}:{}", host, port)).map_err(SplinterError::from)
}

fn to_socket_addr(url: &Url) -> Result<SocketAddr, SplinterError> {
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
    Ok(format!("{}:{}", host, port).parse()?)
}
