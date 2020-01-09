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

use std::env;
use std::fs::{self, metadata, OpenOptions};
use std::io;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};

#[cfg(target_os = "linux")]
use std::os::linux::fs::MetadataExt;
#[cfg(not(target_os = "linux"))]
use std::os::unix::fs::MetadataExt;

use clap::ArgMatches;
use flexi_logger::ReconfigurationHandle;
use openssl::asn1::Asn1Time;
use openssl::bn::{BigNum, MsbOption};
use openssl::error::ErrorStack;
use openssl::hash::MessageDigest;
use openssl::pkey::{PKey, PKeyRef, Private};
use openssl::rsa::Rsa;
use openssl::x509::extension::{BasicConstraints, ExtendedKeyUsage, KeyUsage};
use openssl::x509::{X509NameBuilder, X509Ref, X509};

use crate::error::CliError;

use super::{chown, Action};

pub struct CertGenAction;

const DEFAULT_CERT_DIR: &str = "/etc/splinter/certs/";
const CERT_DIR_ENV: &str = "SPLINTER_CERT_DIR";

const CLIENT_CERT: &str = "client.crt";
const CLIENT_KEY: &str = "client.key";
const SERVER_CERT: &str = "server.crt";
const SERVER_KEY: &str = "server.key";
const CA_CERT: &str = "generated_ca.pem";
const CA_KEY: &str = "generated_ca.key";

impl Action for CertGenAction {
    fn reconfigure_logging<'a>(
        &self,
        arg_matches: Option<&ArgMatches<'a>>,
        logger_handle: &mut ReconfigurationHandle,
    ) -> Result<(), CliError> {
        let args = arg_matches.ok_or_else(|| CliError::RequiresArgs)?;

        if args.is_present("quiet") {
            logger_handle.parse_new_spec("error");
        }

        Ok(())
    }

    fn run<'a>(&mut self, arg_matches: Option<&ArgMatches<'a>>) -> Result<(), CliError> {
        let args = arg_matches.ok_or_else(|| CliError::RequiresArgs)?;

        let common_name = args
            .value_of("common_name")
            .unwrap_or("localhost")
            .to_string();

        let cert_dir_string = args
            .value_of("cert_dir")
            .map(ToOwned::to_owned)
            .or_else(|| env::var(CERT_DIR_ENV).ok())
            .or_else(|| Some(DEFAULT_CERT_DIR.to_string()))
            .unwrap();

        let cert_dir = Path::new(&cert_dir_string);

        // Check if the provided cert directory exists
        if !cert_dir.is_dir() {
            return Err(CliError::ActionError(format!(
                "Cert directory does not exist: {}",
                cert_dir.display()
            )));
        }

        let private_cert_path = cert_dir.join("private/");
        let cert_path = cert_dir.to_path_buf();

        // Check if the provided private key directory for the certs exists, if not create it
        if !private_cert_path.is_dir() {
            fs::create_dir_all(private_cert_path.clone()).map_err(|err| {
                CliError::ActionError(format!("Unable to create private directory: {}", err))
            })?
        }

        // check that both directories are writable
        match cert_dir.metadata() {
            Ok(metadata) => {
                if metadata.permissions().readonly() {
                    return Err(CliError::ActionError(format!(
                        "Cert directory is not writeable: {}",
                        absolute_path(&cert_dir)?,
                    )));
                }
            }
            Err(err) => {
                return Err(CliError::ActionError(format!(
                    "Cannot check if cert directory {} is writable: {}",
                    absolute_path(&cert_dir)?,
                    err
                )));
            }
        }

        match private_cert_path.metadata() {
            Ok(metadata) => {
                if metadata.permissions().readonly() {
                    return Err(CliError::ActionError(format!(
                        "Private cert directory is not writeable: {}",
                        absolute_path(&private_cert_path)?
                    )));
                }
            }
            Err(err) => {
                return Err(CliError::ActionError(format!(
                    "Cannot check if cert directory {} is writable: {}",
                    absolute_path(&private_cert_path)?,
                    err
                )));
            }
        }

        // if skip, check each pair of certificate/key to see if it exists. If not generate the
        // the missing files. If only one of the two files exists, this is an error.
        if args.is_present("skip") {
            return handle_skip(cert_path, private_cert_path, common_name);
        }

        // if force is not present, all files must not exist.
        if !args.is_present("force") {
            let client_cert_path = cert_dir.join(CLIENT_CERT);
            let server_cert_path = cert_dir.join(SERVER_CERT);
            let ca_cert_path = cert_dir.join(CA_CERT);

            let client_key_path = private_cert_path.join(CLIENT_KEY);
            let server_key_path = private_cert_path.join(SERVER_KEY);
            let ca_key_path = private_cert_path.join(CA_KEY);
            let mut errored = false;
            if client_cert_path.exists() {
                error!(
                    "Client certificate already exists: {}",
                    absolute_path(&client_cert_path)?,
                );
                errored = true;
            };

            if client_key_path.exists() {
                error!(
                    "Client key already exists: {}",
                    absolute_path(&client_key_path)?,
                );
                errored = true;
            }

            if server_cert_path.exists() {
                error!(
                    "Server certificate already exists: {}",
                    absolute_path(&server_cert_path)?,
                );
                errored = true;
            }

            if server_key_path.exists() {
                error!(
                    "Server key already exists: {}",
                    absolute_path(&server_key_path)?,
                );
                errored = true;
            }

            if ca_cert_path.exists() {
                error!(
                    "CA certificate already exists: {}",
                    absolute_path(&ca_cert_path)?,
                );
                errored = true;
            }

            if ca_key_path.exists() {
                error!("CA key already exists: {}", absolute_path(&ca_key_path)?);
                errored = true;
            }

            if errored {
                return Err(CliError::ActionError(
                    "Refusing to overwrite files, exiting".into(),
                ));
            } else {
                // if all files need to be generated log what will be written and generate all
                log_writing(&cert_path, &private_cert_path)?;
                create_all_certs(cert_path, private_cert_path, common_name)?;
            }
        } else {
            // if force is true, overwrite all existing files
            log_overwriting(&cert_path, &private_cert_path)?;
            create_all_certs(cert_dir.to_path_buf(), private_cert_path, common_name)?;
        }

        Ok(())
    }
}

