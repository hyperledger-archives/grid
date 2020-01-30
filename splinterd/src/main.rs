// Copyright 2018-2020 Cargill Incorporated
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

#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate clap;

#[cfg(feature = "generate-certs")]
mod certs;
mod config;
mod daemon;
mod registry_config;
mod routes;

use flexi_logger::{style, DeferredNow, LogSpecBuilder, Logger};
use log::Record;

#[cfg(feature = "generate-certs")]
use crate::certs::{make_ca_cert, make_ca_signed_cert, write_file, CertError};
use crate::config::{Config, ConfigError};
#[cfg(feature = "config-toml")]
use crate::config::{ConfigBuilder, TomlConfig};
use crate::daemon::{SplinterDaemonBuilder, StartError};
use clap::{clap_app, crate_version};
use clap::{Arg, ArgMatches};
#[cfg(feature = "generate-certs")]
use openssl::error::ErrorStack;
use splinter::transport::raw::RawTransport;
use splinter::transport::tls::{TlsInitError, TlsTransport};
use splinter::transport::Transport;
#[cfg(feature = "generate-certs")]
use tempdir::TempDir;

use std::env;
use std::error::Error;
use std::fmt;
use std::fs;
#[cfg(not(feature = "config-toml"))]
use std::fs::File;
use std::io;
use std::path::Path;
use std::thread;
use std::time::Duration;

const DEFAULT_STATE_DIR: &str = "/var/lib/splinter/";
const STATE_DIR_ENV: &str = "SPLINTER_STATE_DIR";

const DEFAULT_CERT_DIR: &str = "/etc/splinter/certs/";
const CERT_DIR_ENV: &str = "SPLINTER_CERT_DIR";

const CLIENT_CERT: &str = "client.crt";
const CLIENT_KEY: &str = "private/client.key";
const SERVER_CERT: &str = "server.crt";
const SERVER_KEY: &str = "private/server.key";
const CA_PEM: &str = "ca.pem";

const HEARTBEAT_DEFAULT: u64 = 30;

const DEFAULT_ADMIN_SERVICE_COORDINATOR_TIMEOUT_MILLIS: u64 = 30000;

#[cfg(not(feature = "config-toml"))]
fn load_toml_config(config_file_path: &str) -> Config {
    match File::open(config_file_path) {
        Ok(f) => Config::from_file(f),
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            debug!("Configuration file not found: {}", config_file_path);
            Ok(Config::default())
        }
        Err(err) => Err(ConfigError::from(err)),
    }
    .unwrap_or_else(|err| {
        warn!(
            "Unable to load configuration file {}: {}",
            config_file_path, err
        );
        Config::default()
    })
}

#[cfg(feature = "config-toml")]
fn load_toml_config(config_file_path: &str) -> Config {
    let mut config_builder = ConfigBuilder::new();

    match fs::read_to_string(config_file_path) {
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            debug!("Configuration file not found: {}", config_file_path)
        }
        result => match result.map_err(ConfigError::from).and_then(TomlConfig::new) {
            Ok(toml_config) => {
                config_builder = toml_config.apply_to_builder(config_builder);
            }
            Err(err) => {
                warn!(
                    "Unable to load configuration file {}: {}",
                    config_file_path, err
                );
            }
        },
    }

    config_builder.build()
}

// format for logs
pub fn log_format(
    w: &mut dyn std::io::Write,
    now: &mut DeferredNow,
    record: &Record,
) -> Result<(), std::io::Error> {
    let level = record.level();
    write!(
        w,
        "[{}] T[{:?}] {} [{}] {}",
        now.now().format("%Y-%m-%d %H:%M:%S%.3f"),
        thread::current().name().unwrap_or("<unnamed>"),
        record.level(),
        record.module_path().unwrap_or("<unnamed>"),
        style(level, &record.args()),
    )
}

