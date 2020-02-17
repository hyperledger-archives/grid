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

use openssl::asn1::Asn1Time;
use openssl::bn::{BigNum, MsbOption};
use openssl::error::ErrorStack;
use openssl::hash::MessageDigest;
use openssl::pkey::{PKey, PKeyRef, Private};
use openssl::rsa::Rsa;
use openssl::x509::extension::{BasicConstraints, ExtendedKeyUsage, KeyUsage};
use openssl::x509::{X509NameBuilder, X509Ref, X509};

use std::fs::File;
use std::io;
use std::io::Write;
use std::path::PathBuf;

// Make a certificate and private key for the Certificate  Authority
pub fn make_ca_cert() -> Result<(PKey<Private>, X509), CertError> {
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
pub fn make_ca_signed_cert(
    ca_cert: &X509Ref,
    ca_privkey: &PKeyRef<Private>,
    common_name: &str,
) -> Result<(PKey<Private>, X509), CertError> {
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

// wrie the a file to a temp directory
pub fn write_file(
    mut temp_dir: PathBuf,
    file_name: &str,
    bytes: &[u8],
) -> Result<String, CertError> {
    temp_dir.push(file_name);
    let path = {
        if let Some(path) = temp_dir.to_str() {
            path.to_string()
        } else {
            return Err(CertError::PathError(
                "Path is not valid unicode".to_string(),
            ));
        }
    };
    let mut file = File::create(path.to_string())?;
    file.write_all(bytes)?;

    Ok(path)
}

#[derive(Debug)]
pub enum CertError {
    IoError(io::Error),
    PathError(String),
    OpensslError(ErrorStack),
}

impl From<io::Error> for CertError {
    fn from(io_error: io::Error) -> Self {
        CertError::IoError(io_error)
    }
}

impl From<ErrorStack> for CertError {
    fn from(error_stack: ErrorStack) -> Self {
        CertError::OpensslError(error_stack)
    }
}