// if skip, check each pair of certificate/key to see if it exists. If not generate the
// the missing files. If only one of the two files exists, this is an error.
fn handle_skip(
    cert_dir: PathBuf,
    private_cert_path: PathBuf,
    common_name: String,
) -> Result<(), CliError> {
    let client_cert_path = cert_dir.join(CLIENT_CERT);
    let server_cert_path = cert_dir.join(SERVER_CERT);
    let ca_cert_path = cert_dir.join(CA_CERT);

    let client_key_path = private_cert_path.join(CLIENT_KEY);
    let server_key_path = private_cert_path.join(SERVER_KEY);
    let ca_key_path = private_cert_path.join(CA_KEY);
    let cert_path = cert_dir;
    let mut ca;

    // if all exists, log existence and return
    if client_cert_path.exists()
        && client_key_path.exists()
        && server_cert_path.exists()
        && server_key_path.exists()
        && ca_cert_path.exists()
        && ca_key_path.exists()
    {
        info!(
            "Client certificate exists, skipping: {}",
            absolute_path(&client_cert_path)?,
        );
        info!(
            "Client key exists, skipping: {}",
            absolute_path(&client_key_path)?,
        );
        info!(
            "Server certificate exists, skipping: {}",
            absolute_path(&server_cert_path)?,
        );
        info!(
            "Server key exists, skipping: {}",
            absolute_path(&server_key_path)?,
        );
        info!(
            "CA certificate exists, skipping: {}",
            absolute_path(&ca_cert_path)?,
        );
        info!("CA key exists, skipping: {}", absolute_path(&ca_key_path)?);
        return Ok(());
    }

    if (ca_cert_path.exists() || ca_key_path.exists())
        && !(ca_cert_path.exists() && ca_key_path.exists())
    {
        // if one exists without the other return an error
        if ca_cert_path.exists() {
            return Err(CliError::ActionError(format!(
                "Matching key for the certificate is missing: {}/{} ",
                absolute_path(&cert_path)?,
                CA_KEY
            )));
        } else {
            return Err(CliError::ActionError(format!(
                "Matching certificate for the key is missing: {}/{} ",
                absolute_path(&private_cert_path)?,
                CA_CERT
            )));
        }
    }

    if (client_cert_path.exists() || client_key_path.exists())
        && !(client_cert_path.exists() && client_key_path.exists())
    {
        // if one exists without the other return an error
        if client_cert_path.exists() {
            return Err(CliError::ActionError(format!(
                "Matching key for the certificate is missing: {}/{} ",
                absolute_path(&cert_path)?,
                CLIENT_KEY
            )));
        } else {
            return Err(CliError::ActionError(format!(
                "Matching certificate for the key is missing: {}/{} ",
                absolute_path(&private_cert_path)?,
                CLIENT_CERT
            )));
        }
    }

    if (server_cert_path.exists() || server_key_path.exists())
        && !(server_cert_path.exists() && server_key_path.exists())
    {
        // if one exists without the other return an error
        if server_cert_path.exists() {
            return Err(CliError::ActionError(format!(
                "Matching key for the certificate is missing: {}/{} ",
                absolute_path(&cert_path)?,
                SERVER_KEY
            )));
        } else {
            return Err(CliError::ActionError(format!(
                "Matching certificate for the key is missing: {}/{} ",
                absolute_path(&private_cert_path)?,
                SERVER_CERT
            )));
        }
    }

    // if ca files exists, log and read the cert and key from the file
    if ca_cert_path.exists() && ca_key_path.exists() {
        info!(
            "CA certificate exists, skipping: {}",
            absolute_path(&ca_cert_path)?,
        );
        info!("CA key exists, skipping: {}", absolute_path(&ca_key_path)?);
        let ca_cert = get_ca_cert(&ca_cert_path)?;
        let ca_key = get_ca_key(&ca_key_path)?;
        ca = Some((ca_key, ca_cert));
    } else {
        // if the ca files do not exist, generate them
        info!("Writing file: {}/{}", absolute_path(&cert_path)?, CA_CERT);
        info!(
            "Writing file: {}/{}",
            absolute_path(&private_cert_path)?,
            CA_KEY
        );
        let (genearte_ca_key, generate_ca_cert) =
            write_ca(cert_path.clone(), private_cert_path.clone())?;
        ca = Some((genearte_ca_key, generate_ca_cert));
    }

    // if the client files exist log
    if client_cert_path.exists() && client_key_path.exists() {
        info!(
            "Client certificate exists, skipping: {}",
            absolute_path(&client_cert_path)?,
        );
        info!(
            "Client key exists, skipping: {}",
            absolute_path(&client_key_path)?,
        );
    } else {
        // if the client files do not exist, generate them using the ca
        info!(
            "Writing file: {}/{}",
            absolute_path(&cert_path)?,
            CLIENT_CERT
        );
        info!(
            "Writing file: {}/{}",
            absolute_path(&private_cert_path)?,
            CLIENT_KEY
        );
        if let Some((ca_key, ca_cert)) = ca {
            write_client(
                cert_path.clone(),
                private_cert_path.clone(),
                &ca_key,
                &ca_cert,
                &common_name,
            )?;
            ca = Some((ca_key, ca_cert));
        } else {
            // this should never happen
            return Err(CliError::ActionError("CA does not exist".into()));
        }
    }

    if server_cert_path.exists() && server_key_path.exists() {
        info!(
            "Server certificate exists, skipping: {}",
            absolute_path(&server_cert_path)?,
        );
        info!(
            "Server key exists, skipping: {}",
            absolute_path(&server_key_path)?,
        );
    } else {
        // if the server files do not exist, generate them using the ca
        info!(
            "Writing file: {}/{}",
            absolute_path(&cert_path)?,
            SERVER_CERT
        );
        info!(
            "Writing file: {}/{}",
            absolute_path(&private_cert_path)?,
            SERVER_KEY
        );
        if let Some((ca_key, ca_cert)) = ca {
            write_server(
                cert_path,
                private_cert_path,
                &ca_key,
                &ca_cert,
                &common_name,
            )?;
        } else {
            // this should never happen
            return Err(CliError::ActionError("CA does not exist".into()));
        }
    }
    Ok(())
}

