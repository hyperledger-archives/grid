extern crate rustls;
extern crate webpki;
#[macro_use]
extern crate protobuf;
extern crate bytes;

use rustls::{
    AllowAnyAuthenticatedClient, Certificate, ClientConfig, ClientSession, NoClientAuth,
    PrivateKey, ServerConfig, ServerSession, Session, Stream, SupportedCipherSuite,
};
use std::collections::HashMap;
use std::fs;
use std::io::{stdout, BufReader, ErrorKind, Read, Write};
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream, ToSocketAddrs};
use std::sync::{Arc, Mutex, mpsc};
use std::{time, thread};
use bytes::{BytesMut, Bytes, BufMut};

/// Shorthand for the transmit half of the message channel.
pub type Tx = mpsc::Sender<Bytes>;

/// Shorthand for the receive half of the message channel.
pub type Rx = mpsc::Receiver<Bytes>;

pub struct Shared {
    pub peers: HashMap<SocketAddr, Tx>,
    pub services: HashMap<SocketAddr, Tx>,
    pub channels: HashMap<String,  Channel>,
}

impl Shared {
    /// Create a new, empty, instance of `Shared`.
    pub fn new() -> Shared {
        Shared {
            peers: HashMap::new(),
            services: HashMap::new(),
            channels: HashMap::new(),
        }
    }
}

pub struct Channel {
    channel_id: String,
    peers: HashMap<String, Connection>,
}

impl Channel {
    fn new(channel_id: String, peers: HashMap<String, Connection>) -> Channel {
        Channel { channel_id, peers }
    }
}

// ClientConfig and ServerConfig should be made once and reused for each connection that is
// created.
#[derive(Debug)]
pub enum SessionType {
    client(rustls::ClientSession),
    server(rustls::ServerSession),
}

pub enum ConfigType {
    client(rustls::ClientConfig),
    server(rustls::ServerConfig),
}

pub enum ConnectionType {
    Network,
    Server,
}

pub enum ConnectionState {
    Running,
    Closing,
    Closed
}

/// This is a connection which has been accepted by the server,
/// and is currently being served.
///
/// It has a TCP-level stream, and some
/// other state/metadata.
pub struct Connection {
    connection_state: ConnectionState,
    state: Arc<Mutex<Shared>>,
    addr: SocketAddr,
    socket: TcpStream,
    session: SessionType,
    connection_type: Option<ConnectionType>,
}

impl Connection {
    pub fn new(
        socket: TcpStream,
        session: ConfigType,
        state: Arc<Mutex<Shared>>,
        dns_name: Option<String>,
    ) -> Connection {
        let session = match session {
            ConfigType::client(client_config) => {
                if let Some(name) = dns_name {
                    SessionType::client(create_client_session(client_config, name))
                } else {
                    panic!("No dns_name provided for client session")
                }
            }
            ConfigType::server(server_config) => {
                SessionType::server(create_server_session(server_config))
            }
        };

        // Create a channel for this peer
        // change to bounded channel
        // TODO change to bounded channel
        let (tx, rx) = mpsc::channel();

        let addr = socket.peer_addr().unwrap();
        // Add an entry for this `Peer` in the shared state map.
        state.lock().unwrap()
            .peers.insert(addr, tx);

        Connection {
            connection_state: ConnectionState::Running,
            state,
            addr,
            socket,
            session,
            connection_type: None,
        }
    }

    fn read(&mut self) {
        let mut b = [0; 10240];
        match &mut self.session {
            SessionType::server(session) => {
                session.complete_io(&mut self.socket).unwrap();
                let n = match session.read(&mut b) {
                    Ok(n) => n,
                    Err(ref e) if e.kind() == ErrorKind::WouldBlock => 0,
                    Err(err) => panic!("Whoops {}", err),
                };
                println!(
                    "{:?} {}",
                    String::from_utf8(b.to_vec()[..n].to_vec()).unwrap(),
                    n
                );
            }
            SessionType::client(session) => {
                session.complete_io(&mut self.socket).unwrap();
                let n = match session.read(&mut b) {
                    Ok(n) => n,
                    Err(ref e) if e.kind() == ErrorKind::WouldBlock => 0,
                    Err(err) => panic!("Whoops {}", err),
                };

                println!(
                    "{:?} {}",
                    String::from_utf8(b.to_vec()[..n].to_vec()).unwrap(),
                    n
                );
            }
        };
    }

