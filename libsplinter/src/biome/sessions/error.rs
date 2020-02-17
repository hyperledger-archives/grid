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

use std::error::Error;
use std::fmt;

use jsonwebtoken::errors::{Error as JWTError, ErrorKind};

use super::super::secrets::SecretManagerError;

/// Error for TokenIssuer
#[derive(Debug)]
pub enum TokenIssuerError {
    /// Returned when the TokenIssuer fails to encode a Token
    EncodingError(Box<dyn Error>),
    /// Returned when the TokenIssuer fails to get a valid secret
    SecretError(Box<dyn Error>),
}

impl Error for TokenIssuerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            TokenIssuerError::EncodingError(err) => Some(&**err),
            TokenIssuerError::SecretError(err) => Some(&**err),
        }
    }
}

impl fmt::Display for TokenIssuerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TokenIssuerError::EncodingError(ref s) => write!(f, "failed to issue token: {}", s),
            TokenIssuerError::SecretError(ref s) => write!(f, "failed to fetch secret: {}", s),
        }
    }
}

impl From<JWTError> for TokenIssuerError {
    fn from(err: JWTError) -> TokenIssuerError {
        TokenIssuerError::EncodingError(Box::new(err))
    }
}

impl From<SecretManagerError> for TokenIssuerError {
    fn from(err: SecretManagerError) -> TokenIssuerError {
        TokenIssuerError::SecretError(Box::new(err))
    }
}

/// Error for ClaimsBuilder
#[derive(Debug)]
pub enum ClaimsBuildError {
    /// Returned if a required field is missing
    MissingRequiredField(String),
    /// Returned if a invalid value was provided to the builder
    InvalidValue(String),
}

impl Error for ClaimsBuildError {}

impl fmt::Display for ClaimsBuildError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ClaimsBuildError::MissingRequiredField(ref s) => {
                write!(f, "failed to build claim: {}", s)
            }
            ClaimsBuildError::InvalidValue(ref s) => write!(f, "failed to build claim: {}", s),
        }
    }
}

/// Error for token validation
#[derive(Debug)]
pub enum TokenValidationError {
    /// Returned when validation fails
    ValidationError(Box<dyn Error>),
    /// Returned when the claims in the token are invalid
    InvalidClaim(String),
}

impl Error for TokenValidationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            TokenValidationError::ValidationError(err) => Some(&**err),
            TokenValidationError::InvalidClaim(_) => None,
        }
    }
}

impl fmt::Display for TokenValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TokenValidationError::ValidationError(ref s) => {
                write!(f, "failed to validate claim: {}", s)
            }
            TokenValidationError::InvalidClaim(ref s) => write!(f, "claim is invalid: {}", s),
        }
    }
}

impl From<JWTError> for TokenValidationError {
    fn from(err: JWTError) -> TokenValidationError {
        match err.kind() {
            ErrorKind::InvalidToken => {
                TokenValidationError::InvalidClaim("Token is not valid JWT".to_string())
            }
            ErrorKind::InvalidSignature => {
                TokenValidationError::InvalidClaim("Token signature is not valid".to_string())
            }
            ErrorKind::InvalidAlgorithmName => {
                TokenValidationError::InvalidClaim("Provided algorithm is not valid".to_string())
            }
            ErrorKind::ExpiredSignature => {
                TokenValidationError::InvalidClaim("The token has expired".to_string())
            }
            ErrorKind::InvalidIssuer => {
                TokenValidationError::InvalidClaim("The token has an invalid issuer".to_string())
            }
            ErrorKind::InvalidAudience => {
                TokenValidationError::InvalidClaim("The token has an invalid audience".to_string())
            }
            ErrorKind::InvalidSubject => {
                TokenValidationError::InvalidClaim("The token has an invalid subject".to_string())
            }
            ErrorKind::ImmatureSignature => {
                TokenValidationError::InvalidClaim("The token is not valid yet".to_string())
            }
            ErrorKind::InvalidAlgorithm => {
                TokenValidationError::InvalidClaim("Provided algorithm is not valid".to_string())
            }

            ErrorKind::InvalidEcdsaKey => TokenValidationError::ValidationError(Box::new(err)),
            ErrorKind::InvalidRsaKey => TokenValidationError::ValidationError(Box::new(err)),
            _ => TokenValidationError::ValidationError(Box::new(err)),
        }
    }
}
