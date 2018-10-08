use std::fs::File;
use std::sync::Arc;
use std::mem;
use std::io;
use std::io::{Read, Write, BufReader, ErrorKind};
use webpki;
use std::path::PathBuf;
use rustls;
use rustls::{
    Session,    
    ClientSession,
    ClientConfig,
    TLSError
};
use std::net::{SocketAddr, ToSocketAddrs, TcpStream};
use messaging::protocol::{
    Message,
    MessageType
};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use protobuf;
use url;

pub struct Certs {
    ca_certs: Vec<PathBuf>,
    client_cert: PathBuf,
    client_priv: PathBuf 
}

impl Certs {
    pub fn new(
        ca_certs: Vec<PathBuf>,
        client_cert: PathBuf,
        client_priv: PathBuf
    ) -> Certs {
        Certs {
            ca_certs,
            client_cert,
            client_priv
        }
    }

    pub fn get_ca_certs(&self) -> Result<Vec<File>, SplinterError> {

        let mut files = Vec::new();

        for cert in self.ca_certs.clone() {
            files.push(File::open(if let Some(s) = cert.to_str() {
                s
            } else {
                return Err(SplinterError::CertUtf8Error("ca cert path name is malformed".into()))
            })?);
        }

        Ok(files)
    }

    pub fn get_client_cert(&self) -> Result<File, SplinterError> {
        let cert = if let Some(s) = self.client_cert.to_str() {
            s
        } else {
            return Err(SplinterError::CertUtf8Error("client cert path name is malformed".into()));
        };

        Ok(File::open(cert)?)
    }

    pub fn get_client_priv(&self) -> Result<File, SplinterError> {
        let cert = if let Some(s) = self.client_priv.to_str() {
            s
        } else {
            return Err(SplinterError::CertUtf8Error(
                    "client private key path name is malformed".into()));
        };

        Ok(File::open(cert)?)
    }
}

pub struct SplinterClient {
    session: ClientSession,
    socket: TcpStream
}

impl SplinterClient {

    pub fn connect(
        url: &str,
        certs: Certs
    ) -> Result<SplinterClient, SplinterError> {

        let (hostname, port) = {
            let url = url::Url::parse(url)?;
            let hs = if let Some(hs) = url.host_str() {
                hs.to_string()
            } else {
                return Err(SplinterError::HostNameNotFound);
            };

            let p = if let Some(p) = url.port() {
                p
            } else {
                return Err(SplinterError::HostNameNotFound);
            };

            (hs, p)
        };

        let addr = resolve_hostname(&format!("{}:{}", hostname, port))?;

        let socket = TcpStream::connect((addr.ip(), addr.port()))?;
        socket.set_nonblocking(true)?;

        let config = get_config(certs)?;
        let dns_name = webpki::DNSNameRef::try_from_ascii_str(&hostname)?;
        let session = rustls::ClientSession::new(&Arc::new(config), dns_name);

        Ok(SplinterClient { session, socket })
    }

    pub fn send(&mut self, req: &Message) -> Result<Message, SplinterError> {

        debug!("Performing Handshake");
        loop {
            match self.session.complete_io(&mut self.socket) {
                Ok(_) => break,
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => continue,
                Err(e) => return Err(SplinterError::from(e)),
            };
        }
        debug!("Handshake complete");

        let packed_req = pack_request(req)?;

        let mut send_heartbeat = false;

        info!("Sending message");
        loop {
            self.session.write_tls(&mut self.socket)?;

            if send_heartbeat {
                debug!("Sending heartbeat");

                let mut heartbeat = Message::new();
                heartbeat.set_message_type(MessageType::HEARTBEAT_REQUEST);

                self.session.write(&pack_request(&heartbeat)?)?;
            } else {
                debug!("Writing message");
                self.session.write(&packed_req)?;
                send_heartbeat = true;
            }
            
            debug!("Reading tls");
            match self.session.read_tls(&mut self.socket) {
                Ok(n) => {
                    debug!("TLS Read complete: {}", n);
                }
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                    debug!("No data to read");
                    continue;
                }
                Err(err) => return Err(SplinterError::from(err))
            };

            debug!("Processing new packets");

            self.session.process_new_packets()?;

            // Read first 4 bytes to get length
            let mut msg_len_buff = vec![0; mem::size_of::<u32>()];
            self.session.read_exact(&mut msg_len_buff)?;
            let msg_size = msg_len_buff
                .as_slice()
                .read_u32::<BigEndian>()? as usize;

            // Read Message
            let mut msg_buff = vec![0; msg_size];
            self.session.read_exact(&mut msg_buff)?;

            let response = protobuf::parse_from_bytes::<Message>(&msg_buff)?;

            if response.message_type != MessageType::HEARTBEAT_RESPONSE {
                info!("Response received {:?}", response);
                return Ok(response);
            }
        }
    }
}