// create all certificates and keys from scratch
fn create_all_certs(
    cert_path: PathBuf,
    private_cert_path: PathBuf,
    common_name: String,
) -> Result<(), CliError> {
    // Generate Certificate Authority keys and certificate.
    // These files are not saved
    let (ca_key, ca_cert) = write_ca(cert_path.clone(), private_cert_path.clone())?;
    // Generate client and server keys and certificates

    write_client(
        cert_path.clone(),
        private_cert_path.clone(),
        &ca_key,
        &ca_cert,
        &common_name,
    )?;

    write_server(
        cert_path,
        private_cert_path,
        &ca_key,
        &ca_cert,
        &common_name,
    )?;

    Ok(())
}

// Generate Certificate Authority keys and certificate.
fn write_ca(
    cert_path: PathBuf,
    private_cert_path: PathBuf,
) -> Result<(PKey<Private>, X509), CliError> {
    let (ca_key, ca_cert) = make_ca_cert()?;

    write_file(cert_path, CA_CERT, &ca_cert.to_pem()?)?;

    write_file(
        private_cert_path,
        CA_KEY,
        &ca_key.private_key_to_pem_pkcs8()?,
    )?;

    Ok((ca_key, ca_cert))
}

// Generate server keys and certificate.
fn write_server(
    cert_path: PathBuf,
    private_cert_path: PathBuf,
    ca_key: &PKey<Private>,
    ca_cert: &X509,
    common_name: &str,
) -> Result<(), CliError> {
    let (server_key, server_cert) = make_ca_signed_cert(ca_cert, ca_key, common_name)?;

    write_file(cert_path, SERVER_CERT, &server_cert.to_pem()?)?;

    write_file(
        private_cert_path,
        SERVER_KEY,
        &server_key.private_key_to_pem_pkcs8()?,
    )?;
    Ok(())
}

