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

//! Signing trait implementations backed by the Sawtooth SDK.
//!
//! Requires the "sawtooth-signing-compat" feature enabled
use sawtooth_sdk::signing::{secp256k1, Context};

use crate::hex;

use super::{error::Error, SignatureVerifier, Signer};

/// A Sawtooth Secp256k Signer that references a context.
///
/// The SawtoothSecp256k1RefSigner provides an implementation of the Signer trait, that uses a
/// provided Secp256k1Context.
pub struct SawtoothSecp256k1RefSigner<'c> {
    context: &'c secp256k1::Secp256k1Context,
    private_key: secp256k1::Secp256k1PrivateKey,
    public_key: Vec<u8>,
}

impl<'c> SawtoothSecp256k1RefSigner<'c> {
    pub fn new(
        context: &'c secp256k1::Secp256k1Context,
        private_key: secp256k1::Secp256k1PrivateKey,
    ) -> Result<Self, Error> {
        let public_key = context
            .get_public_key(&private_key)
            .map_err(|err| Error::SigningError(format!("Unable to extract public key: {}", err)))?
            .as_slice()
            .to_vec();
        Ok(Self {
            context,
            private_key,
            public_key,
        })
    }
}

impl<'c> Signer for SawtoothSecp256k1RefSigner<'c> {
    fn sign(&self, message: &[u8]) -> Result<Vec<u8>, Error> {
        self.context
            .sign(message, &self.private_key)
            .map_err(|err| Error::SigningError(format!("Failed to sign message: {}", err)))
            .and_then(|signature| {
                hex::parse_hex(&signature).map_err(|err| {
                    Error::SigningError(format!(
                        "Unable to parse sawtooth signature {} into bytes: {}",
                        signature, err
                    ))
                })
            })
    }

    fn public_key(&self) -> &[u8] {
        &self.public_key
    }
}

/// A Sawtooth Secp256k SignatureVerifier that references a context.
///
/// The SawtoothSecp256k1RefSignatureVeriifier provides an implementation of the SignatureVerifier
/// trait, that uses a provided Secp256k1Context.
pub struct SawtoothSecp256k1RefSignatureVeriifier<'c> {
    context: &'c secp256k1::Secp256k1Context,
}

impl<'c> SawtoothSecp256k1RefSignatureVeriifier<'c> {
    pub fn new(context: &'c secp256k1::Secp256k1Context) -> Self {
        Self { context }
    }
}

impl<'c> SignatureVerifier for SawtoothSecp256k1RefSignatureVeriifier<'c> {
    fn verify(&self, message: &[u8], signature: &[u8], public_key: &[u8]) -> Result<bool, Error> {
        let public_key_hex = hex::to_hex(public_key);
        let public_key =
            secp256k1::Secp256k1PublicKey::from_hex(&public_key_hex).map_err(|err| {
                Error::SignatureVerificationError(format!(
                    "Unable to read public key {}: {}",
                    public_key_hex, err
                ))
            })?;
        let signature_hex = hex::to_hex(signature);
        self.context
            .verify(&signature_hex, message, &public_key)
            .map_err(|err| {
                Error::SignatureVerificationError(format!(
                    "Unable to verify signature {}: {}",
                    signature_hex, err
                ))
            })
    }
}

/// A Sawtooth Secp256k SignatureVerifier that owns a context.
///
/// The SawtoothSecp256k1RefSignatureVeriifier provides an implementation of the SignatureVerifier
/// trait, that uses its own Secp256k1Context.
#[derive(Default)]
pub struct SawtoothSecp256k1SignatureVeriifier {
    context: secp256k1::Secp256k1Context,
}

impl SawtoothSecp256k1SignatureVeriifier {
    pub fn new() -> Self {
        SawtoothSecp256k1SignatureVeriifier::default()
    }
}

impl SignatureVerifier for SawtoothSecp256k1SignatureVeriifier {
    fn verify(&self, message: &[u8], signature: &[u8], public_key: &[u8]) -> Result<bool, Error> {
        SawtoothSecp256k1RefSignatureVeriifier::new(&self.context)
            .verify(message, signature, public_key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::signing::tests::test_signer_implementation;

    static KEY1_PRIV_HEX: &str = "2f1e7b7a130d7ba9da0068b3bb0ba1d79e7e77110302c9f746c3c2a63fe40088";

    #[test]
    fn test_sawtooth_secp256k1() {
        let context = secp256k1::Secp256k1Context::new();
        let private_key = secp256k1::Secp256k1PrivateKey::from_hex(KEY1_PRIV_HEX)
            .expect("unable to read hex private key");

        let sawtooth_signer = SawtoothSecp256k1RefSigner::new(&context, private_key)
            .expect("Unable to create signer");
        let sawtooth_verifier = SawtoothSecp256k1RefSignatureVeriifier::new(&context);

        test_signer_implementation(&sawtooth_signer, &sawtooth_verifier);
    }
}