fn get_config(certs: Certs) -> Result<ClientConfig, SplinterError> {
        let mut config = ClientConfig::new();

        for file in certs.get_ca_certs()? {
            let mut reader = BufReader::new(file);
            config.root_store.add_pem_file(&mut reader)?;
        }

        let client_cert_file = certs.get_client_cert()?;
        let mut reader = BufReader::new(client_cert_file);
        let client_certs = rustls::internal::pemfile::certs(&mut reader)?;

        let client_priv_file = certs.get_client_priv()?;
        let mut reader = BufReader::new(client_priv_file);
        let keys = rustls::internal::pemfile::pkcs8_private_keys(&mut reader)?;

        let privkey = if keys.len() < 1 {
            return Err(SplinterError::PrivateKeyNotFound);
        } else {
            keys[0].clone()
        };

        config.set_single_client_cert(client_certs, privkey);

        config.ciphersuites.push(rustls::ALL_CIPHERSUITES.to_vec()[0]);

        Ok(config)
    }

fn pack_request(req: &Message) -> Result<Vec<u8>, SplinterError> {
    let raw_msg = protobuf::Message::write_to_bytes(req)?;
    let mut buff = Vec::new();

    buff.write_u32::<BigEndian>(raw_msg.len() as u32)?;
    buff.write(&raw_msg)?;

    Ok(buff)
}

fn resolve_hostname(hostname: &str) -> Result<SocketAddr, SplinterError> {
    hostname.to_socket_addrs()?
        .filter(|addr| addr.is_ipv4())
        .next()
        .ok_or(SplinterError::CouldNotResolveHostName)
}

#[derive(Debug)]
pub enum SplinterError {
    DnsError(String),
    IoError(io::Error),
    ProtobufError(protobuf::ProtobufError),
    CertUtf8Error(String),
    UrlParseError(url::ParseError),
    TlsError(TLSError),
    CouldNotResolveHostName,
    PrivateKeyNotFound,
    HostNameNotFound
}

impl From<io::Error> for SplinterError {
    fn from(e: io::Error) -> Self {
        SplinterError::IoError(e)
    }
}

impl From<protobuf::ProtobufError> for SplinterError {
    fn from(e: protobuf::ProtobufError) -> Self {
        SplinterError::ProtobufError(e)
    }
}

impl From<url::ParseError> for SplinterError {
    fn from(e: url::ParseError) -> Self {
        SplinterError::UrlParseError(e)
    }
}

impl From<TLSError> for SplinterError {
    fn from(e: TLSError) -> Self {
        SplinterError::TlsError(e)
    }
}

impl From<()> for SplinterError {
    fn from(_: ()) -> Self {
        SplinterError::DnsError("DNS Error: Invalid name".into())
    }
}

#[cfg(test)]
mod tests {
    use messaging::protocol::{Message, MessageType}; 
    use protobuf;
    use std::mem;
    use splinter_client::pack_request;
    use std::io::Read;
    use byteorder::{BigEndian, ReadBytesExt};

    #[test]
    fn test_pack_request() {
        let mut request = Message::new();
        request.set_message_type(MessageType::HEARTBEAT_REQUEST);

        let expected_size = protobuf::Message::write_to_bytes(&request)
            .unwrap()
            .len();

        let packed_request = pack_request(&request).unwrap();

        assert_eq!(expected_size + mem::size_of::<u32>(), packed_request.len());

        let mut msg_len_buff = vec![0; mem::size_of::<u32>()];
        packed_request
            .as_slice()
            .read_exact(&mut msg_len_buff)
            .unwrap();

        let actual_size = msg_len_buff
            .as_slice()
            .read_u32::<BigEndian>()
            .unwrap() as usize;

        assert_eq!(expected_size, actual_size);

        let actual_request = protobuf::parse_from_bytes::<Message>(&packed_request[mem::size_of::<u32>()..])
            .unwrap();

        assert!(request.get_message_type() == actual_request.get_message_type());
    }
}