fn main() {
    let app = clap_app!(splinterd =>
        (version: crate_version!())
        (about: "Splinter Daemon")
        (@arg config: -c --config +takes_value)
        (@arg node_id: --("node-id") +takes_value
          "Unique ID for the node ")
        (@arg storage: --("storage") +takes_value
          "Storage type used for the node; defaults to yaml")
        (@arg transport: --("transport") +takes_value
          "Transport type for sockets, either raw or tls")
        (@arg network_endpoint: -n --("network-endpoint") +takes_value
          "Endpoint to connect to the network, tcp://ip:port")
        (@arg service_endpoint: --("service-endpoint") +takes_value
          "Endpoint that service will connect to, tcp://ip:port")
        (@arg peers: --peer +takes_value +multiple
          "Endpoint that service will connect to, ip:port")
        (@arg ca_file: --("ca-file") +takes_value
          "File path to the trusted CA certificate")
        (@arg cert_dir: --("cert-dir") +takes_value
          "Path to the directory where the certificates and keys are")
        (@arg client_cert: --("client-cert") +takes_value
          "File path to the certificate for the node when connecting to a node")
        (@arg server_cert: --("server-cert") +takes_value
          "File path to the certificate for the node when connecting to a node")
        (@arg server_key:  --("server-key") +takes_value
          "File path to the key for the node when connecting to a node as server")
        (@arg client_key:  --("client-key") +takes_value
          "File path to the key for the node when connecting to a node as client")
        (@arg insecure:  --("insecure")
          "If set to tls, should accept all peer certificates")
        (@arg bind: --("bind") +takes_value
          "Connection endpoint for REST API")
        (@arg registry_backend: --("registry-backend") +takes_value
          "Backend type for the node registry. Possible values: FILE.")
        (@arg registry_file: --("registry-file") +takes_value
          "File path to the node registry file if registry-backend is FILE.")
        (@arg admin_service_coordinator_timeout: --("admin-timeout") +takes_value
            "The coordinator timeout for admin service proposals (in milliseconds); default is \
             30000 (30 seconds)")
        (@arg verbose: -v --verbose +multiple
          "Increase output verbosity"));

    let app = app.arg(
        Arg::with_name("heartbeat_interval")
            .long("heartbeat")
            .long_help(
                "How often heartbeat should be sent, in seconds; defaults to 30 seconds,\
                 0 means off",
            )
            .takes_value(true),
    );

    #[cfg(feature = "database")]
    let app = app.arg(
        Arg::with_name("database")
            .long("database")
            .long_help("DB connection URL")
            .takes_value(true),
    );

    #[cfg(feature = "biome")]
    let app = app.arg(
        Arg::with_name("biome_enabled")
            .long("enable-biome")
            .long_help("Enable the biome subsystem"),
    );

    #[cfg(feature = "generate-certs")]
    let app = app
        .arg(
            Arg::with_name("generate_certs")
                .long("generate-certs")
                .long_help(
                    "Deprecated: If set, certificates will be generated and insecure will be false; \
                     use only for development",
                ),
        )
        .arg(
            Arg::with_name("common_name")
                .long("common-name")
                .long_help(
                    "Deprecated: The common name that should be used in the generated certificate; \
                     defaults to localhost",
                )
                .takes_value(true),
        );

    let matches = app.get_matches();

    let log_level = match matches.occurrences_of("verbose") {
        0 => log::LevelFilter::Warn,
        1 => log::LevelFilter::Info,
        2 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    };

    let mut log_spec_builder = LogSpecBuilder::new();
    log_spec_builder.default(log_level);
    log_spec_builder.module("hyper", log::LevelFilter::Warn);
    log_spec_builder.module("tokio", log::LevelFilter::Warn);

    Logger::with(log_spec_builder.build())
        .format(log_format)
        .start()
        .expect("Failed to create logger");

    if let Err(err) = start_daemon(matches) {
        error!("Failed to start daemon, {}", err);
        std::process::exit(1);
    }
}

