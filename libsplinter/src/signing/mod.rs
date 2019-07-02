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

//! Simple traits for signing messages and verifing signatures.
pub mod error;

pub use crate::signing::error::Error;

/// Signs messages and returns the signers public key
pub trait Signer {
    fn sign(&self, message: &[u8]) -> Result<Vec<u8>, Error>;
    fn public_key(&self) -> &[u8];
}

// Verifies that the provided signature is valid for the message and public_key
pub trait SignatureVerifier {
    fn verify(&self, message: &[u8], signature: &[u8], pk: &[u8]) -> Result<bool, Error>;
}

#[cfg(test)]
mod tests {
    use super::*;

    // reusable test to test signer and signature verifier implementations
    pub fn test_signer_implementation(
        signer: &dyn Signer,
        signature_verifier: &dyn SignatureVerifier,
    ) {
        let test_message = b"test message to be";
        let signature = signer.sign(test_message).unwrap();
        assert!(signature_verifier
            .verify(test_message, &signature, signer.public_key())
            .unwrap());
    }
}