    fn write(&mut self, buf: &[u8]) {
        match &mut self.session {
            SessionType::server(session) => {
                session.complete_io(&mut self.socket).unwrap();
                let n = session.write(buf).unwrap();
                println!("Wrote {}", n)
            }
            SessionType::client(session) => {
                session.complete_io(&mut self.socket).unwrap();
                let n = session.write(buf).unwrap();
                println!("Wrote {}", n)
            }
        };
    }

    pub fn handle_msg(&mut self) {
        loop {
            self.read();
            self.write(b"test");
        }
    }

    fn add_connection() -> () {
        unimplemented!();
    }

    fn remove_connection() -> () {
        unimplemented!();
    }

    fn add_channel() -> () {
        unimplemented!();
    }

    fn remove_channel() -> () {
        unimplemented!();
    }

}

impl Drop for Connection {
    fn drop(&mut self) {
        self.state.lock().unwrap().peers
            .remove(&self.addr);
    }
}

// Loads the private key associated with a cert for creating the tls config
pub fn load_key(file_path: &str) -> PrivateKey {
    let keyfile = fs::File::open(file_path).expect("cannot open private key file");
    let mut reader = BufReader::new(keyfile);
    let keys = rustls::internal::pemfile::pkcs8_private_keys(&mut reader).unwrap();
    assert!(keys.len() == 1);
    keys[0].clone()
}

// Loads the certifcate that should be connected to a tls config
pub fn load_cert(file_path: &str) -> Vec<Certificate> {
    let certfile = fs::File::open(file_path).expect("cannot open certificate file");
    let mut reader = BufReader::new(certfile);
    rustls::internal::pemfile::certs(&mut reader).unwrap()
}

// TODO allow multiple ca certs
// Creates a Client config for tls communication
pub fn create_client_config(
    ca_certs: Vec<Certificate>,
    client_certs: Vec<Certificate>,
    key: PrivateKey,
    cipher_suite: &'static SupportedCipherSuite,
) -> ClientConfig {
    let mut config = rustls::ClientConfig::new();
    for cert in ca_certs {
        config.root_store.add(&cert);
    }
    config.set_single_client_cert(client_certs, key);
    config.ciphersuites = rustls::ALL_CIPHERSUITES.to_vec();
    config
}

// Creates a Client Session from the ClientConfig and dns_name associated with the server to
// connect to
pub fn create_client_session(config: ClientConfig, dns_name: String) -> ClientSession {
    let dns_name = webpki::DNSNameRef::try_from_ascii_str(&dns_name).unwrap();
    ClientSession::new(&Arc::new(config), dns_name)
}

// Creates a Sever config for tls communication
pub fn create_server_config(
    ca_certs: Vec<Certificate>,
    server_certs: Vec<Certificate>,
    key: PrivateKey,
) -> ServerConfig {
    let mut client_auth_roots = rustls::RootCertStore::empty();
    for cert in ca_certs {
        client_auth_roots.add(&cert).unwrap();
    }

    let auth = AllowAnyAuthenticatedClient::new(client_auth_roots);

    let mut config = ServerConfig::new(auth);
    config.key_log = Arc::new(rustls::KeyLogFile::new());
    config.set_single_cert(server_certs, key);
    config
}

// Creates a Server Session from the ServerConfig
pub fn create_server_session(config: ServerConfig) -> ServerSession {
    ServerSession::new(&Arc::new(config))
}
