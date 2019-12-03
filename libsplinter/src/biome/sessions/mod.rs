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

//! Provides an API for managing user sessions, including issuing and validating JWT tokens

mod claims;
mod error;
mod token_issuer;

use jsonwebtoken::{decode, Validation};
use serde::Serialize;

pub use claims::{Claims, ClaimsBuilder};
pub use error::{ClaimsBuildError, TokenIssuerError, TokenValidationError};
pub use token_issuer::AccessTokenIssuer;

const DEFAULT_LEEWAY: i64 = 10; // default leeway in seconds.

/// Implementers can issue JWT tokens
pub trait TokenIssuer<T: Serialize> {
    /// Issues a JWT token with the given claims
    fn issue_token_with_claims(&self, claims: T) -> Result<String, TokenIssuerError>;
}

/// Deserializes a JWT token, checks that a sigures is valid and checks that the claims are
/// valid. It also and performs the extra validation provided by the caller.
///
/// # Arguments
///
///  * `token` - The serialized token to be validated
///  * `secret` - The secret to be used to validate the token signature
///  * `issuer` - The expected value for the token issuer
///  * `extra_validation` - Closure that performs extra validation, returns Ok(()) if the claims
///  are valid or an error if they are not.
///
/// ```
/// use splinter::biome::sessions::{validate_token, TokenValidationError};
///
/// let token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.\
///              eyJ1c2VyX2lkIjoiY2RmMTIwNzAtNjk1Mi00NTNmLWFiNmMtYjRlMzllZmM3YzA4IiwiZXhwIjo0MTMzO\
///              Dk0NDAwLCJpc3MiOiJzZWxmLWlzc3VlZCIsImFkbWluIjoidHJ1ZSJ9.\
///              km0hcHqWC7HFy02x2V-4QrKArNpzy4fXpBpqdL70e48";
///
/// validate_token(token, "super_secret", "self-issued", |claims| {
///     let custom_claims = claims.custom_claims();
///     let is_admin = custom_claims.get("admin").ok_or_else(|| {
///         TokenValidationError::InvalidClaim("User is not an admin".to_string())
///     })?;
///     match is_admin.as_ref() {
///         "true" => Ok(()),
///         _ =>  Err(TokenValidationError::InvalidClaim("User is not an admin".to_string()))
///     }
/// }).unwrap();
/// ```
pub fn validate_token<F>(
    token: &str,
    secret: &str,
    issuer: &str,
    extra_validation: F,
) -> Result<(), TokenValidationError>
where
    F: Fn(Claims) -> Result<(), TokenValidationError>,
{
    let validation = default_validation(DEFAULT_LEEWAY, issuer);
    let claims = decode::<Claims>(&token, secret.as_ref(), &validation)?.claims;

    extra_validation(claims)
}

fn default_validation(leeway: i64, issuer: &str) -> Validation {
    let mut validation = Validation::default();
    validation.leeway = leeway;
    validation.iss = Some(issuer.to_string());
    validation
}