fn start_daemon(matches: ArgMatches) -> Result<(), UserError> {
    debug!("Loading configuration file");

    // get provided config file or search default location
    let config_file_path = matches
        .value_of("config")
        .unwrap_or("/etc/splinter/splinterd.toml");

    let config = load_toml_config(config_file_path);

    let node_id = matches
        .value_of("node_id")
        .map(String::from)
        .or_else(|| config.node_id())
        .ok_or_else(|| UserError::MissingArgument("node_id".into()))?;

    let storage_type = matches
        .value_of("storage")
        .map(String::from)
        .or_else(|| config.storage())
        .unwrap_or_else(|| String::from("yaml"));

    let transport_type = matches
        .value_of("transport")
        .map(String::from)
        .or_else(|| config.transport())
        .unwrap_or_else(|| String::from("raw"));

    let service_endpoint = matches
        .value_of("service_endpoint")
        .map(String::from)
        .or_else(|| config.service_endpoint())
        .unwrap_or_else(|| "127.0.0.1:8043".to_string());

    let network_endpoint = matches
        .value_of("network_endpoint")
        .map(String::from)
        .or_else(|| config.network_endpoint())
        .unwrap_or_else(|| "127.0.0.1:8044".to_string());

    let initial_peers = matches
        .values_of("peers")
        .map(|values| values.map(String::from).collect::<Vec<String>>())
        .or_else(|| config.peers())
        .unwrap_or_default();

    let heartbeat_interval = value_t!(matches.value_of("heartbeat_interval"), u64)
        .unwrap_or_else(|_| config.heartbeat_interval().unwrap_or(HEARTBEAT_DEFAULT));

    let (transport, transport_log) = get_transport(&transport_type, &matches, &config)?;

    let location = {
        if let Ok(s) = env::var(STATE_DIR_ENV) {
            s
        } else {
            DEFAULT_STATE_DIR.to_string()
        }
    };

    let storage_location = match &storage_type as &str {
        "yaml" => format!("{}{}", location, "circuits.yaml"),
        "memory" => "memory".to_string(),
        _ => {
            return Err(UserError::InvalidArgument(format!(
                "storage type is not supported: {}",
                storage_type
            )))
        }
    };

    let key_registry_location = match &storage_type as &str {
        "yaml" => format!("{}{}", location, "keys.yaml"),
        "memory" => "memory".to_string(),
        _ => {
            return Err(UserError::InvalidArgument(format!(
                "storage type is not supported: {}",
                storage_type
            )))
        }
    };

    let rest_api_endpoint = matches
        .value_of("bind")
        .map(String::from)
        .or_else(|| config.bind())
        .unwrap_or_else(|| "127.0.0.1:8080".to_string());

    #[cfg(feature = "database")]
    let db_url = matches
        .value_of("database")
        .map(String::from)
        .or_else(|| config.database());

    #[cfg(feature = "biome")]
    let biome_enabled: bool = matches.is_present("biome_enabled");

    let registry_backend = matches
        .value_of("registry_backend")
        .map(String::from)
        .or_else(|| config.registry_backend());

    let registry_file = matches
        .value_of("registry_file")
        .map(String::from)
        .or_else(|| config.registry_file());

    // Allow unused mut for experimental features
    #[allow(unused_mut)]
    let mut feature_fields = "".to_string();

    #[cfg(feature = "database")]
    {
        feature_fields = format!("{}, db_url: {:?}", feature_fields, db_url);
    }

    #[cfg(feature = "biome")]
    {
        feature_fields = format!("{}, biome_enabled: {}", feature_fields, biome_enabled);
    }

    let admin_service_coordinator_timeout = matches
        .value_of("admin_service_coordinator_timeout")
        .map(&str::parse::<u64>)
        .transpose()
        .map_err(|err| {
            UserError::InvalidArgument(format!(
                "admin service coordinator timeout is not a valid integer: {}",
                err
            ))
        })?
        .map(Duration::from_millis)
        .or_else(|| config.admin_service_coordinator_timeout())
        .unwrap_or_else(|| Duration::from_millis(DEFAULT_ADMIN_SERVICE_COORDINATOR_TIMEOUT_MILLIS));

    debug!(
        "Configuration: {{ storage_type: {}, storage_location: {}, key_registry_location: {}, {}, \
         service_endpoint: {}, network_endpoint: {}, initial_peers: {:?}, node_id: {}, \
         rest_api_endpoint: {}, registry_backend: {:?}, registry_file: {:?}, \
         heartbeat_interval: {}{} }}",
        storage_type,
        storage_location,
        key_registry_location,
        transport_log,
        service_endpoint,
        network_endpoint,
        initial_peers,
        node_id,
        rest_api_endpoint,
        registry_backend,
        registry_file,
        heartbeat_interval,
        feature_fields,
    );

    let mut daemon_builder = SplinterDaemonBuilder::new()
        .with_storage_location(storage_location)
        .with_key_registry_location(key_registry_location)
        .with_network_endpoint(network_endpoint)
        .with_service_endpoint(service_endpoint)
        .with_initial_peers(initial_peers)
        .with_node_id(node_id)
        .with_rest_api_endpoint(rest_api_endpoint)
        .with_registry_backend(registry_backend)
        .with_storage_type(storage_type)
        .with_heartbeat_interval(heartbeat_interval)
        .with_admin_service_coordinator_timeout(admin_service_coordinator_timeout);

    #[cfg(feature = "database")]
    {
        daemon_builder = daemon_builder.with_db_url(db_url);
    }

    #[cfg(feature = "biome")]
    {
        daemon_builder = daemon_builder.enable_biome(biome_enabled);
    }

    if let Some(registry_file) = registry_file {
        daemon_builder = daemon_builder.with_registry_file(registry_file);
    }

    let mut node = daemon_builder.build().map_err(|err| {
        UserError::daemon_err_with_source("unable to build the Splinter daemon", Box::new(err))
    })?;
    node.start(transport)?;
    Ok(())
}

