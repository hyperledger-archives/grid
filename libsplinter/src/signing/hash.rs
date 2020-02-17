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

//! A signer implemenation that generates hashes, vs real signatures.

use openssl::hash::{hash, MessageDigest};

use super::{error::Error, SignatureVerifier, SignatureVerifierFactory, Signer};

pub struct HashSigner;

impl Signer for HashSigner {
    fn sign(&self, message: &[u8]) -> Result<Vec<u8>, Error> {
        hash(MessageDigest::sha512(), message)
            .map(|digest_bytes| digest_bytes.to_vec())
            .map_err(|err| Error::SigningError(err.to_string()))
    }

    fn public_key(&self) -> &[u8] {
        b"hash_signer"
    }
}

pub struct HashVerifier;

impl SignatureVerifier for HashVerifier {
    fn verify(&self, message: &[u8], signature: &[u8], _public_key: &[u8]) -> Result<bool, Error> {
        let expected_hash = hash(MessageDigest::sha512(), message)
            .map(|digest_bytes| digest_bytes.to_vec())
            .map_err(|err| Error::SigningError(err.to_string()))?;

        Ok(expected_hash == signature)
    }
}

impl SignatureVerifierFactory for HashVerifier {
    fn create_verifier(&self) -> Box<dyn SignatureVerifier> {
        Box::new(HashVerifier)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::signing::tests::test_signer_implementation;

    #[test]
    fn test_hash() {
        let hash_signer = HashSigner;
        let hash_signature_verifier = HashVerifier;
        test_signer_implementation(&hash_signer, &hash_signature_verifier)
    }
}
