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

//! Ursa compatiable Signer and SignatureVerifiers
//!
//! Provides a Signer that is compatiable with both currently supported Hyperledger Ursa
//! signing algorithms, Ed25519Sha512 and Secp256k1. Also provides implementation specific
//! SignatureVerifier for both signing algorithms.

use ursa::keys::PublicKey;
use ursa::signatures::ed25519::Ed25519Sha512;
use ursa::signatures::secp256k1::EcdsaSecp256k1Sha256;
use ursa::signatures::{SignatureScheme, Signer};

use crate::signing::error::Error;
use crate::signing::{SignatureVerifier, Signer as SplinterSigner};

/// A UrsaSigner
///
/// The UrsaSigner provides an implementation of the Signer trait, that uses the provided
/// Ursa PublicKey and Ursa Signer.
pub struct UrsaSigner<'a, 'b, T: 'a + SignatureScheme> {
    signer: Signer<'a, 'b, T>,
    public_key: PublicKey,
}

impl<'a, 'b, T: 'a + SignatureScheme> UrsaSigner<'a, 'b, T> {
    pub fn new(signer: Signer<'a, 'b, T>, public_key: PublicKey) -> Self {
        UrsaSigner { signer, public_key }
    }
}

impl<'a, 'b, T: 'a + SignatureScheme> SplinterSigner for UrsaSigner<'a, 'b, T> {
    fn sign(&self, message: &[u8]) -> Result<Vec<u8>, Error> {
        Ok(self
            .signer
            .sign(message)
            .map_err(|err| Error::SigningError(format!("{}", err)))?)
    }

    fn public_key(&self) -> &[u8] {
        &self.public_key.0
    }
}

/// A UrsaSecp256k1SignatureVerifier
///
/// The UrsaSecp256k1SignatureVerifier provides an implementation of the SignatureVerifier trait,
/// that uses a provided EcdsaSecp256k1Sha256 SignatureScheme.
pub struct UrsaSecp256k1SignatureVerifier<'a> {
    scheme: &'a EcdsaSecp256k1Sha256,
}

impl<'a> UrsaSecp256k1SignatureVerifier<'a> {
    pub fn new(scheme: &'a EcdsaSecp256k1Sha256) -> Self {
        UrsaSecp256k1SignatureVerifier { scheme }
    }
}

impl<'a> SignatureVerifier for UrsaSecp256k1SignatureVerifier<'a> {
    fn verify(&self, message: &[u8], signature: &[u8], pk: &[u8]) -> Result<bool, Error> {
        let public_key = PublicKey(pk.to_vec());
        Ok(self
            .scheme
            .verify(message, signature, &public_key)
            .map_err(|err| Error::SignatureVerificationError(format!("{}", err)))?)
    }
}

/// A UrsaEd25519Sha512SignatureVerifier
///
/// The UrsaEd25519Sha512SignatureVerifier provides an implementation of the SignatureVerifier
/// trait, that uses a provided Ed25519Sha512 SignatureScheme.
pub struct UrsaEd25519Sha512SignatureVerifier<'a> {
    scheme: &'a Ed25519Sha512,
}

impl<'a> UrsaEd25519Sha512SignatureVerifier<'a> {
    pub fn new(scheme: &'a Ed25519Sha512) -> Self {
        UrsaEd25519Sha512SignatureVerifier { scheme }
    }
}

impl<'a> SignatureVerifier for UrsaEd25519Sha512SignatureVerifier<'a> {
    fn verify(&self, message: &[u8], signature: &[u8], pk: &[u8]) -> Result<bool, Error> {
        let public_key = PublicKey(pk.to_vec());
        Ok(self
            .scheme
            .verify(message, signature, &public_key)
            .map_err(|err| Error::SignatureVerificationError(format!("{}", err)))?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signing::tests::test_signer_implementation;

    #[test]
    // Test that the implementation of the signing traits return the same result as the
    // ursa signer and scheme directly for Secp256k1
    fn validate_ursa_secp256k1() {
        let scheme = EcdsaSecp256k1Sha256::new();
        let (public_key, private_key) = scheme.keypair(None).unwrap();
        let signer = Signer::new(&scheme, &private_key);

        let test_message = b"test message to be";
        let signature = signer.sign(test_message).unwrap();

        let ursa_signer = UrsaSigner::new(signer, public_key.clone());
        let ursa_signature = ursa_signer.sign(test_message).unwrap();

        let ursa_signature_verifier = UrsaSecp256k1SignatureVerifier::new(&scheme);

        assert!(scheme
            .verify(test_message, &signature, &public_key)
            .unwrap());

        assert!(ursa_signature_verifier
            .verify(test_message, &ursa_signature, ursa_signer.public_key())
            .unwrap());
    }

    #[test]
    // Test that the implementation of the signing traits return the same result as the
    // ursa signer and scheme directly for Ed25519
    fn validate_ursa_ed25519() {
        let scheme = Ed25519Sha512::new();
        let (public_key, private_key) = scheme.keypair(None).unwrap();
        let signer = Signer::new(&scheme, &private_key);

        let test_message = b"test message to be";
        let signature = signer.sign(test_message).unwrap();

        let ursa_signer = UrsaSigner::new(signer, public_key.clone());
        let ursa_signature = ursa_signer.sign(test_message).unwrap();

        let ursa_signature_verifier = UrsaEd25519Sha512SignatureVerifier::new(&scheme);

        assert!(scheme
            .verify(test_message, &signature, &public_key)
            .unwrap());

        assert!(ursa_signature_verifier
            .verify(test_message, &ursa_signature, ursa_signer.public_key())
            .unwrap());
    }

    #[test]
    // Verify that the implementation of the Secp256k1 traits can be dynamically passed
    fn test_ursa_secp256k1() {
        let scheme = EcdsaSecp256k1Sha256::new();
        let (public_key, private_key) = scheme.keypair(None).unwrap();

        let signer = Signer::new(&scheme, &private_key);
        let ursa_signer = UrsaSigner::new(signer, public_key.clone());
        let ursa_signature_verifier = UrsaSecp256k1SignatureVerifier::new(&scheme);
        test_signer_implementation(&ursa_signer, &ursa_signature_verifier)
    }

    #[test]
    // Verify that the implementation of the Secp256k1 traits can be dynamically passed
    fn test_ursa_ed25519() {
        let scheme = Ed25519Sha512::new();
        let (public_key, private_key) = scheme.keypair(None).unwrap();

        let signer = Signer::new(&scheme, &private_key);
        let ursa_signer = UrsaSigner::new(signer, public_key.clone());
        let ursa_signature_verifier = UrsaEd25519Sha512SignatureVerifier::new(&scheme);
        test_signer_implementation(&ursa_signer, &ursa_signature_verifier)
    }

}