fn get_transport(
    transport_type: &str,
    matches: &clap::ArgMatches,
    config: &Config,
) -> Result<(Box<dyn Transport + Send>, String), GetTransportError> {
    match transport_type {
        "tls" => {
            #[cfg(feature = "generate-certs")]
            {
                if matches.is_present("generate_certs") {
                    warn!("Deprecated: Generating Certs for TLS Transport");

                    let common_name = matches
                        .value_of("common_name")
                        .map(String::from)
                        .unwrap_or_else(|| String::from("localhost"));

                    // Generate Certificate Authority keys and certificate
                    let (ca_key, ca_cert) = make_ca_cert()?;

                    // Create temp directory to store ca.cert
                    let temp_dir = TempDir::new("tls-transport")?;
                    let temp_dir_path = temp_dir.path();

                    // Generate client and server keys and certificates
                    let (client_key, client_cert) =
                        make_ca_signed_cert(&ca_cert, &ca_key, &common_name)?;
                    let (server_key, server_cert) =
                        make_ca_signed_cert(&ca_cert, &ca_key, &common_name)?;

                    let client_cert = write_file(
                        temp_dir_path.to_path_buf(),
                        "client.cert",
                        &client_cert.to_pem()?,
                    )?;

                    let client_key_file = write_file(
                        temp_dir_path.to_path_buf(),
                        "client.key",
                        &client_key.private_key_to_pem_pkcs8()?,
                    )?;

                    let server_cert = write_file(
                        temp_dir_path.to_path_buf(),
                        "server.cert",
                        &server_cert.to_pem()?,
                    )?;

                    let server_key_file = write_file(
                        temp_dir_path.to_path_buf(),
                        "server.key",
                        &server_key.private_key_to_pem_pkcs8()?,
                    )?;

                    warn!("Starting TlsTransport in insecure mode");

                    let log_value = "ca_certs: generated, client_cert: generated, client_key: \
                                     generated, server_cert: generated, server_key: generated"
                        .to_string();

                    // Start transport in insecure mode, do not verify the certs if auto generated,
                    // as the ca will not match
                    let transport = TlsTransport::new(
                        None,
                        client_key_file,
                        client_cert,
                        server_key_file,
                        server_cert,
                    )?;

                    return Ok((Box::new(transport), log_value));
                }
            }

            let cert_location = {
                if let Ok(s) = env::var(CERT_DIR_ENV) {
                    s
                } else {
                    DEFAULT_CERT_DIR.to_string()
                }
            };

            let cert_dir = matches
                .value_of("cert_dir")
                .map(String::from)
                .or_else(|| config.cert_dir())
                .unwrap_or(cert_location);

            let client_cert = matches
                .value_of("client_cert")
                .map(String::from)
                .or_else(|| config.client_cert())
                .or_else(|| {
                    let cert_dir_path = Path::new(&cert_dir);
                    let client_cert = cert_dir_path.join(CLIENT_CERT);
                    if !client_cert.is_file() {
                        error!("Client cert file not found: {:?}", client_cert);
                        return None;
                    }
                    let client_cert: Option<String> = client_cert.to_str().map(ToOwned::to_owned);
                    client_cert
                })
                .ok_or_else(|| {
                    GetTransportError::CertError("must provide a valid client certificate".into())
                })?;

            let server_cert = matches
                .value_of("server_cert")
                .map(String::from)
                .or_else(|| config.server_cert())
                .or_else(|| {
                    let cert_dir_path = Path::new(&cert_dir);
                    let server_cert = cert_dir_path.join(SERVER_CERT);
                    if !server_cert.is_file() {
                        error!("Server cert file not found: {:?}", server_cert);
                        return None;
                    }
                    let server_cert: Option<String> = server_cert.to_str().map(ToOwned::to_owned);
                    server_cert
                })
                .ok_or_else(|| {
                    GetTransportError::CertError("must provide a valid server certificate".into())
                })?;

            let server_key_file = matches
                .value_of("server_key")
                .map(String::from)
                .or_else(|| config.server_key())
                .or_else(|| {
                    let cert_dir_path = Path::new(&cert_dir);
                    let server_key = cert_dir_path.join(SERVER_KEY);
                    if !server_key.is_file() {
                        error!("Server key file not found: {:?}", server_key);
                        return None;
                    }
                    let server_key: Option<String> = server_key.to_str().map(ToOwned::to_owned);
                    server_key
                })
                .ok_or_else(|| {
                    GetTransportError::CertError("must provide a valid server key path".into())
                })?;

            let client_key_file = matches
                .value_of("client_key")
                .map(String::from)
                .or_else(|| config.client_key())
                .or_else(|| {
                    let cert_dir_path = Path::new(&cert_dir);
                    let client_key = cert_dir_path.join(CLIENT_KEY);
                    if !client_key.is_file() {
                        error!("Client key file not found: {:?}", client_key);
                        return None;
                    }
                    let client_key: Option<String> = client_key.to_str().map(ToOwned::to_owned);
                    client_key
                })
                .ok_or_else(|| {
                    GetTransportError::CertError("must provide a valid client key path".into())
                })?;

            let ca_file = {
                if matches.is_present("insecure") {
                    warn!("Starting TlsTransport in insecure mode");
                    None
                } else {
                    let ca_file = matches
                        .value_of("ca_file")
                        .map(String::from)
                        .or_else(|| config.ca_certs())
                        .or_else(|| {
                            let cert_dir_path = Path::new(&cert_dir);
                            let ca_path = cert_dir_path.join(CA_PEM);
                            if !ca_path.is_file() {
                                error!("CA file not found: {:?}", ca_path);
                                return None;
                            }
                            let ca_file: Option<String> =
                                cert_dir_path.join(CA_PEM).to_str().map(ToOwned::to_owned);
                            ca_file
                        })
                        .ok_or_else(|| {
                            GetTransportError::CertError(
                                "must provide a valid file containing ca certs".into(),
                            )
                        })?;
                    Some(ca_file)
                }
            };

            let ca_file_log = {
                if let Some(ca_file) = &ca_file {
                    match fs::canonicalize(&ca_file)?.to_str() {
                        Some(ca_path) => ca_path.to_string(),
                        None => {
                            return Err(GetTransportError::CertError(
                                "CA path is not a valid path".to_string(),
                            ))
                        }
                    }
                } else {
                    "insecure".to_string()
                }
            };

            let log_value = format!(
                "transport_type: tls, ca_certs: {:?}, client_cert: {:?}, \
                 client_key: {:?}, server_cert: {:?}, server_key: {:?}",
                ca_file_log,
                fs::canonicalize(client_cert.clone())?,
                fs::canonicalize(client_key_file.clone())?,
                fs::canonicalize(server_cert.clone())?,
                fs::canonicalize(server_key_file.clone())?,
            );

            let transport = TlsTransport::new(
                ca_file,
                client_key_file,
                client_cert,
                server_key_file,
                server_cert,
            )?;

            Ok((Box::new(transport), log_value))
        }
        "raw" => Ok((
            Box::new(RawTransport::default()),
            "transport_type: raw".to_string(),
        )),
        _ => Err(GetTransportError::NotSupportedError(format!(
            "Transport type {} is not supported",
            transport_type
        ))),
    }
}