// Generate client keys and certificate.
fn write_client(
    cert_path: PathBuf,
    private_cert_path: PathBuf,
    ca_key: &PKey<Private>,
    ca_cert: &X509,
    common_name: &str,
) -> Result<(), CliError> {
    let (server_key, server_cert) = make_ca_signed_cert(ca_cert, ca_key, common_name)?;

    write_file(cert_path, CLIENT_CERT, &server_cert.to_pem()?)?;

    write_file(
        private_cert_path,
        CLIENT_KEY,
        &server_key.private_key_to_pem_pkcs8()?,
    )?;
    Ok(())
}

// Make a certificate and private key for the Certificate  Authority
fn make_ca_cert() -> Result<(PKey<Private>, X509), CliError> {
    // generate private key
    let rsa = Rsa::generate(2048)?;
    let privkey = PKey::from_rsa(rsa)?;

    // build x509 name
    let mut x509_name = X509NameBuilder::new()?;
    x509_name.append_entry_by_text("CN", "generated_ca")?;
    let x509_name = x509_name.build();

    // build x509 cert
    let mut cert_builder = X509::builder()?;
    cert_builder.set_version(2)?;
    cert_builder.set_subject_name(&x509_name)?;
    cert_builder.set_issuer_name(&x509_name)?;
    cert_builder.set_pubkey(&privkey)?;

    let not_before = Asn1Time::days_from_now(0)?;
    cert_builder.set_not_before(&not_before)?;
    let not_after = Asn1Time::days_from_now(365)?;
    cert_builder.set_not_after(&not_after)?;

    cert_builder.append_extension(BasicConstraints::new().critical().ca().build()?)?;
    cert_builder.append_extension(KeyUsage::new().key_cert_sign().build()?)?;

    cert_builder.sign(&privkey, MessageDigest::sha256())?;
    let cert = cert_builder.build();

    // return private key and ca_cert
    Ok((privkey, cert))
}

// Make a certificate and private key signed by the given CA cert and private key
// Cert could act like both server or client
fn make_ca_signed_cert(
    ca_cert: &X509Ref,
    ca_privkey: &PKeyRef<Private>,
    common_name: &str,
) -> Result<(PKey<Private>, X509), CliError> {
    // generate private key
    let rsa = Rsa::generate(2048)?;
    let privkey = PKey::from_rsa(rsa)?;

    // build x509_name
    let mut x509_name = X509NameBuilder::new()?;
    x509_name.append_entry_by_text("CN", &common_name)?;
    let x509_name = x509_name.build();

    // build x509 cert
    let mut cert_builder = X509::builder()?;
    cert_builder.set_version(2)?;
    let serial_number = {
        let mut serial = BigNum::new()?;
        serial.rand(159, MsbOption::MAYBE_ZERO, false)?;
        serial.to_asn1_integer()?
    };
    cert_builder.set_serial_number(&serial_number)?;
    cert_builder.set_subject_name(&x509_name)?;
    cert_builder.set_issuer_name(ca_cert.subject_name())?;
    cert_builder.set_pubkey(&privkey)?;
    let not_before = Asn1Time::days_from_now(0)?;
    cert_builder.set_not_before(&not_before)?;
    let not_after = Asn1Time::days_from_now(365)?;
    cert_builder.set_not_after(&not_after)?;

    // allow keys to be used for both server and client authorization
    cert_builder.append_extension(
        ExtendedKeyUsage::new()
            .server_auth()
            .client_auth()
            .build()?,
    )?;

    // sign the cert by the ca
    cert_builder.sign(&ca_privkey, MessageDigest::sha256())?;
    let cert = cert_builder.build();

    // return private key and cert
    Ok((privkey, cert))
}