#[derive(Debug)]
pub enum UserError {
    TransportError(GetTransportError),
    MissingArgument(String),
    InvalidArgument(String),

    DaemonError {
        context: String,
        source: Option<Box<dyn Error>>,
    },
}

impl UserError {
    pub fn daemon_error(context: &str) -> Self {
        UserError::DaemonError {
            context: context.into(),
            source: None,
        }
    }

    pub fn daemon_err_with_source(context: &str, err: Box<dyn Error>) -> Self {
        UserError::DaemonError {
            context: context.into(),
            source: Some(err),
        }
    }
}

impl Error for UserError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            UserError::TransportError(err) => Some(err),
            UserError::MissingArgument(_) => None,
            UserError::InvalidArgument(_) => None,
            UserError::DaemonError { source, .. } => {
                if let Some(ref err) = source {
                    Some(&**err)
                } else {
                    None
                }
            }
        }
    }
}

impl fmt::Display for UserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            UserError::TransportError(err) => write!(f, "unable to get transport: {}", err),
            UserError::MissingArgument(msg) => write!(f, "missing required argument: {}", msg),
            UserError::InvalidArgument(msg) => write!(f, "required argument is invalid: {}", msg),
            UserError::DaemonError { context, source } => {
                if let Some(ref err) = source {
                    write!(f, "{}: {}", context, err)
                } else {
                    f.write_str(&context)
                }
            }
        }
    }
}