/// write the a file to a temp file name and then rename to final filename
/// this will guarantee that the final file will ony ever contain valid data
fn write_file(path_buf: PathBuf, file_name: &str, bytes: &[u8]) -> Result<(), CliError> {
    let temp_path_buf = path_buf.join(format!(".{}.new", file_name));
    let temp_path = {
        if let Some(path) = temp_path_buf.to_str() {
            path.to_string()
        } else {
            return Err(CliError::ActionError(
                "Path is not valid unicode".to_string(),
            ));
        }
    };

    let final_path_buf = path_buf.join(file_name);
    let final_path = {
        if let Some(path) = final_path_buf.to_str() {
            path.to_string()
        } else {
            return Err(CliError::ActionError(
                "Path is not valid unicode".to_string(),
            ));
        }
    };

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o640)
        .open(temp_path.clone())?;
    file.write_all(bytes)?;

    // change permissions
    let dir = path_buf.as_path();
    let dir_info = metadata(dir).map_err(|err| CliError::EnvironmentError(format!("{}", err)))?;
    #[cfg(not(target_os = "linux"))]
    let (dir_uid, dir_gid) = (dir_info.uid(), dir_info.gid());
    #[cfg(target_os = "linux")]
    let (dir_uid, dir_gid) = (dir_info.st_uid(), dir_info.st_gid());
    chown(temp_path_buf.as_path(), dir_uid, dir_gid)?;

    fs::rename(temp_path, final_path)?;

    Ok(())
}

// helper function to get the absolute_path of a provided path
// this will be used in logging
fn absolute_path(path: &Path) -> Result<String, CliError> {
    let temp_path_buf = fs::canonicalize(path)?;
    if let Some(path) = temp_path_buf.to_str() {
        Ok(path.to_string())
    } else {
        Err(CliError::ActionError(
            "Path is not valid unicode".to_string(),
        ))
    }
}

// create a X509 certficate from a file
fn get_ca_cert(cert_path: &Path) -> Result<X509, CliError> {
    let cert = fs::read(cert_path)?;
    let cert = X509::from_pem(&cert)?;
    Ok(cert)
}

// create a PKey<Private> from a file
fn get_ca_key(key_path: &Path) -> Result<PKey<Private>, CliError> {
    let key = fs::read(key_path)?;
    let rsa = Rsa::private_key_from_pem(&key)?;
    let privkey = PKey::from_rsa(rsa)?;
    Ok(privkey)
}

// helper function to log what files will be written
fn log_writing(cert_path: &PathBuf, private_cert_path: &PathBuf) -> Result<(), CliError> {
    info!("Writing file: {}/{}", absolute_path(cert_path)?, CA_CERT);
    info!(
        "Writing file: {}/{}",
        absolute_path(private_cert_path)?,
        CA_KEY
    );

    info!(
        "Writing file: {}/{}",
        absolute_path(cert_path)?,
        CLIENT_CERT
    );
    info!(
        "Writing file: {}/{}",
        absolute_path(private_cert_path)?,
        CLIENT_KEY
    );

    info!(
        "Writing file: {}/{}",
        absolute_path(cert_path)?,
        SERVER_CERT
    );
    info!(
        "Writing file: {}/{}",
        absolute_path(private_cert_path)?,
        SERVER_KEY
    );
    Ok(())
}

// helper function to log what files will be overwritten
fn log_overwriting(cert_path: &PathBuf, private_cert_path: &PathBuf) -> Result<(), CliError> {
    info!(
        "Overwriting file: {}/{}",
        absolute_path(cert_path)?,
        CA_CERT
    );
    info!(
        "Overwriting file: {}/{}",
        absolute_path(private_cert_path)?,
        CA_KEY
    );

    info!(
        "Overwriting file: {}/{}",
        absolute_path(cert_path)?,
        CLIENT_CERT
    );
    info!(
        "Overwriting file: {}/{}",
        absolute_path(private_cert_path)?,
        CLIENT_KEY
    );

    info!(
        "Overwriting file: {}/{}",
        absolute_path(cert_path)?,
        SERVER_CERT
    );
    info!(
        "Overwriting file: {}/{}",
        absolute_path(private_cert_path)?,
        SERVER_KEY
    );
    Ok(())
}

impl From<io::Error> for CliError {
    fn from(io_error: io::Error) -> Self {
        CliError::ActionError(io_error.to_string())
    }
}

impl From<ErrorStack> for CliError {
    fn from(error_stack: ErrorStack) -> Self {
        CliError::ActionError(error_stack.to_string())
    }
}