impl From<StartError> for UserError {
    fn from(error: StartError) -> Self {
        UserError::daemon_err_with_source("unable to start the Splinter daemon", Box::new(error))
    }
}

impl From<GetTransportError> for UserError {
    fn from(error: GetTransportError) -> Self {
        UserError::TransportError(error)
    }
}

#[derive(Debug)]
pub enum GetTransportError {
    CertError(String),
    NotSupportedError(String),
    TlsTransportError(TlsInitError),
    #[cfg(feature = "generate-certs")]
    OpensslError(ErrorStack),
    IoError(io::Error),
}

impl Error for GetTransportError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            GetTransportError::CertError(_) => None,
            GetTransportError::NotSupportedError(_) => None,
            GetTransportError::TlsTransportError(err) => Some(err),
            #[cfg(feature = "generate-certs")]
            GetTransportError::OpensslError(err) => Some(err),
            GetTransportError::IoError(err) => Some(err),
        }
    }
}

impl fmt::Display for GetTransportError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GetTransportError::CertError(msg) => {
                write!(f, "unable to retrieve certificate: {}", msg)
            }
            GetTransportError::NotSupportedError(msg) => {
                write!(f, "received transport type that is not supported: {}", msg)
            }
            GetTransportError::TlsTransportError(err) => {
                write!(f, "unable to create TLS transport: {}", err)
            }
            #[cfg(feature = "generate-certs")]
            GetTransportError::OpensslError(err) => {
                write!(f, "unable to generate certificates: {}", err)
            }
            GetTransportError::IoError(err) => {
                write!(f, "unable to get transport due to IoError: {}", err)
            }
        }
    }
}

#[cfg(feature = "generate-certs")]
impl From<CertError> for GetTransportError {
    fn from(cert_error: CertError) -> Self {
        GetTransportError::CertError(format!("CertError: {:?}", cert_error))
    }
}

impl From<TlsInitError> for GetTransportError {
    fn from(tls_error: TlsInitError) -> Self {
        GetTransportError::TlsTransportError(tls_error)
    }
}

#[cfg(feature = "generate-certs")]
impl From<ErrorStack> for GetTransportError {
    fn from(error_stack: ErrorStack) -> Self {
        GetTransportError::OpensslError(error_stack)
    }
}

impl From<io::Error> for GetTransportError {
    fn from(io_error: io::Error) -> Self {
        GetTransportError::IoError(io_error)
    }
}
